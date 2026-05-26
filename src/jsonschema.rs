//! JSON Schema definitions for API validation
//!
//! The Redis Enterprise REST API documents `GET /v1/jsonschema` only. Earlier
//! revisions of this module exposed per-resource accessors and a validate
//! endpoint at paths the docs never define; those have been removed.

use crate::client::RestClient;
use crate::error::Result;

/// JSON Schema handler for API schema definitions
pub struct JsonSchemaHandler {
    client: RestClient,
}

impl JsonSchemaHandler {
    /// Create a new handler bound to the given REST client.
    pub fn new(client: RestClient) -> Self {
        JsonSchemaHandler { client }
    }

    /// Get all available schemas - GET /v1/jsonschema
    pub async fn list(&self) -> Result<Vec<String>> {
        self.client.get("/v1/jsonschema").await
    }
}
