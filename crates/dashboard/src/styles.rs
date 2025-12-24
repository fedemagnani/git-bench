//! Minimalistic styles - no external CSS files
//!
//! Clean, practical design with light/dark theme support.

// ============================================================================
// Theme-aware style generators
// ============================================================================

/// Generate app style based on theme
pub fn app_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#1a1a1a", "#e0e0e0")
    } else {
        ("#ffffff", "#1a1a1a")
    };
    format!(
        "min-height: 100vh; \
         font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; \
         font-size: 14px; \
         background: {bg}; \
         color: {fg}; \
         line-height: 1.5; \
         margin: 0; \
         padding: 0;"
    )
}

/// Generate container style
pub fn container_style(_dark: bool) -> &'static str {
    "max-width: 960px; \
     margin: 0 auto; \
     padding: 1rem;"
}

/// Generate header style
pub fn header_style(dark: bool) -> String {
    let border = if dark { "#333" } else { "#ddd" };
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.75rem 1rem; \
         border-bottom: 1px solid {border}; \
         margin-bottom: 1rem;"
    )
}

/// Generate title style
pub fn title_style(_dark: bool) -> &'static str {
    "font-size: 1rem; \
     font-weight: 600; \
     margin: 0;"
}

/// Generate theme toggle button style
pub fn toggle_btn_style(dark: bool) -> String {
    let (bg, fg, border) = if dark {
        ("#333", "#e0e0e0", "#555")
    } else {
        ("#f5f5f5", "#1a1a1a", "#ccc")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         border: 1px solid {border}; \
         padding: 0.25rem 0.5rem; \
         font-family: inherit; \
         font-size: 0.8rem; \
         cursor: pointer;"
    )
}

/// Generate section style (suite/module containers)
pub fn section_style(dark: bool) -> String {
    let border = if dark { "#333" } else { "#ddd" };
    format!(
        "border: 1px solid {border}; \
         margin-bottom: 1rem;"
    )
}

/// Generate section header style
pub fn section_header_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#222", "#333")
    } else {
        ("#f8f8f8", "#ddd")
    };
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.5rem 0.75rem; \
         background: {bg}; \
         border-bottom: 1px solid {border}; \
         cursor: pointer; \
         user-select: none;"
    )
}

/// Generate section title style
pub fn section_title_style(_dark: bool) -> &'static str {
    "font-size: 0.9rem; \
     font-weight: 600; \
     margin: 0;"
}

/// Generate badge style
pub fn badge_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#333", "#888")
    } else {
        ("#eee", "#666")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0.1rem 0.4rem; \
         font-size: 0.75rem; \
         margin-left: 0.5rem;"
    )
}

/// Generate expand button style
pub fn expand_btn_style(dark: bool) -> String {
    let fg = if dark { "#888" } else { "#666" };
    format!(
        "background: none; \
         border: none; \
         color: {fg}; \
         font-size: 0.8rem; \
         cursor: pointer; \
         padding: 0.25rem;"
    )
}

/// Generate module container style
pub fn module_style(dark: bool) -> String {
    let border = if dark { "#333" } else { "#ddd" };
    format!(
        "border: 1px solid {border}; \
         margin: 0.5rem;"
    )
}

/// Generate module header style
pub fn module_header_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#1a1a1a", "#333")
    } else {
        ("#fafafa", "#ddd")
    };
    format!(
        "display: flex; \
         justify-content: space-between; \
         align-items: center; \
         padding: 0.4rem 0.6rem; \
         background: {bg}; \
         border-bottom: 1px solid {border}; \
         cursor: pointer;"
    )
}

/// Generate module title style
pub fn module_title_style(dark: bool) -> String {
    let fg = if dark { "#a0a0ff" } else { "#4040a0" };
    format!(
        "font-size: 0.85rem; \
         font-weight: 600; \
         color: {fg}; \
         margin: 0;"
    )
}

/// Generate chart container style
pub fn chart_style(dark: bool) -> String {
    let border = if dark { "#333" } else { "#ddd" };
    format!(
        "border: 1px solid {border}; \
         margin: 0.5rem;"
    )
}

/// Generate chart header style
pub fn chart_header_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#222", "#333")
    } else {
        ("#f5f5f5", "#ddd")
    };
    format!(
        "display: flex; \
         align-items: center; \
         gap: 0.5rem; \
         padding: 0.3rem 0.6rem; \
         background: {bg}; \
         border-bottom: 1px solid {border};"
    )
}

/// Generate chart title style
pub fn chart_title_style(dark: bool) -> String {
    let fg = if dark { "#80c0ff" } else { "#2060a0" };
    format!(
        "font-size: 0.8rem; \
         font-weight: 600; \
         color: {fg}; \
         margin: 0;"
    )
}

/// Generate unit badge style
pub fn unit_badge_style(dark: bool) -> String {
    let fg = if dark { "#666" } else { "#888" };
    format!(
        "color: {fg}; \
         font-size: 0.7rem;"
    )
}

/// Generate chart legend style
pub fn legend_style(dark: bool) -> String {
    let border = if dark { "#333" } else { "#ddd" };
    format!(
        "display: flex; \
         flex-wrap: wrap; \
         gap: 0.75rem; \
         padding: 0.5rem; \
         border-top: 1px solid {border}; \
         font-size: 0.75rem;"
    )
}

/// Generate legend item style
pub fn legend_item_style(_dark: bool) -> &'static str {
    "display: flex; \
     align-items: center; \
     gap: 0.3rem;"
}

