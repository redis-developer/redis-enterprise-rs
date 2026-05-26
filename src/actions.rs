//! Action management for Redis Enterprise async operations
//!
//! ## Overview
//! - Track long-running operations
//! - Query action status
//! - Cancel or wait for actions

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Action information.
///
/// Represents an action (or state machine) in the cluster. The
/// `GET /v1/actions` endpoint returns a wrapper with two arrays —
/// `actions` and `state-machines` — that share most fields; this
/// struct accepts both shapes with everything outside the common
/// `(action_uid, name, status)` core typed as `Option`.
///
/// Fields like `progress`, `node_uid`, `task_id`, and `creation_time`
/// are typed `Option<String>` because the real API returns them as
/// strings (e.g. `"progress": "100"`, `"node_uid": "1"`), not numbers
/// — the previous numeric typing decode-failed on every real response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action's unique identifier (read-only).
    pub action_uid: String,
    /// Action's name (read-only).
    pub name: String,
    /// Current status of the action.
    ///
    /// Possible values: `queued`, `starting`, `running`, `cancelling`,
    /// `cancelled`, `completed`, `failed`.
    pub status: String,
    /// Percent of completed steps (0–100). Wire type is `string`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
    /// ISO 8601 timestamp when the action was started.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// ISO 8601 timestamp when the action completed or failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Unix-epoch (or ISO-8601) timestamp when the action was created.
    /// Present on entries in the `actions` array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_time: Option<String>,
    /// Task ID associated with the action. Wire type is `string`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Heartbeat timestamp (Unix seconds). Present on entries in the
    /// `state-machines` array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heartbeat: Option<i64>,
    /// Resource the state machine acts on (e.g. `"bdb:1"`). Present on
    /// entries in the `state-machines` array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_name: Option<String>,
    /// Human-readable description of the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Error message if the action failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Database UID associated with the action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bdb_uid: Option<u32>,
    /// Node UID associated with the action. Wire type is `string`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_uid: Option<String>,
}

/// Response wrapper for `GET /v1/actions`.
///
/// The Redis Enterprise API returns
/// `{ "actions": [...], "state-machines": [...] }`. Use
/// [`ActionHandler::list_response`] to get the typed wrapper, or
/// [`ActionHandler::list`] for a single flat `Vec<Action>` combining
/// both arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionsListResponse {
    /// Active and recently completed actions.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// Long-running state machines (e.g. SMCreateBDB).
    #[serde(default, rename = "state-machines")]
    pub state_machines: Vec<Action>,
}

/// Action handler for tracking async operations
/// Handler for action-related operations
pub struct ActionHandler {
    client: RestClient,
}

impl ActionHandler {
    /// Create a new action handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        ActionHandler { client }
    }

    /// List all actions and state machines, flattened into a single vector.
    ///
    /// `GET /v1/actions`. The API returns `{actions, state-machines}`;
    /// this method unwraps the wrapper and concatenates both arrays.
    /// Use [`Self::list_response`] if you need them separated.
    pub async fn list(&self) -> Result<Vec<Action>> {
        let resp: ActionsListResponse = self.client.get("/v1/actions").await?;
        let mut combined = resp.actions;
        combined.extend(resp.state_machines);
        Ok(combined)
    }

    /// List all actions and state machines as the spec-shaped wrapper.
    ///
    /// `GET /v1/actions`. Returns the `{actions, state-machines}`
    /// response shape unchanged for callers that need to distinguish
    /// the two arrays.
    pub async fn list_response(&self) -> Result<ActionsListResponse> {
        self.client.get("/v1/actions").await
    }

    /// Get specific action status
    pub async fn get(&self, action_uid: &str) -> Result<Action> {
        self.client
            .get(&format!("/v1/actions/{}", action_uid))
            .await
    }

    /// Cancel an action
    pub async fn cancel(&self, action_uid: &str) -> Result<()> {
        self.client
            .delete(&format!("/v1/actions/{}", action_uid))
            .await
    }

    /// List actions via v2 API - GET /v2/actions
    pub async fn list_v2(&self) -> Result<Vec<Action>> {
        self.client.get("/v2/actions").await
    }

    /// Get action via v2 API - GET /v2/actions/{uid}
    pub async fn get_v2(&self, action_uid: &str) -> Result<Action> {
        self.client
            .get(&format!("/v2/actions/{}", action_uid))
            .await
    }

    /// List actions for a single database.
    ///
    /// `GET /v1/actions/bdb/{uid}`. Returns the same
    /// `{actions, state-machines}` wrapper as the top-level
    /// `GET /v1/actions`; the two arrays are concatenated.
    pub async fn list_for_bdb(&self, bdb_uid: u32) -> Result<Vec<Action>> {
        let resp: ActionsListResponse = self
            .client
            .get(&format!("/v1/actions/bdb/{}", bdb_uid))
            .await?;
        let mut combined = resp.actions;
        combined.extend(resp.state_machines);
        Ok(combined)
    }

    // Versioned sub-handlers for clearer API
    /// Returns a v1-scoped sub-handler for `/v1/actions` endpoints.
    pub fn v1(&self) -> v1::ActionsV1 {
        v1::ActionsV1::new(self.client.clone())
    }

    /// Returns a v2-scoped sub-handler for `/v2/actions` endpoints.
    pub fn v2(&self) -> v2::ActionsV2 {
        v2::ActionsV2::new(self.client.clone())
    }
}

/// V1 action endpoints (`/v1/actions`).
pub mod v1 {
    use super::{Action, ActionsListResponse, RestClient};
    use crate::error::Result;

    /// V1 sub-handler for `/v1/actions` endpoints.
    pub struct ActionsV1 {
        client: RestClient,
    }

    impl ActionsV1 {
        pub(crate) fn new(client: RestClient) -> Self {
            Self { client }
        }

        /// List actions + state machines, flattened.
        pub async fn list(&self) -> Result<Vec<Action>> {
            let resp: ActionsListResponse = self.client.get("/v1/actions").await?;
            let mut combined = resp.actions;
            combined.extend(resp.state_machines);
            Ok(combined)
        }

        /// Get a single action by UID.
        pub async fn get(&self, action_uid: &str) -> Result<Action> {
            self.client
                .get(&format!("/v1/actions/{}", action_uid))
                .await
        }

        /// Cancel an in-flight action.
        pub async fn cancel(&self, action_uid: &str) -> Result<()> {
            self.client
                .delete(&format!("/v1/actions/{}", action_uid))
                .await
        }

        /// List actions + state machines scoped to a single database, flattened.
        pub async fn list_for_bdb(&self, bdb_uid: u32) -> Result<Vec<Action>> {
            let resp: ActionsListResponse = self
                .client
                .get(&format!("/v1/actions/bdb/{}", bdb_uid))
                .await?;
            let mut combined = resp.actions;
            combined.extend(resp.state_machines);
            Ok(combined)
        }
    }
}

/// V2 action endpoints (`/v2/actions`).
pub mod v2 {
    use super::{Action, RestClient};
    use crate::error::Result;

    /// V2 sub-handler for `/v2/actions` endpoints.
    pub struct ActionsV2 {
        client: RestClient,
    }

    impl ActionsV2 {
        pub(crate) fn new(client: RestClient) -> Self {
            Self { client }
        }

        /// List all v2 actions.
        pub async fn list(&self) -> Result<Vec<Action>> {
            self.client.get("/v2/actions").await
        }

        /// Get a single v2 action by UID.
        pub async fn get(&self, action_uid: &str) -> Result<Action> {
            self.client
                .get(&format!("/v2/actions/{}", action_uid))
                .await
        }
    }
}
