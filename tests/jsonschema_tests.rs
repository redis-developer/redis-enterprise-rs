//! JSON Schema endpoint tests for Redis Enterprise.
//!
//! `JsonSchemaHandler` now only exposes `list()` — the per-resource and
//! validate handlers were removed in the #65 audit follow-up because their
//! `/v1/jsonschema/...` subpaths are not documented in the REST API at any
//! version.

use redis_enterprise::{EnterpriseClient, JsonSchemaHandler};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_jsonschema_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/jsonschema"))
        .and(basic_auth("admin", "password"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!(["bdb", "cluster", "crdb", "node", "user"])),
        )
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = JsonSchemaHandler::new(client);
    let result = handler.list().await.unwrap();
    assert_eq!(result.len(), 5);
    assert!(result.iter().any(|name| name == "bdb"));
}

#[tokio::test]
async fn test_jsonschema_list_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/jsonschema"))
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

    let handler = JsonSchemaHandler::new(client);
    let result = handler.list().await.unwrap();
    assert!(result.is_empty());
}
