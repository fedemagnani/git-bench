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

fn main() {
    tracing_wasm::set_as_global_default();
    launch(App);
}

#[component]
fn App() -> Element {
    // Theme state - default to dark mode
    let dark_mode = use_signal(|| true);
    use_context_provider(|| ThemeCtx(dark_mode));

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

    rsx! {
        div { style: "{app_style(dark)}",
            Header {}

            main { style: "{container_style(dark)}",
                if *loading.read() {
                    LoadingState {}
                } else if let Some(err) = error.read().as_ref() {
                    ErrorState { message: err.clone() }
                } else if let Some(benchmark_data) = data.read().as_ref() {
                    if benchmark_data.entries.is_empty() {
                        EmptyState {}
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
            h1 { style: "{title_style(dark)}", "git-bench" }
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
    rsx! {
        div {
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

/// Suite section - contains the overall suite header and hierarchical module containers
#[component]
fn SuiteSection(suite_name: String, runs: Vec<BenchmarkRun>) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    let mut expanded = use_signal(|| true);
    let hierarchy = build_hierarchy(&runs);

    // Determine if we have hierarchical benchmarks
    // Either we have grandparent modules, or we have parent modules (2-level hierarchy)
    let has_hierarchy = hierarchy.keys().any(|k| k != "_ungrouped")
        || hierarchy.get("_ungrouped").map_or(false, |m| m.keys().any(|k| k != "_ungrouped"));

    let bench_count: usize = runs
        .iter()
        .flat_map(|r| r.benches.iter())
        .map(|b| &b.name)
        .collect::<HashSet<_>>()
        .len();

    rsx! {
        div { style: "{section_style(dark)}",
            div {
                style: "{section_header_style(dark)}",
                onclick: move |_| {
                    let current = *expanded.read();
                    expanded.set(!current);
                },

                div { style: "display: flex; align-items: center;",
                    span { style: "{section_title_style(dark)}", "{suite_name}" }
                    span { style: "{badge_style(dark)}", "{bench_count}" }
                    span { style: "{badge_style(dark)}", "{runs.len()} runs" }
                }

                button { style: "{expand_btn_style(dark)}",
                    if *expanded.read() { "−" } else { "+" }
                }
            }

            if *expanded.read() {
                div { style: "padding: 0.25rem;",
                    if has_hierarchy {
                        for (grandparent, parents) in hierarchy.iter() {
                            if grandparent != "_ungrouped" {
                                ModuleContainer {
                                    key: "{grandparent}",
                                    name: grandparent.clone(),
                                    charts: parents.clone(),
                                    runs: runs.clone()
                                }
                            }
                        }
                        // Handle 2-level hierarchy (parent/test) - render directly as charts
                        if let Some(ungrouped) = hierarchy.get("_ungrouped") {
                            for (parent_name, points) in ungrouped.iter() {
                                if parent_name != "_ungrouped" {
                                    ModuleChart {
                                        key: "{parent_name}",
                                        name: parent_name.clone(),
                                        data_points: points.clone()
                                    }
                                }
                            }
                            // Truly ungrouped (single-level names)
                            if let Some(truly_ungrouped) = ungrouped.get("_ungrouped") {
                                ModuleChart {
                                    name: "other".to_string(),
                                    data_points: truly_ungrouped.clone()
                                }
                            }
                        }
                    } else {
                        FlatBenchmarkView { runs: runs.clone() }
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
) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    let mut expanded = use_signal(|| true);

    let chart_count = charts.len();
    let test_count: usize = charts
        .values()
        .flat_map(|points| points.iter().map(|p| &p.test_name))
        .collect::<HashSet<_>>()
        .len();

    rsx! {
        div { style: "{module_style(dark)}",
            div {
                style: "{module_header_style(dark)}",
                onclick: move |_| {
                    let current = *expanded.read();
                    expanded.set(!current);
                },

                div { style: "display: flex; align-items: center;",
                    span { style: "{module_title_style(dark)}", "{name}" }
                    span { style: "{badge_style(dark)}", "{chart_count}" }
                    span { style: "{badge_style(dark)}", "{test_count} tests" }
                }

                button { style: "{expand_btn_style(dark)}",
                    if *expanded.read() { "−" } else { "+" }
                }
            }

            if *expanded.read() {
                div { style: "padding: 0.25rem;",
                    for (parent_name, points) in charts.iter() {
                        if parent_name != "_ungrouped" {
                            ModuleChart {
                                key: "{parent_name}",
                                name: parent_name.clone(),
                                data_points: points.clone()
                            }
                        }
                    }
                    if let Some(ungrouped_points) = charts.get("_ungrouped") {
                        ModuleChart {
                            name: "other".to_string(),
                            data_points: ungrouped_points.clone()
                        }
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
    commit_message: String,
    date: String,
    /// Values sorted by value descending: (test_name, value, unit, range, color)
    values: Vec<(String, f64, String, Option<String>, String)>,
}

/// Module chart - displays a single chart for a parent module with test lines
#[component]
fn ModuleChart(name: String, data_points: Vec<BenchmarkDataPoint>) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    // Track hovered commit - use Option<usize> for the commit index
    let mut hovered_commit: Signal<Option<usize>> = use_signal(|| None);
    // Track if chart is expanded
    let mut expanded: Signal<bool> = use_signal(|| false);

    let mut series: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    for point in &data_points {
        series
            .entry(point.test_name.clone())
            .or_default()
            .push((point.date.clone(), point.value));
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

    // Build commits_ordered by position index, not by commit_id
    // Each position in the series corresponds to one benchmark run
    let num_points = series.values().next().map_or(0, |v| v.len());
    
    // Group data_points by test_name, keeping the order
    let mut points_by_test: BTreeMap<String, Vec<&BenchmarkDataPoint>> = BTreeMap::new();
    for point in &data_points {
        points_by_test.entry(point.test_name.clone()).or_default().push(point);
    }
    
    // Build one CommitTooltipData per position index
    let commits_ordered: Vec<CommitTooltipData> = (0..num_points)
        .map(|idx| {
            // Get commit info from any test's data at this index
            let reference_point = points_by_test.values().next()
                .and_then(|pts| pts.get(idx))
                .expect("should have data point");
            
            // Gather all test values at this position
            let mut values: Vec<(String, f64, String, Option<String>, String)> = test_names
                .iter()
                .filter_map(|test_name| {
                    points_by_test.get(test_name)
                        .and_then(|pts| pts.get(idx))
                        .map(|p| (
                            p.test_name.clone(),
                            p.value,
                            p.unit.clone(),
                            p.range.clone(),
                            color_map.get(&p.test_name).cloned().unwrap_or_default(),
                        ))
                })
                .collect();
            
            // Sort by value descending
            values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            CommitTooltipData {
                commit_id: reference_point.commit_id.clone(),
                commit_message: reference_point.commit_message.clone(),
                date: reference_point.date.clone(),
                values,
            }
        })
        .collect();

    let latest_values: Vec<(String, f64, Option<String>)> = test_names
        .iter()
        .filter_map(|test| {
            let test_points: Vec<_> = data_points
                .iter()
                .filter(|p| p.test_name == *test)
                .collect();
            test_points
                .last()
                .map(|p| (p.test_name.clone(), p.value, p.range.clone()))
        })
        .collect();

    let num_commits = commits_ordered.len();

    rsx! {
        div { style: "{chart_style(dark)} position: relative;",
            div { style: "{chart_header_style(dark)}",
                span { style: "{chart_title_style(dark)}", "{name}" }
                span { style: "{unit_badge_style(dark)}", "{unit}" }
                button {
                    style: "{expand_btn_style(dark)} margin-left: auto;",
                    onclick: move |_| expanded.set(true),
                    title: "Expand chart",
                    "⛶"
                }
            }

            // Chart + legend side by side
            div { style: "display: flex; align-items: stretch;",
                // Chart area (flexible width)
                div { style: "flex: 1; min-width: 0;",
                    ChartSvg {
                        series: series.clone(),
                        max_value: max_value,
                        commits_ordered: commits_ordered.clone(),
                        hovered_commit: hovered_commit,
                        chart_height: 120.0,
                        chart_width: 400.0,
                        compact: true
                    }
                }

                // Legend (stacked vertically on right)
                div { style: "{legend_right_style(dark)}",
                    for (idx, (test_name, value, _range)) in latest_values.iter().enumerate() {
                        div { style: "{legend_item_vertical_style(dark)}",
                            span { style: "width: 6px; height: 6px; background: {CHART_COLORS[idx % CHART_COLORS.len()]}; flex-shrink: 0;" }
                            span { style: "font-size: 0.7rem;", "{test_name}" }
                            span { style: "font-weight: 600; font-size: 0.7rem;", "{value:.2}" }
                        }
                    }
                }
            }
        }

        // Expanded modal
        if *expanded.read() {
            ExpandedChartModal {
                name: name.clone(),
                unit: unit.clone(),
                series: series.clone(),
                max_value: max_value,
                commits_ordered: commits_ordered.clone(),
                latest_values: latest_values.clone(),
                on_close: move |_| expanded.set(false)
            }
        }
    }
}

/// Reusable SVG chart component
#[component]
fn ChartSvg(
    series: BTreeMap<String, Vec<(String, f64)>>,
    max_value: f64,
    commits_ordered: Vec<CommitTooltipData>,
    mut hovered_commit: Signal<Option<usize>>,
    chart_height: f64,
    chart_width: f64,
    compact: bool,
) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    let padding = if compact { 40.0 } else { 60.0 };
    let num_commits = commits_ordered.len();
    let grid_c = grid_color(dark);
    let axis_c = axis_color(dark);
    let max_height_style = if compact { "max-height: 120px;" } else { "max-height: 400px;" };

    // Calculate chart area boundaries for hover detection
    let chart_left = padding;
    let chart_right = chart_width - padding;
    let chart_area_width = chart_right - chart_left;

    // Padding ratio for hover calculation (padding / chart_width)
    let padding_ratio = padding / chart_width;
    
    // Store the element width when mounted
    let mut chart_div_width = use_signal(|| 0.0f64);

    rsx! {
        div {
            style: "padding: 0.5rem; position: relative; cursor: crosshair;",
            onmounted: move |evt| {
                // Get the element dimensions asynchronously
                let mounted_data = evt.data().clone();
                spawn(async move {
                    if let Ok(rect) = mounted_data.get_client_rect().await {
                        chart_div_width.set(rect.width());
                    }
                });
            },
            onmouseleave: move |_| hovered_commit.set(None),
            onmousemove: move |e| {
                // Get element-relative coordinates - this is in screen pixels
                let coords = e.data().element_coordinates();
                let element_x = coords.x;
                
                // Get stored element width
                let div_width = *chart_div_width.read();
                
                if div_width > 0.0 && num_commits > 0 {
                    let fraction = element_x / div_width;
                    // Account for padding in the chart
                    let chart_start = padding_ratio;
                    let chart_end = 1.0 - padding_ratio;
                    
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
                style: "width: 100%; height: auto; {max_height_style} pointer-events: none;",
                view_box: "0 0 {chart_width} {chart_height}",
                "preserveAspectRatio": "xMidYMid meet",

                for i in 0..5 {
                    line {
                        x1: "{padding}",
                        y1: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                        x2: "{chart_width - padding}",
                        y2: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                        stroke: "{grid_c}",
                        "stroke-width": "1",
                        style: "pointer-events: none;"
                    }
                }

                for i in 0..5 {
                    text {
                        x: "{padding - 4.0}",
                        y: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0) + 3.0}",
                        fill: "{axis_c}",
                        "font-size": if compact { "8" } else { "12" },
                        "text-anchor": "end",
                        style: "pointer-events: none;",
                        "{format_value(max_value * (1.0 - i as f64 / 4.0))}"
                    }
                }

                for (idx, (test_name, points)) in series.iter().enumerate() {
                    if !points.is_empty() {
                        {
                            let color = CHART_COLORS[idx % CHART_COLORS.len()];
                            let path = generate_line_path(points, max_value, chart_width, chart_height, padding);
                            let point_r = if compact { "4" } else { "6" };
                            rsx! {
                                path {
                                    key: "{test_name}-line",
                                    d: "{path}",
                                    fill: "none",
                                    stroke: "{color}",
                                    "stroke-width": if compact { "1.5" } else { "2" },
                                    style: "pointer-events: none;"
                                }
                                for (i, (_, value)) in points.iter().enumerate() {
                                    {
                                        let x = padding + (chart_width - 2.0 * padding) * (i as f64 / (points.len().max(1) - 1).max(1) as f64);
                                        let y = padding + (chart_height - 2.0 * padding) * (1.0 - value / max_value.max(1.0));
                                        rsx! {
                                            circle {
                                                key: "{test_name}-point-{i}",
                                                cx: "{x}",
                                                cy: "{y}",
                                                r: "{point_r}",
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

                if let Some(idx) = *hovered_commit.read() {
                    if idx < num_commits {
                        {
                            let x = padding + (chart_width - 2.0 * padding) * (idx as f64 / (num_commits.max(1) - 1).max(1) as f64);
                            rsx! {
                                line {
                                    x1: "{x}", y1: "{padding}", x2: "{x}", y2: "{chart_height - padding}",
                                    stroke: "{CHART_COLORS[0]}", "stroke-width": "2", "stroke-dasharray": "4,4", opacity: "0.7",
                                    style: "pointer-events: none;"
                                }
                            }
                        }
                    }
                }
            }

            // Tooltip - positioned within the chart container
            if let Some(idx) = *hovered_commit.read() {
                if let Some(commit_data) = commits_ordered.get(idx) {
                    {
                        // Position tooltip based on point position
                        let point_pct = (idx as f64 / (num_commits.max(1) - 1).max(1) as f64) * 100.0;
                        // If point is on left half, show tooltip to the right; otherwise to the left
                        let tooltip_left = if point_pct < 50.0 {
                            format!("calc({}% + 20px)", point_pct.max(5.0))
                        } else {
                            format!("calc({}% - 220px)", point_pct.min(95.0))
                        };
                        let commit_short = &commit_data.commit_id[..7.min(commit_data.commit_id.len())];
                        rsx! {
                            div {
                                style: "{compact_tooltip_style(dark)} top: 8px; left: {tooltip_left}; pointer-events: none;",

                                div { style: "margin-bottom: 0.2rem;",
                                    code { style: "font-size: 0.65rem; {muted_style(dark)}", "{commit_short}" }
                                }
                                for (test_name, value, unit, _range, color) in commit_data.values.iter() {
                                    div { style: "display: flex; align-items: center; gap: 0.2rem; font-size: 0.65rem;",
                                        span { style: "width: 5px; height: 5px; background: {color}; flex-shrink: 0;" }
                                        span { "{test_name}" }
                                        span { style: "font-weight: 600;", "{value:.2}" }
                                        span { style: "{muted_style(dark)}", "{unit}" }
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

/// Expanded chart modal for full-screen viewing
#[component]
fn ExpandedChartModal(
    name: String,
    unit: String,
    series: BTreeMap<String, Vec<(String, f64)>>,
    max_value: f64,
    commits_ordered: Vec<CommitTooltipData>,
    latest_values: Vec<(String, f64, Option<String>)>,
    on_close: EventHandler<()>,
) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();
    let mut hovered_commit: Signal<Option<usize>> = use_signal(|| None);

    rsx! {
        div {
            style: "{modal_overlay_style(dark)}",
            onclick: move |_| on_close.call(()),

            div {
                style: "{modal_content_style(dark)}",
                onclick: move |evt| evt.stop_propagation(),

                div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;",
                    div { style: "display: flex; align-items: center; gap: 0.5rem;",
                        span { style: "{chart_title_style(dark)} font-size: 1.2rem;", "{name}" }
                        span { style: "{unit_badge_style(dark)}", "{unit}" }
                    }
                    button {
                        style: "{close_btn_style(dark)}",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                ChartSvg {
                    series: series.clone(),
                    max_value: max_value,
                    commits_ordered: commits_ordered.clone(),
                    hovered_commit: hovered_commit,
                    chart_height: 400.0,
                    chart_width: 800.0,
                    compact: false
                }

                div { style: "{legend_style(dark)} flex-wrap: wrap; justify-content: center; margin-top: 1rem;",
                    for (idx, (test_name, value, range)) in latest_values.iter().enumerate() {
                        div { style: "{legend_item_style(dark)}",
                            span { style: "width: 10px; height: 10px; background: {CHART_COLORS[idx % CHART_COLORS.len()]};" }
                            span { "{test_name}" }
                            span { style: "font-weight: 600;", "{value:.2}" }
                            if let Some(r) = range {
                                span { style: "{muted_style(dark)}", "{r}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Ungrouped benchmarks that don't have a grandparent module
#[component]
fn UngroupedBenchmarks(
    charts: BTreeMap<String, Vec<BenchmarkDataPoint>>,
    #[allow(unused)] runs: Vec<BenchmarkRun>,
) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

    rsx! {
        div { style: "{module_style(dark)} opacity: 0.7;",
            div { style: "{module_header_style(dark)}",
                span { style: "{module_title_style(dark)}", "ungrouped" }
            }
            div { style: "padding: 0.25rem;",
                for (parent_name, points) in charts.iter() {
                    ModuleChart {
                        key: "{parent_name}",
                        name: parent_name.clone(),
                        data_points: points.clone()
                    }
                }
            }
        }
    }
}

/// Flat benchmark view for non-hierarchical benchmarks
#[component]
fn FlatBenchmarkView(runs: Vec<BenchmarkRun>) -> Element {
    let ThemeCtx(dark_mode) = use_context::<ThemeCtx>();
    let dark = *dark_mode.read();

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

    let bench_names: Vec<String> = series.keys().cloned().collect();
    let max_value = series
        .values()
        .flat_map(|p| p.iter().map(|(_, v)| *v))
        .fold(0.0f64, |a, b| a.max(b));

    let chart_height = 150.0;
    let chart_width = 500.0;
    let padding = 45.0;
    let grid_c = grid_color(dark);
    let axis_c = axis_color(dark);

    rsx! {
        div { style: "{chart_style(dark)}",
            div { style: "padding: 0.5rem;",
                svg {
                    style: "width: 100%; height: auto; max-height: 150px;",
                    view_box: "0 0 {chart_width} {chart_height}",
                    "preserveAspectRatio": "xMidYMid meet",

                    for i in 0..5 {
                        line {
                            x1: "{padding}",
                            y1: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                            x2: "{chart_width - padding}",
                            y2: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0)}",
                            stroke: "{grid_c}", "stroke-width": "1"
                        }
                    }

                    for i in 0..5 {
                        text {
                            x: "{padding - 4.0}",
                            y: "{padding + (chart_height - 2.0 * padding) * (i as f64 / 4.0) + 3.0}",
                            fill: "{axis_c}", "font-size": "8", "text-anchor": "end",
                            "{format_value(max_value * (1.0 - i as f64 / 4.0))}"
                        }
                    }

                    for (idx, (bench_name, points)) in series.iter().enumerate() {
                        if !points.is_empty() {
                            {
                                let color = CHART_COLORS[idx % CHART_COLORS.len()];
                                let path = generate_line_path(points, max_value, chart_width, chart_height, padding);
                                rsx! {
                                    path { key: "{bench_name}-line", d: "{path}", fill: "none", stroke: "{color}", "stroke-width": "1.5" }
                                    for (i, (_, value)) in points.iter().enumerate() {
                                        {
                                            let x = padding + (chart_width - 2.0 * padding) * (i as f64 / (points.len().max(1) - 1).max(1) as f64);
                                            let y = padding + (chart_height - 2.0 * padding) * (1.0 - value / max_value.max(1.0));
                                            rsx! { circle { key: "{bench_name}-point-{i}", cx: "{x}", cy: "{y}", r: "3", fill: "{color}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "{legend_style(dark)}",
                    for (idx, name) in bench_names.iter().enumerate() {
                        div { style: "{legend_item_style(dark)}",
                            span { style: "width: 8px; height: 8px; background: {CHART_COLORS[idx % CHART_COLORS.len()]};" }
                            span { "{name}" }
                        }
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
    // For very large values (likely ops/sec or similar), use K/M/G suffixes
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
