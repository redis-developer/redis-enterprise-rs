//! Redis module management
//!
//! ## Overview
//! - List available modules
//! - Query module versions
//! - Configure module settings

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Platform-specific information for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Platform dependencies (typically an empty object)
    #[serde(default)]
    pub dependencies: Value,

    /// SHA256 checksum of the module binary for this platform
    pub sha256: Option<String>,
}

/// Module information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Unique identifier (read-only).
    pub uid: String,
    /// Module name (e.g. `"search"`, `"timeseries"`).
    pub module_name: Option<String>,
    /// Version (read-only).
    pub version: Option<u32>,
    /// Semantic version string.
    pub semantic_version: Option<String>,
    /// Author name.
    pub author: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Homepage URL.
    pub homepage: Option<String>,
    /// License identifier.
    pub license: Option<String>,
    /// Default command-line arguments.
    pub command_line_args: Option<String>,
    /// List of module capabilities.
    pub capabilities: Option<Vec<String>>,
    /// Minimum compatible Redis version.
    pub min_redis_version: Option<String>,
    /// Compatible Redis version range.
    pub compatible_redis_version: Option<String>,
    /// Display name shown in the UI.
    pub display_name: Option<String>,
    /// Whether the module is bundled with Redis Enterprise.
    pub is_bundled: Option<bool>,

    // Additional fields from API audit
    /// Whether the module supports BigStore (Auto Tiering) version 2
    pub bigstore_version_2_support: Option<bool>,

    /// Name of the capability this module provides
    pub capability_name: Option<String>,

    /// Redis command used to configure this module
    pub config_command: Option<String>,

    /// CRDB (Conflict-free Replicated Database) configuration
    /// The API returns an empty object {} for modules without CRDB support
    pub crdb: Option<Value>,

    /// Module dependencies
    /// The API returns an empty object {} for modules without dependencies
    pub dependencies: Option<Value>,

    /// Contact email address of the module author
    pub email: Option<String>,

    /// Minimum Redis Enterprise version required for this module
    pub min_redis_pack_version: Option<String>,

    /// Platform-specific information for this module
    /// Maps platform names (e.g., 'rhel9/x86_64', 'rhel8/x86_64') to platform details
    #[serde(default)]
    pub platforms: Option<HashMap<String, PlatformInfo>>,

    /// SHA256 checksum of the module binary for verification
    pub sha256: Option<String>,
}

/// Module handler for managing Redis modules
pub struct ModuleHandler {
    client: RestClient,
}

/// Alias for backwards compatibility and intuitive plural naming
pub type ModulesHandler = ModuleHandler;

