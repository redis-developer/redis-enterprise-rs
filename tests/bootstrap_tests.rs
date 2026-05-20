//! Bootstrap endpoint tests for Redis Enterprise

use redis_enterprise::{
    BootstrapConfig, BootstrapHandler, ClusterBootstrap, CredentialsBootstrap, EnterpriseClient,
    NodeBootstrap, NodePaths,
};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Test helper functions
fn success_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

fn created_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(201).set_body_json(body)
}

fn no_content_response() -> ResponseTemplate {
    ResponseTemplate::new(204)
}

// Produces the real wire shape: { "bootstrap_status": { "state", "start_time", "end_time" }, "local_node_info": {...} }.
// The old `(status, progress, message)` shape collapsed all three into the inner object; only
// `state` is documented in the REST API (the other two are kept here for backward-compatible
// test signatures but discarded — they were never on the wire).
fn bootstrap_status_response(
    state: &str,
    _progress: Option<f32>,
    _message: Option<&str>,
) -> serde_json::Value {
    json!({
        "bootstrap_status": {
            "state": state,
            "start_time": "2026-05-20T15:00:00Z",
            "end_time": null
        },
        "local_node_info": {
            "architecture": "x86_64",
            "os_family": "ubuntu"
        }
    })
}

fn cluster_bootstrap_config() -> BootstrapConfig {
    BootstrapConfig {
        action: "cluster_create".to_string(),
        cluster: Some(ClusterBootstrap {
            name: "test-cluster".to_string(),
            dns_suffixes: Some(vec!["cluster.local".to_string()]),
            rack_aware: Some(false),
        }),
        node: Some(NodeBootstrap {
            paths: Some(NodePaths {
                persistent_path: Some("/opt/redislabs/persist".to_string()),
                ephemeral_path: Some("/opt/redislabs/tmp".to_string()),
            }),
        }),
        credentials: Some(CredentialsBootstrap {
            username: "admin".to_string(),
            password: "secure123".to_string(),
        }),
    }
}

fn join_node_config() -> BootstrapConfig {
    BootstrapConfig {
        action: "join_cluster".to_string(),
        cluster: None,
        node: Some(NodeBootstrap {
            paths: Some(NodePaths {
                persistent_path: Some("/opt/redislabs/persist".to_string()),
                ephemeral_path: Some("/opt/redislabs/tmp".to_string()),
            }),
        }),
        credentials: Some(CredentialsBootstrap {
            username: "admin".to_string(),
            password: "secure123".to_string(),
        }),
    }
}

#[tokio::test]
async fn test_bootstrap_create_cluster() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(created_response(bootstrap_status_response(
            "in_progress",
            Some(10.0),
            Some("Initializing cluster"),
        )))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let config = cluster_bootstrap_config();
    let result = handler.create(config).await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.bootstrap_status.state, "in_progress");
}

#[tokio::test]
async fn test_bootstrap_status_in_progress() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(bootstrap_status_response(
            "in_progress",
            Some(75.5),
            Some("Configuring cluster nodes"),
        )))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let result = handler.status().await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.bootstrap_status.state, "in_progress");
}

#[tokio::test]
async fn test_bootstrap_status_completed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(bootstrap_status_response(
            "completed",
            Some(100.0),
            Some("Cluster initialization completed successfully"),
        )))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let result = handler.status().await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.bootstrap_status.state, "completed");
}

#[tokio::test]
async fn test_bootstrap_status_failed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(bootstrap_status_response(
            "failed",
            Some(45.0),
            Some("Failed to connect to cluster node"),
        )))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let result = handler.status().await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.bootstrap_status.state, "failed");
}

#[tokio::test]
async fn test_bootstrap_status_not_started() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Bootstrap not initiated"
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let result = handler.status().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_bootstrap_join_node() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bootstrap/join"))
        .and(basic_auth("admin", "password"))
        .respond_with(created_response(bootstrap_status_response(
            "in_progress",
            Some(5.0),
            Some("Joining node to cluster"),
        )))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let config = join_node_config();
    let result = handler.join(config).await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.bootstrap_status.state, "in_progress");
}

#[tokio::test]
async fn test_bootstrap_reset() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(no_content_response())
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let result = handler.reset().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_bootstrap_create_minimal_config() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(created_response(bootstrap_status_response(
            "in_progress",
            Some(0.0),
            Some("Starting bootstrap process"),
        )))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);

    // Minimal config - just action and credentials
    let config = BootstrapConfig {
        action: "minimal_cluster".to_string(),
        cluster: None,
        node: None,
        credentials: Some(CredentialsBootstrap {
            username: "admin".to_string(),
            password: "minimal123".to_string(),
        }),
    };

    let result = handler.create(config).await;

    assert!(result.is_ok());
    let status = result.unwrap();
    assert_eq!(status.bootstrap_status.state, "in_progress");
}

#[tokio::test]
async fn test_bootstrap_status_decodes_recorded_fixture() {
    // Regression guard for #62: the recorded fixture has the canonical
    // wrapper shape — bootstrap_status with state/start_time/end_time
    // plus a sizeable local_node_info object. The previous
    // (status/progress/message) struct decode-failed against this.
    let mock_server = MockServer::start().await;

    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/bootstrap_status.json"))
            .expect("fixture should be valid JSON");

    Mock::given(method("GET"))
        .and(path("/v1/bootstrap"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(fixture))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = BootstrapHandler::new(client);
    let response = handler.status().await.expect("fixture should decode");

    assert_eq!(response.bootstrap_status.state, "completed");
    assert_eq!(
        response.bootstrap_status.start_time.as_deref(),
        Some("2025-10-14T00:01:06Z")
    );
    assert_eq!(
        response.bootstrap_status.end_time.as_deref(),
        Some("2025-10-14T00:01:22Z")
    );

    let node = response
        .local_node_info
        .expect("fixture has local_node_info");
    assert_eq!(node["os_family"], "ubuntu");
}
