//! Active-Active (CRDB) database management
//!
//! ## Overview
//! - Create and manage Active-Active databases
//! - Configure cross-region replication
//! - Monitor CRDB status

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

/// CRDB (Active-Active Database) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Crdb {
    /// The GUID of the Active-Active database
    pub guid: String,
    /// Name of Active-Active database
    pub name: String,
    /// Current status of the Active-Active database
    pub status: String,
    /// Database memory size limit, in bytes
    pub memory_size: u64,
    /// List of participating instances in the Active-Active setup
    pub instances: Vec<CrdbInstance>,
    /// Whether communication encryption is enabled
    pub encryption: Option<bool>,
    /// Database on-disk persistence policy
    pub data_persistence: Option<String>,
    /// Whether database replication is enabled
    pub replication: Option<bool>,
    /// Data eviction policy (e.g., 'allkeys-lru', 'volatile-lru')
    pub eviction_policy: Option<String>,
}

/// CRDB instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdbInstance {
    /// Unique instance ID
    pub id: u32,
    /// Cluster fully qualified name
    pub cluster: String,
    /// Human-readable cluster name
    pub cluster_name: Option<String>,
    /// Current status of this instance
    pub status: String,
    /// List of endpoint addresses for this instance
    pub endpoints: Option<Vec<String>>,
}

/// Create CRDB request
///
/// # Examples
///
/// ```rust,no_run
/// use redis_enterprise::{CreateCrdbRequest, CreateCrdbInstance};
///
/// let request = CreateCrdbRequest::builder()
///     .name("global-cache")
///     .memory_size(1024 * 1024 * 1024) // 1GB
///     .instances(vec![
///         CreateCrdbInstance::builder()
///             .cluster("cluster1.example.com")
///             .cluster_url("https://cluster1.example.com:9443")
///             .username("admin")
///             .password("password")
///             .build(),
///         CreateCrdbInstance::builder()
///             .cluster("cluster2.example.com")
///             .cluster_url("https://cluster2.example.com:9443")
///             .username("admin")
///             .password("password")
///             .build()
///     ])
///     .encryption(true)
///     .data_persistence("aof")
///     .build();
/// ```
#[derive(Debug, Serialize, TypedBuilder)]
pub struct CreateCrdbRequest {
    /// Name of the Active-Active database
    #[builder(setter(into))]
    pub name: String,
    /// Database memory size limit, in bytes
    pub memory_size: u64,
    /// List of participating cluster instances
    pub instances: Vec<CreateCrdbInstance>,
    /// Whether to encrypt communication between instances
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub encryption: Option<bool>,
    /// Database on-disk persistence policy ('disabled', 'aof', 'snapshot')
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub data_persistence: Option<String>,
    /// Data eviction policy when memory limit is reached
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub eviction_policy: Option<String>,
}

/// Create CRDB instance
#[derive(Debug, Serialize, TypedBuilder)]
pub struct CreateCrdbInstance {
    /// Cluster fully qualified name, used to uniquely identify the cluster
    #[builder(setter(into))]
    pub cluster: String,
    /// Cluster access URL (for example `https://cluster1.example.com:9443`)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub cluster_url: Option<String>,
    /// Username for cluster authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub username: Option<String>,
    /// Password for cluster authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub password: Option<String>,
}

/// Module version change requested as part of a CRDB upgrade.
///
/// The module UIDs are deprecated by Redis Enterprise Software 7.8.2 and
/// later, but remain part of the documented request contract for older
/// supported cluster versions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TypedBuilder)]
pub struct CrdbModuleUpgrade {
    /// UID of the currently installed module version.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub current_module: Option<String>,
    /// UID of the module version to install.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub new_module: Option<String>,
    /// Arguments to pass to the upgraded module.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub new_module_args: Option<String>,
}

/// Request for upgrading an Active-Active database.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, TypedBuilder)]
pub struct CrdbUpgradeRequest {
    /// Allow the upgrade to discard data when required.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub force_discard: Option<bool>,
    /// Restart the database even when an in-place upgrade is possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub force_restart: Option<bool>,
    /// Retain the current CRDT protocol version after upgrading.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub keep_crdt_protocol_version: Option<bool>,
    /// Confirm that data loss is acceptable when required by the upgrade.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub may_discard_data: Option<bool>,
    /// Module changes to apply during the upgrade.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub modules: Option<Vec<CrdbModuleUpgrade>>,
    /// Maximum number of shards to upgrade in parallel.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub parallel_shards_upgrade: Option<u64>,
    /// Preserve existing database roles during the upgrade.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub preserve_roles: Option<bool>,
    /// Redis version to install.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub redis_version: Option<String>,
}

