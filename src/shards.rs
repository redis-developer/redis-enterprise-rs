//! Shards management for Redis Enterprise
//!
//! ## Overview
//! - List and query resources
//! - Create and update configurations
//! - Monitor status and metrics

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Response for a single metric query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResponse {
    /// Interval label for the metric series.
    pub interval: String,
    /// List of Unix-epoch timestamps for the data points.
    pub timestamps: Vec<i64>,
    /// List of metric values, aligned to `timestamps`.
    pub values: Vec<Value>,
}

/// Shard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    /// Unique identifier (read-only).
    pub uid: String,
    /// Database (BDB) UID this entity belongs to.
    pub bdb_uid: u32,
    /// Node UID this entity belongs to.
    pub node_uid: String,
    /// Role.
    pub role: String,
    /// Current status.
    pub status: String,
    /// Hash slot range owned by this shard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slots: Option<String>,
    /// Used memory in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_memory: Option<u64>,
    /// Percent progress (0-100) of in-flight backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_progress: Option<f64>,
    /// Percent progress (0-100) of in-flight import.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_progress: Option<f64>,
    /// All nodes that this shard is associated with
    pub all_nodes: Option<Vec<u32>>,
    /// Assigned slots for this shard
    pub assigned_slots: Option<String>,
    /// Client certificate subject validation type
    pub client_cert_subject_validation_type: Option<String>,
    /// Redis info for this shard
    pub redis_info: Option<Value>,
    /// Roles assigned to this shard
    pub roles: Option<Vec<String>>,
}

/// Shard stats information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    /// Unique identifier (read-only).
    pub uid: String,
    /// Per-interval metric series for the resource.
    pub intervals: Vec<StatsInterval>,
}

/// One interval of statistics, with aligned timestamps and values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsInterval {
    /// Interval label for the metric series.
    pub interval: String,
    /// List of Unix-epoch timestamps for the data points.
    pub timestamps: Vec<i64>,
    /// List of metric values, aligned to `timestamps`.
    pub values: Vec<Value>,
}

/// Shard handler for managing shards
pub struct ShardHandler {
    client: RestClient,
}

impl ShardHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        ShardHandler { client }
    }

    /// List all shards
    pub async fn list(&self) -> Result<Vec<Shard>> {
        self.client.get("/v1/shards").await
    }

    /// Get specific shard information
    pub async fn get(&self, uid: &str) -> Result<Shard> {
        self.client.get(&format!("/v1/shards/{}", uid)).await
    }

    /// Get shard statistics
    pub async fn stats(&self, uid: &str) -> Result<ShardStats> {
        self.client.get(&format!("/v1/shards/stats/{}", uid)).await
    }

    /// Get shard statistics for a specific metric
    pub async fn stats_metric(&self, uid: &str, metric: &str) -> Result<MetricResponse> {
        self.client
            .get(&format!("/v1/shards/{}/stats/{}", uid, metric))
            .await
    }

    // raw variant removed: use stats_metric()

    /// Get shards for a specific database
    pub async fn list_by_database(&self, bdb_uid: u32) -> Result<Vec<Shard>> {
        self.client
            .get(&format!("/v1/bdbs/{}/shards", bdb_uid))
            .await
    }

    /// Get shards for a specific node
    pub async fn list_by_node(&self, node_uid: u32) -> Result<Vec<Shard>> {
        self.client
            .get(&format!("/v1/nodes/{}/shards", node_uid))
            .await
    }

    // Aggregate raw helpers removed; use StatsHandler for aggregates

    /// Global failover - POST /v1/shards/actions/failover
    pub async fn failover_all(&self, body: ShardActionRequest) -> Result<Action> {
        self.client.post("/v1/shards/actions/failover", &body).await
    }

    /// Global migrate - POST /v1/shards/actions/migrate
    pub async fn migrate_all(&self, body: ShardActionRequest) -> Result<Action> {
        self.client.post("/v1/shards/actions/migrate", &body).await
    }

    /// Per-shard failover - POST /v1/shards/{uid}/actions/failover
    pub async fn failover(&self, uid: &str, body: ShardActionRequest) -> Result<Action> {
        self.client
            .post(&format!("/v1/shards/{}/actions/failover", uid), &body)
            .await
    }

    /// Per-shard migrate - POST /v1/shards/{uid}/actions/migrate
    pub async fn migrate(&self, uid: &str, body: ShardActionRequest) -> Result<Action> {
        self.client
            .post(&format!("/v1/shards/{}/actions/migrate", uid), &body)
            .await
    }
}

/// Request body for shard action endpoints (failover, migrate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardActionRequest {
    /// List of shard UIDs targeted by this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_uids: Option<Vec<String>>,
}

/// Response from a shard action endpoint with the tracking UID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action UID for tracking async operations (read-only).
    pub action_uid: String,
    /// Current status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
