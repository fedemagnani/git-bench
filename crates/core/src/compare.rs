//! Benchmark comparison logic

use crate::data::{BenchmarkResult, BenchmarkRun, ComparisonResult};
use std::collections::HashMap;

/// Configuration for benchmark comparison
#[derive(Debug, Clone)]
pub struct CompareConfig {
    /// Alert threshold as a ratio (e.g., 2.0 means 200%)
    pub alert_threshold: f64,
    /// Fail threshold as a ratio (defaults to alert_threshold)
    pub fail_threshold: Option<f64>,
}

impl Default for CompareConfig {
    fn default() -> Self {
        Self {
            alert_threshold: 2.0, // 200%
            fail_threshold: None,
        }
    }
}

impl CompareConfig {
    /// Create config from percentage strings (e.g., "150%")
    pub fn from_percentages(alert: &str, fail: Option<&str>) -> Result<Self, String> {
        let alert_threshold = parse_percentage(alert)?;
        let fail_threshold = fail.map(parse_percentage).transpose()?;

        if let Some(ft) = fail_threshold {
            if ft < alert_threshold {
                return Err("fail-threshold must be >= alert-threshold".to_string());
            }
        }

        Ok(Self {
            alert_threshold,
            fail_threshold,
        })
    }

    /// Get the effective fail threshold
    pub fn effective_fail_threshold(&self) -> f64 {
        self.fail_threshold.unwrap_or(self.alert_threshold)
    }
}

/// Parse a percentage string like "150%" to a ratio (1.5)
fn parse_percentage(s: &str) -> Result<f64, String> {
    let s = s.trim();
    let s = s.strip_suffix('%').unwrap_or(s);
    let value: f64 = s.parse().map_err(|_| format!("Invalid percentage: {}", s))?;
    Ok(value / 100.0)
}

/// Result of comparing benchmark runs
#[derive(Debug, Clone)]
pub struct CompareReport {
    /// Individual comparison results
    pub comparisons: Vec<ComparisonResult>,
    /// Benchmarks that triggered alerts
    pub alerts: Vec<ComparisonResult>,
    /// Benchmarks that should cause failure
    pub failures: Vec<ComparisonResult>,
    /// New benchmarks (no previous data)
    pub new_benchmarks: Vec<BenchmarkResult>,
    /// Removed benchmarks (in previous but not current)
    pub removed_benchmarks: Vec<BenchmarkResult>,
}

impl CompareReport {
    /// Check if any alerts were triggered
    pub fn has_alerts(&self) -> bool {
        !self.alerts.is_empty()
    }

    /// Check if any failures were triggered
    pub fn has_failures(&self) -> bool {
        !self.failures.is_empty()
    }

    /// Generate a summary string
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        if self.comparisons.is_empty() && self.new_benchmarks.is_empty() {
            return "No benchmark comparisons available.".to_string();
        }

        lines.push("## Benchmark Comparison Report\n".to_string());

        if !self.comparisons.is_empty() {
            lines.push("### Comparisons\n".to_string());
            lines.push("| Benchmark | Previous | Current | Change |".to_string());
            lines.push("|-----------|----------|---------|--------|".to_string());

            for comp in &self.comparisons {
                let change_str = if comp.percentage_change >= 0.0 {
                    format!("+{:.2}%", comp.percentage_change)
                } else {
                    format!("{:.2}%", comp.percentage_change)
                };

                let indicator = if comp.is_regression {
                    "🔴"
                } else if comp.percentage_change < -5.0 {
                    "🟢"
                } else {
                    "⚪"
                };

                lines.push(format!(
                    "| {} | {:.2} {} | {:.2} {} | {} {} |",
                    comp.name,
                    comp.previous,
                    comp.unit,
                    comp.current,
                    comp.unit,
                    indicator,
                    change_str
                ));
            }
            lines.push(String::new());
        }

