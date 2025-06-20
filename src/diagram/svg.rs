//! SVG rendering for event model diagrams.
//!
//! This module converts an EventModelDiagram into SVG output.

use super::EventModelDiagram;

/// Escapes special XML characters in text.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl EventModelDiagram {
    /// Renders the diagram to SVG.
    pub fn to_svg(&self) -> String {
        format!(
            r#"<svg width="1200" height="800" xmlns="http://www.w3.org/2000/svg">
    <rect width="1200" height="800" fill="white" />
    <text x="600" y="40" text-anchor="middle" font-family="Arial, sans-serif" font-size="24" font-weight="bold" fill="{}">{}</text>
</svg>"#,
            "#24292e",
            xml_escape(self.workflow_title())
        )
    }
}
