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
        const TITLE_HEIGHT: f64 = 60.0;
        const SWIMLANE_HEIGHT: f64 = 200.0;
        const SWIMLANE_LABEL_WIDTH: f64 = 150.0;
        const CANVAS_WIDTH: f64 = 1200.0;

        let canvas_height = TITLE_HEIGHT + (self.swimlanes().len() as f64 * SWIMLANE_HEIGHT) + 50.0;

        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
    <rect width="{}" height="{}" fill="white" />
    <text x="600" y="40" text-anchor="middle" font-family="Liberation Sans, Arial, sans-serif" font-size="24" font-weight="bold" fill="{}">{}</text>"#,
            CANVAS_WIDTH,
            canvas_height,
            CANVAS_WIDTH,
            canvas_height,
            "#24292e",
            xml_escape(self.workflow_title())
        );

        // Draw swimlanes
        for (index, swimlane) in self.swimlanes().iter().enumerate() {
            let y = TITLE_HEIGHT + (index as f64 * SWIMLANE_HEIGHT);

            // Draw swimlane background
            svg.push_str(&format!(
                r#"
    <rect x="0" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1" />"#,
                y, CANVAS_WIDTH, SWIMLANE_HEIGHT, "#f6f8fa", "#e1e4e8"
            ));

            // Draw swimlane label area
            svg.push_str(&format!(
                r#"
    <rect x="0" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1" />"#,
                y, SWIMLANE_LABEL_WIDTH, SWIMLANE_HEIGHT, "#f6f8fa", "#e1e4e8"
            ));

            // Draw rotated swimlane label
            let label_x = SWIMLANE_LABEL_WIDTH / 2.0;
            let label_y = y + (SWIMLANE_HEIGHT / 2.0);

            svg.push_str(&format!(
                r#"
    <g transform="translate({},{}) rotate(-90)">
        <text x="0" y="0" text-anchor="middle" font-family="Liberation Sans, Arial, sans-serif" font-size="16" font-weight="bold" fill="{}">{}</text>
    </g>"#,
                label_x,
                label_y,
                "#000000",
                xml_escape(&swimlane.label())
            ));
        }

        svg.push_str("\n</svg>");
        svg
    }
}
