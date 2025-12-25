//! git-bench CLI - Continuous benchmarking for cargo

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

mod alert;
mod error;
mod git;
mod github;

use alert::AlertConfig;
use git_bench_core::{
    compare_with_previous, parse_from_string, BenchmarkData, BenchmarkRun, CompareConfig,
};
use github::{GitHubActionsEnv, GitHubClient};

/// git-bench: Continuous benchmarking for cargo projects
#[derive(Parser, Debug)]
#[command(name = "git-bench")]
#[command(author, version, about, long_about = None)]
struct Cli {
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
    /// Show benchmark history
    History(HistoryArgs),
}

#[derive(Parser, Debug)]
struct RunArgs {
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    #[arg(short, long, default_value = "cargo")]
    name: String,

    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    #[arg(long, default_value = "gh-pages")]
    gh_pages_branch: String,

    #[arg(long, default_value = "dev/bench")]
    benchmark_data_dir_path: String,

    #[arg(long, env = "GITHUB_TOKEN")]
    github_token: Option<String>,

    #[arg(long, env = "GITHUB_SHA")]
    git_ref: Option<String>,

    #[arg(long)]
    external_data_json_path: Option<PathBuf>,

    #[arg(long, default_value = "false")]
    auto_push: bool,

    #[arg(long, default_value = "false")]
    comment_always: bool,

    #[arg(long, default_value = "true")]
    save_data_file: bool,

    #[arg(long, default_value = "200%")]
    alert_threshold: String,

    #[arg(long, default_value = "false")]
    comment_on_alert: bool,

    #[arg(long, default_value = "false")]
    fail_on_alert: bool,

    #[arg(long)]
    fail_threshold: Option<String>,

    #[arg(long)]
    alert_comment_cc_users: Option<String>,

    #[arg(long)]
    max_items_in_chart: Option<usize>,

    #[arg(long, default_value = "false")]
    skip_fetch_gh_pages: bool,

    /// Path to dashboard dist directory (if provided, dashboard will be deployed with data)
    #[arg(long)]
    dashboard_dir: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct StoreArgs {
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    #[arg(short, long, default_value = "cargo")]
    name: String,

    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    #[arg(long)]
    git_ref: Option<String>,

    #[arg(long)]
    max_items: Option<usize>,
}

#[derive(Parser, Debug)]
struct CompareArgs {
    #[arg(short, long, value_name = "FILE")]
    output_file: PathBuf,

    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    #[arg(short, long, default_value = "cargo")]
    name: String,

    #[arg(long, default_value = "200%")]
    alert_threshold: String,

    #[arg(long, default_value = "markdown")]
    format: String,
}

#[derive(Parser, Debug)]
struct HistoryArgs {
    #[arg(long, default_value = "benchmark-data.json")]
    data_file: PathBuf,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long, default_value = "10")]
    limit: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

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
        Commands::History(args) => history_command(args),
    }
}

