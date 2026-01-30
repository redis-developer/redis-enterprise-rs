//! Error types for REST API operations

use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum RestError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("API error: {message} (code: {code})")]
    ApiError { code: u16, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Not connected to REST API")]
    NotConnected,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Request timed out")]
    Timeout,

    #[error("Rate limited{}", .retry_after.map(|d| format!(" (retry after {:?})", d)).unwrap_or_default())]
    RateLimited { retry_after: Option<Duration> },

    #[error("Resource already exists")]
    AlreadyExists,

    #[error("Conflict: {0}")]
    Conflict(String),

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
}

pub type Result<T> = std::result::Result<T, RestError>;
