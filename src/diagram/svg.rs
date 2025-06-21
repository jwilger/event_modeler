//! SVG rendering for event model diagrams.
//!
//! This module provides functionality to render event model diagrams as SVG.

use super::{EventModelDiagram, Result};

// Constants for SVG dimensions and text coordinates
const DEFAULT_WIDTH: u32 = 1200;
const DEFAULT_HEIGHT: u32 = 800;
const MARGIN: u32 = 10;
const TITLE_FONT_SIZE: u32 = 14;
const TITLE_Y: u32 = 25;

// Colors
const BACKGROUND_COLOR: &str = "#f8f8f8"; // Light gray background
const TEXT_COLOR: &str = "#333333"; // Dark gray text

/// Renders an event model diagram to SVG format.
///
/// This function takes a constructed diagram and produces the SVG representation.
pub fn render_to_svg(diagram: &EventModelDiagram) -> Result<String> {
    // Step 1: Canvas with workflow title
    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
  <!-- Canvas background -->
  <rect x="0" y="0" width="{}" height="{}" fill="{}" stroke="none"/>
  
  <!-- Workflow title -->
  <text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" font-weight="normal" fill="{}">
    {}
  </text>
</svg>"#,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        BACKGROUND_COLOR,
        MARGIN,
        TITLE_Y,
        TITLE_FONT_SIZE,
        TEXT_COLOR,
        diagram.workflow_title().as_str()
    );

    Ok(svg)
}
