//! Redis ACL management
//!
//! ## Overview
//! - Configure Redis ACLs
//! - Manage user permissions
//! - Query ACL rules

use crate::error::Result;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Redis ACL information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisAcl {
    pub uid: u32,
    pub name: String,
    pub acl: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// List of database UIDs this ACL is associated with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bdbs: Option<Vec<u32>>,
}

/// Create or update Redis ACL request
#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct CreateRedisAclRequest {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub acl: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub description: Option<String>,
}

define_handler!(
    /// Redis ACL handler for managing ACLs
    pub struct RedisAclHandler;
);

impl_crud!(RedisAclHandler {
    list => RedisAcl, "/v1/redis_acls";
    get(u32) => RedisAcl, "/v1/redis_acls/{}";
    delete(u32), "/v1/redis_acls/{}";
    create(CreateRedisAclRequest) => RedisAcl, "/v1/redis_acls";
    update(u32, CreateRedisAclRequest) => RedisAcl, "/v1/redis_acls/{}";
});

/// Alias for backwards compatibility and intuitive plural naming
pub type RedisAclsHandler = RedisAclHandler;

// Custom methods
impl RedisAclHandler {
    /// Validate an ACL payload - POST /v1/redis_acls/validate
    pub async fn validate(&self, body: CreateRedisAclRequest) -> Result<AclValidation> {
        self.client.post("/v1/redis_acls/validate", &body).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclValidation {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
