//! Error types for REST API operations

use std::time::Duration;
use thiserror::Error;

/// Errors returned by Redis Enterprise REST API operations.
#[derive(Error, Debug, Clone)]
pub enum RestError {
    /// The provided base URL could not be parsed.
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// The underlying HTTP request failed (network or transport error).
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    /// Authentication failed (invalid credentials).
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// The server returned a structured API error with HTTP status code.
    #[error("API error: {message} (code: {code})")]
    ApiError {
        /// HTTP status code returned by the server.
        code: u16,
        /// Server-provided error message.
        message: String,
    },

    /// Request or response could not be serialized/deserialized.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Response could not be parsed into the expected shape.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Could not establish a connection to the REST API.
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// TLS handshake or certificate validation failed.
    #[error("TLS certificate error: {0}")]
    TlsError(String),

    /// Client has not been connected to the REST API yet.
    #[error("Not connected to REST API")]
    NotConnected,

    /// Client-side validation of input parameters failed.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// The requested resource was not found (HTTP 404).
    #[error("Resource not found")]
    NotFound,

    /// The request was unauthorized (HTTP 401).
    #[error("Unauthorized")]
    Unauthorized,

    /// The server returned a 5xx error.
    #[error("Server error: {0}")]
    ServerError(String),

    /// The request timed out.
    #[error("Request timed out")]
    Timeout,

    /// The client has been rate limited by the server (HTTP 429).
    #[error("Rate limited{}", .retry_after.map(|d| format!(" (retry after {:?})", d)).unwrap_or_default())]
    RateLimited {
        /// Optional retry-after duration suggested by the server.
        retry_after: Option<Duration>,
    },

    /// The resource already exists (HTTP 409).
    #[error("Resource already exists")]
    AlreadyExists,

    /// A conflict occurred while processing the request (HTTP 409).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The SDK method represents an operation that supported Redis Software
    /// versions do not register.
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// The cluster is busy or temporarily unavailable (HTTP 503).
    #[error("Cluster is busy or unavailable")]
    ClusterBusy,
}

impl From<reqwest::Error> for RestError {
    fn from(err: reqwest::Error) -> Self {
        RestError::RequestFailed(err.to_string())
    }
}

impl From<serde_json::Error> for RestError {
    fn from(err: serde_json::Error) -> Self {
        RestError::SerializationError(err.to_string())
    }
}

impl RestError {
    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, RestError::NotFound)
            || matches!(self, RestError::ApiError { code, .. } if *code == 404)
    }

    /// Check if this is an authentication error
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, RestError::Unauthorized)
            || matches!(self, RestError::AuthenticationFailed)
            || matches!(self, RestError::ApiError { code, .. } if *code == 401)
    }

    /// Check if this is a server error
    pub fn is_server_error(&self) -> bool {
        matches!(self, RestError::ServerError(_))
            || matches!(self, RestError::ApiError { code, .. } if *code >= 500)
    }

    /// Check if this is a timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self, RestError::Timeout)
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, RestError::RateLimited { .. })
            || matches!(self, RestError::ApiError { code, .. } if *code == 429)
    }

    /// Check if this is a conflict/already exists error
    pub fn is_conflict(&self) -> bool {
        matches!(self, RestError::AlreadyExists)
            || matches!(self, RestError::Conflict(_))
            || matches!(self, RestError::ApiError { code, .. } if *code == 409)
    }

    /// Check if this is a cluster busy error
    pub fn is_cluster_busy(&self) -> bool {
        matches!(self, RestError::ClusterBusy)
            || matches!(self, RestError::ApiError { code, .. } if *code == 503)
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        self.is_timeout()
            || self.is_rate_limited()
            || self.is_cluster_busy()
            || self.is_server_error()
    }

    /// Check if this is a bad request / validation error
    pub fn is_bad_request(&self) -> bool {
        matches!(self, RestError::ValidationError(_))
            || matches!(self, RestError::ApiError { code, .. } if *code == 400)
    }
}

/// Result alias used throughout the crate for Redis Enterprise REST API operations.
pub type Result<T> = std::result::Result<T, RestError>;

/// Return a consistent local error for a retired SDK operation.
pub(crate) fn unsupported_operation<T>(operation: &str) -> Result<T> {
    Err(RestError::UnsupportedOperation(operation.to_string()))
}
