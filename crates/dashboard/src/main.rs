//! Dioxus-based benchmark dashboard for git-bench
//!
//! A pure Rust frontend that compiles to WebAssembly.
//! 100% Rust - no manually written JS/TS/CSS.
//! All styling is inline in Rust code.
//!
//! ## Hierarchical Benchmark Organization
//!
//! Benchmarks are organized hierarchically based on their names:
//! - `grandparent::parent::test` creates a container for `grandparent`, a chart for `parent`,
//!   and plots `test` as a line on that chart.
//! - Multiple tests under the same `grandparent::parent` are plotted on the same chart.
//! - Multiple parents under the same `grandparent` are different charts in the same container.

use dioxus::prelude::*;
use git_bench_core::{BenchmarkData, BenchmarkRun};
use gloo_net::http::Request;
use std::collections::{BTreeMap, HashMap, HashSet};

mod styles;

use styles::*;

/// Global theme context - true = dark mode
#[derive(Clone, Copy)]
struct ThemeCtx(Signal<bool>);

/// Global selection context for from/to commit comparison
#[derive(Clone, Copy)]
struct SelectionCtx {
    from_idx: Signal<Option<usize>>,  // run index (older/start)
    to_idx: Signal<Option<usize>>,    // run index (newer/end)
}

/// Global GitHub repo URL context
#[derive(Clone)]
struct RepoCtx(Signal<Option<String>>);

const DATA_URL: &str = "data.json";

/// Represents a parsed benchmark name hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BenchmarkPath {
    /// Grandparent module (container level)
    grandparent: Option<String>,
    /// Parent module (chart level)
    parent: Option<String>,
    /// Test name (line on chart)
    test: String,
}

impl BenchmarkPath {
    /// Parse a benchmark name into its hierarchical components
    /// Supports both `::` (Rust style) and `/` (Criterion style) separators
    fn parse(name: &str) -> Self {
        // Try `::` first (Rust module style), then `/` (Criterion style)
        let parts: Vec<&str> = if name.contains("::") {
            name.split("::").collect()
        } else {
            name.split('/').collect()
        };
        
        match parts.len() {
            1 => BenchmarkPath {
                grandparent: None,
                parent: None,
                test: parts[0].to_string(),
            },
            2 => BenchmarkPath {
                grandparent: None,
                parent: Some(parts[0].to_string()),
                test: parts[1].to_string(),
            },
            _ => BenchmarkPath {
                grandparent: Some(parts[0].to_string()),
                parent: Some(parts[1..parts.len() - 1].join("/")),
                test: parts[parts.len() - 1].to_string(),
            },
        }
    }

    /// Get the display name for grouping at grandparent level
    fn grandparent_key(&self) -> String {
        self.grandparent
            .clone()
            .unwrap_or_else(|| "_ungrouped".to_string())
    }

    /// Get the display name for grouping at parent level
    fn parent_key(&self) -> String {
        match (&self.grandparent, &self.parent) {
            (Some(_), Some(p)) => p.clone(),
            (None, Some(p)) => p.clone(),
            _ => "_ungrouped".to_string(),
        }
    }
}

/// Hierarchical data structure for benchmarks
/// grandparent -> parent -> test -> data points
type HierarchicalData = BTreeMap<String, BTreeMap<String, Vec<BenchmarkDataPoint>>>;

#[derive(Debug, Clone, PartialEq)]
struct BenchmarkDataPoint {
    test_name: String,
    date: String,
    value: f64,
    unit: String,
    range: Option<String>,
    commit_id: String,
    commit_message: String,
}

/// Run info for the sidebar (each benchmark run, not just unique commits)
#[derive(Debug, Clone, PartialEq)]
struct RunInfo {
    /// Unique run identifier (index in original order)
    run_idx: usize,
    /// Commit ID
    commit_id: String,
    short_id: String,
    message: String,
    author: String,
    /// Precise date/time string (YYYY-MM-DD HH:MM:SS)
    date: String,
    /// Raw timestamp for sorting
    timestamp: i64,
}

/// Build hierarchical data from flat benchmark runs
fn build_hierarchy(runs: &[BenchmarkRun]) -> HierarchicalData {
    let mut hierarchy: HierarchicalData = BTreeMap::new();

    for run in runs {
        let date = run.date.format("%m/%d").to_string();
        for bench in &run.benches {
            let path = BenchmarkPath::parse(&bench.name);
            let grandparent_key = path.grandparent_key();
            let parent_key = path.parent_key();

            let point = BenchmarkDataPoint {
                test_name: path.test.clone(),
                date: date.clone(),
                value: bench.value,
                unit: bench.unit.clone(),
                range: bench.range.clone(),
                commit_id: run.commit.id.clone(),
                commit_message: run.commit.message.clone(),
            };

            hierarchy
                .entry(grandparent_key)
                .or_default()
                .entry(parent_key)
                .or_default()
                .push(point);
        }
    }

    hierarchy
}

