//! Response helpers for building wiremock responses
//!
//! # Example
//!
//! ```ignore
//! use redis_enterprise::testing::responses;
//! use redis_enterprise::testing::fixtures::DatabaseFixture;
//!
//! // Success responses
//! let ok = responses::success(json!({"status": "ok"}));
//! let created = responses::created(DatabaseFixture::new(1, "my-db").build());
//! let no_content = responses::no_content();
//!
//! // Error responses
//! let not_found = responses::not_found("Database not found");
//! let unauthorized = responses::unauthorized();
//! let conflict = responses::conflict("Resource already exists");
//! ```

use serde_json::{Value, json};
use std::time::Duration;
use wiremock::ResponseTemplate;

/// Create a 200 OK response with JSON body
pub fn success(body: impl Into<Value>) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body.into())
}

/// Create a 201 Created response with JSON body
pub fn created(body: impl Into<Value>) -> ResponseTemplate {
    ResponseTemplate::new(201).set_body_json(body.into())
}

/// Create a 204 No Content response
pub fn no_content() -> ResponseTemplate {
    ResponseTemplate::new(204)
}

/// Create a 400 Bad Request response
pub fn bad_request(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(400).set_body_json(json!({
        "error": message,
        "code": 400
    }))
}

/// Create a 401 Unauthorized response
pub fn unauthorized() -> ResponseTemplate {
    ResponseTemplate::new(401).set_body_json(json!({
        "error": "Unauthorized",
        "code": 401
    }))
}

/// Create a 404 Not Found response
pub fn not_found(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(404).set_body_json(json!({
        "error": message,
        "code": 404
    }))
}

/// Create a 409 Conflict response
pub fn conflict(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(409).set_body_json(json!({
        "error": message,
        "code": 409
    }))
}

/// Create a 429 Rate Limited response
pub fn rate_limited(retry_after: Option<Duration>) -> ResponseTemplate {
    let mut response = ResponseTemplate::new(429).set_body_json(json!({
        "error": "Rate limited",
        "code": 429
    }));

    if let Some(duration) = retry_after {
        response = response.insert_header("Retry-After", duration.as_secs().to_string());
    }

    response
}

/// Create a 500 Internal Server Error response
pub fn server_error(message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(500).set_body_json(json!({
        "error": message,
        "code": 500
    }))
}

/// Create a 503 Service Unavailable (cluster busy) response
pub fn cluster_busy() -> ResponseTemplate {
    ResponseTemplate::new(503).set_body_json(json!({
        "error": "Cluster is busy or unavailable",
        "code": 503
    }))
}

/// Create a custom error response with any status code
pub fn error(code: u16, message: impl Into<String>) -> ResponseTemplate {
    let message = message.into();
    ResponseTemplate::new(code).set_body_json(json!({
        "error": message,
        "code": code
    }))
}

/// Create a response with a delay (for testing timeouts)
pub fn delayed(response: ResponseTemplate, delay: Duration) -> ResponseTemplate {
    response.set_delay(delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let _resp = success(json!({"status": "ok"}));
    }

    #[test]
    fn test_error_responses() {
        let _not_found = not_found("Resource not found");
        let _unauthorized = unauthorized();
        let _conflict = conflict("Already exists");
        let _rate_limited = rate_limited(Some(Duration::from_secs(60)));
        let _server_error = server_error("Internal error");
        let _cluster_busy = cluster_busy();
    }
}
