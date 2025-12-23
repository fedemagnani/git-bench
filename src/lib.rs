//! git-bench - A Rust implementation of github-action-benchmark for cargo
//!
//! This library provides functionality for continuous benchmarking with cargo,
//! including benchmark parsing, comparison, alerting, and dashboard generation.
//!
//! # Features
//!
//! - Parse cargo bench output (both libtest and Criterion formats)
//! - Store benchmark history in JSON format
//! - Compare benchmarks and detect regressions
//! - Generate HTML dashboards with Chart.js
//! - GitHub integration for comments and alerts
//!
//! # Example
//!
//! ```no_run
//! use git_bench::{parser, data, compare, html};
//!
//! // Parse benchmark output
//! let output = std::fs::read_to_string("bench_output.txt").unwrap();
//! let results = parser::parse_from_string(&output).unwrap();
//!
//! // Load existing data
//! let mut data = data::BenchmarkData::load_from_file("bench_data.json".as_ref()).unwrap();
//!
//! // Compare with previous
//! let config = compare::CompareConfig::default();
//! if let Some(prev_run) = data.get_previous_run("my-suite") {
//!     let report = compare::compare_with_previous(&results, Some(prev_run), &config);
//!     println!("{}", report.summary());
//! }
//! ```

pub mod alert;
pub mod compare;
pub mod data;
pub mod error;
pub mod git;
pub mod github;
pub mod html;
pub mod parser;

pub use error::{Error, Result};

