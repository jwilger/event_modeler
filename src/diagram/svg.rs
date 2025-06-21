//! SVG rendering for event model diagrams.
//!
//! This module provides functionality to render event model diagrams as SVG.

use super::{EventModelDiagram, Result};
use crate::event_model::yaml_types;
use crate::infrastructure::types::NonEmpty;

// Constants for SVG dimensions and text coordinates
const MIN_WIDTH: u32 = 1200; // Minimum reasonable width
const PADDING: u32 = 20; // Consistent padding around elements
const TITLE_FONT_SIZE: u32 = 12;
const TITLE_Y: u32 = 35;

// Swimlane constants
const MIN_SWIMLANE_HEIGHT: u32 = 200; // Minimum height for empty swimlane
const SWIMLANE_LABEL_WIDTH: u32 = 80; // Width for rotated labels
const SWIMLANE_LABEL_FONT_SIZE: u32 = 10;
const HEADER_HEIGHT: u32 = 50; // Space for title area

// Slice constants
const SLICE_HEADER_HEIGHT: u32 = 30; // Height of slice header area
const MIN_SLICE_WIDTH: u32 = 300; // Minimum width per slice
const SLICE_HEADER_FONT_SIZE: u32 = 11;

// Colors
const BACKGROUND_COLOR: &str = "#f8f8f8"; // Light gray background
const TEXT_COLOR: &str = "#333333"; // Dark gray text
const SWIMLANE_BORDER_COLOR: &str = "#cccccc"; // Light gray for borders

/// Renders an event model diagram to SVG format.
///
/// This function takes a constructed diagram and produces the SVG representation.
pub fn render_to_svg(diagram: &EventModelDiagram) -> Result<String> {
    let swimlanes = diagram.swimlanes();
    let num_swimlanes = swimlanes.len();
    let slices = diagram.slices();
    let num_slices = slices.len();

    // Calculate dynamic dimensions
    // Width based on number of slices, with minimum width
    let total_width = if num_slices > 0 {
        SWIMLANE_LABEL_WIDTH + (num_slices as u32 * MIN_SLICE_WIDTH)
    } else {
        MIN_WIDTH
    };

    // Each swimlane gets minimum height for now
    // TODO: In future steps, height will grow based on content in each swimlane
    let swimlane_heights: Vec<u32> = vec![MIN_SWIMLANE_HEIGHT; num_swimlanes];
    let total_swimlane_height: u32 = swimlane_heights.iter().sum();
    let swimlanes_start_y = HEADER_HEIGHT + SLICE_HEADER_HEIGHT;
    let total_height = swimlanes_start_y + total_swimlane_height + PADDING;

    let mut svg_content = String::new();

    // SVG header
    svg_content.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
  <!-- Canvas background -->
  <rect x="0" y="0" width="{}" height="{}" fill="{}" stroke="none"/>
  
  <!-- Workflow title -->
  <text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" font-weight="normal" fill="{}">
    {}
  </text>
"#,
        total_width,
        total_height,
        total_width,
        total_height,
        BACKGROUND_COLOR,
        PADDING,
        TITLE_Y,
        TITLE_FONT_SIZE,
        TEXT_COLOR,
        diagram.workflow_title().as_str()
    ));

    // Render slice headers
    if !slices.is_empty() {
        svg_content.push_str(&render_slice_headers(
            slices,
            SWIMLANE_LABEL_WIDTH,
            total_width,
            total_height,
        ));
    }

    // Render swimlanes
    svg_content.push_str(&render_swimlanes(
        swimlanes,
        &swimlane_heights,
        swimlanes_start_y,
        total_width,
    ));

    // Close SVG
    svg_content.push_str("</svg>");

    Ok(svg_content)
}

/// Renders the swimlanes with labels and dividers.
fn render_swimlanes(
    swimlanes: &NonEmpty<yaml_types::Swimlane>,
    swimlane_heights: &[u32],
    start_y: u32,
    total_width: u32,
) -> String {
    let mut svg = String::new();

    svg.push_str("  <!-- Swimlanes -->\n");

    let mut current_y = start_y;

    // Draw top border of first swimlane
    svg.push_str(&format!(
        r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
"#,
        0, current_y, total_width, current_y, SWIMLANE_BORDER_COLOR
    ));

    for (index, (swimlane, &height)) in swimlanes.iter().zip(swimlane_heights.iter()).enumerate() {
        // Draw horizontal line between swimlanes (not before the first one)
        if index > 0 {
            svg.push_str(&format!(
                r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
"#,
                0, current_y, total_width, current_y, SWIMLANE_BORDER_COLOR
            ));
        }

        // Draw rotated label on the left
        let label_x = SWIMLANE_LABEL_WIDTH / 2;
        let label_y = current_y + (height / 2);

        svg.push_str(&format!(
            r#"  <text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" fill="{}" text-anchor="middle" transform="rotate(-90 {} {})">
    {}
  </text>
"#,
            label_x,
            label_y,
            SWIMLANE_LABEL_FONT_SIZE,
            TEXT_COLOR,
            label_x,
            label_y,
            swimlane.name.clone().into_inner().as_str()
        ));

        // Draw vertical line to separate label area from content area
        svg.push_str(&format!(
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
"#,
            SWIMLANE_LABEL_WIDTH,
            current_y,
            SWIMLANE_LABEL_WIDTH,
            current_y + height,
            SWIMLANE_BORDER_COLOR
        ));

        current_y += height;
    }

    // Draw bottom border
    svg.push_str(&format!(
        r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
"#,
        0, current_y, total_width, current_y, SWIMLANE_BORDER_COLOR
    ));

    svg
}

/// Renders the slice headers with dividers.
fn render_slice_headers(
    slices: &[yaml_types::Slice],
    start_x: u32,
    total_width: u32,
    total_height: u32,
) -> String {
    let mut svg = String::new();

    svg.push_str("  <!-- Slice headers -->\n");

    // Slices are now in a Vec, so order is preserved
    let slice_width = (total_width - start_x) / slices.len() as u32;

    for (index, slice) in slices.iter().enumerate() {
        let x_position = start_x + (index as u32 * slice_width);

        // Draw vertical divider through all swimlanes (except before the first slice)
        if index > 0 {
            svg.push_str(&format!(
                r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
"#,
                x_position,
                HEADER_HEIGHT,
                x_position,
                total_height - PADDING,
                SWIMLANE_BORDER_COLOR
            ));
        }

        // Draw slice header text (centered in slice)
        let text_x = x_position + (slice_width / 2);
        let text_y = HEADER_HEIGHT + (SLICE_HEADER_HEIGHT / 2) + 3; // +3 for vertical centering

        svg.push_str(&format!(
            r#"  <text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" fill="{}" text-anchor="middle">
    {}
  </text>
"#,
            text_x,
            text_y,
            SLICE_HEADER_FONT_SIZE,
            TEXT_COLOR,
            // The slice name is already in display format from the YAML
            slice.name.clone().into_inner().as_str()
        ));
    }

    // Draw horizontal line below slice headers
    svg.push_str(&format!(
        r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>
"#,
        start_x,
        HEADER_HEIGHT + SLICE_HEADER_HEIGHT,
        total_width,
        HEADER_HEIGHT + SLICE_HEADER_HEIGHT,
        SWIMLANE_BORDER_COLOR
    ));

    svg
}