impl ModuleHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        ModuleHandler { client }
    }

    /// List all modules
    pub async fn list(&self) -> Result<Vec<Module>> {
        self.client.get("/v1/modules").await
    }

    /// Get specific module
    pub async fn get(&self, uid: &str) -> Result<Module> {
        self.client.get(&format!("/v1/modules/{}", uid)).await
    }

    /// Upload new module (tries v2 first, falls back to v1)
    ///
    /// Note: Some Redis Enterprise versions (particularly RE 8.x) do not support
    /// module upload via the REST API. In those cases, use the Admin UI or
    /// node-level CLI tools (rladmin) to upload modules.
    pub async fn upload(&self, module_data: Vec<u8>, file_name: &str) -> Result<Value> {
        // Try v2 first (returns action_uid for async tracking)
        match self
            .client
            .post_multipart("/v2/modules", module_data.clone(), "module", file_name)
            .await
        {
            Ok(response) => Ok(response),
            Err(crate::error::RestError::NotFound) => {
                // v2 endpoint doesn't exist, try v1
                match self
                    .client
                    .post_multipart("/v1/modules", module_data, "module", file_name)
                    .await
                {
                    Ok(response) => Ok(response),
                    Err(crate::error::RestError::ApiError { code: 405, .. }) => {
                        Err(crate::error::RestError::ValidationError(
                            "Module upload via REST API is not supported in this Redis Enterprise version. \
                             Use the Admin UI or rladmin CLI to upload modules.".to_string()
                        ))
                    }
                    Err(e) => Err(e),
                }
            }
            Err(crate::error::RestError::ApiError { code: 405, .. }) => {
                Err(crate::error::RestError::ValidationError(
                    "Module upload via REST API is not supported in this Redis Enterprise version. \
                     Use the Admin UI or rladmin CLI to upload modules.".to_string()
                ))
            }
            Err(e) => Err(e),
        }
    }

    /// Delete module
    pub async fn delete(&self, uid: &str) -> Result<()> {
        self.client.delete(&format!("/v1/modules/{}", uid)).await
    }

    /// Retired module update helper.
    #[deprecated(note = "Redis Software does not register PUT /v1/modules/{uid}")]
    pub async fn update(&self, _uid: &str, _updates: Value) -> Result<Module> {
        crate::error::unsupported_operation("update module")
    }

    /// Configure modules for a specific database - POST /v1/modules/config/bdb/{uid}
    pub async fn config_bdb(&self, bdb_uid: u32, config: Value) -> Result<Module> {
        self.client
            .post(&format!("/v1/modules/config/bdb/{}", bdb_uid), &config)
            .await
    }

    /// List custom module artifacts on the local node.
    ///
    /// `GET /v2/local/modules/user-defined/artifacts`. Live verification on
    /// Redis Enterprise Software 8.0.10-81 returned `200 OK` with `[]` on a
    /// cluster with no custom modules uploaded.
    pub async fn list_user_defined_artifacts(&self) -> Result<Value> {
        self.client
            .get("/v2/local/modules/user-defined/artifacts")
            .await
    }

    /// Upload a custom module artifact to the local node.
    ///
    /// `POST /v2/local/modules/user-defined/artifacts`. The endpoint expects
    /// a multipart upload; the file is sent under the `module` form field
    /// to match the existing v1/v2 module upload behaviour. Live
    /// verification on 8.0.10-81 returned `400 no_module` when an empty
    /// body was sent.
    pub async fn upload_user_defined_artifact(
        &self,
        module_data: Vec<u8>,
        file_name: &str,
    ) -> Result<Value> {
        self.client
            .post_multipart(
                "/v2/local/modules/user-defined/artifacts",
                module_data,
                "module",
                file_name,
            )
            .await
    }

    /// Register a previously-uploaded artifact as a cluster-wide
    /// user-defined module configuration.
    ///
    /// `POST /v2/modules/user-defined`. The body shape is version-specific
    /// (it must reference an artifact that has already been uploaded via
    /// [`upload_user_defined_artifact`](Self::upload_user_defined_artifact)
    /// and describe how the cluster should load it); pass a `Value`
    /// matching the documented payload. Live verification on 8.0.10-81
    /// returned `406 invalid_module` when an empty body was sent.
    pub async fn register_user_defined(&self, body: Value) -> Result<Value> {
        self.client.post_raw("/v2/modules/user-defined", body).await
    }

    /// Delete a custom module configuration cluster-wide.
    ///
    /// `DELETE /v2/modules/user-defined/{uid}`. Live verification on
    /// 8.0.10-81 returned `404 module_delete_failed` against a uid that
    /// does not exist on the cluster.
    pub async fn delete_user_defined(&self, uid: &str) -> Result<()> {
        self.client
            .delete(&format!("/v2/modules/user-defined/{}", uid))
            .await
    }

    /// Delete a custom module artifact from the local node.
    ///
    /// `DELETE /v2/local/modules/user-defined/artifacts/{module_name}/{version}`.
    pub async fn delete_user_defined_artifact(
        &self,
        module_name: &str,
        version: &str,
    ) -> Result<()> {
        self.client
            .delete(&format!(
                "/v2/local/modules/user-defined/artifacts/{}/{}",
                module_name, version
            ))
            .await
    }
}
