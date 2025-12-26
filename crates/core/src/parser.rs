//! Parser for cargo bench output
//!
//! Cargo bench output format (libtest):
//! ```text
//! test bench_name ... bench:       1,234 ns/iter (+/- 56)
//! ```
//!
//! Criterion output format:
//! ```text
//! bench_name          time:   [1.2345 µs 1.2456 µs 1.2567 µs]
//! ```

use crate::data::BenchmarkResult;
use crate::error::{Error, Result};
use regex::Regex;
use std::collections::HashMap;

/// Parser for cargo benchmark output
pub struct CargoParser {
    /// Regex for libtest bench format
    libtest_regex: Regex,
    /// Regex for criterion format
    criterion_regex: Regex,
}

impl CargoParser {
    /// Create a new cargo parser
    pub fn new() -> Result<Self> {
        // libtest format: test bench_name ... bench:       1,234 ns/iter (+/- 56)
        let libtest_regex = Regex::new(
            r"test\s+(\S+)\s+\.\.\.\s+bench:\s+([\d,]+)\s+(\w+/\w+)(?:\s+\(\+/-\s+([\d,]+)\))?",
        )?;

        // Criterion format: bench_name          time:   [1.2345 µs 1.2456 µs 1.2567 µs]
        // We capture the middle value (mean/median)
        let criterion_regex = Regex::new(
            r"^(\S+)\s+time:\s+\[([\d.]+)\s*(\w+)\s+([\d.]+)\s*(\w+)\s+([\d.]+)\s*(\w+)\]",
        )?;

        Ok(Self {
            libtest_regex,
            criterion_regex,
        })
    }

    /// Parse cargo bench output and return benchmark results
    pub fn parse(&self, output: &str) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        // Track which benchmarks we've already parsed (to handle multi-line criterion output)
        let mut seen: HashMap<String, bool> = HashMap::new();

        for line in output.lines() {
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Try libtest format first
            if let Some(result) = self.parse_libtest_line(line) {
                if !seen.contains_key(&result.name) {
                    seen.insert(result.name.clone(), true);
                    results.push(result);
                }
                continue;
            }

            // Try criterion format
            if let Some(result) = self.parse_criterion_line(line) {
                if !seen.contains_key(&result.name) {
                    seen.insert(result.name.clone(), true);
                    results.push(result);
                }
            }
        }

        if results.is_empty() {
            return Err(Error::ParseError(
                "No benchmark results found in output. Make sure you're running `cargo bench` and the output is correct.".to_string()
            ));
        }

        Ok(results)
    }

    /// Parse a single libtest bench line
    fn parse_libtest_line(&self, line: &str) -> Option<BenchmarkResult> {
        let captures = self.libtest_regex.captures(line)?;

        let name = captures.get(1)?.as_str().to_string();
        let value_str = captures.get(2)?.as_str().replace(',', "");
        let unit = captures.get(3)?.as_str().to_string();
        let range = captures
            .get(4)
            .map(|m| format!("+/- {}", m.as_str().replace(',', "")));

        let value: f64 = value_str.parse().ok()?;

        Some(BenchmarkResult {
            name,
            value,
            unit,
            range,
            extra: HashMap::new(),
        })
    }

    /// Parse a single criterion bench line
    fn parse_criterion_line(&self, line: &str) -> Option<BenchmarkResult> {
        let captures = self.criterion_regex.captures(line)?;

        let name = captures.get(1)?.as_str().to_string();

        // Get the middle value (median/mean)
        let value_str = captures.get(4)?.as_str();
        let unit = captures.get(5)?.as_str();

        let value: f64 = value_str.parse().ok()?;

        // Convert unit to nanoseconds for consistency
        let (normalized_value, normalized_unit) = self.normalize_time_unit(value, unit);

        // Calculate range from low and high values
        let low: f64 = captures.get(2)?.as_str().parse().ok()?;
        let high: f64 = captures.get(6)?.as_str().parse().ok()?;
        let (low_norm, _) = self.normalize_time_unit(low, unit);
        let (high_norm, _) = self.normalize_time_unit(high, unit);
        let range = Some(format!(
            "[{:.4} {}, {:.4} {}]",
            low_norm, normalized_unit, high_norm, normalized_unit
        ));

        let mut extra = HashMap::new();
        extra.insert("low".to_string(), format!("{:.4}", low_norm));
        extra.insert("high".to_string(), format!("{:.4}", high_norm));

        Some(BenchmarkResult {
            name,
            value: normalized_value,
            unit: normalized_unit,
            range,
            extra,
        })
    }

    /// Normalize time units to nanoseconds
    fn normalize_time_unit(&self, value: f64, unit: &str) -> (f64, String) {
        match unit {
            "ps" => (value / 1000.0, "ns".to_string()),
            "ns" => (value, "ns".to_string()),
            "µs" | "us" => (value * 1000.0, "ns".to_string()),
            "ms" => (value * 1_000_000.0, "ns".to_string()),
            "s" => (value * 1_000_000_000.0, "ns".to_string()),
            _ => (value, unit.to_string()),
        }
    }
}

