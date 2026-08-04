//! Usage report endpoint tests for Redis Software's streamed NDJSON format.

use futures::StreamExt;
use redis_enterprise::usage_report::MAX_USAGE_REPORT_LINE_BYTES;
use redis_enterprise::{EnterpriseClient, RestError, UsageReportHandler, UsageReportRecord};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CHECKSUM: &str = "0123456789abcdef0123456789abcdef";

fn test_report_line() -> String {
    json!({
        "cluster_name": "test-cluster",
        "cluster_uuid": "00000000-0000-0000-0000-000000000001",
        "date": "2026-08-04T17:00:00Z",
        "software_version": "8.2.0-25",
        "api_version": "1",
        "bdb_uid": "1",
        "type": "core",
        "shard_type": "normal",
        "dominant_shard_criteria": "mem",
        "provisioned_memory": 1073741824u64,
        "used_memory": 5242880u64,
        "master_shards_count": 1,
        "no_eviction": false,
        "persistence": false,
        "backup": false,
        "using_redis_search": true,
        "ops_sec": 12.5,
        "replication": false,
        "active_active": false,
        "license": {
            "ram_shards_in_use": 1,
            "ram_shards_limit": 100
        },
        "future_field": "preserved"
    })
    .to_string()
}

fn test_client(mock_server: &MockServer) -> EnterpriseClient {
    EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .expect("test client should build")
}

async fn mock_usage_report(mock_server: &MockServer, status: u16, body: impl Into<String>) {
    Mock::given(method("GET"))
        .and(path("/v1/usage_report"))
        .and(basic_auth("admin", "password"))
        .respond_with(
            ResponseTemplate::new(status)
                .insert_header("content-type", "text/html; charset=utf-8")
                .set_body_string(body),
        )
        .expect(1)
        .mount(mock_server)
        .await;
}

#[tokio::test]
async fn usage_report_list_collects_ndjson_and_consumes_checksum() {
    let mock_server = MockServer::start().await;
    mock_usage_report(
        &mock_server,
        200,
        format!("{}\n{CHECKSUM}\n", test_report_line()),
    )
    .await;

    let reports = UsageReportHandler::new(test_client(&mock_server))
        .list()
        .await
        .expect("valid NDJSON should decode");

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].cluster_name.as_deref(), Some("test-cluster"));
    assert_eq!(reports[0].bdb_uid.as_deref(), Some("1"));
    assert_eq!(reports[0].ops_sec, Some(12.5));
    assert_eq!(
        reports[0].additional_fields.get("future_field"),
        Some(&json!("preserved"))
    );
}

#[tokio::test]
async fn usage_report_stream_exposes_the_final_checksum() {
    let mock_server = MockServer::start().await;
    mock_usage_report(
        &mock_server,
        200,
        format!("{}\n{CHECKSUM}\n", test_report_line()),
    )
    .await;

    let mut stream = UsageReportHandler::new(test_client(&mock_server))
        .stream()
        .await
        .expect("stream request should start");

    assert!(matches!(
        stream.next().await,
        Some(Ok(UsageReportRecord::Report(_)))
    ));
    match stream.next().await {
        Some(Ok(UsageReportRecord::Checksum(checksum))) => assert_eq!(checksum, CHECKSUM),
        other => panic!("expected final checksum record, got {other:?}"),
    }
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn usage_report_checksum_only_and_empty_bodies_are_explicitly_empty() {
    for body in [format!("{CHECKSUM}\n"), String::new()] {
        let mock_server = MockServer::start().await;
        mock_usage_report(&mock_server, 200, body).await;

        let reports = UsageReportHandler::new(test_client(&mock_server))
            .list()
            .await
            .expect("empty report should succeed");
        assert!(reports.is_empty());
    }
}

#[tokio::test]
async fn usage_report_malformed_record_does_not_leak_the_response_line() {
    let mock_server = MockServer::start().await;
    mock_usage_report(
        &mock_server,
        200,
        format!("{{\"customer_secret\":\"do-not-leak\"\n{CHECKSUM}\n"),
    )
    .await;

    let error = UsageReportHandler::new(test_client(&mock_server))
        .list()
        .await
        .expect_err("malformed NDJSON should fail");

    assert!(matches!(error, RestError::ParseError(_)));
    assert!(error.to_string().contains("record 1"));
    assert!(!error.to_string().contains("do-not-leak"));
}

#[tokio::test]
async fn usage_report_requires_the_checksum_after_json_records() {
    let mock_server = MockServer::start().await;
    mock_usage_report(&mock_server, 200, format!("{}\n", test_report_line())).await;

    let error = UsageReportHandler::new(test_client(&mock_server))
        .list()
        .await
        .expect_err("missing checksum should fail");

    assert!(matches!(error, RestError::ParseError(_)));
    assert!(error.to_string().contains("without the final MD5 checksum"));
}

#[tokio::test]
async fn usage_report_rejects_records_after_the_checksum() {
    let mock_server = MockServer::start().await;
    mock_usage_report(
        &mock_server,
        200,
        format!("{CHECKSUM}\n{}\n", test_report_line()),
    )
    .await;

    let error = UsageReportHandler::new(test_client(&mock_server))
        .list()
        .await
        .expect_err("checksum must be final");

    assert!(matches!(error, RestError::ParseError(_)));
    assert!(error.to_string().contains("after the final checksum"));
}

#[tokio::test]
async fn usage_report_http_error_preserves_status_without_body_leakage() {
    let mock_server = MockServer::start().await;
    mock_usage_report(&mock_server, 503, "customer-secret-do-not-leak").await;

    let result = UsageReportHandler::new(test_client(&mock_server))
        .stream()
        .await;
    let error = match result {
        Ok(_) => panic!("HTTP 503 should fail before returning a stream"),
        Err(error) => error,
    };

    assert!(matches!(error, RestError::ClusterBusy));
    assert!(!error.to_string().contains("customer-secret-do-not-leak"));
}

#[tokio::test]
async fn usage_report_bounds_each_streamed_line() {
    let mock_server = MockServer::start().await;
    mock_usage_report(
        &mock_server,
        200,
        "x".repeat(MAX_USAGE_REPORT_LINE_BYTES + 1),
    )
    .await;

    let error = UsageReportHandler::new(test_client(&mock_server))
        .list()
        .await
        .expect_err("oversized line should fail");

    assert!(matches!(error, RestError::ParseError(_)));
    assert!(error.to_string().contains("exceeds the"));
}