/// Extract all runs (each benchmark execution) sorted by date descending (newest first)
fn extract_runs(runs: &[BenchmarkRun]) -> Vec<RunInfo> {
    let mut run_infos: Vec<RunInfo> = runs.iter()
        .enumerate()
        .map(|(idx, run)| {
            let author_name = run.commit.author.as_ref()
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            RunInfo {
                run_idx: idx,
                commit_id: run.commit.id.clone(),
                short_id: run.commit.id[..7.min(run.commit.id.len())].to_string(),
                message: run.commit.message.clone(),
                author: author_name,
                date: run.date.format("%Y-%m-%d %H:%M:%S").to_string(),
                timestamp: run.date.timestamp(),
            }
        })
        .collect();
    
    // Sort by timestamp descending (newest first)
    run_infos.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    run_infos
}

fn main() {
    tracing_wasm::set_as_global_default();
    launch(App);
}

#[component]
fn App() -> Element {
    // Theme state - default to dark mode
    let dark_mode = use_signal(|| true);
    use_context_provider(|| ThemeCtx(dark_mode));
    
    // Selection state for from/to (indices into runs list)
    // Will be initialized with defaults when data loads
    let from_idx = use_signal(|| None::<usize>);
    let to_idx = use_signal(|| None::<usize>);
    use_context_provider(|| SelectionCtx { from_idx, to_idx });
    
    // GitHub repo URL (can be configured)
    let repo_url = use_signal(|| Some("https://github.com/user/repo".to_string()));
    use_context_provider(|| RepoCtx(repo_url));

    let mut data = use_signal(|| None::<BenchmarkData>);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);

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

    let dark = *dark_mode.read();
    let body_bg = if dark { "#0d1117" } else { "#ffffff" };

    rsx! {
        // Global style to fix html/body background
        style { 
            "html, body {{ margin: 0; padding: 0; background: {body_bg}; min-height: 100vh; }} \
             #main {{ min-height: 100vh; }}"
        }
        div { style: "{app_style(dark)}",
            Header {}

            div { style: "display: flex; flex: 1; overflow: hidden;",
                if *loading.read() {
                    main { style: "{main_content_style(dark)}",
                    LoadingState {}
                    }
                } else if let Some(err) = error.read().as_ref() {
                    main { style: "{main_content_style(dark)}",
                    ErrorState { message: err.clone() }
                    }
                } else if let Some(benchmark_data) = data.read().as_ref() {
                    if benchmark_data.entries.is_empty() {
                        main { style: "{main_content_style(dark)}",
                        EmptyState {}
                        }
                    } else {
                        Dashboard { data: benchmark_data.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn Header() -> Element {
    let ThemeCtx(mut dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    rsx! {
        header { style: "{header_style(dark)}",
            div { style: "display: flex; align-items: center; gap: 0.5rem;",
                span { style: "font-size: 1.2rem;", "⑂" }
            h1 { style: "{title_style(dark)}", "git-bench" }
            }
            button {
                style: "{toggle_btn_style(dark)}",
                onclick: move |_| {
                    let current = *dark_mode.read();
                    dark_mode.set(!current);
                },
                if dark { "☀ light" } else { "☾ dark" }
            }
        }
    }
}

#[component]
fn LoadingState() -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    rsx! {
        div { style: "{loading_style(dark)}",
            "Loading..."
        }
    }
}

#[component]
fn ErrorState(message: String) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    rsx! {
        div { style: "{error_style(dark)}",
            strong { "Error: " }
            "{message}"
        }
    }
}

#[component]
fn EmptyState() -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    rsx! {
        div { style: "{empty_style(dark)}",
            p { "No benchmark data." }
            code { style: "{code_style(dark)}", "cargo bench | git-bench run" }
        }
    }
}

#[component]
fn Dashboard(data: BenchmarkData) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let SelectionCtx { mut from_idx, mut to_idx } = use_context::<SelectionCtx>();
    let dark = *dark_mode.read();
    
    // Extract all runs from all suites (use first suite for sidebar)
    let first_suite_runs: Vec<RunInfo> = data.entries
        .values()
        .next()
        .map(|runs| extract_runs(runs))
        .unwrap_or_default();
    
    // Set default from/to if not set (only once)
    // Runs are sorted newest-first, so:
    //   - to = first item's run_idx (newest/latest)
    //   - from = second item's run_idx (previous)
    // Note: run_idx is the original order index, not sorted position
    let to_default = first_suite_runs.first().map(|r| r.run_idx);
    let from_default = first_suite_runs.get(1).map(|r| r.run_idx);
    
    use_effect(move || {
        // Read both values first to avoid partial updates
        let from_is_none = from_idx.read().is_none();
        let to_is_none = to_idx.read().is_none();
        
        if let (Some(from_val), Some(to_val)) = (from_default, to_default) {
            if from_is_none && to_is_none {
                from_idx.set(Some(from_val));
                to_idx.set(Some(to_val));
            }
        } else if let Some(to_val) = to_default {
            if to_is_none {
                to_idx.set(Some(to_val));
            }
        }
    });

    rsx! {
        // Left sidebar with runs
        RunsSidebar { runs: first_suite_runs.clone() }
        
        // Main content area
        main { style: "{main_content_style(dark)}",
            for (suite_name, runs) in data.entries.iter() {
                SuiteSection {
                    key: "{suite_name}",
                    suite_name: suite_name.clone(),
                    runs: runs.clone()
                }
            }
        }
    }
}

/// Left sidebar showing runs list
#[component]
fn RunsSidebar(runs: Vec<RunInfo>) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let SelectionCtx { mut from_idx, mut to_idx } = use_context::<SelectionCtx>();
    let RepoCtx(repo_url) = use_context::<RepoCtx>();
    let dark = *dark_mode.read();

    let mut search_query = use_signal(|| String::new());
    
    // Build a map from run_idx to timestamp for constraint checking
    let timestamp_map: HashMap<usize, i64> = runs.iter()
        .map(|r| (r.run_idx, r.timestamp))
        .collect();
    
    let filtered_runs: Vec<&RunInfo> = runs
        .iter()
        .filter(|r| {
            let query = search_query.read().to_lowercase();
            if query.is_empty() {
                return true;
            }
            r.commit_id.to_lowercase().contains(&query) 
                || r.message.to_lowercase().contains(&query)
                || r.author.to_lowercase().contains(&query)
        })
        .collect();

    rsx! {
        aside { style: "{sidebar_style(dark)}",
            // Search input
            div { style: "padding: 0.75rem;",
                input {
                    style: "{search_input_style(dark)}",
                    r#type: "text",
                    placeholder: "Search commits...",
                    value: "{search_query}",
                    oninput: move |e| search_query.set(e.value())
                }
            }
            
            // Runs header
            div { style: "{sidebar_section_header(dark)}",
                span { "Commits" }
            }
            
            // Help text
            div { style: "padding: 0.5rem 0.75rem; font-size: 0.7rem; {muted_style(dark)}",
                "Click "
                span { style: "opacity: 0.7;", "◉" }
                " to set FROM or "
                span { style: "opacity: 0.7;", "⇌" }
                " to set TO"
            }
            
            // Runs list
            div { style: "flex: 1; overflow-y: auto;",
                for run in filtered_runs {
                    {
                        let run_idx = run.run_idx;
                        let run_timestamp = run.timestamp;
                        let is_from = *from_idx.read() == Some(run_idx);
                        let is_to = *to_idx.read() == Some(run_idx);
                        let repo = repo_url.read().clone();
                        let commit_url = repo.map(|r| format!("{}/commit/{}", r, run.commit_id.clone()));
                        let ts_map = timestamp_map.clone();
                        let ts_map2 = timestamp_map.clone();
                        
                        rsx! {
                            div {
                                key: "run-{run_idx}",
                                style: "{commit_item_style(dark, is_from || is_to)}",
                                
                                // Left indicator
                                if is_from || is_to {
                                    div { style: "{commit_indicator_style(dark)}" }
                                }
                                
                                // Run info
                                div { style: "flex: 1; min-width: 0; padding-left: 0.5rem;",
                                    div { style: "display: flex; align-items: center; gap: 0.5rem;",
                                        // Commit hash link
                                        if let Some(url) = &commit_url {
                                            a {
                                                href: "{url}",
                                                target: "_blank",
                                                style: "{commit_hash_link_style(dark)}",
                                                "{run.short_id}"
                                            }
                                        } else {
                                            span { style: "font-family: monospace; font-size: 0.8rem;", "{run.short_id}" }
                                        }
                                        
                                        // Badges
                                        if is_to {
                                            span { style: "{badge_compare_style(dark)}", "TO" }
                                        }
                                        if is_from {
                                            span { style: "{badge_baseline_style(dark)}", "FROM" }
                                        }
                                    }
                                    div { style: "font-size: 0.7rem; {muted_style(dark)} margin-top: 0.2rem;",
                                        "{run.date}"
                                    }
                                }
                                
                                // Action buttons
                                div { style: "display: flex; gap: 0.25rem;",
                                    button {
                                        style: "{icon_btn_style(dark, is_from)}",
                                        title: "Set as FROM (older commit)",
                onclick: move |_| {
                                            let current_from = *from_idx.read();
                                            let current_to = *to_idx.read();
                                            
                                            if current_from == Some(run_idx) {
                                                from_idx.set(None);
                                            } else {
                                                from_idx.set(Some(run_idx));
                                                // If new from is newer than to, also update to
                                                if let Some(to_i) = current_to {
                                                    if let Some(&to_ts) = ts_map.get(&to_i) {
                                                        // From should be older (smaller timestamp)
                                                        // If new from timestamp > to timestamp, update to
                                                        if run_timestamp > to_ts {
                                                            to_idx.set(Some(run_idx));
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        "◉"
                                    }
                                    button {
                                        style: "{icon_btn_style(dark, is_to)}",
                                        title: "Set as TO (newer commit)",
                                        onclick: move |_| {
                                            let current_from = *from_idx.read();
                                            let current_to = *to_idx.read();
                                            
                                            if current_to == Some(run_idx) {
                                                to_idx.set(None);
                                            } else {
                                                to_idx.set(Some(run_idx));
                                                // If new to is older than from, also update from
                                                if let Some(from_i) = current_from {
                                                    if let Some(&from_ts) = ts_map2.get(&from_i) {
                                                        // To should be newer (larger timestamp)
                                                        // If new to timestamp < from timestamp, update from
                                                        if run_timestamp < from_ts {
                                                            from_idx.set(Some(run_idx));
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        "⇌"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Suite section - contains the overall suite header and hierarchical module containers
#[component]
fn SuiteSection(suite_name: String, runs: Vec<BenchmarkRun>) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    let hierarchy = build_hierarchy(&runs);
    let runs_info = extract_runs(&runs);

    // Determine if we have hierarchical benchmarks
    let has_hierarchy = hierarchy.keys().any(|k| k != "_ungrouped")
        || hierarchy.get("_ungrouped").map_or(false, |m| m.keys().any(|k| k != "_ungrouped"));

    rsx! {
        div { style: "margin-bottom: 2rem;",
            // Suite title
            h2 { style: "{suite_title_style(dark)}", "{suite_name}" }

                    if has_hierarchy {
                        for (grandparent, parents) in hierarchy.iter() {
                            if grandparent != "_ungrouped" {
                                ModuleContainer {
                                    key: "{grandparent}",
                                    name: grandparent.clone(),
                                    charts: parents.clone(),
                            runs: runs.clone(),
                            runs_info: runs_info.clone()
                                }
                            }
                        }
                        // Handle 2-level hierarchy (parent/test) - render directly as charts
                        if let Some(ungrouped) = hierarchy.get("_ungrouped") {
                            for (parent_name, points) in ungrouped.iter() {
                                if parent_name != "_ungrouped" {
                            BenchmarkChart {
                                        key: "{parent_name}",
                                        name: parent_name.clone(),
                                data_points: points.clone(),
                                runs_info: runs_info.clone()
                                    }
                                }
                            }
                            // Truly ungrouped (single-level names)
                            if let Some(truly_ungrouped) = ungrouped.get("_ungrouped") {
                        BenchmarkChart {
                                    name: "other".to_string(),
                            data_points: truly_ungrouped.clone(),
                            runs_info: runs_info.clone()
                                }
                            }
                        }
                    } else {
                // Flat view when no hierarchy
                for (_grandparent, parents) in hierarchy.iter() {
                    for (parent_name, points) in parents.iter() {
                        BenchmarkChart {
                            key: "{parent_name}",
                            name: if parent_name == "_ungrouped" { "benchmarks".to_string() } else { parent_name.clone() },
                            data_points: points.clone(),
                            runs_info: runs_info.clone()
                        }
                    }
                }
            }
        }
    }
}

/// Module container - groups multiple charts under a grandparent module
#[component]
fn ModuleContainer(
    name: String,
    charts: BTreeMap<String, Vec<BenchmarkDataPoint>>,
    #[allow(unused)] runs: Vec<BenchmarkRun>,
    runs_info: Vec<RunInfo>,
) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    rsx! {
        div { style: "{container_card_style(dark)}",
            // Container header
            div { style: "{container_header_style(dark)}",
                span { style: "{container_title_style(dark)}", "{name}" }
            }

            div { style: "padding: 1rem;",
                    for (parent_name, points) in charts.iter() {
                        if parent_name != "_ungrouped" {
                        BenchmarkChart {
                                key: "{parent_name}",
                                name: parent_name.clone(),
                            data_points: points.clone(),
                            runs_info: runs_info.clone()
                            }
                        }
                    }
                    if let Some(ungrouped_points) = charts.get("_ungrouped") {
                    BenchmarkChart {
                            name: "other".to_string(),
                        data_points: ungrouped_points.clone(),
                        runs_info: runs_info.clone()
                    }
                }
            }
        }
    }
}

/// Data for a single commit's tooltip
#[derive(Debug, Clone, PartialEq)]
struct CommitTooltipData {
    commit_id: String,
    commit_short: String,
    /// Values: (test_name, value, unit, color)
    values: Vec<(String, f64, String, String)>,
}

/// Main benchmark chart component
#[component]
fn BenchmarkChart(name: String, data_points: Vec<BenchmarkDataPoint>, runs_info: Vec<RunInfo>) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let SelectionCtx { from_idx, to_idx } = use_context::<SelectionCtx>();
    let dark = *dark_mode.read();

    // Track hovered commit
    let mut hovered_commit: Signal<Option<usize>> = use_signal(|| None);
    // Track if metrics comparison is expanded
    let mut metrics_expanded = use_signal(|| true);

    // Build series data
    let mut series: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    for point in &data_points {
        series
            .entry(point.test_name.clone())
            .or_default()
            .push((point.commit_id.clone(), point.value));
    }

    let test_names: Vec<String> = series.keys().cloned().collect();
    let unit = data_points
        .first()
        .map(|p| p.unit.clone())
        .unwrap_or_default();

    let color_map: HashMap<String, String> = test_names
        .iter()
        .enumerate()
        .map(|(idx, name)| {
            (
                name.clone(),
                CHART_COLORS[idx % CHART_COLORS.len()].to_string(),
            )
        })
        .collect();

    let max_value = series
        .values()
        .flat_map(|points| points.iter().map(|(_, v)| *v))
        .fold(0.0f64, |a, b| a.max(b));

    // Get unique commits in order for this chart
    let chart_commits: Vec<String> = {
        let mut seen = HashSet::new();
        data_points
            .iter()
            .filter_map(|p| {
                if seen.insert(p.commit_id.clone()) {
                    Some(p.commit_id.clone())
                } else {
                    None
                }
            })
            .collect()
    };
    
    let num_commits = chart_commits.len();

    // Build tooltip data for each commit position
    let mut points_by_test: BTreeMap<String, Vec<&BenchmarkDataPoint>> = BTreeMap::new();
    for point in &data_points {
        points_by_test.entry(point.test_name.clone()).or_default().push(point);
    }
    
    let num_points = series.values().next().map_or(0, |v| v.len());
    let commits_tooltip: Vec<CommitTooltipData> = (0..num_points)
        .map(|idx| {
            let reference_point = points_by_test.values().next()
                .and_then(|pts| pts.get(idx))
                .expect("should have data point");
            
            let values: Vec<(String, f64, String, String)> = test_names
                .iter()
                .filter_map(|test_name| {
                    points_by_test.get(test_name)
                        .and_then(|pts| pts.get(idx))
                        .map(|p| (
                            p.test_name.clone(),
                            p.value,
                            p.unit.clone(),
                            color_map.get(&p.test_name).cloned().unwrap_or_default(),
                        ))
                })
                .collect();
            
            CommitTooltipData {
                commit_id: reference_point.commit_id.clone(),
                commit_short: reference_point.commit_id[..7.min(reference_point.commit_id.len())].to_string(),
                values,
            }
        })
        .collect();

    // Get last run info for header
    let last_run = runs_info.last();
    let modified_info = last_run.map(|r| format!("Modified: {} by {}", r.date, r.author));

    // Calculate metrics comparison (from vs to or latest vs previous)
    let from_selection = *from_idx.read();
    let to_selection = *to_idx.read();
    let metrics_comparison = calculate_metrics_comparison(&data_points, &test_names, from_selection, to_selection);

    // Map selection indices to chart positions
    // from_selection and to_selection are run_idx values, we need to find their commit_ids
    // and map to chart_commits positions
    let from_chart_pos: Option<usize> = from_selection.and_then(|run_i| {
        runs_info.iter()
            .find(|r| r.run_idx == run_i)
            .and_then(|r| chart_commits.iter().position(|c| c == &r.commit_id))
    });
    let to_chart_pos: Option<usize> = to_selection.and_then(|run_i| {
        runs_info.iter()
            .find(|r| r.run_idx == run_i)
            .and_then(|r| chart_commits.iter().position(|c| c == &r.commit_id))
    });

    rsx! {
        div { style: "{chart_card_style(dark)}",
            // Chart header
            div { style: "{chart_header_style(dark)}",
                div { style: "display: flex; align-items: center; gap: 0.5rem;",
                span { style: "{chart_title_style(dark)}", "{name}" }
                span { style: "{unit_badge_style(dark)}", "{unit}" }
                }
                if let Some(info) = &modified_info {
                    span { style: "font-size: 0.75rem; {muted_style(dark)}", "{info}" }
                }
            }

            // Chart SVG
            ChartSvg {
                series: series.clone(),
                max_value: max_value,
                chart_commits: chart_commits.clone(),
                commits_tooltip: commits_tooltip.clone(),
                hovered_commit: hovered_commit,
                from_chart_pos: from_chart_pos,
                to_chart_pos: to_chart_pos,
                chart_height: 200.0,
                chart_width: 600.0
            }

            // Legend
            div { style: "{chart_legend_style(dark)}",
                for (idx, test_name) in test_names.iter().enumerate() {
                    div { style: "display: flex; align-items: center; gap: 0.4rem;",
                        span { style: "width: 12px; height: 12px; border-radius: 50%; background: {CHART_COLORS[idx % CHART_COLORS.len()]};" }
                        span { "{test_name}" }
                    }
                }
            }

            // Metrics Comparison (collapsible)
            if !metrics_comparison.is_empty() {
                div { style: "{metrics_section_style(dark)}",
                    button {
                        style: "{metrics_toggle_style(dark)}",
                        onclick: move |_| {
                            let current = *metrics_expanded.read();
                            metrics_expanded.set(!current);
                        },
                        span { if *metrics_expanded.read() { "▼" } else { "▶" } }
                        span { " Metrics Comparison" }
                    }
                    
                    if *metrics_expanded.read() {
                        div { style: "margin-top: 0.5rem;",
                            // Table header
                            div { style: "{metrics_table_header_style(dark)}",
                                span { style: "flex: 2;", "Variant" }
                                span { style: "flex: 1; text-align: right;", "From" }
                                span { style: "flex: 1; text-align: right;", "To" }
                                span { style: "flex: 1; text-align: right;", "Change" }
                            }
                            
                            // Table rows
                            for (test_name, from_value, to_value, change_pct, color) in metrics_comparison.iter() {
                                div { style: "{metrics_table_row_style(dark)}",
                                    div { style: "flex: 2; display: flex; align-items: center; gap: 0.4rem;",
                                        span { style: "width: 8px; height: 8px; border-radius: 50%; background: {color};" }
                                        span { "{test_name}" }
                                    }
                                    span { 
                                        style: "flex: 1; text-align: right; {muted_style(dark)}",
                                        {
                                            match from_value {
                                                Some(v) => format!("{:.2}", v),
                                                None => "—".to_string(),
                                            }
                                        }
                                    }
                                    span { 
                                        style: "flex: 1; text-align: right;",
                                        "{to_value:.2}"
                                    }
                                    span { 
                                        style: "flex: 1; text-align: right; font-weight: 500; color: {change_color(*change_pct)};",
                                        "{format_change(*change_pct)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Metrics comparison result: (test_name, from_value, to_value, change_pct, color)
fn calculate_metrics_comparison(
    data_points: &[BenchmarkDataPoint],
    test_names: &[String],
    from_idx: Option<usize>,
    to_idx: Option<usize>,
) -> Vec<(String, Option<f64>, f64, f64, String)> {
    let mut result = Vec::new();
    
    for (idx, test_name) in test_names.iter().enumerate() {
        let test_points: Vec<_> = data_points
            .iter()
            .filter(|p| p.test_name == *test_name)
            .collect();
        
        if test_points.is_empty() {
            continue;
        }
        
        // Get from and to values using indices
        let (from_value, to_value) = match (from_idx, to_idx) {
            (Some(from_i), Some(to_i)) => {
                let from_val = test_points.get(from_i).map(|p| p.value);
                let to_val = test_points.get(to_i).map(|p| p.value);
                (from_val, to_val)
            }
            _ => {
                // Default to comparing last two if we have enough data
                if test_points.len() >= 2 {
                    let prev = test_points.get(test_points.len() - 2).map(|p| p.value);
                    let curr = test_points.last().map(|p| p.value);
                    (prev, curr)
                } else {
                    (None, None)
                }
            }
        };
        
        if let Some(to_val) = to_value {
            let change_pct = match from_value {
                Some(from_val) if from_val != 0.0 => ((to_val - from_val) / from_val) * 100.0,
                _ => 0.0,
            };
            let color = CHART_COLORS[idx % CHART_COLORS.len()].to_string();
            result.push((test_name.clone(), from_value, to_val, change_pct, color));
        }
    }
    
    result
}

fn change_color(change_pct: f64) -> &'static str {
    if change_pct > 5.0 {
        "#ff6b6b"  // Red for regression
    } else if change_pct < -5.0 {
        "#51cf66"  // Green for improvement
    } else {
        "#868e96"  // Gray for neutral
    }
}

fn format_change(change_pct: f64) -> String {
    if change_pct >= 0.0 {
        format!("+{:.1}%", change_pct)
    } else {
        format!("{:.1}%", change_pct)
    }
}

/// Reusable SVG chart component
#[component]
fn ChartSvg(
    series: BTreeMap<String, Vec<(String, f64)>>,
    max_value: f64,
    chart_commits: Vec<String>,
    commits_tooltip: Vec<CommitTooltipData>,
    mut hovered_commit: Signal<Option<usize>>,
    from_chart_pos: Option<usize>,
    to_chart_pos: Option<usize>,
    chart_height: f64,
    chart_width: f64,
) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    let padding_left = 50.0;
    let padding_right = 20.0;
    let padding_top = 20.0;
    let padding_bottom = 40.0;
    
    let num_commits = chart_commits.len();
    let grid_c = grid_color(dark);
    let axis_c = axis_color(dark);

    let padding_ratio_left = padding_left / chart_width;
    let padding_ratio_right = padding_right / chart_width;
    
    let mut chart_div_width = use_signal(|| 0.0f64);

    rsx! {
        div {
            style: "padding: 0.5rem; position: relative; cursor: crosshair;",
            onmounted: move |evt| {
                let mounted_data = evt.data().clone();
                spawn(async move {
                    if let Ok(rect) = mounted_data.get_client_rect().await {
                        chart_div_width.set(rect.width());
                    }
                });
            },
            onmouseleave: move |_| hovered_commit.set(None),
            onmousemove: move |e| {
                let coords = e.data().element_coordinates();
                let element_x = coords.x;
                let div_width = *chart_div_width.read();
                
                if div_width > 0.0 && num_commits > 0 {
                    let fraction = element_x / div_width;
                    let chart_start = padding_ratio_left;
                    let chart_end = 1.0 - padding_ratio_right;
                    
                    if fraction >= chart_start && fraction <= chart_end {
                        let chart_fraction = (fraction - chart_start) / (chart_end - chart_start);
                        if num_commits > 1 {
                            let commit_idx = (chart_fraction * (num_commits - 1) as f64).round() as usize;
                            let clamped_idx = commit_idx.min(num_commits.saturating_sub(1));
                            hovered_commit.set(Some(clamped_idx));
                        } else {
                            hovered_commit.set(Some(0));
                        }
                    } else if fraction < chart_start {
                        hovered_commit.set(Some(0));
                    } else {
                        hovered_commit.set(Some(num_commits - 1));
                    }
                }
            },

            svg {
                style: "width: 100%; height: auto; max-height: 200px; pointer-events: none;",
                view_box: "0 0 {chart_width} {chart_height}",
                "preserveAspectRatio": "xMidYMid meet",

                // Horizontal grid lines
                for i in 0..5 {
                    line {
                        x1: "{padding_left}",
                        y1: "{padding_top + (chart_height - padding_top - padding_bottom) * (i as f64 / 4.0)}",
                        x2: "{chart_width - padding_right}",
                        y2: "{padding_top + (chart_height - padding_top - padding_bottom) * (i as f64 / 4.0)}",
                        stroke: "{grid_c}",
                        "stroke-width": "1",
                        style: "pointer-events: none;"
                    }
                }

                // Y-axis labels
                for i in 0..5 {
                    text {
                        x: "{padding_left - 8.0}",
                        y: "{padding_top + (chart_height - padding_top - padding_bottom) * (i as f64 / 4.0) + 4.0}",
                        fill: "{axis_c}",
                        "font-size": "10",
                        "text-anchor": "end",
                        style: "pointer-events: none;",
                        "{format_value(max_value * (1.0 - i as f64 / 4.0))}"
                    }
                }

                // X-axis commit labels
                for (i, commit_id) in chart_commits.iter().enumerate() {
                    {
                        let x = padding_left + (chart_width - padding_left - padding_right) * (i as f64 / (num_commits.max(1) - 1).max(1) as f64);
                        let short_id = &commit_id[..7.min(commit_id.len())];
                        // Only show every Nth label if too many
                        let show_label = num_commits <= 8 || i % ((num_commits / 6).max(1)) == 0 || i == num_commits - 1;
                        rsx! {
                            if show_label {
                                text {
                                    x: "{x}",
                                    y: "{chart_height - 10.0}",
                                    fill: "{axis_c}",
                                    "font-size": "9",
                                    "text-anchor": "middle",
                                    style: "pointer-events: none; font-family: monospace;",
                                    "{short_id}"
                                }
                            }
                        }
                    }
                }

                // Data lines and points
                for (idx, (test_name, points)) in series.iter().enumerate() {
                    if !points.is_empty() {
                        {
                            let color = CHART_COLORS[idx % CHART_COLORS.len()];
                            let path = generate_line_path_v2(points, max_value, chart_width, chart_height, padding_left, padding_right, padding_top, padding_bottom);
                            rsx! {
                                path {
                                    key: "{test_name}-line",
                                    d: "{path}",
                                    fill: "none",
                                    stroke: "{color}",
                                    "stroke-width": "2",
                                    style: "pointer-events: none;"
                                }
                                for (i, (_, value)) in points.iter().enumerate() {
                                    {
                                        let x = padding_left + (chart_width - padding_left - padding_right) * (i as f64 / (points.len().max(1) - 1).max(1) as f64);
                                        let y = padding_top + (chart_height - padding_top - padding_bottom) * (1.0 - value / max_value.max(1.0));
                                        rsx! {
                                            circle {
                                                key: "{test_name}-point-{i}",
                                                cx: "{x}",
                                                cy: "{y}",
                                                r: "5",
                                                fill: "{color}",
                                                style: "pointer-events: none;"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Selection markers (FROM and TO) - subtle dashed lines
                // FROM marker
                if let Some(from_pos) = from_chart_pos {
                    if from_pos < num_commits {
                        {
                            let x = padding_left + (chart_width - padding_left - padding_right) * (from_pos as f64 / (num_commits.max(1) - 1).max(1) as f64);
                            let marker_color = "#74c0fc";  // Soft blue for both markers
                            rsx! {
                                line {
                                    x1: "{x}", y1: "{padding_top}", x2: "{x}", y2: "{chart_height - padding_bottom}",
                                    stroke: "{marker_color}", "stroke-width": "1.5", opacity: "0.5",
                                    "stroke-dasharray": "4,3",
                                    style: "pointer-events: none;"
                                }
                            }
                        }
                    }
                }

                // TO marker
                if let Some(to_pos) = to_chart_pos {
                    if to_pos < num_commits {
                        {
                            let x = padding_left + (chart_width - padding_left - padding_right) * (to_pos as f64 / (num_commits.max(1) - 1).max(1) as f64);
                            let marker_color = "#74c0fc";  // Soft blue for both markers
                            rsx! {
                                line {
                                    x1: "{x}", y1: "{padding_top}", x2: "{x}", y2: "{chart_height - padding_bottom}",
                                    stroke: "{marker_color}", "stroke-width": "1.5", opacity: "0.5",
                                    "stroke-dasharray": "4,3",
                                    style: "pointer-events: none;"
                                }
                            }
                        }
                    }
                }

                // Hover line
                if let Some(idx) = *hovered_commit.read() {
                    if idx < num_commits {
                        {
                            let x = padding_left + (chart_width - padding_left - padding_right) * (idx as f64 / (num_commits.max(1) - 1).max(1) as f64);
                            rsx! {
                                line {
                                    x1: "{x}", y1: "{padding_top}", x2: "{x}", y2: "{chart_height - padding_bottom}",
                                    stroke: "#4080ff", "stroke-width": "2", "stroke-dasharray": "4,4", opacity: "0.8",
                                    style: "pointer-events: none;"
                                }
                            }
                        }
                    }
                }
            }

            // Tooltip
            if let Some(idx) = *hovered_commit.read() {
                if let Some(commit_data) = commits_tooltip.get(idx) {
                    {
                        let point_pct = (idx as f64 / (num_commits.max(1) - 1).max(1) as f64) * 100.0;
                        let tooltip_style_pos = if point_pct < 50.0 {
                            format!("right: auto; left: calc({}% + 20px);", point_pct.max(5.0))
                        } else {
                            format!("left: auto; right: calc({}% + 20px);", (100.0 - point_pct).max(5.0))
                        };
                        rsx! {
                            div {
                                style: "{hover_tooltip_style(dark)} top: 20px; {tooltip_style_pos}",

                                div { style: "font-family: monospace; font-weight: 600; margin-bottom: 0.3rem;",
                                    "{commit_data.commit_short}"
                                }
                                for (test_name, value, unit, color) in commit_data.values.iter() {
                                    div { style: "display: flex; align-items: center; gap: 0.3rem; font-size: 0.75rem;",
                                        span { style: "width: 8px; height: 8px; border-radius: 50%; background: {color};" }
                                        span { style: "color: {color};", "{test_name}" }
                                        span { style: "font-weight: 500;", " : " }
                                        span { "{value:.0}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn generate_line_path_v2(
    points: &[(String, f64)],
    max_value: f64,
    width: f64,
    height: f64,
    padding_left: f64,
    padding_right: f64,
    padding_top: f64,
    padding_bottom: f64,
) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut path = String::new();
    let n = points.len().max(1);
    let chart_width = width - padding_left - padding_right;
    let chart_height = height - padding_top - padding_bottom;

    for (i, (_, value)) in points.iter().enumerate() {
        let x = padding_left + chart_width * (i as f64 / (n - 1).max(1) as f64);
        let y = padding_top + chart_height * (1.0 - value / max_value.max(1.0));

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
        format!("{:.1}G", value / 1_000_000_000.0)
    } else if value >= 1_000_000.0 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.1}K", value / 1_000.0)
    } else if value >= 1.0 {
        format!("{:.0}", value)
    } else if value >= 0.001 {
        format!("{:.2}", value)
    } else {
        format!("{:.3}", value)
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