impl Default for CargoParser {
    fn default() -> Self {
        Self::new().expect("Failed to create parser - regex compilation failed")
    }
}

/// Parse benchmark output from a file
pub fn parse_from_file(path: &std::path::Path) -> Result<Vec<BenchmarkResult>> {
    let content = std::fs::read_to_string(path).map_err(|e| Error::FileReadError {
        path: path.display().to_string(),
        source: e,
    })?;

    let parser = CargoParser::new()?;
    parser.parse(&content)
}

/// Parse benchmark output from a string
pub fn parse_from_string(output: &str) -> Result<Vec<BenchmarkResult>> {
    let parser = CargoParser::new()?;
    parser.parse(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_libtest_format() {
        let output = r#"
running 3 tests
test bench_add ... bench:         123 ns/iter (+/- 5)
test bench_multiply ... bench:       1,234 ns/iter (+/- 56)
test bench_divide ... bench:      12,345 ns/iter (+/- 678)

test result: ok. 0 passed; 0 failed; 0 ignored; 3 measured; 0 filtered out
        "#;

        let parser = CargoParser::new().unwrap();
        let results = parser.parse(output).unwrap();

        assert_eq!(results.len(), 3);

        assert_eq!(results[0].name, "bench_add");
        assert_eq!(results[0].value, 123.0);
        assert_eq!(results[0].unit, "ns/iter");
        assert_eq!(results[0].range, Some("+/- 5".to_string()));

        assert_eq!(results[1].name, "bench_multiply");
        assert_eq!(results[1].value, 1234.0);

        assert_eq!(results[2].name, "bench_divide");
        assert_eq!(results[2].value, 12345.0);
    }

    #[test]
    fn test_parse_criterion_format() {
        let output = r#"
bench_fibonacci         time:   [1.2345 µs 1.2456 µs 1.2567 µs]
bench_sorting           time:   [10.123 ns 10.456 ns 10.789 ns]
        "#;

        let parser = CargoParser::new().unwrap();
        let results = parser.parse(output).unwrap();

        assert_eq!(results.len(), 2);

        assert_eq!(results[0].name, "bench_fibonacci");
        // 1.2456 µs = 1245.6 ns
        assert!((results[0].value - 1245.6).abs() < 0.1);
        assert_eq!(results[0].unit, "ns");

        assert_eq!(results[1].name, "bench_sorting");
        assert!((results[1].value - 10.456).abs() < 0.001);
    }

    #[test]
    fn test_parse_empty_output() {
        let output = "";
        let parser = CargoParser::new().unwrap();
        let result = parser.parse(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_no_benchmarks() {
        let output = r#"
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
        "#;

        let parser = CargoParser::new().unwrap();
        let result = parser.parse(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_libtest_hierarchical_names() {
        let output = r#"
running 4 tests
test math::arithmetic::bench_add ... bench:         100 ns/iter (+/- 5)
test math::arithmetic::bench_sub ... bench:         110 ns/iter (+/- 6)
test math::geometry::bench_area ... bench:         200 ns/iter (+/- 10)
test standalone_bench ... bench:         50 ns/iter (+/- 2)

test result: ok. 0 passed; 0 failed; 0 ignored; 4 measured; 0 filtered out
        "#;

        let parser = CargoParser::new().unwrap();
        let results = parser.parse(output).unwrap();

        assert_eq!(results.len(), 4);

        // grandparent::parent::bench format
        assert_eq!(results[0].name, "math::arithmetic::bench_add");
        assert_eq!(results[0].value, 100.0);

        assert_eq!(results[1].name, "math::arithmetic::bench_sub");
        assert_eq!(results[1].value, 110.0);

        // grandparent::parent::bench with different parent
        assert_eq!(results[2].name, "math::geometry::bench_area");
        assert_eq!(results[2].value, 200.0);

        // standalone (no hierarchy)
        assert_eq!(results[3].name, "standalone_bench");
        assert_eq!(results[3].value, 50.0);
    }

    #[test]
    fn test_parse_criterion_hierarchical_names() {
        let output = r#"
math/arithmetic/add     time:   [100.00 ns 105.00 ns 110.00 ns]
math/arithmetic/sub     time:   [110.00 ns 115.00 ns 120.00 ns]
math/geometry/area      time:   [200.00 ns 210.00 ns 220.00 ns]
standalone              time:   [50.00 ns 55.00 ns 60.00 ns]
        "#;

        let parser = CargoParser::new().unwrap();
        let results = parser.parse(output).unwrap();

        assert_eq!(results.len(), 4);

        // grandparent/parent/bench format (Criterion style)
        assert_eq!(results[0].name, "math/arithmetic/add");
        assert!((results[0].value - 105.0).abs() < 0.01);

        assert_eq!(results[1].name, "math/arithmetic/sub");
        assert!((results[1].value - 115.0).abs() < 0.01);

        assert_eq!(results[2].name, "math/geometry/area");
        assert!((results[2].value - 210.0).abs() < 0.01);

        // standalone (no hierarchy)
        assert_eq!(results[3].name, "standalone");
        assert!((results[3].value - 55.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_deep_hierarchy() {
        let output = r#"
running 2 tests
test level1::level2::level3::level4::deep_bench ... bench:         500 ns/iter (+/- 25)
test a::b::c ... bench:         300 ns/iter (+/- 15)

test result: ok. 0 passed; 0 failed; 0 ignored; 2 measured; 0 filtered out
        "#;

        let parser = CargoParser::new().unwrap();
        let results = parser.parse(output).unwrap();

        assert_eq!(results.len(), 2);

        // Very deep hierarchy (4+ levels)
        assert_eq!(
            results[0].name,
            "level1::level2::level3::level4::deep_bench"
        );
        assert_eq!(results[0].value, 500.0);

        // Minimal hierarchy (3 levels)
        assert_eq!(results[1].name, "a::b::c");
        assert_eq!(results[1].value, 300.0);
    }

    #[test]
    fn test_parse_mixed_formats_with_hierarchy() {
        let output = r#"
running 2 tests
test crypto::hashing::bench_sha256 ... bench:       1,500 ns/iter (+/- 75)
test crypto::signing::bench_ecdsa ... bench:       5,000 ns/iter (+/- 250)

crypto/encryption/aes   time:   [2.0000 µs 2.1000 µs 2.2000 µs]
crypto/encryption/rsa   time:   [10.000 ms 10.500 ms 11.000 ms]

test result: ok. 0 passed; 0 failed; 0 ignored; 2 measured; 0 filtered out
        "#;

        let parser = CargoParser::new().unwrap();
        let results = parser.parse(output).unwrap();

        assert_eq!(results.len(), 4);

        // libtest with :: separator
        assert_eq!(results[0].name, "crypto::hashing::bench_sha256");
        assert_eq!(results[0].value, 1500.0);

        assert_eq!(results[1].name, "crypto::signing::bench_ecdsa");
        assert_eq!(results[1].value, 5000.0);

        // criterion with / separator
        assert_eq!(results[2].name, "crypto/encryption/aes");
        // 2.1 µs = 2100 ns
        assert!((results[2].value - 2100.0).abs() < 1.0);

        assert_eq!(results[3].name, "crypto/encryption/rsa");
        // 10.5 ms = 10_500_000 ns
        assert!((results[3].value - 10_500_000.0).abs() < 1000.0);
    }
}
