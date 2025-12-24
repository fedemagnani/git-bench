//! Test data generator for dashboard edge case testing
//!
//! Generates various benchmark data scenarios to test the dashboard's handling of:
//! - New benchmarks appearing mid-history
//! - Benchmarks being renamed/removed
//! - Different hierarchy structures
//! - Performance regressions and improvements
//!
//! Usage:
//!   cargo run --bin test_data_generator [scenario]
//!
//! Scenarios:
//!   simple        - 3 commits, 2 benchmarks
//!   new-bench     - Benchmark appears mid-history  
//!   renamed       - Benchmark gets renamed (simulates refactoring)
//!   regression    - Shows performance regression/recovery
//!   deep-hierarchy - Complex grandparent::parent::test structure
//!   full          - All scenarios combined (default)

use chrono::{Duration, Utc};
use git_bench_core::{AuthorInfo, BenchmarkData, BenchmarkResult, BenchmarkRun, CommitInfo};
use std::collections::HashMap;

fn main() {
    let scenario = std::env::args().nth(1).unwrap_or_else(|| "full".to_string());
    
    let data = match scenario.as_str() {
        "simple" => generate_simple(),
        "new-bench" => generate_new_benchmark_appears(),
        "renamed" => generate_renamed_benchmark(),
        "regression" => generate_regression_scenario(),
        "deep-hierarchy" => generate_deep_hierarchy(),
        "full" | _ => generate_full_test_data(),
    };
    
    let json = serde_json::to_string_pretty(&data).expect("Failed to serialize");
    std::fs::write("test-data.json", &json).expect("Failed to write test-data.json");
    
    println!("✅ Generated test-data.json with scenario: {}", scenario);
    println!("\nAvailable scenarios:");
    println!("  simple         - 3 commits, 2 benchmarks");
    println!("  new-bench      - Benchmark appears mid-history");
    println!("  renamed        - Benchmark gets renamed");
    println!("  regression     - Shows performance regression");
    println!("  deep-hierarchy - Complex nested structure");
    println!("  full           - All scenarios combined (default)");
    println!("\nTo use: cp test-data.json dist/data.json && make dev");
}

fn make_commit(id: &str, message: &str, days_ago: i64) -> CommitInfo {
    CommitInfo {
        id: id.to_string(),
        message: message.to_string(),
        timestamp: Utc::now() - Duration::days(days_ago),
        url: Some(format!("https://github.com/user/repo/commit/{}", id)),
        author: Some(AuthorInfo {
            name: "Test Author".to_string(),
            email: Some("test@example.com".to_string()),
            username: Some("testuser".to_string()),
        }),
    }
}

fn make_bench(name: &str, value: f64) -> BenchmarkResult {
    BenchmarkResult {
        name: name.to_string(),
        value,
        unit: "ns".to_string(),
        range: Some(format!("[{:.2} ns, {:.2} ns]", value * 0.95, value * 1.05)),
        extra: HashMap::new(),
    }
}

fn make_run(commit: CommitInfo, benches: Vec<BenchmarkResult>, days_ago: i64) -> BenchmarkRun {
    BenchmarkRun {
        commit,
        date: Utc::now() - Duration::days(days_ago),
        tool: "cargo".to_string(),
        benches,
    }
}

fn make_benchmark_data(entries: HashMap<String, Vec<BenchmarkRun>>) -> BenchmarkData {
    BenchmarkData {
        last_update: Some(Utc::now()),
        repo_url: Some("https://github.com/user/repo".to_string()),
        entries,
    }
}

/// Simple scenario: 3 commits with stable benchmarks
fn generate_simple() -> BenchmarkData {
    let mut entries = HashMap::new();
    
    entries.insert("example".to_string(), vec![
        make_run(
            make_commit("abc1234", "Initial implementation", 3),
            vec![
                make_bench("fibonacci/iterative", 5.0),
                make_bench("fibonacci/recursive", 1000.0),
            ],
            3,
        ),
        make_run(
            make_commit("def5678", "Add caching", 2),
            vec![
                make_bench("fibonacci/iterative", 4.5),
                make_bench("fibonacci/recursive", 950.0),
            ],
            2,
        ),
        make_run(
            make_commit("ghi9012", "Optimize loop", 1),
            vec![
                make_bench("fibonacci/iterative", 4.0),
                make_bench("fibonacci/recursive", 900.0),
            ],
            1,
        ),
    ]);
    
    make_benchmark_data(entries)
}

/// New benchmark appears mid-history
fn generate_new_benchmark_appears() -> BenchmarkData {
    let mut entries = HashMap::new();
    
    entries.insert("example".to_string(), vec![
        make_run(
            make_commit("aaa1111", "Initial", 5),
            vec![
                make_bench("sorting/bubble", 5000.0),
            ],
            5,
        ),
        make_run(
            make_commit("bbb2222", "Add quicksort", 4),
            vec![
                make_bench("sorting/bubble", 4800.0),
                make_bench("sorting/quick", 500.0), // NEW!
            ],
            4,
        ),
        make_run(
            make_commit("ccc3333", "Add mergesort", 3),
            vec![
                make_bench("sorting/bubble", 4600.0),
                make_bench("sorting/quick", 480.0),
                make_bench("sorting/merge", 450.0), // NEW!
            ],
            3,
        ),
        make_run(
            make_commit("ddd4444", "Optimize all", 2),
            vec![
                make_bench("sorting/bubble", 4400.0),
                make_bench("sorting/quick", 350.0),
                make_bench("sorting/merge", 380.0),
            ],
            2,
        ),
    ]);
    
    make_benchmark_data(entries)
}

