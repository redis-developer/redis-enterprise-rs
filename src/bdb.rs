//! Database (BDB) management for Redis Enterprise
//!
//! ## Overview
//! - Create, list, update, and delete databases
//! - Execute database actions (backup, restore, import, export)
//! - Monitor database status and metrics
//! - Configure database endpoints and sharding
//!
//! ## Examples
//!
//! ### Creating a Database
//! ```no_run
//! use redis_enterprise::{EnterpriseClient, CreateDatabaseRequest};
//!
//! # async fn example(client: EnterpriseClient) -> Result<(), Box<dyn std::error::Error>> {
//! // Simple cache database
//! let cache_db = CreateDatabaseRequest::builder()
//!     .name("my-cache")
//!     .memory_size(1_073_741_824)  // 1GB
//!     .eviction_policy("allkeys-lru")
//!     .persistence("disabled")
//!     .build();
//!
//! let db = client.databases().create(cache_db).await?;
//! println!("Created database with ID: {}", db.uid);
//! # Ok(())
//! # }
//! ```
//!
//! ### Database Actions
//! ```no_run
//! # use redis_enterprise::EnterpriseClient;
//! # async fn example(client: EnterpriseClient) -> Result<(), Box<dyn std::error::Error>> {
//! let db_id = 1;
//!
//! // Export to remote location
//! let export = client.databases().export(db_id, "ftp://backup.site/db.rdb").await?;
//! println!("Export initiated: {:?}", export.action_uid);
//!
//! // Import from backup
//! let import = client.databases().import(db_id, "ftp://backup.site/db.rdb", true).await?;
//! println!("Import started: {:?}", import.action_uid);
//! # Ok(())
//! # }
//! ```
//!
//! ### Monitoring Databases
//! ```no_run
//! # use redis_enterprise::EnterpriseClient;
//! # async fn example(client: EnterpriseClient) -> Result<(), Box<dyn std::error::Error>> {
//! // List all databases
//! let databases = client.databases().list().await?;
//! for db in databases {
//!     println!("{}: {} MB used", db.name, db.memory_used.unwrap_or(0) / 1_048_576);
//! }
//!
//! // Get database endpoints
//! let endpoints = client.databases().endpoints(1).await?;
//! for endpoint in endpoints {
//!     println!("Endpoint: {:?}:{:?}", endpoint.dns_name, endpoint.port);
//! }
//! # Ok(())
//! # }
//! ```

use crate::client::RestClient;
use crate::error::Result;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;
use typed_builder::TypedBuilder;

// Aliases for easier use
/// Alias for [`DatabaseInfo`]; the BDB ("Berkeley DB") naming is legacy Redis Enterprise terminology.
pub type Database = DatabaseInfo;
/// Alias for [`DatabaseHandler`]; retained for backwards compatibility.
pub type BdbHandler = DatabaseHandler;
/// Stream of database state updates yielded by the database watcher.
pub type DatabaseWatchStream<'a> =
    Pin<Box<dyn Stream<Item = Result<(DatabaseInfo, Option<String>)>> + Send + 'a>>;

/// Response from database action operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseActionResponse {
    /// The action UID for tracking async operations
    pub action_uid: String,
    /// Description of the action
    pub description: Option<String>,
}

/// Response from import operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResponse {
    /// The action UID for tracking the import operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_uid: Option<String>,
    /// Import status
    pub status: Option<String>,
}

/// Response from export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    /// The action UID for tracking the export operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_uid: Option<String>,
    /// Export status
    pub status: Option<String>,
}

/// Module information for database upgrade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUpgrade {
    /// Module name
    pub module_name: String,
    /// Module version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version: Option<String>,
    /// Module arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_args: Option<String>,
}

/// Request for database upgrade operation
///
/// # Examples
///
/// ```rust,no_run
/// use redis_enterprise::bdb::DatabaseUpgradeRequest;
///
/// // Upgrade to latest Redis version with role preservation
/// let request = DatabaseUpgradeRequest::builder()
///     .preserve_roles(true)
///     .build();
///
/// // Upgrade to specific version
/// let request = DatabaseUpgradeRequest::builder()
///     .redis_version("7.4.2")
///     .preserve_roles(true)
///     .parallel_shards_upgrade(2)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default, TypedBuilder)]
pub struct DatabaseUpgradeRequest {
    /// Target Redis version (optional, defaults to latest)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub redis_version: Option<String>,

    /// Preserve master/replica roles (requires extra failover)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub preserve_roles: Option<bool>,

    /// Restart shards even if no version change
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub force_restart: Option<bool>,

    /// Allow data loss in non-replicated, non-persistent databases
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub may_discard_data: Option<bool>,

    /// Force data discard even if replicated/persistent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub force_discard: Option<bool>,

    /// Keep current CRDT protocol version
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub keep_crdt_protocol_version: Option<bool>,

    /// Maximum parallel shard upgrades (default: all shards)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub parallel_shards_upgrade: Option<u32>,

    /// Modules to upgrade alongside Redis
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub modules: Option<Vec<ModuleUpgrade>>,
}

