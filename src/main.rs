//! git-bench CLI - Continuous benchmarking for cargo
//!
//! A Rust implementation of github-action-benchmark focused on cargo compatibility.

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use git_bench::{
    alert::{self, AlertConfig},
    compare::{self, CompareConfig},
    data::{BenchmarkData, BenchmarkRun},
    git,
    github::{GitHubActionsEnv, GitHubClient},
    html::{self, DashboardConfig},
    parser,
};

/// git-bench: Continuous benchmarking for cargo projects
#[derive(Parser, Debug)]
#[command(name = "git-bench")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run benchmarks and record results
    Run(RunArgs),

    /// Parse benchmark output and store results
    Store(StoreArgs),

    /// Compare benchmark results
    Compare(CompareArgs),

    /// Generate HTML dashboard
    Dashboard(DashboardArgs),

    /// Show benchmark history
    History(HistoryArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    /// Path to the benchmark output file
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    /// Name of the benchmark suite
    #[arg(short, long, default_value = "cargo")]
    name: String,

    /// Path to benchmark data JSON file
    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    /// GitHub Pages branch name
    #[arg(long, default_value = "gh-pages")]
    gh_pages_branch: String,

    /// Path to benchmark data directory (relative to repo root)
    #[arg(long, default_value = "dev/bench")]
    benchmark_data_dir_path: String,

    /// GitHub token for API operations
    #[arg(long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,

    /// Git ref to use for reporting
    #[arg(long, env = "GITHUB_SHA")]
    git_ref: Option<String>,

    /// External JSON file path (alternative to gh-pages)
    #[arg(long)]
    external_data_json_path: Option<PathBuf>,

    /// Automatically push to GitHub Pages
    #[arg(long, default_value = "false")]
    auto_push: bool,

    /// Always leave a commit comment
    #[arg(long, default_value = "false")]
    comment_always: bool,

    /// Save data to file (set to false for PR comparison)
    #[arg(long, default_value = "true")]
    save_data_file: bool,

    /// Alert threshold percentage (e.g., "200%")
    #[arg(long, default_value = "200%")]
    alert_threshold: String,

    /// Comment on alert
    #[arg(long, default_value = "false")]
    comment_on_alert: bool,

    /// Fail on alert
    #[arg(long, default_value = "false")]
    fail_on_alert: bool,

    /// Fail threshold percentage (defaults to alert-threshold)
    #[arg(long)]
    fail_threshold: Option<String>,

    /// Users to mention in alert comments (comma-separated)
    #[arg(long)]
    alert_comment_cc_users: Option<String>,

    /// Maximum items to keep in chart
    #[arg(long)]
    max_items_in_chart: Option<usize>,

    /// Skip fetching gh-pages branch
    #[arg(long, default_value = "false")]
    skip_fetch_gh_pages: bool,

    /// Repository URL for external storage
    #[arg(long)]
    gh_repository: Option<String>,
}

#[derive(Parser, Debug)]
struct StoreArgs {
    /// Path to the benchmark output file
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    /// Name of the benchmark suite
    #[arg(short, long, default_value = "cargo")]
    name: String,

    /// Path to benchmark data JSON file
    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    /// Git ref (commit SHA)
    #[arg(long)]
    git_ref: Option<String>,

    /// Maximum items to keep
    #[arg(long)]
    max_items: Option<usize>,
}

#[derive(Parser, Debug)]
struct CompareArgs {
    /// Path to the current benchmark output file
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    /// Path to benchmark data JSON file
    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    /// Name of the benchmark suite
    #[arg(short, long, default_value = "cargo")]
    name: String,

    /// Alert threshold percentage
    #[arg(long, default_value = "200%")]
    alert_threshold: String,

    /// Output format (text, json, markdown)
    #[arg(long, default_value = "markdown")]
    format: String,
}

#[derive(Parser, Debug)]
struct DashboardArgs {
    /// Path to benchmark data JSON file
    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    /// Output directory for dashboard
    #[arg(short, long, default_value = "dev/bench")]
    output_dir: PathBuf,

    /// Dashboard title
    #[arg(long, default_value = "Benchmark Results")]
    title: String,
}

#[derive(Parser, Debug)]
struct HistoryArgs {
    /// Path to benchmark data JSON file
    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    /// Name of the benchmark suite (show all if not specified)
    #[arg(short, long)]
    name: Option<String>,

    /// Number of recent runs to show
    #[arg(short, long, default_value = "10")]
    limit: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    match cli.command {
        Commands::Run(args) => run_command(args),
        Commands::Store(args) => store_command(args),
        Commands::Compare(args) => compare_command(args),
        Commands::Dashboard(args) => dashboard_command(args),
        Commands::History(args) => history_command(args),
    }
}

