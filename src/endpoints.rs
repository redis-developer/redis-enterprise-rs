//! Database endpoint configuration and monitoring
//!
//! ## Overview
//! - Configure database endpoints
//! - Query endpoint statistics
//! - Manage endpoint routing

use crate::client::RestClient;
use crate::error::{RestError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Unique identifier (read-only).
    pub uid: String,
    /// Database (BDB) UID this entity belongs to.
    pub bdb_uid: u32,
    /// Node UID this entity belongs to.
    pub node_uid: u32,
    /// Address.
    pub addr: String,
    /// TCP port.
    pub port: u16,
    /// DNS name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_name: Option<String>,
    /// Role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Whether SSL/TLS is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,
    /// Current status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Description of the endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Error code if endpoint has an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// Endpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
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

/// Endpoints handler
pub struct EndpointsHandler {
    client: RestClient,
}

impl EndpointsHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        EndpointsHandler { client }
    }

    /// Retired endpoint-list helper.
    #[deprecated(note = "Redis Software only exposes the endpoint statistics collection")]
    pub async fn list(&self) -> Result<Vec<Endpoint>> {
        crate::error::unsupported_operation("list endpoints")
    }

    /// Retired endpoint lookup helper.
    #[deprecated(note = "Redis Software only exposes the endpoint statistics collection")]
    pub async fn get(&self, _uid: &str) -> Result<Endpoint> {
        crate::error::unsupported_operation("get endpoint")
    }

    /// Get statistics for one endpoint from the canonical global collection.
    pub async fn stats(&self, uid: &str) -> Result<EndpointStats> {
        self.all_stats()
            .await?
            .into_iter()
            .find(|stats| stats.uid == uid)
            .ok_or(RestError::NotFound)
    }

    /// Get all endpoint statistics
    pub async fn all_stats(&self) -> Result<Vec<EndpointStats>> {
        self.client.get("/v1/endpoints/stats").await
    }

    /// Retired database endpoint-list helper.
    #[deprecated(note = "Redis Software does not register database-scoped endpoint routes")]
    pub async fn list_by_database(&self, _bdb_uid: u32) -> Result<Vec<Endpoint>> {
        crate::error::unsupported_operation("list database endpoints")
    }

    /// Retired node endpoint-list helper.
    #[deprecated(note = "Redis Software does not register node-scoped endpoint routes")]
    pub async fn list_by_node(&self, _node_uid: u32) -> Result<Vec<Endpoint>> {
        crate::error::unsupported_operation("list node endpoints")
    }
}
