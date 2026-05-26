//! Service configuration and management
//!
//! ## Overview
//! - Create cluster-wide services via `POST /v1/services`
//!
//! Older revisions of this module exposed list / get / update / status /
//! start / stop / restart handlers that hit `/v1/services` subpaths the
//! Redis Enterprise REST API never documented. They have been removed —
//! per-node service management lives on `LocalHandler` against
//! `/v1/local/services` instead, and cluster-wide configuration goes
//! through `/v1/cluster/services_configuration`.

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Service configuration as returned by `POST /v1/services`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Unique identifier for the service
    pub service_id: String,
    /// Human-readable name of the service
    pub name: String,
    /// Type of service (e.g., "mdns_server", "cm_server", "stats_archiver")
    pub service_type: String,
    /// Whether the service is enabled
    pub enabled: bool,
    /// Service-specific configuration parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    /// Current operational status of the service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// List of node UIDs where this service is running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_uids: Option<Vec<u32>>,
}

/// Services handler
pub struct ServicesHandler {
    client: RestClient,
}

impl ServicesHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        ServicesHandler { client }
    }

    /// Create a service - POST /v1/services
    pub async fn create(&self, body: Value) -> Result<Service> {
        self.client.post("/v1/services", &body).await
    }
}