/// Main run command - the full workflow
fn run_command(args: RunArgs) -> Result<()> {
    info!("Running benchmark workflow");

    // Load GitHub Actions environment
    let gh_env = GitHubActionsEnv::from_env();
    let is_github_actions = GitHubActionsEnv::is_github_actions();

    if is_github_actions {
        debug!("Running in GitHub Actions environment");
    }

    // Parse benchmark output
    info!("Parsing benchmark output from {:?}", args.output_file);
    let output = std::fs::read_to_string(&args.output_file)
        .with_context(|| format!("Failed to read benchmark output file: {:?}", args.output_file))?;

    let results = parser::parse_from_string(&output)
        .with_context(|| "Failed to parse benchmark output")?;

    info!("Parsed {} benchmark results", results.len());

    // Get commit info
    let repo_path = std::env::current_dir()?;
    let commit_ref = args.git_ref.as_deref().or(gh_env.sha.as_deref());
    let commit = git::get_commit_info(&repo_path, commit_ref)
        .with_context(|| "Failed to get commit info")?;

    debug!("Commit: {} - {}", &commit.id[..7], commit.message);

    // Determine data file path
    let data_file = if let Some(ref external_path) = args.external_data_json_path {
        external_path.clone()
    } else {
        args.data_file.clone()
    };

    // Load existing data
    let mut data = BenchmarkData::load_from_file(&data_file)
        .unwrap_or_else(|_| {
            info!("Creating new benchmark data file");
            BenchmarkData::new()
        });

    // Create comparison config
    let compare_config = CompareConfig::from_percentages(
        &args.alert_threshold,
        args.fail_threshold.as_deref(),
    ).map_err(|e| anyhow::anyhow!("Invalid threshold configuration: {}", e))?;

    // Compare with previous run
    let previous_run = data.get_latest_run(&args.name);
    let report = compare::compare_with_previous(&results, previous_run, &compare_config);

    // Print comparison report
    println!("{}", report.summary());

    // Output GitHub Actions annotations
    if is_github_actions {
        print!("{}", alert::format_github_actions_alert(&report));
    }

    // Handle alerts
    let alert_config = AlertConfig {
        comment_on_alert: args.comment_on_alert,
        fail_on_alert: args.fail_on_alert,
        alert_comment_cc_users: args.alert_comment_cc_users,
    };

    // Create GitHub comment if needed
    if (args.comment_always || (args.comment_on_alert && report.has_alerts()))
        && args.github_token.is_some()
    {
        if let Some((owner, repo)) = gh_env.get_owner_repo() {
            let token = args.github_token.as_ref().or(gh_env.token.as_ref());
            if let Some(token) = token {
                info!("Creating commit comment on GitHub");
                match create_github_comment(&owner, &repo, &commit.id, &report, &alert_config, token) {
                    Ok(url) => info!("Created comment: {}", url),
                    Err(e) => warn!("Failed to create comment: {}", e),
                }
            }
        }
    }

    // Save data if enabled
    if args.save_data_file {
        let mut commit_with_url = commit.clone();
        commit_with_url.url = gh_env.commit_url(&commit.id);

        let run = BenchmarkRun {
            commit: commit_with_url,
            date: Utc::now(),
            tool: "cargo".to_string(),
            benches: results,
        };

        data.add_run(&args.name, run, args.max_items_in_chart);
        data.save_to_file(&data_file)
            .with_context(|| "Failed to save benchmark data")?;

        info!("Saved benchmark data to {:?}", data_file);

        // Generate dashboard if using gh-pages
        if args.external_data_json_path.is_none() {
            let dashboard_config = DashboardConfig {
                title: format!("{} Benchmarks", args.name),
                output_dir: args.benchmark_data_dir_path.clone(),
            };

            html::write_dashboard(&data, &dashboard_config, &repo_path)
                .with_context(|| "Failed to generate dashboard")?;

            info!("Generated dashboard in {}", args.benchmark_data_dir_path);
        }
    }

    // Check if we should fail
    if alert::should_fail(&report, &alert_config) {
        error!("Benchmark alert triggered - failing workflow");
        std::process::exit(1);
    }

    Ok(())
}