/// Version-aware database information from the REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DatabaseInfo {
    // Core database identification and status
    /// Database's unique ID (read-only).
    pub uid: u32,
    /// Database name.
    pub name: String,
    /// TCP port on which the database is available (read-only).
    pub port: Option<u16>,
    /// Current status of the database (e.g. `"active"`, `"pending"`).
    pub status: Option<String>,
    /// Database memory limit in bytes (0 for unlimited).
    pub memory_size: Option<u64>,
    /// Current memory usage in bytes (read-only).
    pub memory_used: Option<u64>,

    /// Database type (e.g., "redis", "memcached")
    #[serde(rename = "type")]
    pub type_: Option<String>,
    /// Database version (read-only).
    pub version: Option<String>,

    /// Account and action tracking
    pub account_id: Option<u32>,
    /// UID of the most recent action affecting this database (read-only).
    pub action_uid: Option<String>,

    // Sharding and placement
    /// Number of database shards.
    pub shards_count: Option<u32>,
    /// List of shard UIDs that compose the database.
    pub shard_list: Option<Vec<u32>>,
    /// Whether the database is sharded.
    pub sharding: Option<bool>,
    /// Shard placement strategy (e.g. `"dense"`, `"sparse"`).
    pub shards_placement: Option<String>,
    /// Whether in-memory replication is enabled.
    pub replication: Option<bool>,

    // Endpoints and networking
    /// Endpoints exposed by this database. See [`EndpointInfo`].
    pub endpoints: Option<Vec<EndpointInfo>>,
    /// Primary endpoint address (read-only).
    pub endpoint: Option<String>,
    /// List of endpoint IP addresses (read-only).
    pub endpoint_ip: Option<Vec<String>>,
    /// Node UID that currently hosts the endpoint (read-only).
    pub endpoint_node: Option<u32>,
    /// DNS name pointing to the master endpoint (read-only).
    pub dns_address_master: Option<String>,

    // Data persistence and backup
    /// Persistence policy (e.g. `"disabled"`, `"aof"`, `"snapshot"`).
    pub persistence: Option<String>,
    /// Data persistence mode (alias for `persistence` on some API versions).
    pub data_persistence: Option<String>,
    /// Eviction policy when the database reaches its memory limit (e.g. `"allkeys-lru"`).
    pub eviction_policy: Option<String>,

    // Timestamps
    /// Timestamp when the database was created (ISO-8601).
    pub created_time: Option<String>,
    /// Timestamp of the most recent configuration change (ISO-8601).
    pub last_changed_time: Option<String>,
    /// Timestamp of the most recent successful backup (ISO-8601).
    pub last_backup_time: Option<String>,
    /// Timestamp of the most recent successful export (ISO-8601).
    pub last_export_time: Option<String>,

    // Security and authentication
    /// Whether weak hashing is permitted for mTLS connections.
    pub mtls_allow_weak_hashing: Option<bool>,
    /// Whether outdated/expired certificates are permitted for mTLS connections.
    pub mtls_allow_outdated_certs: Option<bool>,
    /// Redis password used for client authentication.
    pub authentication_redis_pass: Option<String>,
    /// Admin password used for elevated authentication.
    pub authentication_admin_pass: Option<String>,
    /// SASL password (Memcached databases).
    pub authentication_sasl_pass: Option<String>,
    /// SASL username (Memcached databases).
    pub authentication_sasl_uname: Option<String>,
    /// List of SSL client certificates accepted for authentication.
    pub authentication_ssl_client_certs: Option<Vec<Value>>,
    /// List of SSL certificates trusted for CRDT (Active-Active) connections.
    pub authentication_ssl_crdt_certs: Option<Vec<Value>>,
    /// List of authorized subjects for certificate-based authentication.
    pub authorized_subjects: Option<Vec<Value>>,
    /// Whether inter-node data encryption is enabled.
    pub data_internode_encryption: Option<bool>,
    /// Whether SSL/TLS is required for client connections.
    pub ssl: Option<bool>,
    /// TLS mode for client connections (e.g. `"enabled"`, `"disabled"`).
    pub tls_mode: Option<String>,
    /// Client certificate enforcement mode (e.g. `"enabled"`, `"disabled"`).
    pub enforce_client_authentication: Option<String>,
    /// Whether the default Redis user is enabled.
    pub default_user: Option<bool>,
    /// ACL configuration
    pub acl: Option<Value>,
    /// Client certificate subject validation type
    pub client_cert_subject_validation_type: Option<String>,
    /// Compare key hslot
    pub compare_key_hslot: Option<bool>,
    /// DNS suffixes for endpoints
    pub dns_suffixes: Option<Vec<String>>,
    /// Group UID for the database
    pub group_uid: Option<u32>,
    /// Redis cluster mode enabled
    pub redis_cluster_enabled: Option<bool>,

    // CRDT/Active-Active fields
    /// Whether this database participates in a CRDT (Active-Active) deployment.
    pub crdt: Option<bool>,
    /// Whether CRDT (Active-Active) functionality is enabled.
    pub crdt_enabled: Option<bool>,
    /// CRDT configuration version.
    pub crdt_config_version: Option<u32>,
    /// Replica ID assigned to this database in the CRDT cluster.
    pub crdt_replica_id: Option<u32>,
    /// Comma-separated list of replica IDs that have been removed from the CRDT.
    pub crdt_ghost_replica_ids: Option<String>,
    /// CRDT feature-set version.
    pub crdt_featureset_version: Option<u32>,
    /// CRDT protocol version.
    pub crdt_protocol_version: Option<u32>,
    /// Globally unique identifier for the CRDT.
    pub crdt_guid: Option<String>,
    /// Modules configuration for the CRDT.
    pub crdt_modules: Option<String>,
    /// Comma-separated list of CRDT replica endpoints.
    pub crdt_replicas: Option<String>,
    /// List of CRDT replica sources.
    pub crdt_sources: Option<Vec<Value>>,
    /// CRDT sync state (e.g. `"enabled"`, `"disabled"`).
    pub crdt_sync: Option<String>,
    /// Seconds before a stalled CRDT sync connection raises an alarm.
    pub crdt_sync_connection_alarm_timeout_seconds: Option<u32>,
    /// Whether CRDT sync is distributed across shards.
    pub crdt_sync_dist: Option<bool>,
    /// Whether the CRDT syncer auto-unlatches on out-of-memory conditions.
    pub crdt_syncer_auto_oom_unlatch: Option<bool>,
    /// XADD stream ID uniqueness mode for CRDT.
    pub crdt_xadd_id_uniqueness_mode: Option<String>,
    /// Whether CRDT causal consistency is enabled.
    pub crdt_causal_consistency: Option<bool>,
    /// Replication backlog size for CRDT (bytes, or `"auto"`).
    pub crdt_repl_backlog_size: Option<String>,

    // Replication settings
    /// Whether persistence is enabled on the master shard.
    pub master_persistence: Option<bool>,
    /// Whether slave (replica) high availability is enabled.
    pub slave_ha: Option<bool>,
    /// Priority for slave HA replica selection.
    pub slave_ha_priority: Option<u32>,
    /// Whether replica shards are read-only.
    pub replica_read_only: Option<bool>,
    /// List of upstream replica sources for Replica-Of mode.
    pub replica_sources: Option<Vec<Value>>,
    /// Replica sync state (e.g. `"enabled"`, `"paused"`).
    pub replica_sync: Option<String>,
    /// Seconds before a stalled replica sync connection raises an alarm.
    pub replica_sync_connection_alarm_timeout_seconds: Option<u32>,
    /// Whether replica sync is distributed across shards.
    pub replica_sync_dist: Option<bool>,
    /// Replication backlog size (bytes, or `"auto"`).
    pub repl_backlog_size: Option<String>,

    // Connection and performance settings
    /// Maximum number of client connections (alias for `maxclients` on some versions).
    pub max_connections: Option<u32>,
    /// Maximum number of concurrent client connections.
    pub maxclients: Option<u32>,
    /// Number of proxy connection threads.
    pub conns: Option<u32>,
    /// Proxy connection type (e.g. `"per-thread"`, `"per-shard"`).
    pub conns_type: Option<String>,
    /// Maximum number of pipelined commands per client.
    pub max_client_pipeline: Option<u32>,
    /// Maximum number of pipelined commands.
    pub max_pipelined: Option<u32>,

    // AOF (Append Only File) settings
    /// AOF (append-only file) fsync policy (e.g. `"appendfsync-every-sec"`).
    pub aof_policy: Option<String>,
    /// Maximum AOF file size in bytes.
    pub max_aof_file_size: Option<u64>,
    /// Maximum AOF load time in seconds.
    pub max_aof_load_time: Option<u32>,

    // Active defragmentation settings
    /// Active defragmentation toggle (e.g. `"enabled"`, `"disabled"`).
    pub activedefrag: Option<String>,
    /// Maximum CPU percentage used by active defrag.
    pub active_defrag_cycle_max: Option<u32>,
    /// Minimum CPU percentage used by active defrag.
    pub active_defrag_cycle_min: Option<u32>,
    /// Minimum amount of fragmentation waste (bytes) before defrag starts.
    pub active_defrag_ignore_bytes: Option<String>,
    /// Maximum number of fields scanned per defrag cycle.
    pub active_defrag_max_scan_fields: Option<u32>,
    /// Lower fragmentation threshold (percent) for active defrag.
    pub active_defrag_threshold_lower: Option<u32>,
    /// Upper fragmentation threshold (percent) for active defrag.
    pub active_defrag_threshold_upper: Option<u32>,

    // Backup settings
    /// Whether periodic backup is enabled.
    pub backup: Option<bool>,
    /// Reason for the most recent backup failure (read-only).
    pub backup_failure_reason: Option<String>,
    /// Number of backup snapshots retained.
    pub backup_history: Option<u32>,
    /// Backup interval in seconds.
    pub backup_interval: Option<u32>,
    /// Offset (in seconds) from midnight when scheduled backups run.
    pub backup_interval_offset: Option<u32>,
    /// Backup destination location (URI or storage config object).
    pub backup_location: Option<Value>,
    /// Percent progress (0–100) of the in-flight backup (read-only).
    pub backup_progress: Option<f64>,
    /// Current backup status (e.g. `"idle"`, `"running"`, `"failed"`).
    pub backup_status: Option<String>,

    // Import/Export settings
    /// List of dataset import source descriptors.
    pub dataset_import_sources: Option<Vec<Value>>,
    /// Reason for the most recent import failure (read-only).
    pub import_failure_reason: Option<String>,
    /// Percent progress (0–100) of the in-flight import (read-only).
    pub import_progress: Option<f64>,
    /// Current import status (e.g. `"idle"`, `"running"`, `"failed"`).
    pub import_status: Option<String>,
    /// Reason for the most recent export failure (read-only).
    pub export_failure_reason: Option<String>,
    /// Percent progress (0–100) of the in-flight export (read-only).
    pub export_progress: Option<f64>,
    /// Current export status (e.g. `"idle"`, `"running"`, `"failed"`).
    pub export_status: Option<String>,
    /// Skip-analyze policy applied during import.
    pub skip_import_analyze: Option<String>,

    // Monitoring and metrics
    /// Whether all metrics are exported.
    pub metrics_export_all: Option<bool>,
    /// Whether the text monitor log is generated for this database.
    pub generate_text_monitor: Option<bool>,
    /// Whether email alerts are enabled for this database.
    pub email_alerts: Option<bool>,

    // Modules and features
    /// List of Redis modules loaded into the database.
    pub module_list: Option<Vec<Value>>,
    /// Search configuration - can be bool or object depending on API version
    #[serde(default)]
    pub search: Option<Value>,
    /// Timeseries configuration - can be bool or object depending on API version
    #[serde(default)]
    pub timeseries: Option<Value>,

    // BigStore/Flash storage settings
    /// Whether Redis on Flash (BigStore) is enabled.
    pub bigstore: Option<bool>,
    /// RAM portion of the database (bytes) when BigStore is enabled.
    pub bigstore_ram_size: Option<u64>,
    /// Maximum percentage of memory used by RAM in BigStore.
    pub bigstore_max_ram_ratio: Option<u32>,
    /// Per-shard RAM weights for BigStore.
    pub bigstore_ram_weights: Option<Vec<Value>>,
    /// BigStore version in use.
    pub bigstore_version: Option<u32>,

    // Network and proxy settings
    /// Proxy policy (e.g. `"single"`, `"all-master-shards"`, `"all-nodes"`).
    pub proxy_policy: Option<String>,
    /// Whether OSS-cluster API compatibility is enabled.
    pub oss_cluster: Option<bool>,
    /// Preferred endpoint type advertised by the OSS-cluster API (e.g. `"hostname"`, `"ip"`).
    pub oss_cluster_api_preferred_endpoint_type: Option<String>,
    /// Preferred IP type advertised by the OSS-cluster API (e.g. `"internal"`, `"external"`).
    pub oss_cluster_api_preferred_ip_type: Option<String>,
    /// Whether OSS-style hash-slot sharding is enabled.
    pub oss_sharding: Option<bool>,

    // Redis-specific settings
    /// Redis Enterprise database server version (read-only).
    pub redis_version: Option<String>,
    /// Whether RESP3 protocol is enabled.
    pub resp3: Option<bool>,
    /// Comma-separated list of disabled Redis commands.
    pub disabled_commands: Option<String>,

    // Clustering and sharding
    /// Hash-slot policy (e.g. `"legacy"`, `"16k"`).
    pub hash_slots_policy: Option<String>,
    /// List of regular expressions used to extract shard keys.
    pub shard_key_regex: Option<Vec<Value>>,
    /// Whether cross-slot multi-key commands are blocked.
    pub shard_block_crossslot_keys: Option<bool>,
    /// Whether commands referencing foreign keys are blocked.
    pub shard_block_foreign_keys: Option<bool>,
    /// Whether implicit shard keys are used.
    pub implicit_shard_key: Option<bool>,

    // Node placement and rack awareness
    /// List of node UIDs to avoid when placing shards.
    pub avoid_nodes: Option<Vec<String>>,
    /// List of node UIDs preferred for placing shards.
    pub use_nodes: Option<Vec<String>>,
    /// Whether rack-aware shard placement is enabled.
    pub rack_aware: Option<bool>,

    // Operational settings
    /// Whether automatic Redis upgrades are enabled.
    pub auto_upgrade: Option<bool>,
    /// Whether this is an internal/system database.
    pub internal: Option<bool>,
    /// Whether database connection auditing is enabled.
    pub db_conns_auditing: Option<bool>,
    /// Whether the replica flushes its dataset on full sync.
    pub flush_on_fullsync: Option<bool>,
    /// Whether selective flush is used for replica sync.
    pub use_selective_flush: Option<bool>,

    // Sync and replication control
    /// Database sync mode.
    pub sync: Option<String>,
    /// List of sync sources for Replica-Of configurations.
    pub sync_sources: Option<Vec<Value>>,
    /// Number of dedicated threads used by the syncer.
    pub sync_dedicated_threads: Option<u32>,
    /// Syncer mode (e.g. `"distributed"`, `"centralized"`).
    pub syncer_mode: Option<String>,
    /// Log level used by the syncer.
    pub syncer_log_level: Option<String>,
    /// Whether the syncer supports on-the-fly reconfiguration.
    pub support_syncer_reconf: Option<bool>,

    // Gradual sync settings
    /// Gradual-source sync mode.
    pub gradual_src_mode: Option<String>,
    /// Maximum number of sources synced concurrently in gradual mode.
    pub gradual_src_max_sources: Option<u32>,
    /// Gradual sync mode.
    pub gradual_sync_mode: Option<String>,
    /// Maximum number of shards synced concurrently per source in gradual mode.
    pub gradual_sync_max_shards_per_source: Option<u32>,

    // Slave and buffer settings
    /// Replica output buffer limits.
    pub slave_buffer: Option<String>,

    // Snapshot settings
    /// Snapshot policy entries (RDB save points).
    pub snapshot_policy: Option<Vec<Value>>,

    // Scheduling and recovery
    /// Proxy scheduling policy (e.g. `"cmp"`, `"mnp"`, `"mru"`).
    pub sched_policy: Option<String>,
    /// Seconds to wait before automatic recovery (negative disables).
    pub recovery_wait_time: Option<i32>,

    // Performance and optimization
    /// MULTI command optimization mode.
    pub multi_commands_opt: Option<String>,
    /// Configured ingress throughput cap.
    pub throughput_ingress: Option<f64>,
    /// Maximum number of keys tracked by the client-side caching tracking table.
    pub tracking_table_max_keys: Option<u32>,
    /// Whether the WAIT command is allowed.
    pub wait_command: Option<bool>,

    // Legacy and deprecated fields
    /// Background operations currently running on the database (read-only).
    pub background_op: Option<Vec<Value>>,

    // Advanced configuration
    /// Whether MKMS (multi-key multi-slot) commands are allowed.
    pub mkms: Option<bool>,
    /// Role-based permissions for the database.
    pub roles_permissions: Option<Vec<Value>>,
    /// Free-form tags attached to the database.
    pub tags: Option<Vec<String>>,
    /// Current topology epoch counter (read-only).
    pub topology_epoch: Option<u32>,

    /// Additive or version-specific response fields not yet modeled explicitly.
    ///
    /// Redis Software adds advanced database controls between supported release
    /// families. Retaining them prevents typed reads from silently discarding
    /// fields while stable public fields are promoted deliberately.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// Database endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EndpointInfo {
    /// Unique identifier for the endpoint
    pub uid: Option<String>,
    /// List of IP addresses for the endpoint
    pub addr: Option<Vec<String>>,
    /// Port number for the endpoint
    pub port: Option<u16>,
    /// DNS name for the endpoint
    pub dns_name: Option<String>,
    /// Proxy policy for the endpoint
    pub proxy_policy: Option<String>,
    /// Address type (e.g., "internal", "external")
    pub addr_type: Option<String>,
    /// OSS cluster API preferred IP type
    pub oss_cluster_api_preferred_ip_type: Option<String>,
    /// OSS cluster API preferred endpoint type (`ip` or `hostname`).
    pub oss_cluster_api_preferred_endpoint_type: Option<String>,
    /// List of proxy UIDs to exclude
    pub exclude_proxies: Option<Vec<u32>>,
    /// List of proxy UIDs to include
    pub include_proxies: Option<Vec<u32>>,
    /// Additive or version-specific endpoint fields.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// Module configuration for database creation
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct ModuleConfig {
    /// Module name to load (e.g. `"search"`, `"timeseries"`, `"bf"`).
    #[builder(setter(into))]
    pub module_name: String,
    /// Arguments passed to the module when it loads.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub module_args: Option<String>,
}

