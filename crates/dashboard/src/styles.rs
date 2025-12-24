//! Minimalistic styles - no external CSS files
//!
//! Clean, practical design with light/dark theme support.

// ============================================================================
// Theme-aware style generators
// ============================================================================

/// Generate app style based on theme
pub fn app_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#0d1117", "#c9d1d9")
    } else {
        ("#ffffff", "#1a1a1a")
    };
    format!(
        "min-height: 100vh; \
         display: flex; \
         flex-direction: column; \
         font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif; \
         font-size: 14px; \
         background: {bg}; \
         color: {fg}; \
         line-height: 1.5; \
         margin: 0; \
         padding: 0;"
    )
}

/// Generate header style
pub fn header_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#161b22", "#30363d")
    } else {
        ("#f6f8fa", "#d0d7de")
    };
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.75rem 1rem; \
         background: {bg}; \
         border-bottom: 1px solid {border};"
    )
}

/// Generate title style
pub fn title_style(_dark: bool) -> &'static str {
    "font-size: 1.1rem; \
     font-weight: 600; \
     margin: 0;"
}

/// Generate theme toggle button style
pub fn toggle_btn_style(dark: bool) -> String {
    let (bg, fg, border) = if dark {
        ("#21262d", "#c9d1d9", "#30363d")
    } else {
        ("#f6f8fa", "#1a1a1a", "#d0d7de")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         border: 1px solid {border}; \
         padding: 0.35rem 0.75rem; \
         font-family: inherit; \
         font-size: 0.8rem; \
         border-radius: 6px; \
         cursor: pointer;"
    )
}

/// Sidebar style
pub fn sidebar_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#161b22", "#30363d")
    } else {
        ("#f6f8fa", "#d0d7de")
    };
    format!(
        "width: 280px; \
         min-width: 280px; \
         background: {bg}; \
         border-right: 1px solid {border}; \
         display: flex; \
         flex-direction: column; \
         overflow: hidden;"
    )
}

/// Search input style
pub fn search_input_style(dark: bool) -> String {
    let (bg, fg, border, placeholder) = if dark {
        ("#0d1117", "#c9d1d9", "#30363d", "#6e7681")
    } else {
        ("#ffffff", "#1a1a1a", "#d0d7de", "#6e7681")
    };
    format!(
        "width: 100%; \
         box-sizing: border-box; \
         padding: 0.5rem 0.75rem; \
         background: {bg}; \
         color: {fg}; \
         border: 1px solid {border}; \
         border-radius: 6px; \
         font-family: inherit; \
         font-size: 0.85rem; \
         outline: none;"
    )
}

/// Sidebar section header
pub fn sidebar_section_header(dark: bool) -> String {
    let border = if dark { "#30363d" } else { "#d0d7de" };
    format!(
        "padding: 0.5rem 0.75rem; \
         font-weight: 600; \
         font-size: 0.8rem; \
         border-bottom: 1px solid {border};"
    )
}

/// Commit item style
pub fn commit_item_style(dark: bool, selected: bool) -> String {
    let (bg, border) = if dark {
        if selected { ("#1f2937", "#3b82f6") } else { ("transparent", "transparent") }
    } else {
        if selected { ("#eff6ff", "#3b82f6") } else { ("transparent", "transparent") }
    };
    format!(
        "display: flex; \
         align-items: center; \
         padding: 0.5rem 0.75rem; \
         border-left: 3px solid {border}; \
         background: {bg}; \
         cursor: default;"
    )
}

/// Commit indicator (left blue dot)
pub fn commit_indicator_style(_dark: bool) -> &'static str {
    "width: 6px; \
     height: 6px; \
     border-radius: 50%; \
     background: #3b82f6; \
     flex-shrink: 0;"
}

/// Commit hash link style
pub fn commit_hash_link_style(dark: bool) -> String {
    let fg = if dark { "#58a6ff" } else { "#0969da" };
    format!(
        "font-family: 'SF Mono', 'Fira Code', monospace; \
         font-size: 0.85rem; \
         color: {fg}; \
         text-decoration: none; \
         font-weight: 500;"
    )
}

