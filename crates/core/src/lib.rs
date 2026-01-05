//! git-bench-core - Shared types and parsing for git-bench
//!
//! This crate contains WASM-compatible code that can be shared between
//! the CLI and the dashboard.
//!
//! # Features
//!
//! - Parse cargo bench output (both libtest and Criterion formats)
//! - Data structures for benchmark results
//! - Benchmark comparison logic

pub mod compare;
pub mod data;
pub mod error;
pub mod parser;

pub use compare::{compare_runs, compare_with_previous, CompareConfig, CompareReport};
pub use data::{
    AuthorInfo, BenchmarkData, BenchmarkResult, BenchmarkRun, CommitInfo, ComparisonResult,
};
pub use error::{Error, Result};
pub use parser::{parse_from_file, parse_from_string, CargoParser};




