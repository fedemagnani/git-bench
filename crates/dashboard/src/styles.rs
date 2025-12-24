//! Brutalist minimalistic styles - no-style-please aesthetic
//!
//! Pure black/white with accent colors adapted for each theme.

// ============================================================================
// Theme-aware color palette
// ============================================================================

/// Accent color for primary actions (links, selections)
pub fn accent_color(dark: bool) -> &'static str {
    if dark { "#00ffff" } else { "#0066cc" }
}

/// Color for positive changes / improvements
pub fn positive_color(dark: bool) -> &'static str {
    if dark { "#00ff00" } else { "#008800" }
}

/// Color for negative changes / regressions
pub fn negative_color(dark: bool) -> &'static str {
    if dark { "#ff0099" } else { "#cc0066" }
}

/// Neutral/muted text color
pub fn muted_color(dark: bool) -> &'static str {
    if dark { "#888888" } else { "#666666" }
}

/// Border color
pub fn border_color(dark: bool) -> &'static str {
    if dark { "#333333" } else { "#cccccc" }
}

/// Subtle background for selected items
pub fn selected_bg(dark: bool) -> &'static str {
    if dark { "#111111" } else { "#f0f0f0" }
}

/// Chart colors - adapted for each theme
pub fn chart_colors(dark: bool) -> [&'static str; 10] {
    if dark {
        // Neon colors for dark mode
        [
            "#00ff00", // lime
            "#00ffff", // cyan
            "#ff00ff", // magenta
            "#ffff00", // yellow
            "#ff6600", // orange
            "#ff0099", // pink
            "#00ff99", // mint
            "#9900ff", // purple
            "#ff3300", // red
            "#00ccff", // blue
        ]
    } else {
        // Darker, more saturated colors for light mode
        [
            "#008800", // dark green
            "#0066cc", // dark blue
            "#990099", // dark magenta
            "#cc9900", // dark yellow/gold
            "#cc4400", // dark orange
            "#cc0066", // dark pink
            "#009966", // dark mint
            "#6600cc", // dark purple
            "#cc2200", // dark red
            "#0088cc", // medium blue
        ]
    }
}

// ============================================================================
// Theme-aware style generators
// ============================================================================

/// Generate app style based on theme
pub fn app_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#000000", "#ffffff")
    } else {
        ("#ffffff", "#000000")
    };
    format!(
        "min-height: 100vh; \
         display: flex; \
         flex-direction: column; \
         font-family: 'Courier New', Courier, monospace; \
         font-size: 13px; \
         background: {bg}; \
         color: {fg}; \
         line-height: 1.6; \
         margin: 0; \
         padding: 0;"
    )
}

/// Generate header style
pub fn header_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.5rem 1rem; \
         border-bottom: 1px solid {border};"
    )
}

/// Generate title style
pub fn title_style(_dark: bool) -> &'static str {
    "font-size: 1rem; \
     font-weight: normal; \
     margin: 0; \
     letter-spacing: 0.05em;"
}

/// Generate theme toggle button style
pub fn toggle_btn_style(dark: bool) -> String {
    let (bg, fg, border) = if dark {
        ("#000", "#fff", "#fff")
    } else {
        ("#fff", "#000", "#000")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         border: 1px solid {border}; \
         padding: 0.25rem 0.5rem; \
         font-family: inherit; \
         font-size: 0.75rem; \
         cursor: pointer;"
    )
}

/// Sidebar style
pub fn sidebar_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "width: 240px; \
         min-width: 240px; \
         border-right: 1px solid {border}; \
         display: flex; \
         flex-direction: column; \
         overflow: hidden;"
    )
}

/// Search input style
pub fn search_input_style(dark: bool) -> String {
    let (bg, fg) = if dark { ("#000", "#fff") } else { ("#fff", "#000") };
    let border = border_color(dark);
    format!(
        "width: 100%; \
         box-sizing: border-box; \
         padding: 0.4rem; \
         background: {bg}; \
         color: {fg}; \
         border: 1px solid {border}; \
         font-family: inherit; \
         font-size: 0.8rem; \
         outline: none;"
    )
}

/// Sidebar section header
pub fn sidebar_section_header(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "padding: 0.4rem 0.6rem; \
         font-size: 0.7rem; \
         text-transform: uppercase; \
         letter-spacing: 0.1em; \
         border-bottom: 1px solid {border};"
    )
}

/// Commit item style
pub fn commit_item_style(dark: bool, selected: bool) -> String {
    let bg = if selected { selected_bg(dark) } else { "transparent" };
    let border = if selected { accent_color(dark) } else { "transparent" };
    format!(
        "display: flex; \
         align-items: center; \
         padding: 0.4rem 0.6rem; \
         border-left: 2px solid {border}; \
         background: {bg}; \
         cursor: default;"
    )
}

/// Commit indicator (left dot)
pub fn commit_indicator_style(dark: bool) -> String {
    let color = accent_color(dark);
    format!(
        "width: 4px; \
         height: 4px; \
         background: {color}; \
         flex-shrink: 0;"
    )
}

/// Commit hash link style
pub fn commit_hash_link_style(dark: bool) -> String {
    let color = accent_color(dark);
    format!(
        "font-family: inherit; \
         font-size: 0.8rem; \
         color: {color}; \
         text-decoration: none;"
    )
}

/// Badge style for TO
pub fn badge_compare_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#222", "#888")
    } else {
        ("#eee", "#666")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0 0.3rem; \
         font-size: 0.6rem; \
         text-transform: uppercase;"
    )
}

/// Badge style for FROM
pub fn badge_baseline_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#222", "#888")
    } else {
        ("#eee", "#666")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0 0.3rem; \
         font-size: 0.6rem; \
         text-transform: uppercase;"
    )
}