/// Create database request
///
/// # Examples
///
/// ```rust,no_run
/// use redis_enterprise::{CreateDatabaseRequest, ModuleConfig};
///
/// let request = CreateDatabaseRequest::builder()
///     .name("my-database")
///     .memory_size(1024 * 1024 * 1024) // 1GB
///     .redis_version("7.4") // Optional; choose a version advertised by /v1/nodes
///     .port(12000)
///     .replication(true)
///     .persistence("aof")
///     .eviction_policy("volatile-lru")
///     .shards_count(2)
///     .authentication_redis_pass("secure-password")
///     .build();
/// ```
#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct CreateDatabaseRequest {
    /// Database name.
    #[builder(setter(into))]
    pub name: String,
    /// Database memory limit in bytes (0 for unlimited).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub memory_size: Option<u64>,
    /// Redis database engine version to provision (for example, `"7.4"`).
    ///
    /// Supported values are advertised by each node's
    /// `supported_database_versions` field. When omitted, Redis Software
    /// chooses its configured default.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub redis_version: Option<String>,
    /// TCP port on which the database is available (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub port: Option<u16>,
    /// Whether in-memory replication is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub replication: Option<bool>,
    /// Persistence policy (e.g. `"disabled"`, `"aof"`, `"snapshot"`).
    ///
    /// Redis Software names this field `data_persistence` on the wire. The
    /// shorter Rust field and builder name are retained for source
    /// compatibility.
    #[serde(
        rename = "data_persistence",
        alias = "persistence",
        skip_serializing_if = "Option::is_none"
    )]
    #[builder(default, setter(into, strip_option))]
    pub persistence: Option<String>,
    /// Eviction policy when the database reaches its memory limit (e.g. `"allkeys-lru"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub eviction_policy: Option<String>,
    /// Whether the database is sharded.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub sharding: Option<bool>,
    /// Number of database shards.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub shards_count: Option<u32>,
    /// Shard count.
    #[serde(skip_serializing_if = "Option::is_none", alias = "shard_count")]
    #[builder(default, setter(strip_option))]
    pub shard_count: Option<u32>,
    /// Proxy policy (e.g. `"single"`, `"all-master-shards"`, `"all-nodes"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub proxy_policy: Option<String>,
    /// Whether rack-aware shard placement is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub rack_aware: Option<bool>,
    /// List of Redis modules loaded into the database.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub module_list: Option<Vec<ModuleConfig>>,
    /// Whether this database participates in a CRDT (Active-Active) deployment.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub crdt: Option<bool>,
    /// Redis password used for client authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub authentication_redis_pass: Option<String>,
}

