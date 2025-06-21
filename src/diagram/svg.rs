//! SVG rendering for event model diagrams.
//!
//! This module provides functionality to render event model diagrams as SVG.

use super::{EventModelDiagram, Result};

/// Renders an event model diagram to SVG format.
///
/// This function takes a constructed diagram and produces the SVG representation.
pub fn render_to_svg(diagram: &EventModelDiagram) -> Result<String> {
    // For now, just render a minimal SVG with the workflow title
    let svg = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600">
  <text x="20" y="40" font-family="Arial, sans-serif" font-size="24" font-weight="bold">
    {}
  </text>
</svg>"#,
        diagram.workflow_title().as_str()
    );

    Ok(svg)
}