        if !self.new_benchmarks.is_empty() {
            lines.push("### New Benchmarks\n".to_string());
            for bench in &self.new_benchmarks {
                lines.push(format!("- **{}**: {:.2} {}", bench.name, bench.value, bench.unit));
            }
            lines.push(String::new());
        }

        if !self.removed_benchmarks.is_empty() {
            lines.push("### Removed Benchmarks\n".to_string());
            for bench in &self.removed_benchmarks {
                lines.push(format!("- **{}** (was {:.2} {})", bench.name, bench.value, bench.unit));
            }
            lines.push(String::new());
        }

        if !self.alerts.is_empty() {
            lines.push("### ⚠️ Performance Alerts\n".to_string());
            for alert in &self.alerts {
                lines.push(format!(
                    "- **{}**: {:.2}% regression ({:.2} {} → {:.2} {})",
                    alert.name,
                    alert.percentage_change,
                    alert.previous,
                    alert.unit,
                    alert.current,
                    alert.unit
                ));
            }
            lines.push(String::new());
        }

        if !self.failures.is_empty() {
            lines.push("### 🚨 Critical Regressions (Failing)\n".to_string());
            for failure in &self.failures {
                lines.push(format!(
                    "- **{}**: {:.2}% regression exceeds threshold",
                    failure.name, failure.percentage_change
                ));
            }
        }

