//! Action tests for database (BDB) operations
//!
//! Tests for export, import, backup, restore, upgrade, and other database actions.

use crate::common::{success_response, test_client};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer};

#[tokio::test]
async fn test_database_export() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/actions/export"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"task_id": "export-123"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().export(1, "ftp://backup/db1.rdb").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_database_import() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/actions/import"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"task_id": "import-456"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client
        .databases()
        .import(1, "ftp://backup/db1.rdb", true)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_database_backup() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/actions/backup"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"backup_id": "backup-789"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().backup(1).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_database_restore() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/actions/restore"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(
            json!({"action_uid": "act-restore-1", "status": "restored"}),
        ))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().restore(1, Some("backup-789")).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_database_upgrade_module() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/actions/upgrade"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(
            json!({"action_uid": "act-up-1", "status": "upgraded"}),
        ))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().upgrade(1, "search", "2.0").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_database_optimize_shards_placement_status() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1/actions/optimize_shards_placement"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"status": "ok"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().optimize_shards_placement(1).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap()["status"], "ok");
}

#[tokio::test]
async fn test_database_recover() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/actions/recover"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({"action_uid": "act-1"})))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().recover(1).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().action_uid, "act-1");
}

#[tokio::test]
async fn test_database_upgrade_redis_version() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs/1/upgrade"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({
            "action_uid": "591d9dcb-ddd7-48a9-a04d-bd5d4d6834d0",
            "uid": 1,
            "name": "test-db",
            "status": "active",
            "redis_version": "7.4",
            "version": "7.4.2",
            "memory_size": 1073741824,
            "type": "redis",
            "replication": false,
            "persistence": "disabled",
            "port": 18367,
            "shards_count": 1,
            "oss_cluster": false
        })))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);

    let request = redis_enterprise::bdb::DatabaseUpgradeRequest {
        redis_version: Some("7.4.2".to_string()),
        preserve_roles: Some(true),
        force_restart: Some(false),
        may_discard_data: Some(false),
        force_discard: Some(false),
        keep_crdt_protocol_version: Some(false),
        parallel_shards_upgrade: None,
        modules: None,
    };

    let result = client.databases().upgrade_redis_version(1, request).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.action_uid, "591d9dcb-ddd7-48a9-a04d-bd5d4d6834d0");
}
