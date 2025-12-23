//! Dioxus-based benchmark dashboard for git-bench
//!
//! A pure Rust frontend that compiles to WebAssembly.
//! This dashboard loads benchmark data from data.json and displays
//! it in an interactive UI with charts.

use dioxus::prelude::*;
use gloo_net::http::Request;
use std::collections::HashMap;

mod types;
use types::{BenchmarkData, BenchmarkRun};

const DATA_URL: &str = "data.json";

fn main() {
    tracing_wasm::set_as_global_default();
    launch(App);
}

#[component]
fn App() -> Element {
    // State for benchmark data
    let mut data = use_signal(|| None::<BenchmarkData>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);

    // Load data on mount
    use_effect(move || {
        spawn(async move {
            match load_benchmark_data().await {
                Ok(benchmark_data) => {
                    data.set(Some(benchmark_data));
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(e));
                    loading.set(false);
                }
            }
        });
    });

    rsx! {
        style { {include_str!("styles.css")} }

        div { class: "app",
            Header {}

            main { class: "container",
                if *loading.read() {
                    LoadingSpinner {}
                } else if let Some(err) = error.read().as_ref() {
                    ErrorMessage { message: err.clone() }
                } else if let Some(benchmark_data) = data.read().as_ref() {
                    if benchmark_data.entries.is_empty() {
                        EmptyState {}
                    } else {
                        Dashboard { data: benchmark_data.clone() }
                    }
                }
            }

            Footer {}
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header { class: "header",
            div { class: "header-content",
                h1 { class: "title",
                    span { class: "icon", "📊" }
                    " Benchmark Dashboard"
                }
                p { class: "subtitle", "Continuous Performance Tracking" }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer { class: "footer",
            p {
                "Powered by "
                a { href: "https://github.com/yourusername/git-bench", "git-bench" }
                " • Built with "
                a { href: "https://dioxuslabs.com", "Dioxus" }
            }
        }
    }
}

#[component]
fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "loading",
            div { class: "spinner" }
            p { "Loading benchmark data..." }
        }
    }
}

#[component]
fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "error-container",
            div { class: "error-icon", "⚠️" }
            h2 { "Failed to Load Data" }
            p { class: "error-message", "{message}" }
            p { class: "error-hint",
                "Make sure "
                code { "data.json" }
                " exists in the same directory."
            }
        }
    }
}

#[component]
fn EmptyState() -> Element {
    rsx! {
        div { class: "empty-state",
            div { class: "empty-icon", "📈" }
            h2 { "No Benchmark Data Yet" }
            p { "Run your benchmarks to start tracking performance." }
            code { class: "command", "cargo bench | git-bench run -o -" }
        }
    }
}

#[component]
fn Dashboard(data: BenchmarkData) -> Element {
    let suites: Vec<_> = data.entries.iter().collect();

    rsx! {
        div { class: "dashboard",
            // Stats overview
            StatsOverview { data: data.clone() }

            // Benchmark suites
            for (name, runs) in suites {
                BenchmarkSuite {
                    key: "{name}",
                    name: name.clone(),
                    runs: runs.clone()
                }
            }
        }
    }
}

#[component]
fn StatsOverview(data: BenchmarkData) -> Element {
    let total_suites = data.entries.len();
    let total_runs: usize = data.entries.values().map(|v| v.len()).sum();
    let total_benchmarks: usize = data
        .entries
        .values()
        .flat_map(|runs| runs.iter().flat_map(|r| r.benches.iter()))
        .map(|b| &b.name)
        .collect::<std::collections::HashSet<_>>()
        .len();

    let last_update = data
        .last_update
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    rsx! {
        div { class: "stats-overview",
            div { class: "stat-card",
                div { class: "stat-value", "{total_suites}" }
                div { class: "stat-label", "Suites" }
            }
            div { class: "stat-card",
                div { class: "stat-value", "{total_benchmarks}" }
                div { class: "stat-label", "Benchmarks" }
            }
            div { class: "stat-card",
                div { class: "stat-value", "{total_runs}" }
                div { class: "stat-label", "Total Runs" }
            }
            div { class: "stat-card",
                div { class: "stat-value stat-date", "{last_update}" }
                div { class: "stat-label", "Last Updated" }
            }
        }
    }
}

