//! Error types for git-bench CLI (native-only errors)

use thiserror::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// CLI-specific error types (includes native dependencies)
#[derive(Error, Debug)]
pub enum Error {
    #[error("Core error: {0}")]
    Core(#[from] git_bench_core::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Failed to write file: {path}")]
    FileWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("{0}")]
    Other(String),
}

