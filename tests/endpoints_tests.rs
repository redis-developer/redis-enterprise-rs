//! Endpoints tests for Redis Enterprise

use redis_enterprise::{EndpointsHandler, EnterpriseClient};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Test helper functions
fn success_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

fn test_endpoint_stats_data() -> serde_json::Value {
    json!({
        "uid": "endpoint-1",
        "intervals": [
            {
                "interval": "1sec",
                "timestamps": [1640995200, 1640995260, 1640995320],
                "values": [
                    {"ops_per_sec": 1000, "hits_per_sec": 800},
                    {"ops_per_sec": 1100, "hits_per_sec": 850},
                    {"ops_per_sec": 1050, "hits_per_sec": 820}
                ]
            },
            {
                "interval": "1hour",
                "timestamps": [1640991600, 1640995200],
                "values": [
                    {"ops_per_sec": 950, "hits_per_sec": 750},
                    {"ops_per_sec": 1050, "hits_per_sec": 820}
                ]
            }
        ]
    })
}

fn test_endpoint_stats_minimal_data() -> serde_json::Value {
    json!({
        "uid": "endpoint-2",
        "intervals": []
    })
}

#[tokio::test]
async fn test_endpoint_stats() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/endpoints/stats"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_endpoint_stats_data(),
            test_endpoint_stats_minimal_data()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = EndpointsHandler::new(client);
    let result = handler.stats("endpoint-1").await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.uid, "endpoint-1");
    assert_eq!(stats.intervals.len(), 2);

    // Check first interval
    assert_eq!(stats.intervals[0].interval, "1sec");
    assert_eq!(stats.intervals[0].timestamps.len(), 3);
    assert_eq!(stats.intervals[0].values.len(), 3);

    // Check second interval
    assert_eq!(stats.intervals[1].interval, "1hour");
    assert_eq!(stats.intervals[1].timestamps.len(), 2);
    assert_eq!(stats.intervals[1].values.len(), 2);
}

#[tokio::test]
async fn test_endpoint_stats_minimal() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/endpoints/stats"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_endpoint_stats_data(),
            test_endpoint_stats_minimal_data()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = EndpointsHandler::new(client);
    let result = handler.stats("endpoint-2").await;

    assert!(result.is_ok());
    let stats = result.unwrap();
    assert_eq!(stats.uid, "endpoint-2");
    assert_eq!(stats.intervals.len(), 0);
}

#[tokio::test]
async fn test_endpoint_stats_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/endpoints/stats"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_endpoint_stats_data(),
            test_endpoint_stats_minimal_data()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = EndpointsHandler::new(client);
    let result = handler.stats("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_endpoints_all_stats() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/endpoints/stats"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_endpoint_stats_data(),
            test_endpoint_stats_minimal_data()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = EndpointsHandler::new(client);
    let result = handler.all_stats().await;

    assert!(result.is_ok());
    let all_stats = result.unwrap();
    assert_eq!(all_stats.len(), 2);

    // Check first endpoint stats
    assert_eq!(all_stats[0].uid, "endpoint-1");
    assert_eq!(all_stats[0].intervals.len(), 2);

    // Check second endpoint stats
    assert_eq!(all_stats[1].uid, "endpoint-2");
    assert_eq!(all_stats[1].intervals.len(), 0);
}

#[tokio::test]
async fn test_endpoints_all_stats_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/endpoints/stats"))
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

    let handler = EndpointsHandler::new(client);
    let result = handler.all_stats().await;

    assert!(result.is_ok());
    let all_stats = result.unwrap();
    assert_eq!(all_stats.len(), 0);
}