/// Response wrapper for `GET /v1/crdbs`.
///
/// The API returns `{"crdbs": [...]}`. Kept private to the module since
/// [`CrdbHandler::list`] unwraps it; only exposed in the public API via
/// the flat `Vec<Crdb>` return.
#[derive(Debug, Clone, Deserialize)]
struct CrdbsListResponse {
    #[serde(default)]
    crdbs: Vec<Crdb>,
}

/// CRDB handler for managing Active-Active databases
pub struct CrdbHandler {
    client: RestClient,
}

impl CrdbHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        CrdbHandler { client }
    }

    /// List all CRDBs.
    ///
    /// `GET /v1/crdbs`. The API wraps the array under a `crdbs` key
    /// (`{"crdbs": [...]}`); this method unwraps it so callers get a
    /// flat `Vec<Crdb>`.
    pub async fn list(&self) -> Result<Vec<Crdb>> {
        let resp: CrdbsListResponse = self.client.get("/v1/crdbs").await?;
        Ok(resp.crdbs)
    }

    /// Get specific CRDB
    pub async fn get(&self, guid: &str) -> Result<Crdb> {
        self.client.get(&format!("/v1/crdbs/{}", guid)).await
    }

    /// Create new CRDB
    pub async fn create(&self, request: CreateCrdbRequest) -> Result<Crdb> {
        self.client.post("/v1/crdbs", &request).await
    }

    /// Update an existing CRDB.
    ///
    /// `PATCH /v1/crdbs/{crdb_guid}`. The Redis Enterprise REST API
    /// documents the CRDB update verb as `PATCH`; the previous
    /// implementation used `PUT` and returned 405 on recent cluster
    /// versions.
    pub async fn update(&self, guid: &str, updates: Value) -> Result<Crdb> {
        let response = self
            .client
            .patch_raw(&format!("/v1/crdbs/{}", guid), updates)
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Delete CRDB
    pub async fn delete(&self, guid: &str) -> Result<()> {
        self.client.delete(&format!("/v1/crdbs/{}", guid)).await
    }

    /// Get tasks for a CRDB.
    ///
    /// Redis Software exposes tasks as a global collection, not beneath an
    /// individual CRDB. Preserve this convenience method by filtering the
    /// canonical collection client-side.
    pub async fn tasks(&self, guid: &str) -> Result<Value> {
        let tasks: Vec<Value> = self.client.get("/v1/crdb_tasks").await?;
        Ok(Value::Array(
            tasks
                .into_iter()
                .filter(|task| task.get("crdb_guid").and_then(Value::as_str) == Some(guid))
                .collect(),
        ))
    }

    /// Flush all data from an Active-Active database.
    ///
    /// `PUT /v1/crdbs/{crdb_guid}/flush`. The request body is intentionally
    /// `serde_json::Value` because the documented payload is a small set of
    /// optional flags (e.g. `{}` for the default flush) whose accepted shape
    /// is version-specific; pass `json!({})` for the common case.
    pub async fn flush(&self, guid: &str, body: Value) -> Result<Value> {
        self.client
            .put_raw(&format!("/v1/crdbs/{}/flush", guid), body)
            .await
    }

    /// Retrieve the health report for an Active-Active database.
    ///
    /// `GET /v1/crdbs/{crdb_guid}/health_report`. The response is returned
    /// as `serde_json::Value` because the report is a richly structured
    /// document whose shape evolves across cluster versions.
    pub async fn health_report(&self, guid: &str) -> Result<Value> {
        self.client
            .get(&format!("/v1/crdbs/{}/health_report", guid))
            .await
    }

    /// Purge data from an instance that was forcibly removed from an
    /// Active-Active database.
    ///
    /// `PUT /v1/crdbs/{crdb_guid}/purge`. The body identifies the
    /// instance(s) to purge; pass a `Value` matching the documented
    /// shape for the cluster version under test.
    pub async fn purge(&self, guid: &str, body: Value) -> Result<Value> {
        self.client
            .put_raw(&format!("/v1/crdbs/{}/purge", guid), body)
            .await
    }

    /// Submit a configuration update against an Active-Active database.
    ///
    /// `POST /v1/crdbs/{crdb_guid}/updates`. The body shape is version-
    /// specific; pass a `Value` with the desired field changes.
    pub async fn updates(&self, guid: &str, body: Value) -> Result<Value> {
        self.client
            .post_raw(&format!("/v1/crdbs/{}/updates", guid), body)
            .await
    }

    /// Upgrade an Active-Active database.
    ///
    /// Calls `POST /v1/crdbs/{crdb_guid}/upgrade`. The response is returned
    /// as `serde_json::Value` because the currently documented CRDB task
    /// object differs from the crate's legacy task model and varies across
    /// supported cluster versions.
    pub async fn upgrade(&self, guid: &str, request: CrdbUpgradeRequest) -> Result<Value> {
        self.client
            .post(&format!("/v1/crdbs/{}/upgrade", guid), &request)
            .await
    }
}
