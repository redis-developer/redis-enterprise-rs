//! CRUD tests for database (BDB) operations

use crate::common::{
    created_response, no_content_response, success_response, test_client, test_database,
};
use redis_enterprise::bdb::CreateDatabaseRequest;
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer};

#[tokio::test]
async fn test_database_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_database(),
            {
                "uid": 2,
                "name": "test-db-2",
                "type": "redis",
                "memory_size": 2147483648u64,
                "port": 12001,
                "status": "active"
            }
        ])))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().list().await;

    assert!(result.is_ok());
    let databases = result.unwrap();
    assert_eq!(databases.len(), 2);
}

#[tokio::test]
async fn test_database_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/bdbs/1"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_database()))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().info(1).await;

    assert!(result.is_ok());
    let db = result.unwrap();
    assert_eq!(db.uid, 1);
    assert_eq!(db.name, "test-db");
}

#[tokio::test]
async fn test_database_create() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/bdbs"))
        .and(basic_auth("admin", "password"))
        .respond_with(created_response(test_database()))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let request_data = CreateDatabaseRequest::builder()
        .name("test-db")
        .memory_size(1073741824)
        .port(12000)
        .build();
    let request = client.databases().create(request_data).await;

    assert!(request.is_ok());
    let db = request.unwrap();
    assert_eq!(db.uid, 1);
    assert_eq!(db.name, "test-db");
}

#[tokio::test]
async fn test_database_delete() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/bdbs/1"))
        .and(basic_auth("admin", "password"))
        .respond_with(no_content_response())
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let result = client.databases().delete(1).await;

    assert!(result.is_ok());
}
