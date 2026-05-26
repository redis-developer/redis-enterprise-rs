//! Specialized types for Redis Enterprise API objects

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Database group - represents a group of databases sharing a memory pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BdbGroup {
    /// Unique identifier (read-only).
    pub uid: u32,
    /// Memory pool size in bytes.
    pub memory_size: u64,
    /// List of member database UIDs.
    pub members: Vec<u32>,
}

/// Database connections auditing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConnsAuditingConfig {
    /// Audit protocol (e.g. `"TCP"`, `"syslog"`).
    pub audit_protocol: String,
    /// Audit collector address.
    pub audit_address: String,
    /// Audit collector port.
    pub audit_port: u16,
    /// Reconnect interval in seconds.
    pub audit_reconnect_interval: Option<u32>,
    /// Maximum number of reconnect attempts.
    pub audit_reconnect_max_attempts: Option<u32>,
}

/// Cluster check result for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Overall cluster test result (e.g. `"PASS"`, `"FAIL"`).
    pub cluster_test_result: Option<String>,
    /// Per-node check results. See [`NodeCheckResult`].
    pub nodes: Option<Vec<NodeCheckResult>>,
}

/// Node check result for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCheckResult {
    /// Node UID.
    pub node_uid: u32,
    /// Check status.
    pub status: String,
    /// List of individual checks performed on the node.
    pub checks: Option<Vec<Value>>,
}

/// Services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfiguration {
    /// Cluster manager (CM) server configuration. See [`ServiceConfig`].
    pub cm_server: Option<ServiceConfig>,
    /// CRDB coordinator configuration. See [`ServiceConfig`].
    pub crdb_coordinator: Option<ServiceConfig>,
    /// CRDB worker configuration. See [`ServiceConfig`].
    pub crdb_worker: Option<ServiceConfig>,
    /// mDNS server configuration. See [`ServiceConfig`].
    pub mdns_server: Option<ServiceConfig>,
    /// PowerDNS server configuration. See [`ServiceConfig`].
    pub pdns_server: Option<ServiceConfig>,
    /// Redis server configuration. See [`ServiceConfig`].
    pub redis_server: Option<ServiceConfig>,
    /// saslauthd service configuration. See [`ServiceConfig`].
    pub saslauthd: Option<ServiceConfig>,
    /// Stats archiver service configuration. See [`ServiceConfig`].
    pub stats_archiver: Option<ServiceConfig>,
}

/// Individual service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Whether the service is enabled.
    pub enabled: bool,
    /// Port the service listens on.
    pub port: Option<u16>,
    /// Service-specific settings (free-form).
    pub settings: Option<Value>,
}

/// OCSP (Online Certificate Status Protocol) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcspConfig {
    /// OCSP responder URL.
    pub ocsp_url: String,
    /// OCSP response timeout in seconds.
    pub ocsp_response_timeout_seconds: Option<u32>,
    /// OCSP query frequency in seconds.
    pub query_frequency_seconds: Option<u32>,
    /// OCSP recovery query frequency in seconds.
    pub recovery_frequency_seconds: Option<u32>,
    /// Maximum number of OCSP recovery attempts.
    pub recovery_max_tries: Option<u32>,
    /// OCSP responder certificate (PEM).
    pub responder_cert: Option<String>,
}

/// OCSP status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcspStatus {
    /// Name of the certificate.
    pub cert_name: String,
    /// OCSP status (e.g. `"good"`, `"revoked"`).
    pub ocsp_status: String,
    /// Timestamp when the OCSP response was produced (ISO-8601).
    pub produced_at: Option<String>,
    /// URL of the OCSP responder.
    pub responder_url: Option<String>,
    /// Timestamp of the current update (ISO-8601).
    pub this_update: Option<String>,
    /// Timestamp of the next scheduled update (ISO-8601).
    pub next_update: Option<String>,
}

/// JWT authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtAuthorize {
    /// JSON Web Key Set (JWKS) URI used to verify JWTs.
    pub jwks_uri: String,
}

/// State machine status (for long-running operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    /// State machine ID.
    pub id: String,
    /// State machine name.
    pub name: String,
    /// Current state.
    pub state: String,
    /// Check status.
    pub status: String,
    /// Percent progress (0–100).
    pub progress: Option<f64>,
    /// Creation timestamp (ISO-8601).
    pub created: Option<String>,
    /// Last update timestamp (ISO-8601).
    pub updated: Option<String>,
    /// Error message if the state machine failed.
    pub error: Option<String>,
}

/// Module metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    /// State machine name.
    pub name: String,
    /// Module version.
    pub version: String,
    /// Semantic version string.
    pub semantic_version: Option<String>,
    /// Minimum compatible Redis version.
    pub min_redis_version: Option<String>,
    /// Minimum compatible Redis Enterprise version.
    pub min_redis_pack_version: Option<String>,
    /// List of capability names exposed by the module.
    pub capabilities: Option<Vec<String>>,
    /// Default command-line arguments.
    pub command_line_args: Option<String>,
    /// Configuration command exposed by the module.
    pub config_command: Option<String>,
    /// List of module dependencies.
    pub dependencies: Option<Vec<String>>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Display name shown in the UI.
    pub display_name: Option<String>,
    /// Maintainer email address.
    pub email: Option<String>,
    /// Module homepage URL.
    pub homepage: Option<String>,
    /// Whether the module ships bundled with Redis Enterprise.
    pub is_bundled: Option<bool>,
    /// Module license identifier.
    pub license: Option<String>,
    /// Supported operating systems.
    pub os_list: Option<Vec<String>>,
    /// SHA-256 checksum of the module package.
    pub sha256: Option<String>,
    /// Unique identifier (read-only).
    pub uid: Option<String>,
    /// Supported CPU architectures.
    pub architecture_list: Option<Vec<String>>,
    /// Module author name.
    pub author: Option<String>,
}

/// Database command statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCommand {
    /// Redis command name.
    pub command: String,
    /// Number of times the command has been executed.
    pub count: u64,
}

/// Action v2 - enhanced action tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionV2 {
    /// Action unique identifier (read-only).
    pub action_uid: String,
    /// State machine name.
    pub name: String,
    /// Action type (e.g. `"backup"`, `"restore"`).
    pub action_type: String,
    /// Unix-epoch timestamp when the action was created.
    pub creation_time: u64,
    /// Percent progress (0–100).
    pub progress: f64,
    /// Check status.
    pub status: String,
    /// Additional action information. See [`ActionAdditionalInfo`].
    pub additional_info: Option<ActionAdditionalInfo>,
}

/// Additional info for ActionV2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionAdditionalInfo {
    /// Human-readable description.
    pub description: Option<String>,
    /// Error message if the state machine failed.
    pub error: Option<String>,
    /// Type of object the action operates on (e.g. `"bdb"`, `"node"`).
    pub object_type: Option<String>,
    /// UID of the object the action operates on.
    pub object_uid: Option<String>,
}

// Types are already public, no need for re-export
