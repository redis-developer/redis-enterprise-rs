//! Tests for the documented migration lookup resource.

use redis_enterprise::{EnterpriseClient, MigrationsHandler};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(mock_server: &MockServer) -> EnterpriseClient {
    EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap()
}

fn migration() -> serde_json::Value {
    json!({
        "migration_id": "migration-123",
        "source": {"endpoint_type": "external", "host": "source.redis.example"},
        "target": {"endpoint_type": "bdb", "bdb_uid": 1},
        "status": "running"
    })
}

#[tokio::test]
async fn get_migration_uses_documented_route() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/migrations/migration-123"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(200).set_body_json(migration()))
        .mount(&mock_server)
        .await;

    let migration = MigrationsHandler::new(client(&mock_server))
        .get("migration-123")
        .await
        .unwrap();
    assert_eq!(migration.migration_id, "migration-123");
}

#[tokio::test]
async fn get_missing_migration_propagates_not_found() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/migrations/missing"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let result = MigrationsHandler::new(client(&mock_server))
        .get("missing")
        .await;
    assert!(result.is_err());
}
