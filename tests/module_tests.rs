//! Module endpoint tests for Redis Enterprise

use redis_enterprise::{EnterpriseClient, ModuleHandler};
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

fn test_module() -> serde_json::Value {
    json!({
        "uid": "1",
        "module_name": "RedisSearch",
        "version": 20601,
        "semantic_version": "2.6.1",
        "capabilities": ["search", "index"]
    })
}

/// Test module with platforms field as returned by the actual API
/// The API returns platforms as a map/object, not an array
fn test_module_with_platforms() -> serde_json::Value {
    json!({
        "uid": "2",
        "module_name": "search",
        "semantic_version": "2.10.15",
        "version": 21015,
        "author": "RedisLabs",
        "description": "High performance search index on top of Redis",
        "homepage": "http://redisearch.io",
        "license": "Redis Source Available License 2.0",
        "command_line_args": "",
        "capabilities": ["search", "index"],
        "min_redis_version": "7.4",
        "compatible_redis_version": "7.4",
        "display_name": "RediSearch 2",
        "is_bundled": true,
        "platforms": {
            "rhel9/x86_64": {
                "dependencies": {},
                "sha256": "6bad5fdb464af8ecdf98b63dc26d65e08774826fbc11b0b5a8da363cb97fbd8c"
            },
            "rhel8/x86_64": {
                "dependencies": {},
                "sha256": "abc123def456789"
            }
        }
    })
}

#[tokio::test]
async fn test_module_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/modules"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_module(),
            {
                "uid": "2",
                "module_name": "RedisJSON",
                "version": 20400,
                "semantic_version": "2.4.0",
                "capabilities": ["json"]
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ModuleHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let modules = result.unwrap();
    assert_eq!(modules.len(), 2);
}

#[tokio::test]
async fn test_module_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/modules/1"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_module()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ModuleHandler::new(client);
    let result = handler.get("1").await;

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.uid, "1");
    assert_eq!(module.module_name, Some("RedisSearch".to_string()));
}

#[tokio::test]
async fn test_module_upload() {
    let mock_server = MockServer::start().await;

    // Mock v2 endpoint as not found
    Mock::given(method("POST"))
        .and(path("/v2/modules"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    // Mock v1 endpoint as success
    Mock::given(method("POST"))
        .and(path("/v1/modules"))
        .and(basic_auth("admin", "password"))
        .respond_with(created_response(test_module()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ModuleHandler::new(client);
    let result = handler.upload(vec![1, 2, 3, 4], "test.zip").await; // Mock binary data

    assert!(result.is_ok());
    let response = result.unwrap();
    // Response is now a Value, not a Module
    assert_eq!(response["uid"], "1");
    assert_eq!(response["module_name"], "RedisSearch");
}

#[tokio::test]
async fn test_module_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/modules/1"))
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

    let handler = ModuleHandler::new(client);
    let result = handler.delete("1").await;

    assert!(result.is_ok());
}

/// Test that demonstrates the bug: platforms field is returned as a map by the API
/// but the Module struct expects it as a Vec<String>
#[tokio::test]
async fn test_module_list_with_platforms_map() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/modules"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([test_module_with_platforms()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ModuleHandler::new(client);
    let result = handler.list().await;

    // This should succeed after the fix
    assert!(
        result.is_ok(),
        "Failed to deserialize module with platforms map: {:?}",
        result.err()
    );
    let modules = result.unwrap();
    assert_eq!(modules.len(), 1);

    let module = &modules[0];
    assert_eq!(module.uid, "2");
    assert_eq!(module.module_name, Some("search".to_string()));

    // After the fix, platforms should be accessible as a HashMap
    assert!(
        module.platforms.is_some(),
        "Platforms field should be present"
    );
}

/// Test getting a single module with platforms field
#[tokio::test]
async fn test_module_get_with_platforms_map() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/modules/2"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_module_with_platforms()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ModuleHandler::new(client);
    let result = handler.get("2").await;

    // This should succeed after the fix
    assert!(
        result.is_ok(),
        "Failed to deserialize module with platforms map: {:?}",
        result.err()
    );
    let module = result.unwrap();
    assert_eq!(module.uid, "2");
    assert_eq!(module.module_name, Some("search".to_string()));
    assert!(
        module.platforms.is_some(),
        "Platforms field should be present"
    );
}
