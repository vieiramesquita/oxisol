//! Error types for `oxisol-io`.

use thiserror::Error;

/// Unified I/O and parsing error type.
#[derive(Debug, Error)]
pub enum IoError {
    /// Filesystem I/O failure.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Parse error with contextual message.
    #[error("parse error: {0}")]
    Parse(String),
    /// Invalid NetCDF-3 binary structure or unsupported format.
    #[error("NetCDF-3 error: {0}")]
    NetCdf(String),
    /// JSON checkpoint serialization/deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience result alias.
pub type Result<T> = std::result::Result<T, IoError>;
