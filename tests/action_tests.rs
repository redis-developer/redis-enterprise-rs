//! Action endpoint tests for Redis Enterprise

use redis_enterprise::{ActionHandler, EnterpriseClient};
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

fn test_action() -> serde_json::Value {
    // Note: real API returns progress as a string, not a number — the
    // type matches what `cat tests/fixtures/actions_list.json` shows.
    json!({
        "action_uid": "action-123-abc",
        "name": "database_backup",
        "status": "running",
        "progress": "45",
        "start_time": "2023-01-01T12:00:00Z",
        "end_time": null,
        "description": "Backing up database test-db",
        "error": null
    })
}

fn completed_action() -> serde_json::Value {
    json!({
        "action_uid": "action-456-def",
        "name": "database_restore",
        "status": "completed",
        "progress": "100",
        "start_time": "2023-01-01T11:00:00Z",
        "end_time": "2023-01-01T11:30:00Z",
        "description": "Restored database from backup",
        "error": null
    })
}

fn failed_action() -> serde_json::Value {
    json!({
        "action_uid": "action-789-ghi",
        "name": "node_add",
        "status": "failed",
        "progress": "25",
        "start_time": "2023-01-01T10:00:00Z",
        "end_time": "2023-01-01T10:15:00Z",
        "description": "Adding new node to cluster",
        "error": "Connection timeout to new node"
    })
}

#[tokio::test]
async fn test_action_list() {
    let mock_server = MockServer::start().await;

    // Regression guard for #62: real API returns the wrapper shape
    // `{actions: [...], state-machines: [...]}`, not a bare array.
    Mock::given(method("GET"))
        .and(path("/v1/actions"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({
            "actions": [
                test_action(),
                completed_action(),
                failed_action()
            ],
            "state-machines": []
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let actions = result.unwrap();
    assert_eq!(actions.len(), 3);

    // Verify first action details
    let running_action = &actions[0];
    assert_eq!(running_action.action_uid, "action-123-abc");
    assert_eq!(running_action.name, "database_backup");
    assert_eq!(running_action.status, "running");
    // Regression guard for #63: progress is a string on the wire.
    assert_eq!(running_action.progress.as_deref(), Some("45"));
    assert!(running_action.end_time.is_none());
    assert!(running_action.error.is_none());
}

#[tokio::test]
async fn test_action_list_empty() {
    let mock_server = MockServer::start().await;

    // Empty case: API returns the wrapper with both arrays empty.
    Mock::given(method("GET"))
        .and(path("/v1/actions"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({
            "actions": [],
            "state-machines": []
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.list().await;

    assert!(result.is_ok());
    let actions = result.unwrap();
    assert_eq!(actions.len(), 0);
}

#[tokio::test]
async fn test_action_get_running() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/actions/action-123-abc"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(test_action()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.get("action-123-abc").await;

    assert!(result.is_ok());
    let action = result.unwrap();
    assert_eq!(action.action_uid, "action-123-abc");
    assert_eq!(action.name, "database_backup");
    assert_eq!(action.status, "running");
    assert_eq!(action.progress.as_deref(), Some("45"));
    assert_eq!(
        action.description,
        Some("Backing up database test-db".to_string())
    );
    assert!(action.end_time.is_none());
    assert!(action.error.is_none());
}

#[tokio::test]
async fn test_action_get_completed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/actions/action-456-def"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(completed_action()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.get("action-456-def").await;

    assert!(result.is_ok());
    let action = result.unwrap();
    assert_eq!(action.action_uid, "action-456-def");
    assert_eq!(action.status, "completed");
    assert_eq!(action.progress.as_deref(), Some("100"));
    assert!(action.end_time.is_some());
    assert!(action.error.is_none());
}

#[tokio::test]
async fn test_action_get_failed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/actions/action-789-ghi"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(failed_action()))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.get("action-789-ghi").await;

    assert!(result.is_ok());
    let action = result.unwrap();
    assert_eq!(action.action_uid, "action-789-ghi");
    assert_eq!(action.status, "failed");
    assert_eq!(action.progress.as_deref(), Some("25"));
    assert_eq!(
        action.error,
        Some("Connection timeout to new node".to_string())
    );
    assert!(action.end_time.is_some());
}

#[tokio::test]
async fn test_action_cancel() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/actions/action-123-abc"))
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

    let handler = ActionHandler::new(client);
    let result = handler.cancel("action-123-abc").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_action_cancel_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/actions/nonexistent-action"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Action not found"
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.cancel("nonexistent-action").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_action_get_nonexistent() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/actions/nonexistent-action"))
        .and(basic_auth("admin", "password"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Action not found"
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);
    let result = handler.get("nonexistent-action").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_action_list_state_machines_combined_into_flat_vec() {
    // Regression guard for #62 + #63: the wrapper carries both arrays,
    // shapes diverge between them (state-machines use heartbeat: i64,
    // object_name: String; actions use creation_time / task_id /
    // string-typed progress + node_uid).
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/actions"))
        .and(basic_auth("admin", "password"))
        .respond_with(success_response(json!({
            "actions": [
                {
                    "action_uid": "8fc32a11-4688-4057-9237-cd77fccd726f",
                    "creation_time": "1760400087",
                    "name": "retry_bdb",
                    "node_uid": "1",
                    "progress": "100",
                    "status": "completed",
                    "task_id": "8fc32a11-4688-4057-9237-cd77fccd726f"
                }
            ],
            "state-machines": [
                {
                    "action_uid": "04d7b6ea-f377-4d40-9b23-9a8f291988df",
                    "heartbeat": 1760400082,
                    "name": "SMCreateBDB",
                    "object_name": "bdb:1",
                    "status": "completed"
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = EnterpriseClient::builder()
        .base_url(mock_server.uri())
        .username("admin")
        .password("password")
        .build()
        .unwrap();

    let handler = ActionHandler::new(client);

    // Flat view: actions first, state machines appended.
    let actions = handler.list().await.unwrap();
    assert_eq!(actions.len(), 2);

    let action_entry = &actions[0];
    assert_eq!(action_entry.name, "retry_bdb");
    assert_eq!(action_entry.progress.as_deref(), Some("100"));
    assert_eq!(action_entry.node_uid.as_deref(), Some("1"));
    assert_eq!(action_entry.creation_time.as_deref(), Some("1760400087"));
    assert!(action_entry.task_id.is_some());

    let sm_entry = &actions[1];
    assert_eq!(sm_entry.name, "SMCreateBDB");
    assert_eq!(sm_entry.heartbeat, Some(1760400082));
    assert_eq!(sm_entry.object_name.as_deref(), Some("bdb:1"));

    // list_response gives the spec-shaped wrapper directly.
    let wrapped = handler.list_response().await.unwrap();
    assert_eq!(wrapped.actions.len(), 1);
    assert_eq!(wrapped.state_machines.len(), 1);
}