/// Badge style for TO
pub fn badge_compare_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#30363d", "#8b949e")
    } else {
        ("#e1e4e8", "#57606a")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0.1rem 0.4rem; \
         border-radius: 4px; \
         font-size: 0.6rem; \
         font-weight: 500; \
         text-transform: uppercase;"
    )
}

/// Badge style for FROM
pub fn badge_baseline_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#30363d", "#8b949e")
    } else {
        ("#e1e4e8", "#57606a")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0.1rem 0.4rem; \
         border-radius: 4px; \
         font-size: 0.6rem; \
         font-weight: 500; \
         text-transform: uppercase;"
    )
}

/// Icon button style
pub fn icon_btn_style(dark: bool, active: bool) -> String {
    let (bg, fg) = if dark {
        if active { ("#3b82f6", "#ffffff") } else { ("transparent", "#6e7681") }
    } else {
        if active { ("#3b82f6", "#ffffff") } else { ("transparent", "#6e7681") }
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         border: none; \
         width: 24px; \
         height: 24px; \
         border-radius: 4px; \
         cursor: pointer; \
         font-size: 0.9rem; \
         display: flex; \
         align-items: center; \
         justify-content: center;"
    )
}

/// Main content area style
pub fn main_content_style(dark: bool) -> String {
    let bg = if dark { "#0d1117" } else { "#ffffff" };
    format!(
        "flex: 1; \
         padding: 1.5rem 2rem; \
         overflow-y: auto; \
         background: {bg};"
    )
}

/// Suite title style
pub fn suite_title_style(dark: bool) -> String {
    let fg = if dark { "#c9d1d9" } else { "#1a1a1a" };
    format!(
        "font-size: 1.5rem; \
         font-weight: 600; \
         color: {fg}; \
         margin: 0 0 1rem 0;"
    )
}

/// Container card style (for grandparent grouping)
pub fn container_card_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#161b22", "#30363d")
    } else {
        ("#ffffff", "#d0d7de")
    };
    format!(
        "background: {bg}; \
         border: 1px solid {border}; \
         border-radius: 8px; \
         margin-bottom: 1.5rem; \
         overflow: hidden;"
    )
}

/// Container header style
pub fn container_header_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#21262d", "#30363d")
    } else {
        ("#f6f8fa", "#d0d7de")
    };
    format!(
        "padding: 0.75rem 1rem; \
         background: {bg}; \
         border-bottom: 1px solid {border};"
    )
}

/// Container title style
pub fn container_title_style(dark: bool) -> String {
    let fg = if dark { "#c9d1d9" } else { "#1a1a1a" };
    format!(
        "font-size: 1rem; \
         font-weight: 600; \
         color: {fg};"
    )
}

/// Chart card style
pub fn chart_card_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#161b22", "#30363d")
    } else {
        ("#ffffff", "#d0d7de")
    };
    format!(
        "background: {bg}; \
         border: 1px solid {border}; \
         border-radius: 8px; \
         margin-bottom: 1rem; \
         overflow: hidden;"
    )
}

/// Generate chart header style
pub fn chart_header_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#21262d", "#30363d")
    } else {
        ("#f6f8fa", "#d0d7de")
    };
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.75rem 1rem; \
         background: {bg}; \
         border-bottom: 1px solid {border};"
    )
}

/// Generate chart title style
pub fn chart_title_style(dark: bool) -> String {
    let fg = if dark { "#c9d1d9" } else { "#1a1a1a" };
    format!(
        "font-size: 1rem; \
         font-weight: 600; \
         color: {fg};"
    )
}

/// Generate unit badge style
pub fn unit_badge_style(dark: bool) -> String {
    let fg = if dark { "#6e7681" } else { "#6e7681" };
    format!(
        "color: {fg}; \
         font-size: 0.8rem; \
         font-weight: 400;"
    )
}

