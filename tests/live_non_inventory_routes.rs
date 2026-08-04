//! Non-mutating live validation for SDK routes absent from the public-doc inventory.
//!
//! Redis Software responds to `OPTIONS` with `404` for an unknown path and an
//! `Allow` header for a registered path. That lets this audit validate read and
//! write route shapes without executing any mutation:
//!
//! ```text
//! REDIS_ENTERPRISE_EXPECTED_VERSION=8.2.0-25 \
//! cargo test --test live_non_inventory_routes -- --ignored --nocapture
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use reqwest::header::ALLOW;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const NON_INVENTORY_REGISTRY: &str = include_str!("fixtures/live_non_inventory_routes.json");

#[derive(Debug, Deserialize)]
struct RouteRegistry {
    schema_version: u32,
    verified_at: String,
    evidence: String,
    tested_versions: Vec<String>,
    routes: BTreeMap<String, RouteEvidence>,
}

#[derive(Debug, Clone, Deserialize)]
struct RouteEvidence {
    module: String,
    present_versions: Vec<String>,
    disposition: RouteDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RouteDisposition {
    VerifiedUndocumented,
    CompatibilityLegacy,
    Invalid,
}

#[derive(Debug, Clone)]
struct RouteSpec {
    method: String,
    path: String,
    evidence: RouteEvidence,
}

impl RouteSpec {
    fn key(&self) -> String {
        format!("{} {}", self.method, self.path)
    }

    fn probe_path(&self) -> String {
        self.path.replace("{}", "1")
    }
}

#[derive(Debug, Serialize)]
struct RouteResult {
    method: String,
    path: String,
    module: String,
    disposition: RouteDisposition,
    status_code: u16,
    registered_methods: Vec<String>,
    expected_registered: bool,
    observed_registered: bool,
    matches_registry: bool,
}

#[derive(Debug, Serialize)]
struct RouteReport {
    schema_version: u32,
    generated_at: String,
    server_version: String,
    expected_version: String,
    probe: &'static str,
    total: usize,
    matching: usize,
    drifted: usize,
    routes: Vec<RouteResult>,
}

fn load_registry() -> RouteRegistry {
    serde_json::from_str(NON_INVENTORY_REGISTRY)
        .expect("checked-in non-inventory registry should be valid JSON")
}

fn load_routes(registry: &RouteRegistry) -> Vec<RouteSpec> {
    registry
        .routes
        .iter()
        .map(|(key, evidence)| {
            let (method, path) = key
                .split_once(' ')
                .expect("non-inventory route should be METHOD /path");
            RouteSpec {
                method: method.to_string(),
                path: path.to_string(),
                evidence: evidence.clone(),
            }
        })
        .collect()
}

fn output_path(expected_version: &str) -> PathBuf {
    env::var_os("REDIS_ENTERPRISE_NON_INVENTORY_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let family = expected_version.replace(['.', '-'], "_");
            PathBuf::from(format!(
                "target/enterprise-non-inventory-routes-{family}.json"
            ))
        })
}

fn write_json(path: &Path, report: &RouteReport) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("report directory should be creatable");
    }
    let contents = serde_json::to_string_pretty(report).expect("report should serialize");
    fs::write(path, format!("{contents}\n")).expect("report should be writable");
}

fn registered_methods(response: &reqwest::Response) -> Vec<String> {
    let mut methods = response
        .headers()
        .get(ALLOW)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|method| !method.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    methods.sort();
    methods.dedup();
    methods
}

