//! HTML dashboard generator with Chart.js

use crate::data::BenchmarkData;
use crate::error::Result;
use minijinja::{context, Environment};
use std::path::Path;

/// HTML template for the benchmark dashboard
const DASHBOARD_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns@3.0.0/dist/chartjs-adapter-date-fns.bundle.min.js"></script>
    <style>
        :root {
            --bg-primary: #0d1117;
            --bg-secondary: #161b22;
            --bg-tertiary: #21262d;
            --text-primary: #c9d1d9;
            --text-secondary: #8b949e;
            --text-muted: #6e7681;
            --border-color: #30363d;
            --accent-blue: #58a6ff;
            --accent-green: #3fb950;
            --accent-red: #f85149;
            --accent-purple: #a371f7;
            --accent-orange: #d29922;
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            min-height: 100vh;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 2rem;
        }

        header {
            text-align: center;
            margin-bottom: 3rem;
            padding: 2rem;
            background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
            border-radius: 16px;
            border: 1px solid var(--border-color);
        }

        h1 {
            font-size: 2.5rem;
            font-weight: 600;
            background: linear-gradient(135deg, var(--accent-blue) 0%, var(--accent-purple) 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            margin-bottom: 0.5rem;
        }

        .subtitle {
            color: var(--text-secondary);
            font-size: 1.1rem;
        }

        .last-updated {
            color: var(--text-muted);
            font-size: 0.9rem;
            margin-top: 1rem;
        }

        .benchmark-suite {
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: 12px;
            margin-bottom: 2rem;
            overflow: hidden;
        }

        .suite-header {
            padding: 1.25rem 1.5rem;
            background: var(--bg-tertiary);
            border-bottom: 1px solid var(--border-color);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .suite-header h2 {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text-primary);
        }

        .suite-stats {
            display: flex;
            gap: 1rem;
        }

        .stat {
            padding: 0.25rem 0.75rem;
            background: var(--bg-primary);
            border-radius: 20px;
            font-size: 0.85rem;
            color: var(--text-secondary);
        }

        .stat-value {
            font-weight: 600;
            color: var(--accent-blue);
        }

        .chart-container {
            padding: 1.5rem;
            height: 400px;
            position: relative;
        }

        .benchmark-table {
            width: 100%;
            border-collapse: collapse;
        }

        .benchmark-table th,
        .benchmark-table td {
            padding: 1rem 1.5rem;
            text-align: left;
            border-top: 1px solid var(--border-color);
        }

        .benchmark-table th {
            background: var(--bg-tertiary);
            color: var(--text-secondary);
            font-weight: 500;
            font-size: 0.85rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        .benchmark-table tr:hover {
            background: var(--bg-tertiary);
        }

        .benchmark-name {
            font-family: 'SF Mono', 'Fira Code', monospace;
            color: var(--accent-blue);
        }

        .trend-up {
            color: var(--accent-red);
        }

        .trend-down {
            color: var(--accent-green);
        }

        .trend-neutral {
            color: var(--text-muted);
        }

        .commit-link {
            color: var(--accent-purple);
            text-decoration: none;
        }

        .commit-link:hover {
            text-decoration: underline;
        }

        .no-data {
            text-align: center;
            padding: 3rem;
            color: var(--text-muted);
        }

        footer {
            text-align: center;
            padding: 2rem;
            color: var(--text-muted);
            font-size: 0.9rem;
        }

        footer a {
            color: var(--accent-blue);
            text-decoration: none;
        }

        footer a:hover {
            text-decoration: underline;
        }

        @media (max-width: 768px) {
            .container {
                padding: 1rem;
            }

            h1 {
                font-size: 1.75rem;
            }

            .suite-header {
                flex-direction: column;
                gap: 1rem;
            }

            .chart-container {
                height: 300px;
            }

            .benchmark-table th,
            .benchmark-table td {
                padding: 0.75rem;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>📊 {{ title }}</h1>
            <p class="subtitle">Continuous Benchmark Tracking</p>
            {% if last_update %}
            <p class="last-updated">Last updated: {{ last_update }}</p>
            {% endif %}
        </header>

        {% if suites %}
            {% for suite in suites %}
            <div class="benchmark-suite">
                <div class="suite-header">
                    <h2>{{ suite.name }}</h2>
                    <div class="suite-stats">
                        <span class="stat"><span class="stat-value">{{ suite.bench_count }}</span> benchmarks</span>
                        <span class="stat"><span class="stat-value">{{ suite.run_count }}</span> runs</span>
                    </div>
                </div>

                <div class="chart-container">
                    <canvas id="chart-{{ suite.id }}"></canvas>
                </div>

                {% if suite.latest_results %}
                <table class="benchmark-table">
                    <thead>
                        <tr>
                            <th>Benchmark</th>
                            <th>Value</th>
                            <th>Range</th>
                            <th>Commit</th>
                        </tr>
                    </thead>
                    <tbody>
                        {% for result in suite.latest_results %}
                        <tr>
                            <td class="benchmark-name">{{ result.name }}</td>
                            <td>{{ result.value }} {{ result.unit }}</td>
                            <td>{{ result.range | default('-') }}</td>
                            <td>
                                {% if result.commit_url %}
                                <a href="{{ result.commit_url }}" class="commit-link">{{ result.commit_short }}</a>
                                {% else %}
                                {{ result.commit_short }}
                                {% endif %}
                            </td>
                        </tr>
                        {% endfor %}
                    </tbody>
                </table>
                {% endif %}
            </div>
            {% endfor %}
        {% else %}
            <div class="no-data">
                <p>No benchmark data available yet.</p>
                <p>Run your benchmarks to start tracking performance.</p>
            </div>
        {% endif %}

        <footer>
            <p>Generated by <a href="https://github.com/benchmark-action/github-action-benchmark">git-bench</a></p>
        </footer>
    </div>

    <script>
        window.BENCHMARK_DATA = {{ benchmark_data_json | safe }};

        // Color palette for charts
        const colors = [
            '#58a6ff', '#3fb950', '#f85149', '#a371f7', '#d29922',
            '#79c0ff', '#56d364', '#ff7b72', '#bc8cff', '#e3b341'
        ];

        // Initialize charts
        document.addEventListener('DOMContentLoaded', function() {
            const data = window.BENCHMARK_DATA;

            Object.keys(data.entries).forEach((suiteName, suiteIndex) => {
                const runs = data.entries[suiteName];
                if (!runs || runs.length === 0) return;

                const canvasId = 'chart-' + suiteName.replace(/[^a-zA-Z0-9]/g, '-');
                const canvas = document.getElementById(canvasId);
                if (!canvas) return;

                // Group data by benchmark name
                const benchmarks = {};
                runs.forEach(run => {
                    run.benches.forEach(bench => {
                        if (!benchmarks[bench.name]) {
                            benchmarks[bench.name] = [];
                        }
                        benchmarks[bench.name].push({
                            x: new Date(run.date),
                            y: bench.value,
                            commit: run.commit.id.substring(0, 7),
                            message: run.commit.message
                        });
                    });
                });

                // Create datasets
                const datasets = Object.keys(benchmarks).map((name, index) => ({
                    label: name,
                    data: benchmarks[name],
                    borderColor: colors[index % colors.length],
                    backgroundColor: colors[index % colors.length] + '20',
                    fill: false,
                    tension: 0.3,
                    pointRadius: 4,
                    pointHoverRadius: 6
                }));

                new Chart(canvas, {
                    type: 'line',
                    data: { datasets },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        interaction: {
                            mode: 'index',
                            intersect: false
                        },
                        plugins: {
                            legend: {
                                position: 'top',
                                labels: {
                                    color: '#c9d1d9',
                                    usePointStyle: true,
                                    padding: 20
                                }
                            },
                            tooltip: {
                                backgroundColor: '#21262d',
                                titleColor: '#c9d1d9',
                                bodyColor: '#8b949e',
                                borderColor: '#30363d',
                                borderWidth: 1,
                                callbacks: {
                                    afterTitle: function(context) {
                                        const point = context[0].raw;
                                        return 'Commit: ' + point.commit;
                                    },
                                    afterBody: function(context) {
                                        const point = context[0].raw;
                                        return point.message ? '\\n' + point.message : '';
                                    }
                                }
                            }
                        },
                        scales: {
                            x: {
                                type: 'time',
                                time: {
                                    unit: 'day',
                                    displayFormats: {
                                        day: 'MMM d'
                                    }
                                },
                                grid: {
                                    color: '#30363d'
                                },
                                ticks: {
                                    color: '#8b949e'
                                }
                            },
                            y: {
                                beginAtZero: false,
                                grid: {
                                    color: '#30363d'
                                },
                                ticks: {
                                    color: '#8b949e'
                                },
                                title: {
                                    display: true,
                                    text: 'Time (ns)',
                                    color: '#8b949e'
                                }
                            }
                        }
                    }
                });
            });
        });
    </script>
</body>
</html>
"#;

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// Title for the dashboard
    pub title: String,
    /// Path to output directory
    pub output_dir: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            title: "Benchmark Results".to_string(),
            output_dir: "dev/bench".to_string(),
        }
    }
}