/// Database handler for executing database commands
pub struct DatabaseHandler {
    client: RestClient,
}

impl DatabaseHandler {
    /// New.
    pub fn new(client: RestClient) -> Self {
        DatabaseHandler { client }
    }

    /// List all databases (BDB.LIST)
    pub async fn list(&self) -> Result<Vec<DatabaseInfo>> {
        self.client.get("/v1/bdbs").await
    }

    /// Get specific database info (BDB.INFO)
    pub async fn info(&self, uid: u32) -> Result<DatabaseInfo> {
        self.client.get(&format!("/v1/bdbs/{}", uid)).await
    }

    /// Get specific database info (alias for info)
    pub async fn get(&self, uid: u32) -> Result<DatabaseInfo> {
        self.info(uid).await
    }

    /// Create a new database (BDB.CREATE)
    pub async fn create(&self, request: CreateDatabaseRequest) -> Result<DatabaseInfo> {
        self.client.post("/v1/bdbs", &request).await
    }

    /// Update database configuration (BDB.UPDATE)
    pub async fn update(&self, uid: u32, updates: Value) -> Result<DatabaseInfo> {
        self.client
            .put(&format!("/v1/bdbs/{}", uid), &updates)
            .await
    }

    /// Delete a database (BDB.DELETE)
    pub async fn delete(&self, uid: u32) -> Result<()> {
        self.client.delete(&format!("/v1/bdbs/{}", uid)).await
    }

