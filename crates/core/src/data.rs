//! Data structures for benchmark results and storage

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

impl BenchmarkData {
    /// Create a new empty benchmark data store
    pub fn new() -> Self {
        Self::default()
    }

    /// Load benchmark data from a JSON file
    pub fn load_from_file(path: &std::path::Path) -> crate::error::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| crate::error::Error::FileReadError {
                path: path.display().to_string(),
                source: e,
            })?;

        let data: Self = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Save benchmark data to a JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> crate::error::Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| crate::error::Error::FileWriteError {
                path: parent.display().to_string(),
                source: e,
            })?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content).map_err(|e| crate::error::Error::FileWriteError {
            path: path.display().to_string(),
            source: e,
        })?;

        Ok(())
    }

    /// Add a benchmark run to the data store
    pub fn add_run(&mut self, suite_name: &str, run: BenchmarkRun, max_items: Option<usize>) {
        let entries = self.entries.entry(suite_name.to_string()).or_default();
        entries.push(run);

        // Trim old entries if max_items is set
        if let Some(max) = max_items {
            while entries.len() > max {
                entries.remove(0);
            }
        }

        self.last_update = Some(Utc::now());
    }

    /// Get the most recent run for a suite
    pub fn get_latest_run(&self, suite_name: &str) -> Option<&BenchmarkRun> {
        self.entries.get(suite_name).and_then(|runs| runs.last())
    }

    /// Get the previous run (before the most recent) for a suite
    pub fn get_previous_run(&self, suite_name: &str) -> Option<&BenchmarkRun> {
        self.entries.get(suite_name).and_then(|runs| {
            if runs.len() >= 2 {
                runs.get(runs.len() - 2)
            } else {
                None
            }
        })
    }
}

/// Comparison result between two benchmark values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Benchmark name
    pub name: String,
    /// Previous value
    pub previous: f64,
    /// Current value
    pub current: f64,
    /// Ratio (current / previous)
    pub ratio: f64,
    /// Percentage change
    pub percentage_change: f64,
    /// Whether this is a regression (worse performance)
    pub is_regression: bool,
    /// Unit of measurement
    pub unit: String,
}

impl ComparisonResult {
    /// Create a comparison between two benchmark results
    /// For cargo benchmarks, lower is better (smaller time)
    pub fn new(previous: &BenchmarkResult, current: &BenchmarkResult) -> Self {
        let ratio = if previous.value != 0.0 {
            current.value / previous.value
        } else {
            1.0
        };

        let percentage_change = (ratio - 1.0) * 100.0;

        // For time-based benchmarks, higher ratio means regression (slower)
        let is_regression = ratio > 1.0;

        Self {
            name: current.name.clone(),
            previous: previous.value,
            current: current.value,
            ratio,
            percentage_change,
            is_regression,
            unit: current.unit.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_result() {
        let prev = BenchmarkResult {
            name: "test_bench".to_string(),
            value: 100.0,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        };

        let curr = BenchmarkResult {
            name: "test_bench".to_string(),
            value: 150.0,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        };

        let comparison = ComparisonResult::new(&prev, &curr);

        assert_eq!(comparison.ratio, 1.5);
        assert_eq!(comparison.percentage_change, 50.0);
        assert!(comparison.is_regression);
    }

    #[test]
    fn test_comparison_improvement() {
        let prev = BenchmarkResult {
            name: "test_bench".to_string(),
            value: 100.0,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        };

        let curr = BenchmarkResult {
            name: "test_bench".to_string(),
            value: 80.0,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        };

        let comparison = ComparisonResult::new(&prev, &curr);

        assert_eq!(comparison.ratio, 0.8);
        assert!(!comparison.is_regression);
    }
}

