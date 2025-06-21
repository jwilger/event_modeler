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

/// Wraps text to fit within a given width when rotated.
/// Returns a vector of lines.
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    // Split by both whitespace and comma (keeping the comma with the word)
    let mut words = Vec::new();
    for part in text.split_whitespace() {
        if part.contains(',') {
            // Split around comma but keep it
            let comma_parts: Vec<&str> = part.splitn(2, ',').collect();
            if comma_parts.len() == 2 && !comma_parts[1].is_empty() {
                words.push(format!("{},", comma_parts[0]));
                words.push(comma_parts[1].to_string());
            } else {
                words.push(part.to_string());
            }
        } else {
            words.push(part.to_string());
        }
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.is_empty() {
            current_line = word;
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(&word);
        } else {
            lines.push(current_line);
            current_line = word;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

impl EventModelDiagram {
    /// Renders the diagram to SVG.
    pub fn to_svg(&self) -> String {
        const TITLE_HEIGHT: f64 = 60.0;
        const SWIMLANE_HEIGHT: f64 = 200.0;
        const CANVAS_WIDTH: f64 = 1200.0;
        const FONT_SIZE: f64 = 16.0;
        const LINE_HEIGHT: f64 = 20.0; // Line height for wrapped text
        const PADDING: f64 = 20.0; // Padding around text
        const MAX_CHARS_PER_LINE: usize = 20; // Maximum characters per line when wrapped

        // Calculate the maximum width needed for all swimlane labels
        let mut max_label_width = 100.0; // Minimum width

        for swimlane in self.swimlanes() {
            let label = swimlane.label();
            let wrapped = wrap_text(&label, MAX_CHARS_PER_LINE);

            // When rotated -90 degrees:
            // - The height of the text area becomes the width of the label section
            // - The width of the text area becomes limited by the swimlane height

            // Find the longest line to determine minimum width needed
            let max_line_length = wrapped.iter().map(|line| line.len()).max().unwrap_or(0) as f64;

            // Calculate required width based on either:
            // 1. The number of lines (for vertical space when rotated)
            // 2. The longest line (for horizontal space when rotated)
            let text_height = wrapped.len() as f64 * LINE_HEIGHT;
            let text_width = max_line_length * 8.0; // Approximate character width

            // The required width is the maximum of:
            // - Space needed for wrapped lines (text_height when rotated)
            // - Space needed for the longest line (ensures text fits)
            let required_width = text_height.max(text_width) + PADDING * 2.0;

            if required_width > max_label_width {
                max_label_width = required_width;
            }
        }

        let canvas_height = TITLE_HEIGHT + (self.swimlanes().len() as f64 * SWIMLANE_HEIGHT) + 50.0;

        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
    <rect width="{}" height="{}" fill="white" />
    <text x="{}" y="40" text-anchor="start" font-family="Liberation Sans, Arial, sans-serif" font-size="24" font-weight="bold" fill="{}">{}</text>"#,
            CANVAS_WIDTH,
            canvas_height,
            CANVAS_WIDTH,
            canvas_height,
            PADDING, // Use same padding as other elements
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

            // Draw swimlane label area with calculated width
            svg.push_str(&format!(
                r#"
    <rect x="0" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1" />"#,
                y, max_label_width, SWIMLANE_HEIGHT, "#f6f8fa", "#e1e4e8"
            ));

            // Draw rotated swimlane label with wrapped text
            let label_x = max_label_width / 2.0;
            let label_y = y + (SWIMLANE_HEIGHT / 2.0);

            // Wrap the text
            let wrapped_lines = wrap_text(&swimlane.label(), MAX_CHARS_PER_LINE);
            let total_height = wrapped_lines.len() as f64 * LINE_HEIGHT;
            let start_offset = total_height / 2.0;

            svg.push_str(&format!(
                r#"
    <g transform="translate({},{}) rotate(-90)">"#,
                label_x, label_y
            ));

            // Render each line of wrapped text
            for (i, line) in wrapped_lines.iter().enumerate() {
                let y_offset = -start_offset + (i as f64 + 0.5) * LINE_HEIGHT;
                svg.push_str(&format!(
                    r#"
        <text x="0" y="{}" text-anchor="middle" font-family="Liberation Sans, Arial, sans-serif" font-size="{}" font-weight="bold" fill="{}">{}</text>"#,
                    y_offset,
                    FONT_SIZE,
                    "#000000",
                    xml_escape(line)
                ));
            }

            svg.push_str("\n    </g>");
        }

        svg.push_str("\n</svg>");
        svg
    }
}