/// Suite data for template rendering
#[derive(Debug, Clone, serde::Serialize)]
struct SuiteData {
    name: String,
    id: String,
    bench_count: usize,
    run_count: usize,
    latest_results: Vec<LatestResult>,
}

/// Latest result data for template
#[derive(Debug, Clone, serde::Serialize)]
struct LatestResult {
    name: String,
    value: String,
    unit: String,
    range: Option<String>,
    commit_short: String,
    commit_url: Option<String>,
}

/// Generate the HTML dashboard
pub fn generate_dashboard(data: &BenchmarkData, config: &DashboardConfig) -> Result<String> {
    let mut env = Environment::new();
    env.add_template("dashboard", DASHBOARD_TEMPLATE)?;

    let template = env.get_template("dashboard")?;

    // Prepare suite data
    let suites: Vec<SuiteData> = data
        .entries
        .iter()
        .map(|(name, runs)| {
            let latest_results = if let Some(latest_run) = runs.last() {
                latest_run
                    .benches
                    .iter()
                    .map(|bench| LatestResult {
                        name: bench.name.clone(),
                        value: format!("{:.2}", bench.value),
                        unit: bench.unit.clone(),
                        range: bench.range.clone(),
                        commit_short: latest_run.commit.id.chars().take(7).collect(),
                        commit_url: latest_run.commit.url.clone(),
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // Count unique benchmarks across all runs
            let bench_count = runs
                .iter()
                .flat_map(|r| r.benches.iter().map(|b| &b.name))
                .collect::<std::collections::HashSet<_>>()
                .len();

            SuiteData {
                id: name.replace(|c: char| !c.is_alphanumeric(), "-"),
                name: name.clone(),
                bench_count,
                run_count: runs.len(),
                latest_results,
            }
        })
        .collect();

    let last_update = data.last_update.map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string());

    let benchmark_data_json = serde_json::to_string(data)?;

    let html = template.render(context! {
        title => &config.title,
        last_update => last_update,
        suites => suites,
        benchmark_data_json => benchmark_data_json,
    })?;

    Ok(html)
}

/// Write the dashboard to a file
pub fn write_dashboard(data: &BenchmarkData, config: &DashboardConfig, base_path: &Path) -> Result<()> {
    let output_dir = base_path.join(&config.output_dir);
    std::fs::create_dir_all(&output_dir)?;

    let html = generate_dashboard(data, config)?;
    let index_path = output_dir.join("index.html");

    std::fs::write(&index_path, html)?;

    // Also write the raw data as JSON
    let data_path = output_dir.join("data.json");
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(&data_path, json)?;

    Ok(())
}

/// Check if index.html exists in the output directory
pub fn dashboard_exists(base_path: &Path, output_dir: &str) -> bool {
    base_path.join(output_dir).join("index.html").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{BenchmarkResult, BenchmarkRun, CommitInfo};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_generate_dashboard_empty() {
        let data = BenchmarkData::new();
        let config = DashboardConfig::default();

        let html = generate_dashboard(&data, &config).unwrap();
        assert!(html.contains("No benchmark data available"));
    }

    #[test]
    fn test_generate_dashboard_with_data() {
        let mut data = BenchmarkData::new();

        let run = BenchmarkRun {
            commit: CommitInfo {
                id: "abc123def456".to_string(),
                message: "Test commit".to_string(),
                timestamp: Utc::now(),
                url: Some("https://github.com/test/repo/commit/abc123".to_string()),
                author: None,
            },
            date: Utc::now(),
            tool: "cargo".to_string(),
            benches: vec![BenchmarkResult {
                name: "test_bench".to_string(),
                value: 123.45,
                unit: "ns/iter".to_string(),
                range: Some("+/- 5".to_string()),
                extra: HashMap::new(),
            }],
        };

        data.add_run("test-suite", run, None);

        let config = DashboardConfig {
            title: "Test Dashboard".to_string(),
            ..Default::default()
        };

        let html = generate_dashboard(&data, &config).unwrap();
        assert!(html.contains("Test Dashboard"));
        assert!(html.contains("test-suite"));
        assert!(html.contains("test_bench"));
        assert!(html.contains("123.45"));
    }
}