    /// Get database stats (BDB.STATS)
    pub async fn stats(&self, uid: u32) -> Result<Value> {
        self.client.get(&format!("/v1/bdbs/stats/{}", uid)).await
    }

    /// Get database metrics through the canonical database statistics route.
    ///
    /// Redis Software does not register `/v1/bdbs/metrics/{uid}`. Keep this
    /// convenience alias for callers that use metrics terminology.
    pub async fn metrics(&self, uid: u32) -> Result<Value> {
        self.stats(uid).await
    }

    /// Export database (BDB.EXPORT)
    pub async fn export(&self, uid: u32, export_location: &str) -> Result<ExportResponse> {
        let body = serde_json::json!({
            "export_location": export_location
        });
        self.client
            .post(&format!("/v1/bdbs/{}/actions/export", uid), &body)
            .await
    }

    /// Import database (BDB.IMPORT)
    pub async fn import(
        &self,
        uid: u32,
        import_location: &str,
        flush: bool,
    ) -> Result<ImportResponse> {
        let body = serde_json::json!({
            "import_location": import_location,
            "flush": flush
        });
        self.client
            .post(&format!("/v1/bdbs/{}/actions/import", uid), &body)
            .await
    }

    /// Flush all keys from a database.
    ///
    /// `PUT /v1/bdbs/{uid}/flush`. This is the path-segment-style action
    /// the REST API documents; the previous implementation POSTed to
    /// `/v1/bdbs/{uid}/actions/flush`, which is not in the spec.
    pub async fn flush(&self, uid: u32) -> Result<DatabaseActionResponse> {
        let response = self
            .client
            .put_raw(&format!("/v1/bdbs/{}/flush", uid), serde_json::json!({}))
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Get database shards (BDB.SHARDS)
    pub async fn shards(&self, uid: u32) -> Result<Value> {
        self.client.get(&format!("/v1/bdbs/{}/shards", uid)).await
    }

    /// Retired database endpoints helper.
    #[deprecated(note = "Redis Software does not register database-scoped endpoint routes")]
    pub async fn endpoints(&self, _uid: u32) -> Result<Vec<EndpointInfo>> {
        crate::error::unsupported_operation("list database endpoints")
    }

    /// Optimize shards placement (status) - GET
    pub async fn optimize_shards_placement(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!(
                "/v1/bdbs/{}/actions/optimize_shards_placement",
                uid
            ))
            .await
    }