/// Generate legend style for right-side vertical stacking
pub fn legend_right_style(dark: bool) -> String {
    let border = if dark { "#333" } else { "#ddd" };
    format!(
        "display: flex; \
         flex-direction: column; \
         justify-content: center; \
         gap: 0.4rem; \
         padding: 0.5rem 0.75rem; \
         border-left: 1px solid {border}; \
         font-size: 0.7rem; \
         min-width: 100px; \
         max-width: 140px;"
    )
}

/// Generate legend item style for vertical layout
pub fn legend_item_vertical_style(_dark: bool) -> &'static str {
    "display: flex; \
     align-items: center; \
     gap: 0.25rem; \
     white-space: nowrap; \
     overflow: hidden;"
}

/// Generate tooltip style (interactive - no pointer-events: none)
pub fn tooltip_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#2a2a2a", "#444")
    } else {
        ("#fff", "#ccc")
    };
    format!(
        "position: absolute; \
         background: {bg}; \
         border: 1px solid {border}; \
         padding: 0.5rem; \
         font-size: 0.75rem; \
         z-index: 100; \
         min-width: 180px; \
         box-shadow: 0 2px 8px rgba(0,0,0,0.2);"
    )
}

/// Generate compact tooltip style for hover (smaller, less invasive)
pub fn compact_tooltip_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("rgba(30, 30, 30, 0.95)", "#444")
    } else {
        ("rgba(255, 255, 255, 0.95)", "#ccc")
    };
    format!(
        "position: absolute; \
         background: {bg}; \
         border: 1px solid {border}; \
         padding: 0.3rem 0.4rem; \
         font-size: 0.65rem; \
         z-index: 100; \
         box-shadow: 0 1px 4px rgba(0,0,0,0.15);"
    )
}

/// Generate commit link style
pub fn commit_link_style(dark: bool) -> String {
    let fg = if dark { "#80c0ff" } else { "#2060a0" };
    format!(
        "color: {fg}; \
         text-decoration: none; \
         font-family: inherit;"
    )
}

/// Generate commit link hover style
pub fn commit_link_hover_style(dark: bool) -> String {
    let fg = if dark { "#80c0ff" } else { "#2060a0" };
    format!("color: {fg}; text-decoration: underline;")
}

/// Generate loading style
pub fn loading_style(_dark: bool) -> &'static str {
    "padding: 2rem; \
     text-align: center;"
}

/// Generate error style
pub fn error_style(dark: bool) -> String {
    let border = if dark { "#a04040" } else { "#d04040" };
    format!(
        "padding: 1rem; \
         border: 1px solid {border}; \
         margin: 1rem 0;"
    )
}

/// Generate empty state style
pub fn empty_style(_dark: bool) -> &'static str {
    "padding: 2rem; \
     text-align: center; \
     opacity: 0.7;"
}

/// Generate link style
pub fn link_style(dark: bool) -> String {
    let fg = if dark { "#80c0ff" } else { "#2060a0" };
    format!("color: {fg};")
}

/// Generate muted text style
pub fn muted_style(dark: bool) -> String {
    let fg = if dark { "#666" } else { "#888" };
    format!("color: {fg};")
}

/// Generate code/mono style
pub fn code_style(dark: bool) -> String {
    let (bg, fg) = if dark {
        ("#222", "#80ff80")
    } else {
        ("#f5f5f5", "#208020")
    };
    format!(
        "background: {bg}; \
         color: {fg}; \
         padding: 0.2rem 0.4rem; \
         font-size: 0.85rem;"
    )
}

/// SVG grid line color
pub fn grid_color(dark: bool) -> &'static str {
    if dark { "#333" } else { "#ddd" }
}

/// SVG axis label color
pub fn axis_color(dark: bool) -> &'static str {
    if dark { "#666" } else { "#888" }
}

/// Chart line colors (work for both themes)
pub const CHART_COLORS: [&str; 10] = [
    "#4080ff", "#40c040", "#ff6060", "#a080ff", "#ff8000",
    "#00c0c0", "#ff40ff", "#80c000", "#ff4080", "#4040ff",
];

/// Modal overlay style (fullscreen backdrop)
pub fn modal_overlay_style(dark: bool) -> String {
    let bg = if dark {
        "rgba(0, 0, 0, 0.85)"
    } else {
        "rgba(0, 0, 0, 0.6)"
    };
    format!(
        "position: fixed; \
         top: 0; \
         left: 0; \
         right: 0; \
         bottom: 0; \
         background: {bg}; \
         display: flex; \
         justify-content: center; \
         align-items: center; \
         z-index: 1000; \
         padding: 2rem;"
    )
}

/// Modal content style
pub fn modal_content_style(dark: bool) -> String {
    let (bg, border) = if dark {
        ("#1a1a1a", "#444")
    } else {
        ("#ffffff", "#ccc")
    };
    format!(
        "background: {bg}; \
         border: 1px solid {border}; \
         padding: 1.5rem; \
         max-width: 90vw; \
         max-height: 90vh; \
         overflow: auto; \
         min-width: 600px;"
    )
}

/// Close button style
pub fn close_btn_style(dark: bool) -> String {
    let (fg, hover_bg) = if dark {
        ("#888", "#333")
    } else {
        ("#666", "#eee")
    };
    format!(
        "background: none; \
         border: none; \
         color: {fg}; \
         font-size: 1.2rem; \
         cursor: pointer; \
         padding: 0.25rem 0.5rem; \
         line-height: 1;"
    )
}