async fn server_version(client: &Client, base_url: &str, user: &str, password: &str) -> String {
    let response = client
        .get(format!("{base_url}/v1/nodes"))
        .basic_auth(user, Some(password))
        .send()
        .await
        .expect("server version request should complete")
        .error_for_status()
        .expect("server version request should succeed");
    let value: Value = response
        .json()
        .await
        .expect("node collection should be compatible JSON");
    value
        .as_array()
        .and_then(|nodes| nodes.first())
        .and_then(|node| node.get("software_version"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

#[test]
fn route_registry_is_complete_and_consistent() {
    let registry = load_registry();
    assert_eq!(registry.schema_version, 1);
    assert_eq!(registry.verified_at, "2026-08-04");
    assert_eq!(
        registry.evidence,
        "OPTIONS path registration and Allow header"
    );
    assert_eq!(
        registry.tested_versions,
        [
            "7.4.6-272",
            "7.8.6-286",
            "7.22.2-170",
            "8.0.20-68",
            "8.2.0-25",
        ]
    );

    let routes = load_routes(&registry);
    assert_eq!(
        routes.len(),
        81,
        "issue #105's registry shrinks as routes move to documented canonical paths"
    );

    let mut dispositions = BTreeMap::new();

    for route in routes {
        assert!(
            matches!(
                route.method.as_str(),
                "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
            ),
            "unsupported method in {}",
            route.key()
        );
        assert!(
            route.path.starts_with('/'),
            "invalid path in {}",
            route.key()
        );
        assert!(
            !route.path.contains('{') || route.path.contains("{}"),
            "route fixture should use normalized placeholders: {}",
            route.key()
        );
        assert!(
            !route.evidence.module.trim().is_empty(),
            "route module is required for {}",
            route.key()
        );
        assert!(
            route
                .evidence
                .present_versions
                .iter()
                .all(|version| registry.tested_versions.contains(version)),
            "present versions must be a subset of tested versions for {}",
            route.key()
        );

        let derived_disposition = if route
            .evidence
            .present_versions
            .iter()
            .any(|version| version.starts_with("8."))
        {
            RouteDisposition::VerifiedUndocumented
        } else if route.evidence.present_versions.is_empty() {
            RouteDisposition::Invalid
        } else {
            RouteDisposition::CompatibilityLegacy
        };
        assert_eq!(
            route.evidence.disposition,
            derived_disposition,
            "disposition must follow the reviewed version evidence for {}",
            route.key()
        );
        *dispositions
            .entry(route.evidence.disposition)
            .or_insert(0usize) += 1;
    }

    assert_eq!(
        dispositions.get(&RouteDisposition::VerifiedUndocumented),
        Some(&13)
    );
    assert_eq!(
        dispositions.get(&RouteDisposition::CompatibilityLegacy),
        Some(&9)
    );
    assert_eq!(dispositions.get(&RouteDisposition::Invalid), Some(&59));
}

#[tokio::test]
#[ignore = "requires a disposable Redis Software cluster"]
async fn live_non_inventory_options_audit() {
    let registry = load_registry();
    let base_url = env::var("REDIS_ENTERPRISE_URL")
        .unwrap_or_else(|_| "https://localhost:9443".to_string())
        .trim_end_matches('/')
        .to_string();
    let user =
        env::var("REDIS_ENTERPRISE_USER").unwrap_or_else(|_| "admin@redis.local".to_string());
    let password = env::var("REDIS_ENTERPRISE_PASSWORD")
        .expect("REDIS_ENTERPRISE_PASSWORD is required for live validation");
    let expected_version = env::var("REDIS_ENTERPRISE_EXPECTED_VERSION")
        .expect("REDIS_ENTERPRISE_EXPECTED_VERSION is required for live validation");
    assert!(
        registry.tested_versions.contains(&expected_version),
        "no reviewed non-inventory baseline for Redis Software {expected_version}"
    );
    let insecure = env::var("REDIS_ENTERPRISE_INSECURE")
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);

    let client = Client::builder()
        .danger_accept_invalid_certs(insecure)
        .build()
        .expect("live validation client should build");
    let actual_version = server_version(&client, &base_url, &user, &password).await;
    assert_eq!(
        actual_version, expected_version,
        "live evidence must use the exact expected Redis Software version"
    );

    let mut results = Vec::new();
    for route in load_routes(&registry) {
        let response = client
            .request(Method::OPTIONS, format!("{base_url}{}", route.probe_path()))
            .basic_auth(&user, Some(&password))
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!("OPTIONS {} failed before a response: {error}", route.key())
            });
        let status_code = response.status().as_u16();
        let methods = registered_methods(&response);
        let observed_registered =
            response.status().is_success() && methods.iter().any(|method| method == &route.method);
        let expected_registered = route.evidence.present_versions.contains(&actual_version);
        results.push(RouteResult {
            method: route.method,
            path: route.path,
            module: route.evidence.module,
            disposition: route.evidence.disposition,
            status_code,
            registered_methods: methods,
            expected_registered,
            observed_registered,
            matches_registry: expected_registered == observed_registered,
        });
    }

    let matching = results
        .iter()
        .filter(|route| route.matches_registry)
        .count();
    let report = RouteReport {
        schema_version: 1,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        server_version: actual_version,
        expected_version,
        probe: "OPTIONS path registration and Allow header",
        total: results.len(),
        matching,
        drifted: results.len() - matching,
        routes: results,
    };
    let path = output_path(&report.expected_version);
    write_json(&path, &report);
    println!(
        "wrote sanitized non-inventory route report to {}",
        path.display()
    );

    let drifted = report
        .routes
        .iter()
        .filter(|route| !route.matches_registry)
        .map(|route| {
            format!(
                "{} {}: expected registered={}, observed registered={}, HTTP {}, Allow={}",
                route.method,
                route.path,
                route.expected_registered,
                route.observed_registered,
                route.status_code,
                route.registered_methods.join("|")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        drifted.is_empty(),
        "non-inventory route behavior drifted from reviewed evidence:\n{}",
        drifted.join("\n")
    );
}
