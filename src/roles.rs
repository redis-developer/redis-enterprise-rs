//! Role-based access control
//!
//! ## Overview
//! - Create and manage roles
//! - Configure role permissions
//! - Query role assignments

use crate::error::Result;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Role information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleInfo {
    pub uid: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdb_roles: Option<Vec<BdbRole>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_roles: Option<Vec<String>>,
}

/// Database-specific role permissions
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct BdbRole {
    pub bdb_uid: u32,
    #[builder(setter(into))]
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub redis_acl_uid: Option<u32>,
}

/// Create role request
///
/// # Examples
///
/// ```rust,no_run
/// use redis_enterprise::{CreateRoleRequest, BdbRole};
///
/// let request = CreateRoleRequest::builder()
///     .name("database-admin")
///     .management("admin")
///     .bdb_roles(vec![
///         BdbRole::builder()
///             .bdb_uid(1)
///             .role("admin")
///             .build()
///     ])
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct CreateRoleRequest {
    #[builder(setter(into))]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub management: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub data_access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub bdb_roles: Option<Vec<BdbRole>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub cluster_roles: Option<Vec<String>>,
}

define_handler!(
    /// Roles handler
    pub struct RolesHandler;
);

impl_crud!(RolesHandler {
    list => RoleInfo, "/v1/roles";
    get(u32) => RoleInfo, "/v1/roles/{}";
    delete(u32), "/v1/roles/{}";
    create(CreateRoleRequest) => RoleInfo, "/v1/roles";
    update(u32, CreateRoleRequest) => RoleInfo, "/v1/roles/{}";
});

// Custom methods
impl RolesHandler {
    /// Get built-in roles
    pub async fn built_in(&self) -> Result<Vec<RoleInfo>> {
        self.client.get("/v1/roles/builtin").await
    }

    /// Get users assigned to a role
    pub async fn users(&self, uid: u32) -> Result<Vec<u32>> {
        self.client.get(&format!("/v1/roles/{}/users", uid)).await
    }
}