    /// Recover database (status) - GET
    pub async fn recover_status(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/actions/recover", uid))
            .await
    }

    /// Recover database - POST
    pub async fn recover(&self, uid: u32) -> Result<DatabaseActionResponse> {
        self.client
            .post(
                &format!("/v1/bdbs/{}/actions/recover", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Resume traffic - POST
    pub async fn resume_traffic(&self, uid: u32) -> Result<DatabaseActionResponse> {
        self.client
            .post(
                &format!("/v1/bdbs/{}/actions/resume_traffic", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Stop traffic - POST
    pub async fn stop_traffic(&self, uid: u32) -> Result<DatabaseActionResponse> {
        self.client
            .post(
                &format!("/v1/bdbs/{}/actions/stop_traffic", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Rebalance database - PUT
    pub async fn rebalance(&self, uid: u32) -> Result<DatabaseActionResponse> {
        self.client
            .put(
                &format!("/v1/bdbs/{}/actions/rebalance", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Revamp database - PUT
    pub async fn revamp(&self, uid: u32) -> Result<DatabaseActionResponse> {
        self.client
            .put(
                &format!("/v1/bdbs/{}/actions/revamp", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Reset backup status - PUT
    pub async fn backup_reset_status(&self, uid: u32) -> Result<Value> {
        self.client
            .put(
                &format!("/v1/bdbs/{}/actions/backup_reset_status", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Reset export status - PUT
    pub async fn export_reset_status(&self, uid: u32) -> Result<Value> {
        self.client
            .put(
                &format!("/v1/bdbs/{}/actions/export_reset_status", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Reset import status - PUT
    pub async fn import_reset_status(&self, uid: u32) -> Result<Value> {
        self.client
            .put(
                &format!("/v1/bdbs/{}/actions/import_reset_status", uid),
                &serde_json::json!({}),
            )
            .await
    }

    /// Peer stats for a database - GET
    pub async fn peer_stats(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/peer_stats", uid))
            .await
    }

    /// Peer stats for a specific peer - GET
    pub async fn peer_stats_for(&self, uid: u32, peer_uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/peer_stats/{}", uid, peer_uid))
            .await
    }

    /// Sync source stats for a database - GET
    pub async fn sync_source_stats(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/sync_source_stats", uid))
            .await
    }

    /// Sync source stats for a specific source - GET
    pub async fn sync_source_stats_for(&self, uid: u32, src_uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/sync_source_stats/{}", uid, src_uid))
            .await
    }

    /// Syncer state (all) - GET
    pub async fn syncer_state(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/syncer_state", uid))
            .await
    }

    /// Syncer state for CRDT - GET
    pub async fn syncer_state_crdt(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/syncer_state/crdt", uid))
            .await
    }

    /// Syncer state for replica - GET
    pub async fn syncer_state_replica(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/{}/syncer_state/replica", uid))
            .await
    }

    /// Database passwords delete - DELETE
    pub async fn passwords_delete(&self, uid: u32) -> Result<()> {
        self.client
            .delete(&format!("/v1/bdbs/{}/passwords", uid))
            .await
    }

    /// List all database alerts - GET
    pub async fn alerts_all(&self) -> Result<Value> {
        self.client.get("/v1/bdbs/alerts").await
    }

    /// List alerts for a specific database - GET
    pub async fn alerts_for(&self, uid: u32) -> Result<Value> {
        self.client.get(&format!("/v1/bdbs/alerts/{}", uid)).await
    }

    /// Get a specific alert for a database - GET
    pub async fn alert_detail(&self, uid: u32, alert: &str) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/alerts/{}/{}", uid, alert))
            .await
    }

    /// CRDT source alerts - GET
    pub async fn crdt_source_alerts_all(&self) -> Result<Value> {
        self.client.get("/v1/bdbs/crdt_sources/alerts").await
    }

    /// CRDT source alerts for DB - GET
    pub async fn crdt_source_alerts_for(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/crdt_sources/alerts/{}", uid))
            .await
    }

    /// CRDT source alerts for specific source - GET
    pub async fn crdt_source_alerts_source(&self, uid: u32, source_id: u32) -> Result<Value> {
        self.client
            .get(&format!(
                "/v1/bdbs/crdt_sources/alerts/{}/{}",
                uid, source_id
            ))
            .await
    }

    /// CRDT source alert detail - GET
    pub async fn crdt_source_alert_detail(
        &self,
        uid: u32,
        source_id: u32,
        alert: &str,
    ) -> Result<Value> {
        self.client
            .get(&format!(
                "/v1/bdbs/crdt_sources/alerts/{}/{}/{}",
                uid, source_id, alert
            ))
            .await
    }

    /// Replica source alerts - GET
    pub async fn replica_source_alerts_all(&self) -> Result<Value> {
        self.client.get("/v1/bdbs/replica_sources/alerts").await
    }

    /// Replica source alerts for DB - GET
    pub async fn replica_source_alerts_for(&self, uid: u32) -> Result<Value> {
        self.client
            .get(&format!("/v1/bdbs/replica_sources/alerts/{}", uid))
            .await
    }

    /// Replica source alerts for specific source - GET
    pub async fn replica_source_alerts_source(&self, uid: u32, source_id: u32) -> Result<Value> {
        self.client
            .get(&format!(
                "/v1/bdbs/replica_sources/alerts/{}/{}",
                uid, source_id
            ))
            .await
    }

    /// Replica source alert detail - GET
    pub async fn replica_source_alert_detail(
        &self,
        uid: u32,
        source_id: u32,
        alert: &str,
    ) -> Result<Value> {
        self.client
            .get(&format!(
                "/v1/bdbs/replica_sources/alerts/{}/{}/{}",
                uid, source_id, alert
            ))
            .await
    }

    /// Upgrade database Redis version and/or modules (BDB.UPGRADE)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use redis_enterprise::EnterpriseClient;
    /// # use redis_enterprise::bdb::DatabaseUpgradeRequest;
    /// # async fn example() -> redis_enterprise::Result<()> {
    /// let client = EnterpriseClient::builder()
    ///     .base_url("https://localhost:9443")
    ///     .username("admin")
    ///     .password("password")
    ///     .insecure(true)
    ///     .build()?;
    ///
    /// // Upgrade to latest Redis version
    /// let request = DatabaseUpgradeRequest {
    ///     redis_version: None,  // defaults to latest
    ///     preserve_roles: Some(true),
    ///     ..Default::default()
    /// };
    /// client.databases().upgrade_redis_version(1, request).await?;
    ///
    /// // Upgrade to specific Redis version
    /// let request = DatabaseUpgradeRequest {
    ///     redis_version: Some("7.4.2".to_string()),
    ///     preserve_roles: Some(true),
    ///     ..Default::default()
    /// };
    /// client.databases().upgrade_redis_version(1, request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upgrade_redis_version(
        &self,
        uid: u32,
        request: DatabaseUpgradeRequest,
    ) -> Result<DatabaseActionResponse> {
        self.client
            .post(&format!("/v1/bdbs/{}/upgrade", uid), &request)
            .await
    }

    /// Reset the database admin password.
    ///
    /// `PUT /v1/bdbs/{uid}/reset_admin_pass`. This is the path-segment-style
    /// action the REST API documents. The new password is sent in the body
    /// under the `authentication_redis_pass` key.
    pub async fn reset_admin_pass(
        &self,
        uid: u32,
        new_password: &str,
    ) -> Result<DatabaseActionResponse> {
        let body = serde_json::json!({
            "authentication_redis_pass": new_password
        });
        let response = self
            .client
            .put_raw(&format!("/v1/bdbs/{}/reset_admin_pass", uid), body)
            .await?;
        serde_json::from_value(response).map_err(Into::into)
    }

    /// Reset database password.
    ///
    /// Deprecated alias for [`Self::reset_admin_pass`]. The previous
    /// implementation POSTed to `/v1/bdbs/{uid}/actions/reset_password`,
    /// which is not in the REST API spec.
    #[deprecated(
        since = "0.9.0",
        note = "use `reset_admin_pass`; the action POST path was not in the REST API spec"
    )]
    pub async fn reset_password(
        &self,
        uid: u32,
        new_password: &str,
    ) -> Result<DatabaseActionResponse> {
        self.reset_admin_pass(uid, new_password).await
    }

    /// Check database availability.
    ///
    /// Redis Software returns an empty successful response for this endpoint,
    /// so availability is represented by `Ok(())` rather than a JSON value.
    pub async fn availability(&self, uid: u32) -> Result<()> {
        self.client
            .get_empty(&format!("/v1/bdbs/{}/availability", uid))
            .await
    }

    /// Check local database endpoint availability.
    ///
    /// Redis Software returns an empty successful response for this endpoint,
    /// so availability is represented by `Ok(())` rather than a JSON value.
    pub async fn endpoint_availability(&self, uid: u32) -> Result<()> {
        self.client
            .get_empty(&format!("/v1/local/bdbs/{}/endpoint/availability", uid))
            .await
    }

    /// Create database using v2 API (supports recovery plan)
    pub async fn create_v2(&self, request: Value) -> Result<DatabaseInfo> {
        self.client.post("/v2/bdbs", &request).await
    }

    /// Watch database status changes in real-time
    ///
    /// Polls the database endpoint and yields updates when status changes occur.
    /// Useful for monitoring database operations like upgrades, migrations, backups, etc.
    ///
    /// # Arguments
    /// * `uid` - Database ID to watch
    /// * `poll_interval` - Time to wait between polls
    ///
    /// # Returns
    /// A stream of `(DatabaseInfo, Option<String>)` tuples where:
    /// - `DatabaseInfo` - Current database state
    /// - `Option<String>` - Previous status (None on first poll, Some on status change)
    ///
    /// # Example
    /// ```no_run
    /// use redis_enterprise::EnterpriseClient;
    /// use futures::StreamExt;
    /// use std::time::Duration;
    ///
    /// # async fn example(client: EnterpriseClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let db_handler = client.databases();
    /// let mut stream = db_handler.watch_database(1, Duration::from_secs(5));
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok((db_info, prev_status)) => {
    ///             if let Some(old_status) = prev_status {
    ///                 println!("Status changed: {} -> {}", old_status, db_info.status.unwrap_or_default());
    ///             } else {
    ///                 println!("Initial status: {}", db_info.status.unwrap_or_default());
    ///             }
    ///         }
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn watch_database(&self, uid: u32, poll_interval: Duration) -> DatabaseWatchStream<'_> {
        Box::pin(async_stream::stream! {
            let mut last_status: Option<String> = None;

            loop {
                match self.info(uid).await {
                    Ok(db_info) => {
                        let current_status = db_info.status.clone();

                        // Check if status changed
                        let status_changed = match (&last_status, &current_status) {
                            (Some(old), Some(new)) => old != new,
                            (None, Some(_)) => false, // First poll, not a change
                            (Some(_), None) => true,  // Status disappeared
                            (None, None) => false,
                        };

                        // Yield the database info with previous status if changed
                        if status_changed {
                            yield Ok((db_info, last_status.clone()));
                        } else if last_status.is_none() {
                            // First poll - always yield
                            yield Ok((db_info, None));
                        } else {
                            // Status unchanged - yield current state for monitoring
                            yield Ok((db_info, None));
                        }

                        last_status = current_status;
                    }
                    Err(e) => {
                        yield Err(e);
                        break;
                    }
                }

                sleep(poll_interval).await;
            }
        })
    }
}
