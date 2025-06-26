//! SVG rendering for event model diagrams.
//!
//! This module provides functionality to render event model diagrams as SVG.

use super::{EventModelDiagram, Result};
use crate::event_model::yaml_types;
use crate::infrastructure::types::NonEmpty;
use std::collections::HashMap;

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

// Entity constants
const ENTITY_BOX_WIDTH: u32 = 120; // Width of entity boxes
const ENTITY_BOX_HEIGHT: u32 = 60; // Height of entity boxes
const ENTITY_PADDING: u32 = 10; // Padding inside entity boxes
const ENTITY_MARGIN: u32 = 20; // Margin between entities
const ENTITY_LABEL_FONT_SIZE: u32 = 9; // Font size for entity type labels
const ENTITY_NAME_FONT_SIZE: u32 = 10; // Font size for entity names

// Entity colors
const VIEW_BACKGROUND_COLOR: &str = "#ffffff"; // White for views

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

    // Calculate swimlane heights based on content
    // First, we need to analyze content to determine heights
    let mut swimlane_content_heights: Vec<u32> = vec![0; num_swimlanes];

    // For now, check views in each swimlane (will expand to other entities later)
    for view_def in diagram.views().values() {
        if let Some(swimlane_index) = swimlanes.iter().position(|s| s.id == view_def.swimlane) {
            // Account for entity height plus margins
            swimlane_content_heights[swimlane_index] =
                swimlane_content_heights[swimlane_index].max(ENTITY_BOX_HEIGHT + 2 * ENTITY_MARGIN);
        }
    }

    // Ensure minimum height for each swimlane
    let swimlane_heights: Vec<u32> = swimlane_content_heights
        .iter()
        .map(|&content_height| content_height.max(MIN_SWIMLANE_HEIGHT))
        .collect();

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

    // Render entities (views, commands, etc.)
    svg_content.push_str(&render_entities(
        diagram,
        swimlanes,
        slices,
        &swimlane_heights,
        swimlanes_start_y,
        SWIMLANE_LABEL_WIDTH,
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

/// Process an entity reference and add it to the entities_by_slice_and_swimlane map if it's a view.
fn process_entity_reference<'a>(
    entity_ref: &yaml_types::EntityReference,
    slice_index: usize,
    view_lookup: &HashMap<String, &'a yaml_types::ViewDefinition>,
    entities_by_slice_and_swimlane: &mut HashMap<(usize, &'a yaml_types::SwimlaneId), Vec<String>>,
) {
    if let yaml_types::EntityReference::View(view_path) = entity_ref {
        // Extract the view name from the path (before any dots)
        let view_name_string = view_path.clone().into_inner();
        let view_name_str = view_name_string.as_str();
        let base_view_name = view_name_str.split('.').next().unwrap_or(view_name_str);

        // Find the view definition using the lookup map
        if let Some(view_def) = view_lookup.get(base_view_name) {
            let key = (slice_index, &view_def.swimlane);
            entities_by_slice_and_swimlane
                .entry(key)
                .or_default()
                .push(base_view_name.to_string());
        }
    }
}

/// Renders all entities (views, commands, events, etc.) in their respective positions.
fn render_entities(
    diagram: &EventModelDiagram,
    swimlanes: &NonEmpty<yaml_types::Swimlane>,
    slices: &[yaml_types::Slice],
    swimlane_heights: &[u32],
    swimlanes_start_y: u32,
    start_x: u32,
    total_width: u32,
) -> String {
    let mut svg = String::new();

    svg.push_str("  <!-- Entities -->\n");

    // Create a map of swimlane IDs to their Y positions
    let mut swimlane_y_positions = HashMap::new();
    let mut current_y = swimlanes_start_y;
    for (swimlane, &height) in swimlanes.iter().zip(swimlane_heights.iter()) {
        swimlane_y_positions.insert(&swimlane.id, current_y);
        current_y += height;
    }

    // Calculate slice X positions
    let slice_width = if !slices.is_empty() {
        (total_width - start_x) / slices.len() as u32
    } else {
        total_width - start_x
    };

    // For now, just render views in their slices
    // First, we need to find which views appear in which slices
    let mut entities_by_slice_and_swimlane: HashMap<(usize, &yaml_types::SwimlaneId), Vec<String>> =
        HashMap::new();

    // Build a lookup map from view names to definitions for performance
    let view_lookup: HashMap<String, &yaml_types::ViewDefinition> = diagram
        .views()
        .iter()
        .map(|(name, def)| (name.clone().into_inner().as_str().to_string(), def))
        .collect();

    // Parse slice connections to find view positions
    for (slice_index, slice) in slices.iter().enumerate() {
        for connection in slice.connections.iter() {
            // Process both sides of the connection
            process_entity_reference(
                &connection.from,
                slice_index,
                &view_lookup,
                &mut entities_by_slice_and_swimlane,
            );
            process_entity_reference(
                &connection.to,
                slice_index,
                &view_lookup,
                &mut entities_by_slice_and_swimlane,
            );
        }
    }

    // Remove duplicates while preserving order
    for entities in entities_by_slice_and_swimlane.values_mut() {
        let mut seen = std::collections::HashSet::new();
        entities.retain(|item| seen.insert(item.clone()));
    }

    // Render views
    for ((slice_index, swimlane_id), entity_names) in &entities_by_slice_and_swimlane {
        if let Some(&swimlane_y) = swimlane_y_positions.get(swimlane_id) {
            let slice_x = start_x + (*slice_index as u32 * slice_width);
            let num_entities = entity_names.len();

            // Calculate available space and entity spacing for horizontal layout
            let available_width = slice_width - (2 * ENTITY_MARGIN);

            // Position entities horizontally within the slice
            for (entity_index, entity_name) in entity_names.iter().enumerate() {
                // Calculate entity position for horizontal layout
                let entity_x = if num_entities == 1 {
                    // Center single entity
                    slice_x + (slice_width - ENTITY_BOX_WIDTH) / 2
                } else {
                    // Distribute multiple entities horizontally
                    let entity_plus_margin = ENTITY_BOX_WIDTH + ENTITY_MARGIN;
                    let total_entities_width = num_entities as u32 * ENTITY_BOX_WIDTH
                        + (num_entities as u32 - 1) * ENTITY_MARGIN;

                    if total_entities_width <= available_width {
                        // Enough space - use standard spacing
                        let start_x = slice_x + (slice_width - total_entities_width) / 2;
                        start_x + entity_index as u32 * entity_plus_margin
                    } else {
                        // Not enough space - calculate minimal overlap
                        let max_possible_spacing =
                            (available_width - ENTITY_BOX_WIDTH) / (num_entities as u32 - 1);
                        let spacing = max_possible_spacing.min(entity_plus_margin);
                        slice_x + ENTITY_MARGIN + entity_index as u32 * spacing
                    }
                };

                // Get swimlane index to access height
                let swimlane_index = swimlanes
                    .iter()
                    .position(|s| &s.id == *swimlane_id)
                    .unwrap();
                let swimlane_height = swimlane_heights[swimlane_index];

                // Center entity vertically in swimlane
                let entity_y = swimlane_y + (swimlane_height - ENTITY_BOX_HEIGHT) / 2;

                // Render view box
                svg.push_str(&render_view_box(entity_x, entity_y, entity_name));
            }
        }
    }

    svg
}

/// Formats an entity name by inserting spaces before capital letters.
/// E.g., "LoginScreen" becomes "Login Screen", "UserProfileScreen" becomes "User Profile Screen"
fn format_entity_name(name: &str) -> String {
    let mut result = String::new();
    let mut chars = name.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_uppercase() && !result.is_empty() {
            // Add space before capital letter, unless previous char was also uppercase
            // This handles cases like "XMLParser" -> "XML Parser" correctly
            if let Some(last) = result.chars().last() {
                if !last.is_uppercase() || (chars.peek().is_some_and(|&next| !next.is_uppercase()))
                {
                    result.push(' ');
                }
            }
        }
        result.push(ch);
    }

    result
}

/// Renders a single view box.
fn render_view_box(x: u32, y: u32, name: &str) -> String {
    let mut svg = String::new();

    // Draw the box
    svg.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{ENTITY_BOX_WIDTH}" height="{ENTITY_BOX_HEIGHT}" fill="{VIEW_BACKGROUND_COLOR}" stroke="{SWIMLANE_BORDER_COLOR}" stroke-width="1"/>
"#
    ));

    // Draw the entity type label "View"
    let label_x = x + ENTITY_BOX_WIDTH / 2;
    let label_y = y + ENTITY_PADDING + ENTITY_LABEL_FONT_SIZE;
    svg.push_str(&format!(
        r#"  <text x="{label_x}" y="{label_y}" font-family="Arial, sans-serif" font-size="{ENTITY_LABEL_FONT_SIZE}" fill="{TEXT_COLOR}" text-anchor="middle">View</text>
"#
    ));

    // Draw the entity name (formatted with spaces)
    let formatted_name = format_entity_name(name);
    let name_y = y + ENTITY_BOX_HEIGHT / 2 + ENTITY_NAME_FONT_SIZE / 2;
    svg.push_str(&format!(
        r#"  <text x="{label_x}" y="{name_y}" font-family="Arial, sans-serif" font-size="{ENTITY_NAME_FONT_SIZE}" fill="{TEXT_COLOR}" text-anchor="middle">{formatted_name}</text>
"#
    ));

    svg
}
