//! Alerts endpoint tests for Redis Enterprise

use redis_enterprise::{AlertHandler, AlertSettings, EnterpriseClient};
use serde_json::json;
use wiremock::matchers::{basic_auth, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Test helper functions
fn success_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

fn error_response(code: u16, message: &str) -> ResponseTemplate {
    ResponseTemplate::new(code).set_body_json(json!({
        "error": message,
        "code": code
    }))
}

fn test_alert() -> serde_json::Value {
    json!({
        "uid": "alert-123",
        "name": "node_memory_high",
        "severity": "high",
        "state": "active",
        "entity_type": "node",
        "entity_name": "node-1",
        "entity_uid": "1",
        "threshold": {"value": 80, "unit": "percent"},
        "change_time": "2023-01-01T12:00:00Z",
        "change_value": 85.5,
        "description": "Node memory usage is high"
    })
}

fn test_database_alert() -> serde_json::Value {
    json!({
        "uid": "alert-456",
        "name": "database_latency",
        "severity": "medium",
        "state": "active",
        "entity_type": "database",
        "entity_name": "redis-db",
        "entity_uid": "1",
        "description": "Database latency is elevated"
    })
}

#[tokio::test]
async fn test_alerts_get_settings() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/cluster/alert_settings/node_memory_high"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({
            "enabled": true,
            "threshold": {"value": 80, "unit": "percent"}
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.get_settings("node_memory_high").await;

    assert!(result.is_ok());
    let settings = result.unwrap();
    assert!(settings.enabled);
}

#[tokio::test]
async fn test_alerts_list_by_database() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/alerts"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([test_database_alert()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.list_by_database(1).await;

    assert!(result.is_ok());
    let alerts = result.unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].uid, "alert-456");
    assert_eq!(alerts[0].entity_type.as_ref().unwrap(), "database");
}

#[tokio::test]
async fn test_alerts_list_by_database_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/alerts"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.list_by_database(1).await;

    assert!(result.is_ok());
    let alerts = result.unwrap();
    assert_eq!(alerts.len(), 0);
}

#[tokio::test]
async fn test_alerts_list_by_node() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/nodes/1/alerts"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([test_alert()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.list_by_node(1).await;

    assert!(result.is_ok());
    let alerts = result.unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].uid, "alert-123");
    assert_eq!(alerts[0].entity_type.as_ref().unwrap(), "node");
}

#[tokio::test]
async fn test_alerts_list_by_node_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/nodes/999/alerts"))
        .and(basic_auth("admin", "password"))
        .respond_with(error_response(404, "Node not found"))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.list_by_node(999).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_alerts_list_cluster_alerts() {
    // Regression guard for #62: cluster alerts are returned as a map
    // keyed by alert name, not as a bare array of Alert objects.
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/cluster/alerts"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({
            "cluster_multiple_nodes_down": {
                "enabled": true,
                "state": false,
                "severity": "WARNING",
                "change_time": "2026-05-20T00:00:00Z",
                "change_value": { "dead_nodes": [], "state": false }
            },
            "cluster_ram_overcommit": {
                "enabled": false,
                "state": false,
                "severity": "INFO"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let alerts = handler.list_cluster_alerts().await.unwrap();

    assert_eq!(alerts.len(), 2);
    let multi = alerts.get("cluster_multiple_nodes_down").unwrap();
    assert!(multi.enabled);
    assert!(!multi.state);
    assert_eq!(multi.severity, "WARNING");

    let overcommit = alerts.get("cluster_ram_overcommit").unwrap();
    assert!(!overcommit.enabled);
    assert!(overcommit.change_time.is_none());
}

#[tokio::test]
async fn test_alerts_update_settings() {
    let mock_server = MockServer::start().await;

    let settings = AlertSettings {
        enabled: true,
        threshold: Some(json!({"value": 85, "unit": "percent"})),
        email_recipients: Some(vec!["admin@example.com".to_string()]),
        webhook_url: Some("https://webhook.example.com/alerts".to_string()),
    };

    Mock::given(method("PUT"))
        .and(path("/v1/cluster/alert_settings/node_memory_high"))
        .and(basic_auth("admin", "password"))
        .and(body_json(&settings))
        .respond_with(success_response(json!({
            "enabled": true,
            "threshold": {"value": 85, "unit": "percent"},
            "email_recipients": ["admin@example.com"],
            "webhook_url": "https://webhook.example.com/alerts"
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.update_settings("node_memory_high", settings).await;

    assert!(result.is_ok());
    let updated_settings = result.unwrap();
    assert!(updated_settings.enabled);
    assert!(updated_settings.threshold.is_some());
    assert!(updated_settings.email_recipients.is_some());
    assert!(updated_settings.webhook_url.is_some());
}

#[tokio::test]
async fn test_alerts_update_settings_invalid() {
    let mock_server = MockServer::start().await;

    let settings = AlertSettings {
        enabled: true,
        threshold: Some(json!({"value": "invalid", "unit": "percent"})),
        email_recipients: None,
        webhook_url: None,
    };

    Mock::given(method("PUT"))
        .and(path("/v1/cluster/alert_settings/invalid_alert"))
        .and(basic_auth("admin", "password"))
        .and(body_json(&settings))
        .respond_with(error_response(400, "Invalid alert settings"))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.update_settings("invalid_alert", settings).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_alerts_get_settings_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/cluster/alert_settings/nonexistent_alert"))
        .and(basic_auth("admin", "password"))
        .respond_with(error_response(404, "Alert settings not found"))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = AlertHandler::new(client);
    let result = handler.get_settings("nonexistent_alert").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_cluster_alerts_decodes_recorded_fixture() {
    // Regression guard for #62: the recorded fixture has the real
    // wire shape — a top-level map keyed by alert name where each
    // entry exposes enabled/state/severity/change_time/change_value.
    let mock_server = MockServer::start().await;

    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/cluster_alerts.json"))
            .expect("fixture should be valid JSON");

    Mock::given(method("GET"))
        .and(path("/v1/cluster/alerts"))
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

    let handler = AlertHandler::new(client);
    let alerts = handler
        .list_cluster_alerts()
        .await
        .expect("fixture should decode");

    // The recorded fixture has 12 cluster alerts.
    assert_eq!(alerts.len(), 12);

    let nodes_down = alerts
        .get("cluster_even_node_count")
        .expect("fixture has cluster_even_node_count");
    assert_eq!(nodes_down.severity, "WARNING");
    assert!(
        nodes_down.state,
        "cluster_even_node_count is firing in the fixture"
    );
    assert!(!nodes_down.enabled);
}
