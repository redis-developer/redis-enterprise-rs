//! Common test utilities for BDB tests

use redis_enterprise::EnterpriseClient;
use serde_json::json;
use wiremock::{MockServer, ResponseTemplate};

/// Create a success response with JSON body
pub fn success_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

/// Create a 201 Created response
pub fn created_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(201).set_body_json(body)
}

/// Create a 204 No Content response
pub fn no_content_response() -> ResponseTemplate {
    ResponseTemplate::new(204)
}

/// Standard test database fixture
pub fn test_database() -> serde_json::Value {
    json!({
        "uid": 1,
        "name": "test-db",
        "type": "redis",
        "memory_size": 1073741824,
        "port": 12000,
        "status": "active",
        "master_persistence": false,
        "data_persistence": "disabled",
        "max_aof_file_size": 322122547200u64,
        "recovery_wait_time": -1,
        "skip_import_analyze": "disabled",
        "sync_dedicated_threads": 5
    })
}

/// Create a test client connected to the mock server
pub fn test_client(mock_server: &MockServer) -> EnterpriseClient {
    EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap()
}
