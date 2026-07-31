//! Version-aware live compliance checks driven by the official-doc inventory.
//!
//! Normal test runs exercise the deterministic inventory, resolver, comparison,
//! and baseline machinery. The live test is ignored unless explicitly selected:
//!
//! ```text
//! REDIS_ENTERPRISE_EXPECTED_VERSION=8.2.0-25 \
//! REDIS_ENTERPRISE_IMAGE=redislabs/redis:8.2.0-25.12 \
//! REDIS_ENTERPRISE_COMPLIANCE_RECORD=true \
//! cargo test --test live_compliance -- --ignored --nocapture
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{SecondsFormat, Utc};
use redis_enterprise::{CreateUserRequest, EnterpriseClient, RestError, UpdateUserRequest};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

const INVENTORY_CSV: &str = include_str!("../docs/api-inventory.csv");
const BASELINE_JSON: &str = include_str!("fixtures/live_compliance_baseline.json");
const REPORT_SCHEMA_VERSION: u32 = 1;
const DISPOSABLE_USER_PREFIX: &str = "redis-enterprise-compliance-";

#[derive(Debug, Deserialize)]
struct InventoryRow {
    page: String,
    method: String,
    path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OperationSpec {
    method: String,
    path: String,
    source_pages: BTreeSet<String>,
}

impl OperationSpec {
    fn key(&self) -> String {
        format!("{} {}", self.method, self.path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ComplianceStatus {
    Pass,
    KnownDifference,
    VersionSpecific,
    Skipped,
    Unsupported,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModelStatus {
    Pass,
    DroppedFields,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ModelComparison {
    model: String,
    status: ModelStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dropped_paths: Vec<String>,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OperationReport {
    method: String,
    path: String,
    source_pages: Vec<String>,
    status: ComplianceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_code: Option<u16>,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<ModelComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReportSummary {
    total: usize,
    pass: usize,
    known_difference: usize,
    version_specific: usize,
    skipped: usize,
    unsupported: usize,
    fail: usize,
    model_dropped_fields: usize,
    model_failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ServerCapabilities {
    api_versions: Vec<String>,
    discovered_collections: Vec<String>,
    rbac: bool,
    active_active: bool,
    ldap_mappings: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComplianceReport {
    schema_version: u32,
    generated_at: String,
    server_version: String,
    version_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    capabilities: ServerCapabilities,
    writes_enabled: bool,
    summary: ReportSummary,
    operations: Vec<OperationReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BaselineEntry {
    status: ComplianceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_status: Option<ModelStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dropped_paths: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProfileBaseline {
    operations: BTreeMap<String, BaselineEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VersionBaseline {
    profiles: BTreeMap<String, ProfileBaseline>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComplianceBaseline {
    schema_version: u32,
    versions: BTreeMap<String, VersionBaseline>,
}

#[derive(Debug, Default)]
struct Resources {
    action_uid: Option<String>,
    bdb_uid: Option<String>,
    crdb_guid: Option<String>,
    crdb_task_id: Option<String>,
    ldap_mapping_uid: Option<String>,
    module_uid: Option<String>,
    node_uid: Option<String>,
    proxy_uid: Option<String>,
    redis_acl_uid: Option<String>,
    role_uid: Option<String>,
    shard_uid: Option<String>,
    suffix_name: Option<String>,
    user_uid: Option<String>,
}

impl Resources {
    fn resolve(&self, template: &str) -> Result<String, String> {
        let placeholders =
            Regex::new(r"\{([^}]+)\}|<([^>]+)>").expect("placeholder regex should compile");
        let mut missing = None;
        let resolved = placeholders
            .replace_all(template, |captures: &regex::Captures<'_>| {
                let name = captures
                    .get(1)
                    .or_else(|| captures.get(2))
                    .expect("placeholder capture")
                    .as_str();
                match self.value_for(template, name) {
                    Some(value) => value.to_string(),
                    None => {
                        missing = Some(name.to_string());
                        captures
                            .get(0)
                            .expect("whole placeholder capture")
                            .as_str()
                            .to_string()
                    }
                }
            })
            .into_owned();

        match missing {
            Some(name) => Err(format!("no disposable or discovered value for `{name}`")),
            None => Ok(resolved),
        }
    }

    fn value_for<'a>(&'a self, template: &str, name: &str) -> Option<&'a str> {
        match name {
            "bdb_uid" => self.bdb_uid.as_deref(),
            "crdb_guid" => self.crdb_guid.as_deref(),
            "node_uid" => self.node_uid.as_deref(),
            "task_id" => self.crdb_task_id.as_deref(),
            "name" if template.starts_with("/v1/suffix/") => self.suffix_name.as_deref(),
            "uid" if template.starts_with("/v1/actions/") => self.action_uid.as_deref(),
            "uid" if template.starts_with("/v2/actions/") => self.action_uid.as_deref(),
            "uid" if template.starts_with("/v1/bdbs/") => self.bdb_uid.as_deref(),
            "uid" if template.starts_with("/v1/ldap_mappings/") => self.ldap_mapping_uid.as_deref(),
            "uid" if template.starts_with("/v1/modules/") => self.module_uid.as_deref(),
            "uid" if template.starts_with("/v1/nodes/") => self.node_uid.as_deref(),
            "uid" if template.starts_with("/v1/proxies/") => self.proxy_uid.as_deref(),
            "uid" if template.starts_with("/v1/redis_acls/") => self.redis_acl_uid.as_deref(),
            "uid" if template.starts_with("/v1/roles/") => self.role_uid.as_deref(),
            "uid" if template.starts_with("/v1/shards/") => self.shard_uid.as_deref(),
            "uid" if template.starts_with("/v1/users/") => self.user_uid.as_deref(),
            _ => None,
        }
    }
}

fn load_inventory() -> Vec<OperationSpec> {
    let mut reader = csv::Reader::from_reader(INVENTORY_CSV.as_bytes());
    let mut operations: BTreeMap<String, OperationSpec> = BTreeMap::new();

    for row in reader.deserialize::<InventoryRow>() {
        let row = row.expect("checked-in inventory should be valid CSV");
        let method = row.method.trim().to_ascii_uppercase();
        let path = row.path.trim().to_string();
        let key = format!("{method} {path}");
        operations
            .entry(key)
            .and_modify(|operation| {
                operation.source_pages.insert(row.page.clone());
            })
            .or_insert_with(|| OperationSpec {
                method,
                path,
                source_pages: BTreeSet::from([row.page]),
            });
    }

    operations.into_values().collect()
}

fn load_baseline() -> ComplianceBaseline {
    let baseline: ComplianceBaseline =
        serde_json::from_str(BASELINE_JSON).expect("checked-in compliance baseline should be JSON");
    assert_eq!(baseline.schema_version, REPORT_SCHEMA_VERSION);
    baseline
}

fn records(value: &Value) -> Option<&Vec<Value>> {
    if let Some(array) = value.as_array() {
        return Some(array);
    }
    value.as_object()?.values().find_map(Value::as_array)
}

fn first_field(value: &Value, names: &[&str]) -> Option<String> {
    let item = records(value)?.first()?;
    names.iter().find_map(|name| {
        let field = item.get(*name)?;
        field
            .as_str()
            .map(ToOwned::to_owned)
            .or_else(|| field.as_u64().map(|number| number.to_string()))
    })
}

fn capabilities(snapshots: &BTreeMap<String, Value>) -> ServerCapabilities {
    let mut api_versions = BTreeSet::new();
    for path in snapshots.keys() {
        if let Some(version) = path.split('/').nth(1) {
            api_versions.insert(version.to_string());
        }
    }

    ServerCapabilities {
        api_versions: api_versions.into_iter().collect(),
        discovered_collections: snapshots.keys().cloned().collect(),
        rbac: snapshots.contains_key("/v1/roles") && snapshots.contains_key("/v1/users"),
        active_active: snapshots.contains_key("/v1/crdbs"),
        ldap_mappings: snapshots.contains_key("/v1/ldap_mappings"),
    }
}

async fn discover(
    client: &EnterpriseClient,
) -> (
    String,
    Resources,
    ServerCapabilities,
    BTreeMap<String, Value>,
) {
    let paths = [
        "/v1/actions",
        "/v1/bdbs",
        "/v1/crdb_tasks",
        "/v1/crdbs",
        "/v1/ldap_mappings",
        "/v1/modules",
        "/v1/nodes",
        "/v1/proxies",
        "/v1/redis_acls",
        "/v1/roles",
        "/v1/shards",
        "/v1/suffixes",
        "/v1/users",
        "/v2/actions",
    ];
    let mut snapshots = BTreeMap::new();
    for path in paths {
        if let Ok(value) = client.get_raw(path).await {
            snapshots.insert(path.to_string(), value);
        }
    }

    let nodes = snapshots.get("/v1/nodes");
    let server_version = nodes
        .and_then(|value| first_field(value, &["software_version"]))
        .unwrap_or_else(|| "unknown".to_string());

    let resources = Resources {
        action_uid: snapshots
            .get("/v1/actions")
            .or_else(|| snapshots.get("/v2/actions"))
            .and_then(|value| first_field(value, &["uid", "action_uid", "id"])),
        bdb_uid: snapshots
            .get("/v1/bdbs")
            .and_then(|value| first_field(value, &["uid"])),
        crdb_guid: snapshots
            .get("/v1/crdbs")
            .and_then(|value| first_field(value, &["guid"])),
        crdb_task_id: snapshots
            .get("/v1/crdb_tasks")
            .and_then(|value| first_field(value, &["id", "task_id"])),
        ldap_mapping_uid: snapshots
            .get("/v1/ldap_mappings")
            .and_then(|value| first_field(value, &["uid"])),
        module_uid: snapshots
            .get("/v1/modules")
            .and_then(|value| first_field(value, &["uid", "id"])),
        node_uid: nodes.and_then(|value| first_field(value, &["uid", "id"])),
        proxy_uid: snapshots
            .get("/v1/proxies")
            .and_then(|value| first_field(value, &["uid", "id"])),
        redis_acl_uid: snapshots
            .get("/v1/redis_acls")
            .and_then(|value| first_field(value, &["uid"])),
        role_uid: snapshots
            .get("/v1/roles")
            .and_then(|value| first_field(value, &["uid"])),
        shard_uid: snapshots
            .get("/v1/shards")
            .and_then(|value| first_field(value, &["uid", "id"])),
        suffix_name: snapshots
            .get("/v1/suffixes")
            .and_then(|value| first_field(value, &["name", "suffix"])),
        user_uid: snapshots
            .get("/v1/users")
            .and_then(|value| first_field(value, &["uid"])),
    };

    let capabilities = capabilities(&snapshots);
    (server_version, resources, capabilities, snapshots)
}

fn version_family(version: &str) -> String {
    version
        .split(['.', '-'])
        .take(2)
        .collect::<Vec<_>>()
        .join(".")
}

fn bool_env(name: &str) -> bool {
    env::var(name)
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn is_expensive_read(path: &str) -> bool {
    path.contains("debuginfo")
}

fn status_code(error: &RestError) -> Option<u16> {
    match error {
        RestError::ApiError { code, .. } => Some(*code),
        RestError::Unauthorized | RestError::AuthenticationFailed => Some(401),
        RestError::NotFound => Some(404),
        RestError::Conflict(_) | RestError::AlreadyExists => Some(409),
        RestError::RateLimited { .. } => Some(429),
        RestError::ServerError(_) => Some(500),
        RestError::ClusterBusy => Some(503),
        _ => None,
    }
}

fn sanitized_error(error: &RestError) -> String {
    match error {
        RestError::ApiError { code, .. } => format!("server returned HTTP {code}"),
        RestError::Unauthorized | RestError::AuthenticationFailed => {
            "server rejected compliance credentials".to_string()
        }
        RestError::NotFound => "server returned HTTP 404".to_string(),
        RestError::Conflict(_) | RestError::AlreadyExists => "server returned HTTP 409".to_string(),
        RestError::RateLimited { .. } => "server returned HTTP 429".to_string(),
        RestError::ServerError(_) => "server returned a 5xx response".to_string(),
        RestError::ClusterBusy => "server returned HTTP 503".to_string(),
        RestError::ParseError(_) => "response was not compatible JSON".to_string(),
        RestError::SerializationError(_) => "response serialization failed".to_string(),
        RestError::Timeout => "request timed out".to_string(),
        _ => "request failed before a usable HTTP response".to_string(),
    }
}

fn classify_error(error: &RestError) -> ComplianceStatus {
    match status_code(error) {
        Some(400 | 405 | 406 | 422) => ComplianceStatus::KnownDifference,
        Some(404) => ComplianceStatus::VersionSpecific,
        _ => ComplianceStatus::Fail,
    }
}

async fn probe_get(client: &EnterpriseClient, path: &str) -> Result<Value, RestError> {
    if path == "/v1/cluster/sso/saml/metadata/sp" {
        return client.get_text(path).await.map(Value::String);
    }
    client.get_raw(path).await
}

fn operation_report(
    spec: &OperationSpec,
    status: ComplianceStatus,
    status_code: Option<u16>,
    reason: impl Into<String>,
) -> OperationReport {
    OperationReport {
        method: spec.method.clone(),
        path: spec.path.clone(),
        source_pages: spec.source_pages.iter().cloned().collect(),
        status,
        status_code,
        reason: reason.into(),
        model: None,
    }
}

fn escaped_path_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn collect_dropped_paths(raw: &Value, typed: &Value, path: &str, output: &mut Vec<String>) {
    if output.len() >= 200 {
        return;
    }
    match (raw, typed) {
        (Value::Object(raw), Value::Object(typed)) => {
            for (key, raw_value) in raw {
                let child_path = format!("{}/{}", path, escaped_path_segment(key));
                match typed.get(key) {
                    Some(typed_value) => {
                        collect_dropped_paths(raw_value, typed_value, &child_path, output)
                    }
                    None => output.push(child_path),
                }
                if output.len() >= 200 {
                    break;
                }
            }
        }
        (Value::Array(raw), Value::Array(typed)) => {
            for (raw_value, typed_value) in raw.iter().zip(typed) {
                collect_dropped_paths(raw_value, typed_value, &format!("{path}/*"), output);
                if output.len() >= 200 {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn compare_model(
    model: &str,
    raw: Result<Value, RestError>,
    typed: Result<Value, RestError>,
) -> ModelComparison {
    match (raw, typed) {
        (Ok(raw), Ok(typed)) => {
            let mut dropped_paths = Vec::new();
            collect_dropped_paths(&raw, &typed, "", &mut dropped_paths);
            dropped_paths.sort();
            dropped_paths.dedup();
            let status = if dropped_paths.is_empty() {
                ModelStatus::Pass
            } else {
                ModelStatus::DroppedFields
            };
            let reason = if dropped_paths.is_empty() {
                "raw JSON round-tripped through the typed model without dropped fields".to_string()
            } else {
                format!(
                    "typed round-trip dropped {} JSON paths (capped at 200)",
                    dropped_paths.len()
                )
            };
            ModelComparison {
                model: model.to_string(),
                status,
                dropped_paths,
                reason,
            }
        }
        (Err(error), _) => ModelComparison {
            model: model.to_string(),
            status: ModelStatus::Skipped,
            dropped_paths: Vec::new(),
            reason: format!("raw probe unavailable: {}", sanitized_error(&error)),
        },
        (_, Err(error)) => ModelComparison {
            model: model.to_string(),
            status: ModelStatus::Failed,
            dropped_paths: Vec::new(),
            reason: format!("typed deserialization failed: {}", sanitized_error(&error)),
        },
    }
}

fn to_json<T: Serialize>(result: Result<T, RestError>) -> Result<Value, RestError> {
    result.and_then(|value| serde_json::to_value(value).map_err(Into::into))
}

async fn attach_model_comparisons(
    client: &EnterpriseClient,
    reports: &mut BTreeMap<String, OperationReport>,
) {
    macro_rules! attach {
        ($path:literal, $model:literal, $typed:expr) => {{
            let raw = client.get_raw($path).await;
            let typed = to_json($typed.await);
            if let Some(report) = reports.get_mut(concat!("GET ", $path)) {
                report.model = Some(compare_model($model, raw, typed));
            }
        }};
    }

    attach!("/v1/cluster", "ClusterInfo", client.cluster().info());
    attach!("/v1/nodes", "Vec<Node>", client.nodes().list());
    attach!("/v1/bdbs", "Vec<DatabaseInfo>", client.databases().list());
    attach!("/v1/users", "Vec<User>", client.users().list());
    attach!("/v1/roles", "Vec<Role>", client.roles().list());
    attach!("/v1/modules", "Vec<Module>", client.modules().list());
    attach!("/v1/shards", "Vec<Shard>", client.shards().list());
    attach!("/v1/proxies", "Vec<Proxy>", client.proxies().list());
    attach!("/v1/license", "License", client.license().get());
    attach!(
        "/v1/metrics_config",
        "MetricsConfig",
        client.metrics_config().get()
    );

    let raw = client.get_raw("/v1/crdbs").await;
    let typed = client
        .crdb()
        .list()
        .await
        .and_then(|crdbs| serde_json::to_value(json!({ "crdbs": crdbs })).map_err(Into::into));
    if let Some(report) = reports.get_mut("GET /v1/crdbs") {
        report.model = Some(compare_model("Vec<Crdb>", raw, typed));
    }
}

async fn run_user_lifecycle(
    client: &EnterpriseClient,
    reports: &mut BTreeMap<String, OperationReport>,
) {
    match cleanup_disposable_users(client).await {
        Ok(_) => {}
        Err(error) => {
            mark_user_lifecycle_failed(
                reports,
                format!(
                    "could not inspect stale disposable users: {}",
                    sanitized_error(&error)
                ),
            );
            return;
        }
    }

    let role_uid = match client.roles().list().await {
        Ok(roles) => roles
            .into_iter()
            .find(|role| role.management.as_deref() == Some("admin") || role.name == "Admin")
            .map(|role| role.uid),
        Err(error) => {
            mark_user_lifecycle_failed(
                reports,
                format!(
                    "could not inspect roles for a disposable user: {}",
                    sanitized_error(&error)
                ),
            );
            return;
        }
    };
    let Some(role_uid) = role_uid else {
        for key in [
            "POST /v1/users",
            "GET /v1/users/{uid}",
            "PUT /v1/users/{uid}",
            "DELETE /v1/users/{uid}",
        ] {
            if let Some(report) = reports.get_mut(key) {
                report.status = ComplianceStatus::Skipped;
                report.status_code = None;
                report.reason = "no admin-capable role was available for a disposable user".into();
            }
        }
        return;
    };

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow Unix epoch")
        .as_nanos();
    let request = CreateUserRequest::builder()
        .email(format!("{DISPOSABLE_USER_PREFIX}{unique}@example.invalid"))
        .password("RedisCompliance123!")
        .role_uids(vec![role_uid])
        .name("Redis Enterprise Compliance")
        .email_alerts(false)
        .auth_method("regular")
        .build();

    let created = match client.users().create(request).await {
        Ok(user) => {
            if let Some(report) = reports.get_mut("POST /v1/users") {
                report.status = ComplianceStatus::Pass;
                report.status_code = None;
                report.reason =
                    "created a disposable user; the typed handler does not expose its 2xx status"
                        .into();
            }
            user
        }
        Err(error) => {
            let cleanup_error = cleanup_disposable_users(client).await.err();
            if let Some(report) = reports.get_mut("POST /v1/users") {
                report.status = ComplianceStatus::Fail;
                report.status_code = status_code(&error);
                report.reason = lifecycle_failure_reason("create", &error, cleanup_error.as_ref());
            }
            return;
        }
    };

    let uid = created.uid;
    match client.users().get(uid).await {
        Ok(_) => {
            if let Some(report) = reports.get_mut("GET /v1/users/{uid}") {
                report.status = ComplianceStatus::Pass;
                report.status_code = Some(200);
                report.reason = "read the disposable user".into();
            }
        }
        Err(error) => {
            if let Some(report) = reports.get_mut("GET /v1/users/{uid}") {
                report.status = ComplianceStatus::Fail;
                report.status_code = status_code(&error);
                report.reason = format!("disposable read failed: {}", sanitized_error(&error));
            }
        }
    }

    let update = UpdateUserRequest::builder()
        .name("Redis Enterprise Compliance Updated")
        .email_alerts(false)
        .build();
    match client.users().update(uid, update).await {
        Ok(_) => {
            if let Some(report) = reports.get_mut("PUT /v1/users/{uid}") {
                report.status = ComplianceStatus::Pass;
                report.status_code = None;
                report.reason =
                    "updated the disposable user; the typed handler does not expose its 2xx status"
                        .into();
            }
        }
        Err(error) => {
            if let Some(report) = reports.get_mut("PUT /v1/users/{uid}") {
                report.status = ComplianceStatus::Fail;
                report.status_code = status_code(&error);
                report.reason = format!("disposable update failed: {}", sanitized_error(&error));
            }
        }
    }

    match client.users().delete(uid).await {
        Ok(()) => {
            if let Some(report) = reports.get_mut("DELETE /v1/users/{uid}") {
                report.status = ComplianceStatus::Pass;
                report.status_code = None;
                report.reason =
                    "deleted the disposable user; the typed handler does not expose its 2xx status"
                        .into();
            }
        }
        Err(error) => {
            let cleanup_error = cleanup_disposable_users(client).await.err();
            if let Some(report) = reports.get_mut("DELETE /v1/users/{uid}") {
                report.status = ComplianceStatus::Fail;
                report.status_code = status_code(&error);
                report.reason = lifecycle_failure_reason("delete", &error, cleanup_error.as_ref());
            }
            return;
        }
    }

    for _ in 0..10 {
        match client.users().get(uid).await {
            Err(error) if error.is_not_found() => return,
            Ok(_) => sleep(Duration::from_millis(200)).await,
            Err(_) => return,
        }
    }
    if let Some(report) = reports.get_mut("DELETE /v1/users/{uid}") {
        report.status = ComplianceStatus::Fail;
        report.reason = "disposable user remained visible after delete".into();
    }
}

async fn cleanup_disposable_users(client: &EnterpriseClient) -> Result<usize, RestError> {
    let users = client.users().list().await?;
    let stale_users = users
        .into_iter()
        .filter(|user| user.email.starts_with(DISPOSABLE_USER_PREFIX))
        .collect::<Vec<_>>();
    let count = stale_users.len();
    for user in stale_users {
        client.users().delete(user.uid).await?;
    }
    Ok(count)
}

fn lifecycle_failure_reason(
    operation: &str,
    error: &RestError,
    cleanup_error: Option<&RestError>,
) -> String {
    let mut reason = format!("disposable {operation} failed: {}", sanitized_error(error));
    if let Some(cleanup_error) = cleanup_error {
        reason.push_str(&format!(
            "; follow-up cleanup also failed: {}",
            sanitized_error(cleanup_error)
        ));
    }
    reason
}

fn mark_user_lifecycle_failed(reports: &mut BTreeMap<String, OperationReport>, reason: String) {
    for key in [
        "POST /v1/users",
        "GET /v1/users/{uid}",
        "PUT /v1/users/{uid}",
        "DELETE /v1/users/{uid}",
    ] {
        if let Some(report) = reports.get_mut(key) {
            report.status = ComplianceStatus::Fail;
            report.status_code = None;
            report.reason.clone_from(&reason);
        }
    }
}

fn summarize(operations: &[OperationReport]) -> ReportSummary {
    let count = |status| {
        operations
            .iter()
            .filter(|item| item.status == status)
            .count()
    };
    ReportSummary {
        total: operations.len(),
        pass: count(ComplianceStatus::Pass),
        known_difference: count(ComplianceStatus::KnownDifference),
        version_specific: count(ComplianceStatus::VersionSpecific),
        skipped: count(ComplianceStatus::Skipped),
        unsupported: count(ComplianceStatus::Unsupported),
        fail: count(ComplianceStatus::Fail),
        model_dropped_fields: operations
            .iter()
            .filter(|item| {
                item.model.as_ref().map(|model| model.status) == Some(ModelStatus::DroppedFields)
            })
            .count(),
        model_failed: operations
            .iter()
            .filter(|item| {
                item.model.as_ref().map(|model| model.status) == Some(ModelStatus::Failed)
            })
            .count(),
    }
}

fn baseline_entry(report: &OperationReport) -> BaselineEntry {
    BaselineEntry {
        status: report.status,
        status_code: report.status_code,
        model_status: report.model.as_ref().map(|model| model.status),
        dropped_paths: report
            .model
            .as_ref()
            .map(|model| model.dropped_paths.clone())
            .unwrap_or_default(),
    }
}

fn baseline_candidate(report: &ComplianceReport) -> ComplianceBaseline {
    let operations = report
        .operations
        .iter()
        .map(|operation| {
            (
                format!("{} {}", operation.method, operation.path),
                baseline_entry(operation),
            )
        })
        .collect();
    ComplianceBaseline {
        schema_version: REPORT_SCHEMA_VERSION,
        versions: BTreeMap::from([(
            report.version_family.clone(),
            VersionBaseline {
                profiles: BTreeMap::from([(
                    baseline_profile(report).to_string(),
                    ProfileBaseline { operations },
                )]),
            },
        )]),
    }
}

fn baseline_profile(report: &ComplianceReport) -> &'static str {
    if report.writes_enabled {
        "writes"
    } else {
        "safe"
    }
}

fn compare_baseline(
    report: &ComplianceReport,
    baseline: &ComplianceBaseline,
) -> Result<(), Vec<String>> {
    let Some(version) = baseline.versions.get(&report.version_family) else {
        return Err(vec![format!(
            "no reviewed baseline for Redis Software {}",
            report.version_family
        )]);
    };
    let profile = baseline_profile(report);
    let Some(expected) = version.profiles.get(profile) else {
        return Err(vec![format!(
            "no reviewed {profile} baseline for Redis Software {}",
            report.version_family
        )]);
    };
    let actual: BTreeMap<String, BaselineEntry> = report
        .operations
        .iter()
        .map(|operation| {
            (
                format!("{} {}", operation.method, operation.path),
                baseline_entry(operation),
            )
        })
        .collect();
    let mut differences = Vec::new();
    for (key, expected_entry) in &expected.operations {
        match actual.get(key) {
            Some(actual_entry) if actual_entry == expected_entry => {}
            Some(actual_entry) => differences.push(format!(
                "{key}: expected {expected_entry:?}, observed {actual_entry:?}"
            )),
            None => differences.push(format!("{key}: missing from live report")),
        }
    }
    for key in actual.keys() {
        if !expected.operations.contains_key(key) {
            differences.push(format!("{key}: missing from reviewed baseline"));
        }
    }
    if differences.is_empty() {
        Ok(())
    } else {
        Err(differences)
    }
}

fn report_path(family: &str) -> PathBuf {
    env::var("REDIS_ENTERPRISE_COMPLIANCE_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("target/enterprise-compliance-{family}.json")))
}

fn candidate_path(family: &str) -> PathBuf {
    env::var("REDIS_ENTERPRISE_COMPLIANCE_BASELINE_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(format!(
                "target/enterprise-compliance-baseline-{family}.json"
            ))
        })
}

fn write_json(path: &Path, value: &impl Serialize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("report directory should be creatable");
    }
    let contents = serde_json::to_string_pretty(value).expect("report should serialize");
    fs::write(path, format!("{contents}\n")).expect("report should be writable");
}

#[test]
fn inventory_is_deduplicated_without_losing_source_pages() {
    let operations = load_inventory();
    assert_eq!(operations.len(), 203);
    assert!(operations.iter().all(|operation| {
        matches!(
            operation.method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
        )
    }));
    let cluster = operations
        .iter()
        .find(|operation| operation.key() == "GET /v1/cluster")
        .expect("cluster route should exist");
    assert!(!cluster.source_pages.is_empty());
}

#[test]
fn checked_in_baseline_is_well_formed() {
    let baseline = load_baseline();
    assert_eq!(baseline.schema_version, REPORT_SCHEMA_VERSION);
    for version in baseline.versions.values() {
        assert!(
            version
                .profiles
                .keys()
                .all(|profile| profile == "safe" || profile == "writes")
        );
    }
}

#[test]
fn resource_resolver_is_context_aware_and_refuses_unknown_values() {
    let resources = Resources {
        bdb_uid: Some("7".into()),
        node_uid: Some("3".into()),
        user_uid: Some("11".into()),
        ..Resources::default()
    };
    assert_eq!(resources.resolve("/v1/bdbs/{uid}").unwrap(), "/v1/bdbs/7");
    assert_eq!(resources.resolve("/v1/nodes/{uid}").unwrap(), "/v1/nodes/3");
    assert_eq!(
        resources.resolve("/v1/users/<uid>").unwrap(),
        "/v1/users/11"
    );
    assert!(resources.resolve("/v1/cluster/actions/{action}").is_err());
}

#[test]
fn dropped_path_comparison_records_names_not_values() {
    let raw = json!([
        {"uid": 1, "nested": {"kept": true, "dropped": "secret-one"}},
        {"uid": 2, "nested": {"kept": true, "dropped": "secret-two"}}
    ]);
    let typed = json!([
        {"uid": 1, "nested": {"kept": true}},
        {"uid": 2, "nested": {"kept": true}}
    ]);
    let comparison = compare_model("Example", Ok(raw), Ok(typed));
    assert_eq!(comparison.status, ModelStatus::DroppedFields);
    assert_eq!(comparison.dropped_paths, ["/*/nested/dropped"]);
    assert!(!comparison.reason.contains("secret-one"));
    assert!(!comparison.reason.contains("secret-two"));
}

#[test]
fn lifecycle_failure_reasons_do_not_expose_server_messages() {
    let operation_error = RestError::ApiError {
        code: 422,
        message: "request contained RedisCompliance123!".into(),
    };
    let cleanup_error = RestError::ServerError("user@example.invalid".into());
    let reason = lifecycle_failure_reason("create", &operation_error, Some(&cleanup_error));
    assert!(reason.contains("HTTP 422"));
    assert!(reason.contains("5xx"));
    assert!(!reason.contains("RedisCompliance123!"));
    assert!(!reason.contains("user@example.invalid"));
}

#[test]
fn baseline_comparison_detects_operation_drift() {
    let operation = OperationReport {
        method: "GET".into(),
        path: "/v1/cluster".into(),
        source_pages: vec!["cluster".into()],
        status: ComplianceStatus::Pass,
        status_code: Some(200),
        reason: "ok".into(),
        model: None,
    };
    let report = ComplianceReport {
        schema_version: REPORT_SCHEMA_VERSION,
        generated_at: "2026-07-31T00:00:00Z".into(),
        server_version: "8.2.0-25".into(),
        version_family: "8.2".into(),
        image: None,
        capabilities: ServerCapabilities {
            api_versions: vec!["v1".into()],
            discovered_collections: vec!["/v1/nodes".into()],
            rbac: true,
            active_active: false,
            ldap_mappings: false,
        },
        writes_enabled: false,
        summary: summarize(std::slice::from_ref(&operation)),
        operations: vec![operation],
    };
    let baseline = baseline_candidate(&report);
    assert!(compare_baseline(&report, &baseline).is_ok());

    let mut drifted = report;
    drifted.operations[0].status = ComplianceStatus::VersionSpecific;
    drifted.operations[0].status_code = Some(404);
    assert!(compare_baseline(&drifted, &baseline).is_err());
}

#[tokio::test]
#[ignore = "requires a disposable Redis Enterprise cluster configured via REDIS_ENTERPRISE_* env vars"]
async fn live_inventory_compliance() {
    let client = EnterpriseClient::from_env()
        .unwrap_or_else(|error| panic!("failed to build live Enterprise client: {error}"));
    let expected_version = env::var("REDIS_ENTERPRISE_EXPECTED_VERSION")
        .expect("REDIS_ENTERPRISE_EXPECTED_VERSION must identify the pinned server build");
    let writes_enabled = bool_env("REDIS_ENTERPRISE_LIVE_WRITES");
    let record = bool_env("REDIS_ENTERPRISE_COMPLIANCE_RECORD");
    let operations = load_inventory();
    let (server_version, resources, capabilities, snapshots) = discover(&client).await;

    assert!(
        server_version.starts_with(&expected_version),
        "expected Redis Software {expected_version}, discovered {server_version}"
    );
    let family = version_family(&server_version);
    let mut reports = BTreeMap::new();

    for spec in operations {
        let key = spec.key();
        let report = if spec.method != "GET" {
            operation_report(
                &spec,
                ComplianceStatus::Skipped,
                None,
                if writes_enabled {
                    "write operation has no disposable lifecycle implementation"
                } else {
                    "write operation requires REDIS_ENTERPRISE_LIVE_WRITES=true"
                },
            )
        } else if is_expensive_read(&spec.path) {
            operation_report(
                &spec,
                ComplianceStatus::Skipped,
                None,
                "support-package and debug-info downloads are excluded from safe reads",
            )
        } else {
            match resources.resolve(&spec.path) {
                Err(reason) => operation_report(&spec, ComplianceStatus::Skipped, None, reason),
                Ok(resolved) => {
                    let result = match snapshots.get(&resolved) {
                        Some(value) => Ok(value.clone()),
                        None => probe_get(&client, &resolved).await,
                    };
                    match result {
                        Ok(_) => operation_report(
                            &spec,
                            ComplianceStatus::Pass,
                            Some(200),
                            "safe read returned a compatible response",
                        ),
                        Err(error) => operation_report(
                            &spec,
                            classify_error(&error),
                            status_code(&error),
                            sanitized_error(&error),
                        ),
                    }
                }
            }
        };
        reports.insert(key, report);
    }

    attach_model_comparisons(&client, &mut reports).await;
    if writes_enabled {
        run_user_lifecycle(&client, &mut reports).await;
    }

    let operations: Vec<_> = reports.into_values().collect();
    let report = ComplianceReport {
        schema_version: REPORT_SCHEMA_VERSION,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        server_version,
        version_family: family.clone(),
        image: env::var("REDIS_ENTERPRISE_IMAGE").ok(),
        capabilities,
        writes_enabled,
        summary: summarize(&operations),
        operations,
    };
    let report_path = report_path(&family);
    write_json(&report_path, &report);
    println!(
        "wrote sanitized compliance report to {}",
        report_path.display()
    );

    if record {
        let candidate = baseline_candidate(&report);
        let candidate_path = candidate_path(&family);
        write_json(&candidate_path, &candidate);
        println!(
            "wrote baseline candidate for review to {}",
            candidate_path.display()
        );
        return;
    }

    if let Err(differences) = compare_baseline(&report, &load_baseline()) {
        panic!("live compliance drift:\n{}", differences.join("\n"));
    }
}
