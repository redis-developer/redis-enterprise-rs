//! Monitoring tests for database (BDB) operations
//!
//! Tests for shards, alerts, peer stats, syncer state, and password management.

use crate::common::{no_content_response, success_response, test_client};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer};

#[tokio::test]
async fn test_database_get_shards() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/shards"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            {"shard_id": 1, "role": "master"},
            {"shard_id": 2, "role": "slave"}
        ])))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().shards(1).await;

    assert!(result.is_ok());
    let shards = result.unwrap();
    assert!(shards.is_array());
}

#[tokio::test]
async fn test_database_peer_and_sync_stats() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/peer_stats"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"peers": []})))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/syncer_state"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"state": "ok"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let peers = client.databases().peer_stats(1).await.unwrap();
    assert!(peers["peers"].is_array());

    let state = client.databases().syncer_state(1).await.unwrap();
    assert_eq!(state["state"], "ok");
}

#[tokio::test]
async fn test_bdbs_alerts_and_crdt_detail() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/alerts"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([])))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/crdt_sources/alerts/1/2/high_cpu"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"severity": "critical"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let all = client.databases().alerts_all().await.unwrap();
    assert!(all.is_array());

    let detail = client
        .databases()
        .crdt_source_alert_detail(1, 2, "high_cpu")
        .await
        .unwrap();
    assert_eq!(detail["severity"], "critical");
}

#[tokio::test]
async fn test_passwords_delete_and_reset_status() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/bdbs/1/passwords"))
        .and(basic_auth("admin", "password"))
        .respond_with(no_content_response())
        .mount(&mock_server)
        .await;

    Mock::given(method("PUT"))
        .and(path("/v1/bdbs/1/actions/backup_reset_status"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"status": "reset"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    client.databases().passwords_delete(1).await.unwrap();
    let reset = client.databases().backup_reset_status(1).await.unwrap();
    assert_eq!(reset["status"], "reset");
}
