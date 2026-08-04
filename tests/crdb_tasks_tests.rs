//! CRDB tasks endpoint tests for Redis Enterprise

use redis_enterprise::{CrdbTasksHandler, EnterpriseClient};
use serde_json::json;
use wiremock::matchers::{basic_auth, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Test helper functions
fn success_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

fn no_content_response() -> ResponseTemplate {
    ResponseTemplate::new(204)
}

fn error_response(code: u16, message: &str) -> ResponseTemplate {
    ResponseTemplate::new(code).set_body_json(json!({
        "error": message,
        "code": code
    }))
}

fn test_crdb_task() -> serde_json::Value {
    json!({
        "task_id": "task-123",
        "crdb_guid": "crdb-456",
        "task_type": "sync",
        "status": "running",
        "progress": 45.5,
        "start_time": "2023-01-01T12:00:00Z",
        "end_time": null,
        "error": null
    })
}

fn test_completed_task() -> serde_json::Value {
    json!({
        "task_id": "task-789",
        "crdb_guid": "crdb-456",
        "task_type": "backup",
        "status": "completed",
        "progress": 100.0,
        "start_time": "2023-01-01T11:00:00Z",
        "end_time": "2023-01-01T12:00:00Z",
        "error": null
    })
}

fn test_failed_task() -> serde_json::Value {
    json!({
        "task_id": "task-999",
        "crdb_guid": "crdb-456",
        "task_type": "restore",
        "status": "failed",
        "progress": 75.0,
        "start_time": "2023-01-01T10:00:00Z",
        "end_time": "2023-01-01T10:30:00Z",
        "error": "Connection timeout during restore"
    })
}

fn test_other_crdb_task() -> serde_json::Value {
    json!({
        "task_id": "task-other",
        "crdb_guid": "crdb-other",
        "task_type": "sync",
        "status": "completed"
    })
}

#[tokio::test]
async fn test_crdb_tasks_list() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_crdb_task(),
            test_completed_task(),
            test_failed_task()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(tasks[0].task_id, "task-123");
    assert_eq!(tasks[0].status, "running");
    assert_eq!(tasks[1].status, "completed");
    assert_eq!(tasks[2].status, "failed");
}

#[tokio::test]
async fn test_crdb_tasks_list_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks"))
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

    let handler = CrdbTasksHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn test_crdb_tasks_get() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks/task-123"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_crdb_task()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.get("task-123").await;

    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.task_id, "task-123");
    assert_eq!(task.crdb_guid, "crdb-456");
    assert_eq!(task.task_type, "sync");
    assert_eq!(task.status, "running");
    assert_eq!(task.progress, Some(45.5));
    assert!(task.start_time.is_some());
    assert!(task.end_time.is_none());
    assert!(task.error.is_none());
}

#[tokio::test]
async fn test_crdb_tasks_get_completed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks/task-789"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_completed_task()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.get("task-789").await;

    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.task_id, "task-789");
    assert_eq!(task.status, "completed");
    assert_eq!(task.progress, Some(100.0));
    assert!(task.start_time.is_some());
    assert!(task.end_time.is_some());
    assert!(task.error.is_none());
}

#[tokio::test]
async fn test_crdb_tasks_get_failed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks/task-999"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_failed_task()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.get("task-999").await;

    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.task_id, "task-999");
    assert_eq!(task.status, "failed");
    assert_eq!(task.progress, Some(75.0));
    assert!(task.error.is_some());
    assert_eq!(task.error.unwrap(), "Connection timeout during restore");
}

#[tokio::test]
async fn test_crdb_tasks_get_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks/nonexistent"))
        .and(basic_auth("admin", "password"))
        .respond_with(error_response(404, "Task not found"))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.get("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_crdb_tasks_cancel() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/crdb_tasks/task-123/actions/cancel"))
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

    let handler = CrdbTasksHandler::new(client);
    let result = handler.cancel("task-123").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_crdb_tasks_cancel_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/crdb_tasks/nonexistent/actions/cancel"))
        .and(basic_auth("admin", "password"))
        .respond_with(error_response(404, "Task not found"))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.cancel("nonexistent").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_crdb_tasks_cancel_completed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/crdb_tasks/task-789/actions/cancel"))
        .and(basic_auth("admin", "password"))
        .respond_with(error_response(400, "Cannot cancel completed task"))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.cancel("task-789").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_crdb_tasks_cancel_force() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/crdb_tasks/task-123/actions/cancel"))
        .and(wiremock::matchers::query_param("force", "true"))
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

    let handler = CrdbTasksHandler::new(client);
    let result = handler.cancel_with_force("task-123", true).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_crdb_tasks_list_by_crdb() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([
            test_crdb_task(),
            test_completed_task(),
            test_other_crdb_task()
        ])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.list_by_crdb("crdb-456").await;

    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].crdb_guid, "crdb-456");
    assert_eq!(tasks[1].crdb_guid, "crdb-456");
}

#[tokio::test]
async fn test_crdb_tasks_list_by_crdb_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([test_crdb_task()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.list_by_crdb("crdb-999").await;

    assert!(result.is_ok());
    let tasks = result.unwrap();
    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn test_crdb_tasks_list_by_crdb_nonexistent_is_empty() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/crdb_tasks"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!([test_crdb_task()])))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = CrdbTasksHandler::new(client);
    let result = handler.list_by_crdb("nonexistent").await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