/// Benchmark gets renamed (simulates refactoring)
fn generate_renamed_benchmark() -> BenchmarkData {
    let mut entries = HashMap::new();
    
    entries.insert("example".to_string(), vec![
        make_run(
            make_commit("old1111", "Old naming", 4),
            vec![
                make_bench("math/add", 10.0),
                make_bench("math/multiply", 15.0),
            ],
            4,
        ),
        make_run(
            make_commit("old2222", "Still old naming", 3),
            vec![
                make_bench("math/add", 9.5),
                make_bench("math/multiply", 14.0),
            ],
            3,
        ),
        // Refactoring happens here - names change
        make_run(
            make_commit("new3333", "Refactor: rename to arithmetic", 2),
            vec![
                make_bench("arithmetic/addition", 9.0),      // renamed from math/add
                make_bench("arithmetic/multiplication", 13.0), // renamed from math/multiply
            ],
            2,
        ),
        make_run(
            make_commit("new4444", "Continue with new names", 1),
            vec![
                make_bench("arithmetic/addition", 8.5),
                make_bench("arithmetic/multiplication", 12.0),
            ],
            1,
        ),
    ]);
    
    make_benchmark_data(entries)
}

/// Performance regression scenario
fn generate_regression_scenario() -> BenchmarkData {
    let mut entries = HashMap::new();
    
    entries.insert("example".to_string(), vec![
        make_run(
            make_commit("fast1111", "Baseline - fast", 6),
            vec![
                make_bench("api/request", 100.0),
                make_bench("api/response", 50.0),
            ],
            6,
        ),
        make_run(
            make_commit("fast2222", "Still fast", 5),
            vec![
                make_bench("api/request", 95.0),
                make_bench("api/response", 48.0),
            ],
            5,
        ),
        make_run(
            make_commit("slow3333", "Added logging - REGRESSION!", 4),
            vec![
                make_bench("api/request", 250.0),  // 2.5x slower!
                make_bench("api/response", 120.0), // 2.5x slower!
            ],
            4,
        ),
        make_run(
            make_commit("slow4444", "Still slow", 3),
            vec![
                make_bench("api/request", 240.0),
                make_bench("api/response", 115.0),
            ],
            3,
        ),
        make_run(
            make_commit("fix55555", "Fixed logging - back to normal", 2),
            vec![
                make_bench("api/request", 98.0),
                make_bench("api/response", 52.0),
            ],
            2,
        ),
    ]);
    
    make_benchmark_data(entries)
}

/// Deep hierarchy with grandparent::parent::test structure
fn generate_deep_hierarchy() -> BenchmarkData {
    let mut entries = HashMap::new();
    
    entries.insert("mylib".to_string(), vec![
        make_run(
            make_commit("hier1111", "Initial deep hierarchy", 3),
            vec![
                // crypto::hash::*
                make_bench("crypto::hash::sha256", 500.0),
                make_bench("crypto::hash::sha512", 800.0),
                make_bench("crypto::hash::blake3", 200.0),
                // crypto::encrypt::*
                make_bench("crypto::encrypt::aes128", 1000.0),
                make_bench("crypto::encrypt::aes256", 1500.0),
                // io::file::*
                make_bench("io::file::read", 5000.0),
                make_bench("io::file::write", 8000.0),
                // io::network::*
                make_bench("io::network::connect", 10000.0),
                make_bench("io::network::send", 2000.0),
            ],
            3,
        ),
        make_run(
            make_commit("hier2222", "Optimize crypto", 2),
            vec![
                make_bench("crypto::hash::sha256", 450.0),
                make_bench("crypto::hash::sha512", 720.0),
                make_bench("crypto::hash::blake3", 180.0),
                make_bench("crypto::encrypt::aes128", 900.0),
                make_bench("crypto::encrypt::aes256", 1350.0),
                make_bench("io::file::read", 4800.0),
                make_bench("io::file::write", 7800.0),
                make_bench("io::network::connect", 9500.0),
                make_bench("io::network::send", 1900.0),
            ],
            2,
        ),
        make_run(
            make_commit("hier3333", "Optimize IO", 1),
            vec![
                make_bench("crypto::hash::sha256", 440.0),
                make_bench("crypto::hash::sha512", 700.0),
                make_bench("crypto::hash::blake3", 170.0),
                make_bench("crypto::encrypt::aes128", 880.0),
                make_bench("crypto::encrypt::aes256", 1300.0),
                make_bench("io::file::read", 3500.0),
                make_bench("io::file::write", 5500.0),
                make_bench("io::network::connect", 7000.0),
                make_bench("io::network::send", 1500.0),
            ],
            1,
        ),
    ]);
    
    make_benchmark_data(entries)
}

/// Full test data combining multiple scenarios
fn generate_full_test_data() -> BenchmarkData {
    let mut entries = HashMap::new();
    
    // Combine scenarios into different suites
    let simple = generate_simple();
    let new_bench = generate_new_benchmark_appears();
    let regression = generate_regression_scenario();
    let deep = generate_deep_hierarchy();
    
    for (k, v) in simple.entries {
        entries.insert(format!("simple_{}", k), v);
    }
    for (k, v) in new_bench.entries {
        entries.insert(format!("new_bench_{}", k), v);
    }
    for (k, v) in regression.entries {
        entries.insert(format!("regression_{}", k), v);
    }
    for (k, v) in deep.entries {
        entries.insert(k, v);
    }
    
    make_benchmark_data(entries)
}
