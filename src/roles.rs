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
    /// Unique identifier (read-only).
    pub uid: u32,
    /// Name.
    pub name: String,
    /// Management permission level (e.g. `"admin"`, `"db_member"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management: Option<String>,
    /// Data access permission level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_access: Option<String>,
    /// Per-database role permissions. See [`BdbRole`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdb_roles: Option<Vec<BdbRole>>,
    /// List of cluster-wide role names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_roles: Option<Vec<String>>,
}

/// Database-specific role permissions
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct BdbRole {
    /// Database (BDB) UID this entity belongs to.
    pub bdb_uid: u32,
    /// Role.
    #[builder(setter(into))]
    pub role: String,
    /// UID of the Redis ACL associated with this role binding.
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
    /// Name.
    #[builder(setter(into))]
    pub name: String,
    /// Management permission level (e.g. `"admin"`, `"db_member"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub management: Option<String>,
    /// Data access permission level.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub data_access: Option<String>,
    /// Per-database role permissions. See [`BdbRole`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub bdb_roles: Option<Vec<BdbRole>>,
    /// List of cluster-wide role names.
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
    /// Retired built-in role alias.
    #[deprecated(note = "Redis Software does not register /v1/roles/builtin")]
    pub async fn built_in(&self) -> Result<Vec<RoleInfo>> {
        crate::error::unsupported_operation("list built-in roles")
    }

    /// Get user UIDs assigned to a role.
    ///
    /// Redis Software does not expose a role-scoped users route. Fetch the
    /// canonical users collection and filter its `role_uids` client-side.
    pub async fn users(&self, uid: u32) -> Result<Vec<u32>> {
        let users: Vec<crate::users::User> = self.client.get("/v1/users").await?;
        Ok(users
            .into_iter()
            .filter(|user| {
                user.role_uids
                    .as_ref()
                    .is_some_and(|role_uids| role_uids.contains(&uid))
            })
            .map(|user| user.uid)
            .collect())
    }
}
