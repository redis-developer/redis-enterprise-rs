//! Metrics configuration endpoint tests for Redis Enterprise.

use redis_enterprise::{EnterpriseClient, MetricsConfigUpdate};
use serde_json::json;
use wiremock::matchers::{basic_auth, body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn metrics_config_response() -> serde_json::Value {
    json!({
        "key_distribution_enabled": false,
        "key_size_buckets": "",
        "key_items_buckets": "",
        "local_storage_max_size_mb": 1024,
        "local_storage_retention_days": 8,
        "expose_db_tags": true,
        "metrics_tag_keys_exposed": ["environment", "team"],
        "max_requests_in_flight": 2
    })
}

fn test_client(mock_server: &MockServer) -> EnterpriseClient {
    EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .expect("test client should build")
}

#[tokio::test]
async fn get_metrics_config_uses_documented_route_and_response_shape() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/metrics_config"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(metrics_config_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = test_client(&mock_server)
        .metrics_config()
        .get()
        .await
        .expect("metrics configuration should deserialize");

    assert!(!config.key_distribution_enabled);
    assert_eq!(config.local_storage_max_size_mb, 1024);
    assert_eq!(config.local_storage_retention_days, 8);
    assert!(config.expose_db_tags);
    assert_eq!(config.metrics_tag_keys_exposed, ["environment", "team"]);
    assert_eq!(config.max_requests_in_flight, 2);
}

#[tokio::test]
async fn update_metrics_config_serializes_only_supplied_fields() {
    let mock_server = MockServer::start().await;
    let update = MetricsConfigUpdate::builder()
        .expose_db_tags(true)
        .metrics_tag_keys_exposed(vec!["environment".to_string(), "team".to_string()])
        .max_requests_in_flight(4)
        .build();

    Mock::given(method("PUT"))
        .and(path("/v1/metrics_config"))
        .and(basic_auth("admin", "password"))
        .and(body_json(json!({
            "expose_db_tags": true,
            "metrics_tag_keys_exposed": ["environment", "team"],
            "max_requests_in_flight": 4
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(metrics_config_response()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = test_client(&mock_server)
        .metrics_config()
        .update(update)
        .await
        .expect("metrics configuration should update");

    assert!(config.expose_db_tags);
    assert_eq!(config.metrics_tag_keys_exposed.len(), 2);
}

#[test]
fn empty_metrics_config_update_is_detectable_and_serializes_empty() {
    let update = MetricsConfigUpdate::default();

    assert!(update.is_empty());
    assert_eq!(serde_json::to_value(update).unwrap(), json!({}));
}

#[tokio::test]
async fn empty_metrics_config_update_is_rejected_before_sending() {
    let mock_server = MockServer::start().await;

    let error = test_client(&mock_server)
        .metrics_config()
        .update(MetricsConfigUpdate::default())
        .await
        .expect_err("an empty update should fail validation");

    assert!(error.is_bad_request());
    assert!(error.to_string().contains("at least one field"));
}
