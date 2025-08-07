//! Error types for the Orama client.

use thiserror::Error;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, OramaError>;

/// Main error type for Orama operations
#[derive(Error, Debug)]
pub enum OramaError {
    /// HTTP client errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Authentication errors
    #[error("Authentication failed: {message}")]
    Auth { message: String },

    /// API errors returned from Orama
    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Stream processing errors
    #[error("Stream error: {message}")]
    Stream { message: String },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing errors
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Generic errors
    #[error("Error: {message}")]
    Generic { message: String },
}

impl OramaError {
    /// Create a new authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    /// Create a new API error
    pub fn api<S: Into<String>>(status: u16, message: S) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Create a new stream error
    pub fn stream<S: Into<String>>(message: S) -> Self {
        Self::Stream {
            message: message.into(),
        }
    }

    /// Create a new generic error
    pub fn generic<S: Into<String>>(message: S) -> Self {
        Self::Generic {
            message: message.into(),
        }
    }
}
