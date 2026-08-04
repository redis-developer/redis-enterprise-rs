//! Database migration operations
//!
//! ## Overview
//! - Perform database migrations
//! - Track migration status
//! - Manage migration plans

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Migration task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Unique identifier for this migration task
    pub migration_id: String,
    /// Source endpoint configuration
    pub source: MigrationEndpoint,
    /// Target endpoint configuration
    pub target: MigrationEndpoint,
    /// Sync status of this migration (e.g., "syncing", "in-sync", "out-of-sync")
    pub status: String,
    /// Migration progress as a percentage (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    /// Timestamp when migration started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Timestamp when migration completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Error message if migration failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Migration endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationEndpoint {
    /// Type of endpoint (e.g., "redis", "cluster", "azure-cache")
    pub endpoint_type: String,
    /// Hostname or IP address of the endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Port number of the endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    /// Database UID (for internal cluster migrations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdb_uid: Option<u32>,
    /// Authentication password for the endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Whether to use SSL/TLS for the connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,
}

/// Create migration request
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CreateMigrationRequest {
    /// Source endpoint configuration
    pub source: MigrationEndpoint,
    /// Target endpoint configuration
    pub target: MigrationEndpoint,
    /// Type of migration to perform (e.g., "full", "incremental")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub migration_type: Option<String>,
    /// Redis key pattern to migrate (supports wildcards)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub key_pattern: Option<String>,
    /// Whether to flush the target database before migration
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub flush_target: Option<bool>,
}

/// Migrations handler
pub struct MigrationsHandler {
    client: RestClient,
}

impl MigrationsHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        MigrationsHandler { client }
    }

    /// Retired migration-list helper.
    #[deprecated(note = "Redis Software only exposes migration lookup by UID")]
    pub async fn list(&self) -> Result<Vec<Migration>> {
        crate::error::unsupported_operation("list migrations")
    }

    /// Get specific migration
    pub async fn get(&self, migration_id: &str) -> Result<Migration> {
        self.client
            .get(&format!("/v1/migrations/{}", migration_id))
            .await
    }

    /// Retired standalone migration creation helper.
    #[deprecated(note = "Redis Software does not register POST /v1/migrations")]
    pub async fn create(&self, _request: CreateMigrationRequest) -> Result<Migration> {
        crate::error::unsupported_operation("create migration")
    }

    /// Retired migration start helper.
    #[deprecated(note = "Redis Software does not register migration action routes")]
    pub async fn start(&self, _migration_id: &str) -> Result<Migration> {
        crate::error::unsupported_operation("start migration")
    }

    /// Retired migration pause helper.
    #[deprecated(note = "Redis Software does not register migration action routes")]
    pub async fn pause(&self, _migration_id: &str) -> Result<Migration> {
        crate::error::unsupported_operation("pause migration")
    }

    /// Retired migration resume helper.
    #[deprecated(note = "Redis Software does not register migration action routes")]
    pub async fn resume(&self, _migration_id: &str) -> Result<Migration> {
        crate::error::unsupported_operation("resume migration")
    }

    /// Retired migration cancellation helper.
    #[deprecated(note = "Redis Software does not register migration deletion")]
    pub async fn cancel(&self, _migration_id: &str) -> Result<()> {
        crate::error::unsupported_operation("cancel migration")
    }
}
