//! Alert detection and notification system

use git_bench_core::CompareReport;

/// Alert configuration
#[derive(Debug, Clone, Default)]
pub struct AlertConfig {
    /// Whether to fail the workflow on alerts
    pub fail_on_alert: bool,
    /// Users to mention in alert comments (e.g., "@user1,@user2")
    pub alert_comment_cc_users: Option<String>,
}

/// Generate an alert message for a comparison report
pub fn generate_alert_message(report: &CompareReport, config: &AlertConfig) -> Option<String> {
    if report.alerts.is_empty() {
        return None;
    }

    let mut message = String::new();

    message.push_str("# ⚠️ Performance Alert\n\n");
    message.push_str("The following benchmarks show significant performance regressions:\n\n");

    message.push_str("| Benchmark | Previous | Current | Ratio | Change |\n");
    message.push_str("|-----------|----------|---------|-------|--------|\n");

    for alert in &report.alerts {
        message.push_str(&format!(
            "| {} | {:.2} {} | {:.2} {} | {:.2}x | +{:.1}% |\n",
            alert.name,
            alert.previous,
            alert.unit,
            alert.current,
            alert.unit,
            alert.ratio,
            alert.percentage_change
        ));
    }

    message.push('\n');

    if let Some(cc_users) = &config.alert_comment_cc_users {
        message.push_str(&format!("cc: {}\n", cc_users));
    }

    Some(message)
}

/// Check if the workflow should fail based on the report and config
pub fn should_fail(report: &CompareReport, config: &AlertConfig) -> bool {
    config.fail_on_alert && report.has_failures()
}

/// Format alerts for GitHub Actions workflow commands
pub fn format_github_actions_alert(report: &CompareReport) -> String {
    let mut output = String::new();

    for alert in &report.alerts {
        output.push_str(&format!(
            "::warning title=Performance Regression::Benchmark '{}' regressed by {:.1}% ({:.2} {} → {:.2} {})\n",
            alert.name,
            alert.percentage_change,
            alert.previous,
            alert.unit,
            alert.current,
            alert.unit
        ));
    }

    for failure in &report.failures {
        output.push_str(&format!(
            "::error title=Critical Performance Regression::Benchmark '{}' regressed by {:.1}%, exceeding threshold\n",
            failure.name, failure.percentage_change
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use git_bench_core::{BenchmarkResult, ComparisonResult};
    use std::collections::HashMap;

    fn make_comparison(name: &str, prev: f64, curr: f64) -> ComparisonResult {
        let prev_result = BenchmarkResult {
            name: name.to_string(),
            value: prev,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        };
        let curr_result = BenchmarkResult {
            name: name.to_string(),
            value: curr,
            unit: "ns/iter".to_string(),
            range: None,
            extra: HashMap::new(),
        };
        ComparisonResult::new(&prev_result, &curr_result)
    }

    #[test]
    fn test_generate_alert_message() {
        let report = CompareReport {
            comparisons: vec![make_comparison("test", 100.0, 250.0)],
            alerts: vec![make_comparison("test", 100.0, 250.0)],
            failures: vec![],
            new_benchmarks: vec![],
            removed_benchmarks: vec![],
        };

        let config = AlertConfig {
            fail_on_alert: false,
            alert_comment_cc_users: Some("@developer".to_string()),
        };

        let message = generate_alert_message(&report, &config).unwrap();
        assert!(message.contains("Performance Alert"));
        assert!(message.contains("test"));
        assert!(message.contains("@developer"));
    }

    #[test]
    fn test_no_alert_message_when_no_alerts() {
        let report = CompareReport {
            comparisons: vec![make_comparison("test", 100.0, 100.0)],
            alerts: vec![],
            failures: vec![],
            new_benchmarks: vec![],
            removed_benchmarks: vec![],
        };

        let config = AlertConfig::default();
        let message = generate_alert_message(&report, &config);
        assert!(message.is_none());
    }

    #[test]
    fn test_should_fail() {
        let report = CompareReport {
            comparisons: vec![],
            alerts: vec![],
            failures: vec![make_comparison("test", 100.0, 300.0)],
            new_benchmarks: vec![],
            removed_benchmarks: vec![],
        };

        let config_fail = AlertConfig {
            fail_on_alert: true,
            ..Default::default()
        };

        let config_no_fail = AlertConfig {
            fail_on_alert: false,
            ..Default::default()
        };

        assert!(should_fail(&report, &config_fail));
        assert!(!should_fail(&report, &config_no_fail));
    }

    #[test]
    fn test_format_github_actions_alert() {
        let report = CompareReport {
            comparisons: vec![],
            alerts: vec![make_comparison("slow_function", 100.0, 200.0)],
            failures: vec![],
            new_benchmarks: vec![],
            removed_benchmarks: vec![],
        };

        let output = format_github_actions_alert(&report);
        assert!(output.contains("::warning"));
        assert!(output.contains("slow_function"));
    }
}

