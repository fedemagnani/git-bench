//! Binary to generate the index.html from Rust
//!
//! Run with: cargo run --bin generate_html
//!
//! Generates minimal HTML shell for the Dioxus WASM app.

use std::fs;
use std::path::Path;

fn main() {
    let html = generate_index_html();

    let dist_path = Path::new("dist");
    fs::create_dir_all(dist_path).ok();

    let index_path = dist_path.join("index.html");
    fs::write(&index_path, &html).expect("Failed to write index.html");

    println!("✅ Generated: {}", index_path.display());
}

fn generate_index_html() -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>git-bench</title>
    <style>
{styles}
    </style>
</head>
<body>
    <div id="loading">loading...</div>
    <div id="main"></div>
    <script type="module">
        import init from './git_bench_dashboard.js';
        init().then(() => {{
            document.getElementById('loading').remove();
        }});
    </script>
</body>
</html>"#,
        styles = get_styles(),
    )
}

fn get_styles() -> &'static str {
    r#"        * { margin: 0; padding: 0; box-sizing: border-box; }
        html, body { min-height: 100%; }
        body { font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 14px; }
        #loading { padding: 2rem; text-align: center; font-family: inherit; }
        #main { min-height: 100vh; }"#
}
