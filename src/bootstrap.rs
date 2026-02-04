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

/// Bootstrap status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStatus {
    /// Current status of the bootstrap operation
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Progress percentage (0.0-100.0) of the bootstrap operation
    pub progress: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Status message or error description
    pub message: Option<String>,
}

/// Bootstrap handler for cluster initialization
pub struct BootstrapHandler {
    client: RestClient,
}

impl BootstrapHandler {
    pub fn new(client: RestClient) -> Self {
        BootstrapHandler { client }
    }

    /// Initialize cluster bootstrap
    pub async fn create(&self, config: BootstrapConfig) -> Result<BootstrapStatus> {
        self.client.post("/v1/bootstrap", &config).await
    }

    /// Get bootstrap status
    pub async fn status(&self) -> Result<BootstrapStatus> {
        self.client.get("/v1/bootstrap").await
    }

    /// Join node to existing cluster
    pub async fn join(&self, config: BootstrapConfig) -> Result<BootstrapStatus> {
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