        lines.join("\n")
    }

    /// Generate a short summary for commit comments
    pub fn short_summary(&self) -> String {
        if self.comparisons.is_empty() && self.new_benchmarks.is_empty() {
            return "No benchmark data to compare.".to_string();
        }

        let mut parts = Vec::new();

        let regressions: Vec<_> = self.comparisons.iter().filter(|c| c.is_regression).collect();
        let improvements: Vec<_> = self
            .comparisons
            .iter()
            .filter(|c| !c.is_regression && c.percentage_change < -5.0)
            .collect();

        if !regressions.is_empty() {
            parts.push(format!("🔴 {} regression(s)", regressions.len()));
        }

        if !improvements.is_empty() {
            parts.push(format!("🟢 {} improvement(s)", improvements.len()));
        }

        if !self.new_benchmarks.is_empty() {
            parts.push(format!("🆕 {} new benchmark(s)", self.new_benchmarks.len()));
        }

        if parts.is_empty() {
            "⚪ No significant changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Compare two benchmark runs
pub fn compare_runs(
    previous: &BenchmarkRun,
    current: &BenchmarkRun,
    config: &CompareConfig,
) -> CompareReport {
    let mut comparisons = Vec::new();
    let mut alerts = Vec::new();
    let mut failures = Vec::new();
    let mut new_benchmarks = Vec::new();
    let mut removed_benchmarks = Vec::new();

    // Build a map of previous benchmarks
    let prev_map: HashMap<&str, &BenchmarkResult> = previous
        .benches
        .iter()
        .map(|b| (b.name.as_str(), b))
        .collect();

    // Build a map of current benchmarks
    let curr_map: HashMap<&str, &BenchmarkResult> = current
        .benches
        .iter()
        .map(|b| (b.name.as_str(), b))
        .collect();

    // Compare benchmarks that exist in both
    for curr_bench in &current.benches {
        if let Some(prev_bench) = prev_map.get(curr_bench.name.as_str()) {
            let comparison = ComparisonResult::new(prev_bench, curr_bench);

            // Check for alerts
            if comparison.ratio >= config.alert_threshold {
                alerts.push(comparison.clone());
            }

            // Check for failures
            if comparison.ratio >= config.effective_fail_threshold() {
                failures.push(comparison.clone());
            }

            comparisons.push(comparison);
        } else {
            // New benchmark
            new_benchmarks.push(curr_bench.clone());
        }
    }

    // Find removed benchmarks
    for prev_bench in &previous.benches {
        if !curr_map.contains_key(prev_bench.name.as_str()) {
            removed_benchmarks.push(prev_bench.clone());
        }
    }

    CompareReport {
        comparisons,
        alerts,
        failures,
        new_benchmarks,
        removed_benchmarks,
    }
}

/// Compare current benchmarks against previous data
pub fn compare_with_previous(
    current_benches: &[BenchmarkResult],
    previous_run: Option<&BenchmarkRun>,
    config: &CompareConfig,
) -> CompareReport {
    match previous_run {
        Some(prev) => {
            // Create a temporary current run for comparison
            let current_run = BenchmarkRun {
                commit: prev.commit.clone(), // Placeholder
                date: chrono::Utc::now(),
                tool: "cargo".to_string(),
                benches: current_benches.to_vec(),
            };
            compare_runs(prev, &current_run, config)
        }
        None => {
            // No previous data - all benchmarks are new
            CompareReport {
                comparisons: Vec::new(),
                alerts: Vec::new(),
                failures: Vec::new(),
                new_benchmarks: current_benches.to_vec(),
                removed_benchmarks: Vec::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AuthorInfo, CommitInfo};
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_bench(name: &str, value: f64) -> BenchmarkResult {
        BenchmarkResult {
            name: name.to_string(),
            value,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        }
    }

    fn make_run(benches: Vec<BenchmarkResult>) -> BenchmarkRun {
        BenchmarkRun {
            commit: CommitInfo {
                id: "abc123".to_string(),
                message: "test commit".to_string(),
                timestamp: Utc::now(),
                url: None,
                author: Some(AuthorInfo {
                    name: "Test".to_string(),
                    email: None,
                    username: None,
                }),
            },
            date: Utc::now(),
            tool: "cargo".to_string(),
            benches,
        }
    }

    #[test]
    fn test_compare_runs_regression() {
        let prev = make_run(vec![make_bench("test", 100.0)]);
        let curr = make_run(vec![make_bench("test", 250.0)]);

        let config = CompareConfig::default();
        let report = compare_runs(&prev, &curr, &config);

        assert_eq!(report.comparisons.len(), 1);
        assert!(report.comparisons[0].is_regression);
        assert_eq!(report.comparisons[0].ratio, 2.5);
        assert!(report.has_alerts()); // 250% > 200% threshold
    }

    #[test]
    fn test_compare_runs_improvement() {
        let prev = make_run(vec![make_bench("test", 100.0)]);
        let curr = make_run(vec![make_bench("test", 50.0)]);

        let config = CompareConfig::default();
        let report = compare_runs(&prev, &curr, &config);

        assert_eq!(report.comparisons.len(), 1);
        assert!(!report.comparisons[0].is_regression);
        assert_eq!(report.comparisons[0].ratio, 0.5);
        assert!(!report.has_alerts());
    }

    #[test]
    fn test_compare_runs_new_benchmark() {
        let prev = make_run(vec![make_bench("old", 100.0)]);
        let curr = make_run(vec![make_bench("old", 100.0), make_bench("new", 50.0)]);

        let config = CompareConfig::default();
        let report = compare_runs(&prev, &curr, &config);

        assert_eq!(report.new_benchmarks.len(), 1);
        assert_eq!(report.new_benchmarks[0].name, "new");
    }

    #[test]
    fn test_compare_runs_removed_benchmark() {
        let prev = make_run(vec![make_bench("old", 100.0), make_bench("removed", 50.0)]);
        let curr = make_run(vec![make_bench("old", 100.0)]);

        let config = CompareConfig::default();
        let report = compare_runs(&prev, &curr, &config);

        assert_eq!(report.removed_benchmarks.len(), 1);
        assert_eq!(report.removed_benchmarks[0].name, "removed");
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("150%").unwrap(), 1.5);
        assert_eq!(parse_percentage("200%").unwrap(), 2.0);
        assert_eq!(parse_percentage("100%").unwrap(), 1.0);
        assert_eq!(parse_percentage("50%").unwrap(), 0.5);
        assert_eq!(parse_percentage("150").unwrap(), 1.5);
    }
}