/// Chart legend style
pub fn chart_legend_style(dark: bool) -> String {
    let border = if dark { "#30363d" } else { "#d0d7de" };
    format!(
        "display: flex; \
         flex-wrap: wrap; \
         gap: 1.5rem; \
         padding: 0.75rem 1rem; \
         border-top: 1px solid {border}; \
         font-size: 0.85rem;"
    )
}

/// Metrics section style
pub fn metrics_section_style(dark: bool) -> String {
    let border = if dark { "#30363d" } else { "#d0d7de" };
    format!(
        "padding: 0.75rem 1rem; \
         border-top: 1px solid {border};"
    )
}

/// Metrics toggle button style
pub fn metrics_toggle_style(dark: bool) -> String {
    let fg = if dark { "#c9d1d9" } else { "#1a1a1a" };
    format!(
        "background: none; \
         border: none; \
         color: {fg}; \
         font-size: 0.85rem; \
         font-weight: 500; \
         cursor: pointer; \
         padding: 0; \
         display: flex; \
         align-items: center; \
         gap: 0.4rem;"
    )
}

/// Metrics table header style
pub fn metrics_table_header_style(dark: bool) -> String {
    let fg = if dark { "#6e7681" } else { "#6e7681" };
    format!(
        "display: flex; \
         padding: 0.5rem 0; \
         font-size: 0.75rem; \
         color: {fg}; \
         font-weight: 500; \
         text-transform: uppercase; \
         letter-spacing: 0.5px;"
    )
}

/// Metrics table row style
pub fn metrics_table_row_style(dark: bool) -> String {
    let border = if dark { "#21262d" } else { "#f6f8fa" };
    format!(
        "display: flex; \
         padding: 0.6rem 0; \
         font-size: 0.85rem; \
         border-top: 1px solid {border}; \
         align-items: center;"
    )
}

/// Hover tooltip style (compact)
pub fn hover_tooltip_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("rgba(22, 27, 34, 0.95)", "#30363d")
    } else {
        ("rgba(255, 255, 255, 0.95)", "#d0d7de")
    };
    format!(
        "position: absolute; \
         background: {bg}; \
         border: 1px solid {border}; \
         border-radius: 8px; \
         padding: 0.6rem 0.8rem; \
         font-size: 0.8rem; \
         z-index: 100; \
         box-shadow: 0 4px 12px rgba(0,0,0,0.3); \
         pointer-events: none;"
    )
}

/// Generate muted text style
pub fn muted_style(dark: bool) -> String {
    let fg = if dark { "#6e7681" } else { "#6e7681" };
    format!("color: {fg};")
}

/// Generate loading style
pub fn loading_style(_dark: bool) -> &'static str {
    "padding: 2rem; \
     text-align: center;"
}

/// Generate error style
pub fn error_style(dark: bool) -> String {
    let border = if dark { "#f85149" } else { "#cf222e" };
    format!(
        "padding: 1rem; \
         border: 1px solid {border}; \
         border-radius: 8px; \
         margin: 1rem 0;"
    )
}

/// Generate empty state style
pub fn empty_style(_dark: bool) -> &'static str {
    "padding: 2rem; \
     text-align: center; \
     opacity: 0.7;"
}

/// Generate code/mono style
pub fn code_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#21262d", "#79c0ff")
    } else {
        ("#f6f8fa", "#0550ae")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0.2rem 0.5rem; \
         border-radius: 4px; \
         font-size: 0.85rem; \
         font-family: 'SF Mono', 'Fira Code', monospace;"
    )
}

/// SVG grid line color
pub fn grid_color(dark: bool) -> &'static str {
    if dark { "#21262d" } else { "#eaeef2" }
}

/// SVG axis label color
pub fn axis_color(dark: bool) -> &'static str {
    if dark { "#6e7681" } else { "#6e7681" }
}

/// Chart line colors
pub const CHART_COLORS: [&str; 10] = [
    "#3fb950", // green
    "#58a6ff", // blue
    "#f78166", // orange/red
    "#a371f7", // purple
    "#f9c513", // yellow
    "#39d353", // bright green
    "#79c0ff", // light blue
    "#ff7b72", // red
    "#d2a8ff", // light purple
    "#ffa657", // orange
];
