//! Cluster bootstrap and node joining operations
//!
//! ## Overview
//! - Bootstrap new clusters
//! - Join nodes to existing clusters
//! - Configure initial settings

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

/// Bootstrap configuration for cluster initialization
///
/// # Examples
///
/// ```rust,no_run
/// use redis_enterprise::{BootstrapConfig, ClusterBootstrap, CredentialsBootstrap};
///
/// let config = BootstrapConfig::builder()
///     .action("create_cluster")
///     .cluster(ClusterBootstrap::builder()
///         .name("my-cluster.local")
///         .rack_aware(true)
///         .build())
///     .credentials(CredentialsBootstrap::builder()
///         .username("admin@example.com")
///         .password("secure-password")
///         .build())
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct BootstrapConfig {
    /// Action to perform (e.g., 'create', 'join', 'recover_cluster')
    #[builder(setter(into))]
    pub action: String,
    /// Cluster configuration for initialization
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub cluster: Option<ClusterBootstrap>,
    /// Node configuration for bootstrap
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub node: Option<NodeBootstrap>,
    /// Admin credentials for cluster access
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub credentials: Option<CredentialsBootstrap>,
}

/// Cluster bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct ClusterBootstrap {
    /// Cluster name for identification
    #[builder(setter(into))]
    pub name: String,
    /// DNS suffixes for cluster FQDN resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub dns_suffixes: Option<Vec<String>>,
    /// Enable rack-aware placement for high availability
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub rack_aware: Option<bool>,
}

/// Node bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct NodeBootstrap {
    /// Storage paths configuration for the node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub paths: Option<NodePaths>,
}

/// Node paths configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct NodePaths {
    /// Path for persistent storage (databases, configuration, logs)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub persistent_path: Option<String>,
    /// Path for ephemeral storage (temporary files, caches)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub ephemeral_path: Option<String>,
}

/// Credentials bootstrap configuration
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CredentialsBootstrap {
    /// Admin username for cluster management
    #[builder(setter(into))]
    pub username: String,
    /// Admin password for authentication
    #[builder(setter(into))]
    pub password: String,
}

/// Inner bootstrap state, as carried inside [`BootstrapStatusResponse`].
///
/// The Redis Enterprise REST API uses the field name `state`
/// (not `status`) for the bootstrap lifecycle value, and pairs it with
/// `start_time` and `end_time`. The previous shape (`status` /
/// `progress` / `message`) did not match the wire response —
/// see `tests/fixtures/bootstrap_status.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStatus {
    /// Current bootstrap state (e.g. `"idle"`, `"initializing"`,
    /// `"completed"`, `"failed"`).
    pub state: String,
    /// ISO-8601 timestamp when the bootstrap began.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// ISO-8601 timestamp when the bootstrap reached its terminal state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// Response wrapper for `GET /v1/bootstrap` (and `POST /v1/bootstrap` /
/// `POST /v1/bootstrap/join`).
///
/// The Redis Enterprise API wraps the bootstrap state in a top-level
/// `bootstrap_status` field and includes a `local_node_info` object
/// describing the node that received the request. The previous Rust
/// shape collapsed these into a single struct and used the wrong
/// field names; the resulting decode failed against real responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStatusResponse {
    /// Inner bootstrap state.
    pub bootstrap_status: BootstrapStatus,
    /// Information about the local node that handled the request.
    ///
    /// Typed as `serde_json::Value` because the contents are
    /// version-specific and operator-oriented (CPU/storage info,
    /// supported Redis versions, software version, etc.); see the
    /// recorded `bootstrap_status.json` fixture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_node_info: Option<Value>,
}

/// Bootstrap handler for cluster initialization
pub struct BootstrapHandler {
    client: RestClient,
}

impl BootstrapHandler {
    pub fn new(client: RestClient) -> Self {
        BootstrapHandler { client }
    }

    /// Initialize cluster bootstrap.
    ///
    /// `POST /v1/bootstrap`. Returns the [`BootstrapStatusResponse`]
    /// wrapper (`bootstrap_status` + optional `local_node_info`).
    pub async fn create(&self, config: BootstrapConfig) -> Result<BootstrapStatusResponse> {
        self.client.post("/v1/bootstrap", &config).await
    }

    /// Get current bootstrap status.
    ///
    /// `GET /v1/bootstrap`. Returns the [`BootstrapStatusResponse`]
    /// wrapper.
    pub async fn status(&self) -> Result<BootstrapStatusResponse> {
        self.client.get("/v1/bootstrap").await
    }

    /// Join this node to an existing cluster.
    ///
    /// `POST /v1/bootstrap/join`. Returns the [`BootstrapStatusResponse`]
    /// wrapper.
    pub async fn join(&self, config: BootstrapConfig) -> Result<BootstrapStatusResponse> {
        self.client.post("/v1/bootstrap/join", &config).await
    }

    /// Reset bootstrap (dangerous operation)
    pub async fn reset(&self) -> Result<()> {
        self.client.delete("/v1/bootstrap").await
    }

    /// Validate bootstrap for a specific UID - POST /v1/bootstrap/validate/{uid}
    pub async fn validate_for(&self, uid: u32, body: Value) -> Result<Value> {
        self.client
            .post(&format!("/v1/bootstrap/validate/{}", uid), &body)
            .await
    }

    /// Post a specific bootstrap action - POST /v1/bootstrap/{action}
    pub async fn post_action(&self, action: &str, body: Value) -> Result<Value> {
        self.client
            .post(&format!("/v1/bootstrap/{}", action), &body)
            .await
    }
}
