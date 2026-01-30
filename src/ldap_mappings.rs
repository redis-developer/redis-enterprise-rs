//! LDAP integration and role mapping
//!
//! ## Overview
//! - Configure LDAP mappings
//! - Map LDAP groups to roles
//! - Query mapping status

use crate::error::Result;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// LDAP mapping information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapMapping {
    /// LDAP-mapping's unique uid
    pub uid: u32,
    /// Role's name
    pub name: String,
    /// An LDAP group's distinguished name
    pub dn: String,
    /// Role identifier (deprecated, use role_uids instead)
    pub role: String,
    /// Email address that (if set) is used for alerts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// List of role uids associated with the LDAP group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_uids: Option<Vec<u32>>,
}

/// Create or update LDAP mapping request
#[derive(Debug, Serialize, Deserialize, TypedBuilder)]
pub struct CreateLdapMappingRequest {
    /// Role's name for the LDAP mapping
    #[builder(setter(into))]
    pub name: String,
    /// LDAP group's distinguished name to map
    #[builder(setter(into))]
    pub dn: String,
    /// Role identifier (deprecated, use role_uids instead)
    #[builder(setter(into))]
    pub role: String,
    /// Email address for alert notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub email: Option<String>,
    /// List of role UIDs to associate with this LDAP group
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub role_uids: Option<Vec<u32>>,
}

/// LDAP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapConfig {
    /// Whether LDAP authentication is enabled
    pub enabled: bool,
    /// List of LDAP servers to connect to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<LdapServer>>,
    /// Cache refresh interval in seconds for LDAP data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_refresh_interval: Option<u32>,
    /// LDAP query suffix for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_query_suffix: Option<String>,
    /// LDAP query suffix for authorization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_query_suffix: Option<String>,
    /// Distinguished name for LDAP binding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_dn: Option<String>,
    /// Password for LDAP binding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_pass: Option<String>,
}

/// LDAP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapServer {
    /// LDAP server hostname or IP address
    pub host: String,
    /// LDAP server port number (typically 389 for plain, 636 for SSL)
    pub port: u16,
    /// Whether to use TLS encryption for the LDAP connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_tls: Option<bool>,
    /// Whether to use STARTTLS for upgrading the connection to TLS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starttls: Option<bool>,
}

define_handler!(
    /// LDAP mapping handler
    pub struct LdapMappingHandler;
);

impl_crud!(LdapMappingHandler {
    list => LdapMapping, "/v1/ldap_mappings";
    get(u32) => LdapMapping, "/v1/ldap_mappings/{}";
    delete(u32), "/v1/ldap_mappings/{}";
    create(CreateLdapMappingRequest) => LdapMapping, "/v1/ldap_mappings";
    update(u32, CreateLdapMappingRequest) => LdapMapping, "/v1/ldap_mappings/{}";
});

// Custom methods for LDAP configuration
impl LdapMappingHandler {
    /// Get LDAP configuration
    pub async fn get_config(&self) -> Result<LdapConfig> {
        self.client.get("/v1/cluster/ldap").await
    }

    /// Update LDAP configuration
    pub async fn update_config(&self, config: LdapConfig) -> Result<LdapConfig> {
        self.client.put("/v1/cluster/ldap", &config).await
    }
}
