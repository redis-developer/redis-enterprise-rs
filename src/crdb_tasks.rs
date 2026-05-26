//! Active-Active database task operations
//!
//! ## Overview
//! - Track CRDB async operations
//! - Query task status
//! - Manage replication tasks

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

/// CRDB task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdbTask {
    /// Unique task identifier
    pub task_id: String,
    /// Globally unique Active-Active database ID (GUID)
    pub crdb_guid: String,
    /// Type of task being executed
    pub task_type: String,
    /// Current status of the task (queued, running, completed, failed)
    pub status: String,
    /// Task completion progress as a percentage (0.0-100.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    /// Timestamp when the task was started (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Timestamp when the task was completed or failed (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Error description if the task failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// CRDB task creation request
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CreateCrdbTaskRequest {
    /// Globally unique Active-Active database ID (GUID) for the target CRDB
    #[builder(setter(into))]
    pub crdb_guid: String,
    /// Type of task to create (e.g., "flush", "purge", "update_config")
    #[builder(setter(into))]
    pub task_type: String,
    /// Optional parameters specific to the task type
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub params: Option<Value>,
}

/// CRDB tasks handler
pub struct CrdbTasksHandler {
    client: RestClient,
}

impl CrdbTasksHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        CrdbTasksHandler { client }
    }

    /// List all CRDB tasks
    pub async fn list(&self) -> Result<Vec<CrdbTask>> {
        self.client.get("/v1/crdb_tasks").await
    }

    /// Get specific CRDB task
    pub async fn get(&self, task_id: &str) -> Result<CrdbTask> {
        self.client
            .get(&format!("/v1/crdb_tasks/{}", task_id))
            .await
    }

    /// Create a new CRDB task
    pub async fn create(&self, request: CreateCrdbTaskRequest) -> Result<CrdbTask> {
        self.client.post("/v1/crdb_tasks", &request).await
    }

    /// Cancel a CRDB task
    pub async fn cancel(&self, task_id: &str) -> Result<()> {
        self.client
            .post_action(
                &format!("/v1/crdb_tasks/{}/actions/cancel", task_id),
                &serde_json::json!({}),
            )
            .await
    }

    /// Cancel a CRDB task with optional force mode
    pub async fn cancel_with_force(&self, task_id: &str, force: bool) -> Result<()> {
        let path = if force {
            format!("/v1/crdb_tasks/{}/actions/cancel?force=true", task_id)
        } else {
            format!("/v1/crdb_tasks/{}/actions/cancel", task_id)
        };

        self.client.post_action(&path, &serde_json::json!({})).await
    }

    /// Get tasks for a specific CRDB
    pub async fn list_by_crdb(&self, crdb_guid: &str) -> Result<Vec<CrdbTask>> {
        self.client
            .get(&format!("/v1/crdbs/{}/tasks", crdb_guid))
            .await
    }
}