/// Icon button style
pub fn icon_btn_style(dark: bool, active: bool) -> String {
    let (bg, fg) = if active {
        (accent_color(dark), if dark { "#000" } else { "#fff" })
    } else {
        ("transparent", muted_color(dark))
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         border: none; \
         width: 20px; \
         height: 20px; \
         cursor: pointer; \
         font-size: 0.8rem; \
         display: flex; \
         align-items: center; \
         justify-content: center;"
    )
}

/// Main content area style
pub fn main_content_style(_dark: bool) -> String {
    "flex: 1; \
     padding: 1rem 1.5rem; \
     overflow-y: auto;".to_string()
}

/// Suite title style
pub fn suite_title_style(_dark: bool) -> String {
    "font-size: 1.2rem; \
     font-weight: normal; \
     margin: 0 0 1rem 0; \
     text-transform: uppercase; \
     letter-spacing: 0.1em;".to_string()
}

/// Container card style (for grandparent grouping)
pub fn container_card_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "border: 1px solid {border}; \
         margin-bottom: 1rem;"
    )
}

/// Container header style
pub fn container_header_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "padding: 0.5rem 0.75rem; \
         border-bottom: 1px solid {border};"
    )
}

/// Container title style
pub fn container_title_style(_dark: bool) -> String {
    "font-size: 0.85rem; \
     text-transform: uppercase; \
     letter-spacing: 0.05em;".to_string()
}

/// Chart card style
pub fn chart_card_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "border: 1px solid {border}; \
         margin-bottom: 0.75rem;"
    )
}

/// Generate chart header style
pub fn chart_header_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.5rem 0.75rem; \
         border-bottom: 1px solid {border};"
    )
}

/// Generate chart title style
pub fn chart_title_style(_dark: bool) -> String {
    "font-size: 0.9rem; \
     font-weight: normal;".to_string()
}

/// Generate unit badge style
pub fn unit_badge_style(dark: bool) -> String {
    let color = muted_color(dark);
    format!("color: {color}; font-size: 0.75rem;")
}

/// Chart legend style
pub fn chart_legend_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "display: flex; \
         flex-wrap: wrap; \
         gap: 1rem; \
         padding: 0.5rem 0.75rem; \
         border-top: 1px solid {border}; \
         font-size: 0.75rem;"
    )
}

/// Metrics section style
pub fn metrics_section_style(dark: bool) -> String {
    let border = border_color(dark);
    format!(
        "padding: 0.5rem 0.75rem; \
         border-top: 1px solid {border};"
    )
}

/// Metrics toggle button style
pub fn metrics_toggle_style(_dark: bool) -> String {
    "background: none; \
     border: none; \
     color: inherit; \
     font-family: inherit; \
     font-size: 0.75rem; \
     cursor: pointer; \
     padding: 0; \
     display: flex; \
     align-items: center; \
     gap: 0.3rem;".to_string()
}

/// Metrics table header style
pub fn metrics_table_header_style(dark: bool) -> String {
    let color = muted_color(dark);
    format!(
        "display: flex; \
         padding: 0.3rem 0; \
         font-size: 0.65rem; \
         color: {color}; \
         text-transform: uppercase; \
         letter-spacing: 0.05em;"
    )
}

/// Metrics table row style
pub fn metrics_table_row_style(dark: bool) -> String {
    let border = if dark { "#222" } else { "#eee" };
    format!(
        "display: flex; \
         padding: 0.4rem 0; \
         font-size: 0.8rem; \
         border-top: 1px solid {border}; \
         align-items: center;"
    )
}

/// Hover tooltip style (compact)
pub fn hover_tooltip_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#000", "#444")
    } else {
        ("#fff", "#ccc")
    };
    format!(
        "position: absolute; \
         background: {bg}; \
         border: 1px solid {border}; \
         padding: 0.4rem 0.6rem; \
         font-size: 0.75rem; \
         z-index: 100; \
         pointer-events: none;"
    )
}

/// Generate muted text style
pub fn muted_style(dark: bool) -> String {
    let color = muted_color(dark);
    format!("color: {color};")
}

/// Generate loading style
pub fn loading_style(_dark: bool) -> &'static str {
    "padding: 2rem; \
     text-align: center;"
}

/// Generate error style
pub fn error_style(dark: bool) -> String {
    let color = negative_color(dark);
    format!(
        "padding: 1rem; \
         border: 1px solid {color}; \
         color: {color};"
    )
}

/// Generate empty state style
pub fn empty_style(dark: bool) -> String {
    let color = muted_color(dark);
    format!(
        "padding: 2rem; \
         text-align: center; \
         color: {color};"
    )
}

/// Generate code/mono style
pub fn code_style(dark: bool) -> String {
    let color = accent_color(dark);
    format!(
        "color: {color}; \
         font-family: inherit;"
    )
}

/// SVG grid line color
pub fn grid_color(dark: bool) -> &'static str {
    if dark { "#222" } else { "#eee" }
}

/// SVG axis label color
pub fn axis_color(dark: bool) -> &'static str {
    muted_color(dark)
}

/// Get change color based on percentage and theme
pub fn change_color(dark: bool, change_pct: f64) -> &'static str {
    if change_pct > 5.0 {
        negative_color(dark)
    } else if change_pct < -5.0 {
        positive_color(dark)
    } else {
        muted_color(dark)
    }
}

/// Get selection marker color for charts
pub fn marker_color(dark: bool) -> &'static str {
    accent_color(dark)
}

/// Get hover line color for charts
pub fn hover_color(dark: bool) -> &'static str {
    if dark { "#ff00ff" } else { "#990099" }
}