/// Store benchmark results
fn store_command(args: StoreArgs) -> Result<()> {
    info!("Storing benchmark results");

    // Parse benchmark output
    let output = std::fs::read_to_string(&args.output_file)
        .with_context(|| format!("Failed to read benchmark output file: {:?}", args.output_file))?;

    let results = parser::parse_from_string(&output)
        .with_context(|| "Failed to parse benchmark output")?;

    info!("Parsed {} benchmark results", results.len());

    // Get commit info
    let repo_path = std::env::current_dir()?;
    let commit = git::get_commit_info(&repo_path, args.git_ref.as_deref())
        .with_context(|| "Failed to get commit info")?;

    // Load existing data
    let mut data = BenchmarkData::load_from_file(&args.data_file)
        .unwrap_or_else(|_| BenchmarkData::new());

    // Add new run
    let run = BenchmarkRun {
        commit,
        date: Utc::now(),
        tool: "cargo".to_string(),
        benches: results,
    };

    data.add_run(&args.name, run, args.max_items);
    data.save_to_file(&args.data_file)
        .with_context(|| "Failed to save benchmark data")?;

    info!("Stored benchmark data to {:?}", args.data_file);

    Ok(())
}

/// Compare benchmarks
fn compare_command(args: CompareArgs) -> Result<()> {
    // Parse current benchmark output
    let output = std::fs::read_to_string(&args.output_file)
        .with_context(|| format!("Failed to read benchmark output file: {:?}", args.output_file))?;

    let results = parser::parse_from_string(&output)
        .with_context(|| "Failed to parse benchmark output")?;

    // Load existing data
    let data = BenchmarkData::load_from_file(&args.data_file)
        .with_context(|| "Failed to load benchmark data")?;

    // Create comparison config
    let config = CompareConfig::from_percentages(&args.alert_threshold, None)
        .map_err(|e| anyhow::anyhow!("Invalid threshold: {}", e))?;

    // Compare
    let previous_run = data.get_latest_run(&args.name);
    let report = compare::compare_with_previous(&results, previous_run, &config);

    // Output in requested format
    match args.format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "comparisons": report.comparisons,
                "alerts": report.alerts,
                "failures": report.failures,
                "new_benchmarks": report.new_benchmarks,
                "removed_benchmarks": report.removed_benchmarks,
                "has_alerts": report.has_alerts(),
                "has_failures": report.has_failures(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "markdown" => {
            println!("{}", report.summary());
        }
        "text" | _ => {
            println!("{}", report.short_summary());
            for comp in &report.comparisons {
                let indicator = if comp.is_regression { "↑" } else { "↓" };
                println!(
                    "  {} {}: {:.2} {} -> {:.2} {} ({:+.1}%)",
                    indicator,
                    comp.name,
                    comp.previous,
                    comp.unit,
                    comp.current,
                    comp.unit,
                    comp.percentage_change
                );
            }
        }
    }

    Ok(())
}

/// Generate dashboard
fn dashboard_command(args: DashboardArgs) -> Result<()> {
    info!("Generating dashboard");

    let data = BenchmarkData::load_from_file(&args.data_file)
        .with_context(|| "Failed to load benchmark data")?;

    let config = DashboardConfig {
        title: args.title,
        output_dir: args.output_dir.to_string_lossy().to_string(),
    };

    let repo_path = std::env::current_dir()?;
    html::write_dashboard(&data, &config, &repo_path)
        .with_context(|| "Failed to generate dashboard")?;

    info!("Dashboard generated at {:?}", args.output_dir.join("index.html"));

    Ok(())
}

/// Show benchmark history
fn history_command(args: HistoryArgs) -> Result<()> {
    let data = BenchmarkData::load_from_file(&args.data_file)
        .with_context(|| "Failed to load benchmark data")?;

    let suites: Vec<&String> = if let Some(ref name) = args.name {
        if data.entries.contains_key(name) {
            vec![name]
        } else {
            anyhow::bail!("Suite '{}' not found", name);
        }
    } else {
        data.entries.keys().collect()
    };

    for suite_name in suites {
        println!("## {}\n", suite_name);

        if let Some(runs) = data.entries.get(suite_name) {
            let recent_runs: Vec<_> = runs.iter().rev().take(args.limit).collect();

            for run in recent_runs {
                println!("### {} - {}", &run.commit.id[..7], run.commit.message);
                println!("Date: {}", run.date.format("%Y-%m-%d %H:%M:%S UTC"));
                println!();

                for bench in &run.benches {
                    let range_str = bench.range.as_deref().unwrap_or("-");
                    println!("  - {}: {:.2} {} ({})", bench.name, bench.value, bench.unit, range_str);
                }
                println!();
            }
        }
    }

    Ok(())
}

/// Create a GitHub commit comment
fn create_github_comment(
    owner: &str,
    repo: &str,
    sha: &str,
    report: &compare::CompareReport,
    config: &AlertConfig,
    token: &str,
) -> Result<String> {
    let client = GitHubClient::new(Some(token.to_string()))?;

    let body = if report.has_alerts() {
        alert::generate_alert_message(report, config)
            .unwrap_or_else(|| report.summary())
    } else {
        report.summary()
    };

    let url = client.create_commit_comment(owner, repo, sha, &body)?;
    Ok(url)
}
