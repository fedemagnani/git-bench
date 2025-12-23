//! Error types for git-bench

use thiserror::Error;

/// Result type alias for git-bench operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for git-bench
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

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("GitHub API error: {0}")]
    GitHubError(String),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Benchmark alert: performance regression detected - {0}")]
    AlertError(String),

    #[error("Template error: {0}")]
    TemplateError(#[from] minijinja::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("{0}")]
    Other(String),
}

