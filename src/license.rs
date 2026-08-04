//! License management and validation
//!
//! ## Overview
//! - Query license status
//! - Update license keys
//! - Monitor license expiration

use crate::client::RestClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use typed_builder::TypedBuilder;

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct License {
    /// License key - the actual field name returned by API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// License string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// License type (trial, commercial, etc.)
    #[serde(rename = "type")]
    pub type_: Option<String>,

    /// Mark license expired or not
    pub expired: bool,

    /// License activation date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_date: Option<String>,

    /// License expiration date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,

    /// The cluster name as appears in the license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_name: Option<String>,

    /// Owner of license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// Shards limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shards_limit: Option<u32>,

    /// Amount of RAM shards in use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_shards_in_use: Option<u32>,

    /// Amount of RAM shards allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_shards_limit: Option<u32>,

    /// Amount of flash shards in use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash_shards_in_use: Option<u32>,

    /// Amount of flash shards allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flash_shards_limit: Option<u32>,

    /// Node limit (deprecated in favor of shards_limit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_limit: Option<u32>,

    /// List of features supported by license
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<String>>,

    /// Additive or version-specific license fields.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub additional_fields: BTreeMap<String, Value>,
}

/// License update request
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
pub struct LicenseUpdateRequest {
    /// New license key to install
    #[builder(setter(into))]
    pub license: String,
}

/// License usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseUsage {
    /// Number of shards currently in use
    pub shards_used: u32,
    /// Maximum number of shards allowed by license
    pub shards_limit: u32,
    /// Number of nodes currently in use
    pub nodes_used: u32,
    /// Maximum number of nodes allowed by license
    pub nodes_limit: u32,
    /// Amount of RAM currently in use (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_used: Option<u64>,
    /// Maximum amount of RAM allowed by license (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram_limit: Option<u64>,
}

/// License handler
pub struct LicenseHandler {
    client: RestClient,
}

impl LicenseHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        LicenseHandler { client }
    }

    /// Get current license information
    pub async fn get(&self) -> Result<License> {
        self.client.get("/v1/license").await
    }

    /// Update license
    ///
    /// The Redis Enterprise API may return 200 with an empty body for this
    /// endpoint, so we use put_action (which tolerates empty responses) and
    /// follow up with a GET to return the installed license.
    pub async fn update(&self, request: LicenseUpdateRequest) -> Result<License> {
        self.client.put_action("/v1/license", &request).await?;
        self.get().await
    }

    /// Retired license-usage helper.
    #[deprecated(note = "Redis Software does not register /v1/license/usage")]
    pub async fn usage(&self) -> Result<LicenseUsage> {
        crate::error::unsupported_operation("get license usage")
    }

    /// Retired standalone license-validation helper.
    #[deprecated(note = "install with update and inspect the canonical license resource")]
    pub async fn validate(&self, _license_key: &str) -> Result<License> {
        crate::error::unsupported_operation("validate license without installing it")
    }

    /// Get the cluster license through the canonical license endpoint.
    pub async fn cluster_license(&self) -> Result<License> {
        self.get().await
    }
}