fn run_command(args: RunArgs) -> Result<()> {
    info!("Running benchmark workflow");

    let gh_env = GitHubActionsEnv::from_env();
    let is_github_actions = GitHubActionsEnv::is_github_actions();

    if is_github_actions {
        debug!("Running in GitHub Actions environment");
    }

    info!("Parsing benchmark output from {:?}", args.output_file);
    let output = std::fs::read_to_string(&args.output_file)
        .with_context(|| format!("Failed to read benchmark output file: {:?}", args.output_file))?;

    let results = match parse_from_string(&output) {
        Ok(r) => r,
        Err(_) => {
            info!("No benchmark results found in output, skipping");
            return Ok(());
        }
    };

    if results.is_empty() {
        info!("No benchmark results found, skipping");
        return Ok(());
    }

    info!("Parsed {} benchmark results", results.len());

    let repo_path = std::env::current_dir()?;
    let commit_ref = args.git_ref.as_deref().or(gh_env.sha.as_deref());
    let commit =
        git::get_commit_info(&repo_path, commit_ref).with_context(|| "Failed to get commit info")?;

    debug!("Commit: {} - {}", &commit.id[..7], commit.message);

    let data_file = if let Some(ref external_path) = args.external_data_json_path {
        external_path.clone()
    } else {
        args.data_file.clone()
    };

    let mut data = BenchmarkData::load_from_file(&data_file).unwrap_or_else(|_| {
        info!("Creating new benchmark data file");
        BenchmarkData::new()
    });

    let compare_config = CompareConfig::from_percentages(
        &args.alert_threshold,
        args.fail_threshold.as_deref(),
    )
    .map_err(|e| anyhow::anyhow!("Invalid threshold configuration: {}", e))?;

    let previous_run = data.get_latest_run(&args.name);
    let report = compare_with_previous(&results, previous_run, &compare_config);

    println!("{}", report.summary());

    if is_github_actions {
        print!("{}", alert::format_github_actions_alert(&report));
    }

    let alert_config = AlertConfig {
        fail_on_alert: args.fail_on_alert,
        alert_comment_cc_users: args.alert_comment_cc_users.clone(),
    };

    if (args.comment_always || (args.comment_on_alert && report.has_alerts()))
        && args.github_token.is_some()
    {
        if let Some((owner, repo)) = gh_env.get_owner_repo() {
            let token = args.github_token.as_ref().or(gh_env.token.as_ref());
            if let Some(token) = token {
                info!("Creating commit comment on GitHub");
                match create_github_comment(&owner, &repo, &commit.id, &report, &alert_config, token)
                {
                    Ok(url) => info!("Created comment: {}", url),
                    Err(e) => warn!("Failed to create comment: {}", e),
                }
            }
        }
    }

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

        if args.auto_push {
            info!(
                "Deploying to GitHub Pages branch: {}",
                args.gh_pages_branch
            );

            let gh_config = git::GhPagesConfig {
                branch: &args.gh_pages_branch,
                data_dir: &args.benchmark_data_dir_path,
                remote: "origin",
                skip_fetch: args.skip_fetch_gh_pages,
                dashboard_dir: args.dashboard_dir.as_deref(),
            };

            match git::deploy_to_gh_pages(&repo_path, &data_file, &gh_config) {
                Ok(commit_id) => {
                    if commit_id == "No changes" {
                        info!("No changes to deploy - benchmark data already up to date");
                    } else {
                        info!(
                            "Successfully deployed to gh-pages: {}",
                            &commit_id[..7.min(commit_id.len())]
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to deploy to GitHub Pages: {}", e);
                    if args.fail_on_alert {
                        return Err(e.into());
                    }
                }
            }
        } else {
            info!("To view dashboard, use the Dioxus dashboard: cd crates/dashboard && ./build.sh");
        }
    }

    if alert::should_fail(&report, &alert_config) {
        error!("Benchmark alert triggered - failing workflow");
        std::process::exit(1);
    }

    Ok(())
}

fn store_command(args: StoreArgs) -> Result<()> {
    info!("Storing benchmark results");

    let output = std::fs::read_to_string(&args.output_file)
        .with_context(|| format!("Failed to read benchmark output file: {:?}", args.output_file))?;

    let results = match parse_from_string(&output) {
        Ok(r) if !r.is_empty() => r,
        _ => {
            info!("No benchmark results found, skipping");
            return Ok(());
        }
    };

    info!("Parsed {} benchmark results", results.len());

    let repo_path = std::env::current_dir()?;
    let commit = git::get_commit_info(&repo_path, args.git_ref.as_deref())
        .with_context(|| "Failed to get commit info")?;

    let mut data =
        BenchmarkData::load_from_file(&args.data_file).unwrap_or_else(|_| BenchmarkData::new());

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

fn compare_command(args: CompareArgs) -> Result<()> {
    let output = std::fs::read_to_string(&args.output_file)
        .with_context(|| format!("Failed to read benchmark output file: {:?}", args.output_file))?;

    let results = match parse_from_string(&output) {
        Ok(r) if !r.is_empty() => r,
        _ => {
            info!("No benchmark results found, skipping comparison");
            return Ok(());
        }
    };

    let data = BenchmarkData::load_from_file(&args.data_file)
        .with_context(|| "Failed to load benchmark data")?;

    let config = CompareConfig::from_percentages(&args.alert_threshold, None)
        .map_err(|e| anyhow::anyhow!("Invalid threshold: {}", e))?;

    let previous_run = data.get_latest_run(&args.name);
    let report = compare_with_previous(&results, previous_run, &config);

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
        _ => {
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
                    println!(
                        "  - {}: {:.2} {} ({})",
                        bench.name, bench.value, bench.unit, range_str
                    );
                }
                println!();
            }
        }
    }

    Ok(())
}

fn create_github_comment(
    owner: &str,
    repo: &str,
    sha: &str,
    report: &git_bench_core::CompareReport,
    config: &AlertConfig,
    token: &str,
) -> Result<String> {
    let client = GitHubClient::new(Some(token.to_string()))?;

    let body = if report.has_alerts() {
        alert::generate_alert_message(report, config).unwrap_or_else(|| report.summary())
    } else {
        report.summary()
    };

    let url = client.create_commit_comment(owner, repo, sha, &body)?;
    Ok(url)
}