#[component]
fn BenchmarkSuite(name: String, runs: Vec<BenchmarkRun>) -> Element {
    let mut expanded = use_signal(|| true);

    // Get unique benchmark names
    let bench_names: Vec<String> = runs
        .iter()
        .flat_map(|r| r.benches.iter().map(|b| b.name.clone()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let latest_run = runs.last();

    rsx! {
        div { class: "suite-card",
            // Header
            div {
                class: "suite-header",
                onclick: move |_| {
                    let current = *expanded.read();
                    expanded.set(!current);
                },

                div { class: "suite-info",
                    h2 { class: "suite-name", "{name}" }
                    div { class: "suite-meta",
                        span { class: "badge", "{bench_names.len()} benchmarks" }
                        span { class: "badge", "{runs.len()} runs" }
                    }
                }

                button { class: "expand-btn",
                    if *expanded.read() { "▼" } else { "▶" }
                }
            }

            // Content
            if *expanded.read() {
                div { class: "suite-content",
                    // Chart
                    BenchmarkChart {
                        runs: runs.clone(),
                        bench_names: bench_names.clone()
                    }

                    // Latest results table
                    if let Some(run) = latest_run {
                        LatestResults { run: run.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn BenchmarkChart(runs: Vec<BenchmarkRun>, bench_names: Vec<String>) -> Element {
    // Prepare data for visualization
    // Group by benchmark name
    let mut series: HashMap<String, Vec<(String, f64)>> = HashMap::new();

    for run in &runs {
        let date = run.date.format("%m/%d").to_string();
        for bench in &run.benches {
            series
                .entry(bench.name.clone())
                .or_default()
                .push((date.clone(), bench.value));
        }
    }

    // Colors for different benchmarks
    let colors = [
        "#58a6ff", "#3fb950", "#f85149", "#a371f7", "#d29922",
        "#79c0ff", "#56d364", "#ff7b72", "#bc8cff", "#e3b341",
    ];

    // Calculate chart dimensions
    let max_value = series
        .values()
        .flat_map(|points| points.iter().map(|(_, v)| *v))
        .fold(0.0f64, |a, b| a.max(b));

    let chart_height = 300.0;
    let chart_width = 600.0;
    let padding = 60.0;

    rsx! {
        div { class: "chart-container",
            svg {
                class: "chart",
                view_box: "0 0 {chart_width} {chart_height}",
                "preserveAspectRatio": "xMidYMid meet",

                // Grid lines
                for i in 0..5 {
                    line {
                        x1: "{padding}",
                        y1: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                        x2: "{chart_width - padding}",
                        y2: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                        class: "grid-line"
                    }
                }

                // Y-axis labels
                for i in 0..5 {
                    text {
                        x: "{padding - 10.0}",
                        y: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                        class: "axis-label",
                        "text-anchor": "end",
                        "{format_value(max_value * (1.0 - i as f64 / 4.0))}"
                    }
                }

                // Lines for each benchmark
                for (idx, (bench_name, points)) in series.iter().enumerate() {
                    if !points.is_empty() {
                        {
                            let color = colors[idx % colors.len()];
                            let path = generate_line_path(points, max_value, chart_width, chart_height, padding);
                            rsx! {
                                path {
                                    key: "{bench_name}-line",
                                    d: "{path}",
                                    fill: "none",
                                    stroke: "{color}",
                                    "stroke-width": "2"
                                }
                                // Points
                                for (i, (_, value)) in points.iter().enumerate() {
                                    {
                                        let x = padding + (chart_width - 2.0 * padding) * (i as f64 / (points.len().max(1) - 1).max(1) as f64);
                                        let y = padding + (chart_height - 2.0 * padding) * (1.0 - value / max_value.max(1.0));
                                        rsx! {
                                            circle {
                                                key: "{bench_name}-point-{i}",
                                                cx: "{x}",
                                                cy: "{y}",
                                                r: "4",
                                                fill: "{color}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Legend
            div { class: "chart-legend",
                for (idx, name) in bench_names.iter().enumerate() {
                    div { class: "legend-item",
                        span {
                            class: "legend-color",
                            style: "background-color: {colors[idx % colors.len()]}"
                        }
                        span { class: "legend-label", "{name}" }
                    }
                }
            }
        }
    }
}

fn generate_line_path(
    points: &[(String, f64)],
    max_value: f64,
    width: f64,
    height: f64,
    padding: f64,
) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut path = String::new();
    let n = points.len().max(1);

    for (i, (_, value)) in points.iter().enumerate() {
        let x = padding + (width - 2.0 * padding) * (i as f64 / (n - 1).max(1) as f64);
        let y = padding + (height - 2.0 * padding) * (1.0 - value / max_value.max(1.0));

        if i == 0 {
            path.push_str(&format!("M {:.1} {:.1}", x, y));
        } else {
            path.push_str(&format!(" L {:.1} {:.1}", x, y));
        }
    }

    path
}

fn format_value(value: f64) -> String {
    if value >= 1_000_000_000.0 {
        format!("{:.1}s", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.1}ms", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}µs", value / 1_000.0)
    } else {
        format!("{:.0}ns", value)
    }
}

#[component]
fn LatestResults(run: BenchmarkRun) -> Element {
    let commit_short = &run.commit.id[..7.min(run.commit.id.len())];

    rsx! {
        div { class: "results-table",
            h3 { class: "table-title", "Latest Results" }

            div { class: "commit-info",
                span { class: "commit-hash", "{commit_short}" }
                span { class: "commit-message", "{run.commit.message}" }
            }

            table {
                thead {
                    tr {
                        th { "Benchmark" }
                        th { "Value" }
                        th { "Unit" }
                        th { "Range" }
                    }
                }
                tbody {
                    for bench in &run.benches {
                        tr {
                            td { class: "bench-name", "{bench.name}" }
                            td { class: "bench-value", "{bench.value:.2}" }
                            td { class: "bench-unit", "{bench.unit}" }
                            td { class: "bench-range",
                                {bench.range.clone().unwrap_or_else(|| "-".to_string())}
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn load_benchmark_data() -> Result<BenchmarkData, String> {
    let response = Request::get(DATA_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch data: {}", e))?;

    if !response.ok() {
        return Err(format!(
            "HTTP error: {} {}",
            response.status(),
            response.status_text()
        ));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    serde_json::from_str(&text).map_err(|e| format!("Failed to parse JSON: {}", e))
}
