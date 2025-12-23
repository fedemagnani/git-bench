//! Benchmark data types compatible with git-bench JSON format
//!
//! These types mirror the main git-bench types but are WASM-compatible
//! (no native dependencies like git2, openssl, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single benchmark result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// The measured value (typically nanoseconds per iteration)
    pub value: f64,
    /// Unit of measurement
    pub unit: String,
    /// Range/variance if available (e.g., "+/- 5")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

/// Information about a commit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitInfo {
    /// Git commit SHA
    pub id: String,
    /// Commit message (first line)
    pub message: String,
    /// Commit timestamp
    pub timestamp: DateTime<Utc>,
    /// Commit URL (GitHub)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<AuthorInfo>,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorInfo {
    /// Author name
    pub name: String,
    /// Author email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// GitHub username
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// A benchmark run containing multiple benchmark results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkRun {
    /// Commit information
    pub commit: CommitInfo,
    /// When the benchmark was run
    pub date: DateTime<Utc>,
    /// Tool used (always "cargo" for this implementation)
    pub tool: String,
    /// Individual benchmark results
    pub benches: Vec<BenchmarkResult>,
}

/// Stored benchmark data for a repository
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BenchmarkData {
    /// Last update timestamp
    pub last_update: Option<DateTime<Utc>>,
    /// Repository information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    /// Benchmark entries grouped by benchmark suite name
    pub entries: HashMap<String, Vec<BenchmarkRun>>,
}

