//! Usage report endpoint tests for Redis Enterprise.
//!
//! `UsageReportHandler` now only exposes `list()` — the latest / get / generate
//! / config / csv handlers were removed in the #65 audit follow-up because
//! their `/v1/usage_report/*` subpaths are not documented in the REST API at
//! any version.

use redis_enterprise::{EnterpriseClient, UsageReportHandler};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_report_body() -> serde_json::Value {
    json!([{
        "report_id": "2026-05-26",
        "timestamp": "2026-05-26T00:00:00Z",
        "period_start": "2026-05-25T00:00:00Z",
        "period_end": "2026-05-26T00:00:00Z",
        "cluster_name": "test-cluster",
        "summary": {
            "total_memory_gb": 12.0,
            "total_ops": 0,
            "total_bandwidth_gb": 0.0,
            "database_count": 1,
            "node_count": 3,
            "shard_count": 1
        }
    }])
}

#[tokio::test]
async fn test_usage_report_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/usage_report"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(test_report_body()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = UsageReportHandler::new(client);
    let result = handler.list().await.unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].report_id, "2026-05-26");
    assert_eq!(result[0].cluster_name, "test-cluster");
    let summary = result[0].summary.as_ref().expect("summary present");
    assert_eq!(summary.database_count, 1);
}

#[tokio::test]
async fn test_usage_report_list_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/usage_report"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = UsageReportHandler::new(client);
    let result = handler.list().await.unwrap();
    assert!(result.is_empty());
}
