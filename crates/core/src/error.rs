//! Error types for git-bench-core (WASM-compatible)

use thiserror::Error;

/// Result type alias for git-bench-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that work in both native and WASM environments
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to parse benchmark output: {0}")]
    ParseError(String),

    #[error("Failed to read file: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWriteError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Other(String),
}





