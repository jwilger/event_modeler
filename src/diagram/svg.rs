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

    // First, pre-calculate dimensions for all entities
    let mut entity_dimensions_map: HashMap<String, EntityDimensions> = HashMap::new();
    for view_name in diagram.views().keys() {
        let name_string = view_name.clone().into_inner();
        let name_str = name_string.as_str();
        let dimensions = calculate_entity_dimensions(name_str, "View");
        entity_dimensions_map.insert(name_str.to_string(), dimensions);
    }

    // Build a temporary map for view lookups
    let view_lookup: HashMap<String, &yaml_types::ViewDefinition> = diagram
        .views()
        .iter()
        .map(|(name, def)| (name.clone().into_inner().as_str().to_string(), def))
        .collect();

    // Analyze entities in each slice to determine required widths
    let mut slice_required_widths = vec![MIN_SLICE_WIDTH; num_slices];

    // Count entities in each slice and calculate required space
    for (slice_index, slice) in slices.iter().enumerate() {
        let mut entities_by_swimlane: HashMap<&yaml_types::SwimlaneId, Vec<String>> =
            HashMap::new();

        for connection in slice.connections.iter() {
            // Check both sides of connections for views
            if let yaml_types::EntityReference::View(view_path) = &connection.from {
                let view_name_string = view_path.clone().into_inner();
                let view_name_str = view_name_string.as_str();
                let base_view_name = view_name_str.split('.').next().unwrap_or(view_name_str);

                if let Some(view_def) = view_lookup.get(base_view_name) {
                    entities_by_swimlane
                        .entry(&view_def.swimlane)
                        .or_default()
                        .push(base_view_name.to_string());
                }
            }

            if let yaml_types::EntityReference::View(view_path) = &connection.to {
                let view_name_string = view_path.clone().into_inner();
                let view_name_str = view_name_string.as_str();
                let base_view_name = view_name_str.split('.').next().unwrap_or(view_name_str);

                if let Some(view_def) = view_lookup.get(base_view_name) {
                    entities_by_swimlane
                        .entry(&view_def.swimlane)
                        .or_default()
                        .push(base_view_name.to_string());
                }
            }
        }

        // Remove duplicates and calculate required width
        let mut max_width_in_swimlane = 0u32;
        for entities in entities_by_swimlane.values_mut() {
            let mut seen = std::collections::HashSet::new();
            entities.retain(|item| seen.insert(item.clone()));

            // Calculate total width needed for entities in this swimlane
            let total_entity_width: u32 = entities
                .iter()
                .map(|name| {
                    entity_dimensions_map
                        .get(name)
                        .map(|d| d.width)
                        .unwrap_or(ENTITY_BOX_WIDTH)
                })
                .sum();
            let spacing_width = (entities.len() as u32 + 1) * ENTITY_MARGIN;
            let required_width = total_entity_width + spacing_width;

            max_width_in_swimlane = max_width_in_swimlane.max(required_width);
        }

        // Set slice width based on maximum required in any swimlane
        if max_width_in_swimlane > 0 {
            slice_required_widths[slice_index] = max_width_in_swimlane.max(MIN_SLICE_WIDTH);
        }
    }

    // Calculate total width based on actual requirements
    let total_width = if num_slices > 0 {
        SWIMLANE_LABEL_WIDTH + slice_required_widths.iter().sum::<u32>()
    } else {
        MIN_WIDTH
    };

    // Calculate swimlane heights based on content
    // First, we need to analyze content to determine heights
    let mut swimlane_content_heights: Vec<u32> = vec![0; num_swimlanes];

    // For now, check views in each swimlane (will expand to other entities later)
    for (view_name, view_def) in diagram.views() {
        if let Some(swimlane_index) = swimlanes.iter().position(|s| s.id == view_def.swimlane) {
            let name_string = view_name.clone().into_inner();
            let name_str = name_string.as_str();
            if let Some(dimensions) = entity_dimensions_map.get(name_str) {
                // Account for entity height plus margins
                swimlane_content_heights[swimlane_index] = swimlane_content_heights[swimlane_index]
                    .max(dimensions.height + 2 * ENTITY_MARGIN);
            }
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
            &slice_required_widths,
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
    let render_ctx = EntityRenderContext {
        diagram,
        swimlanes,
        slices,
        slice_widths: &slice_required_widths,
        swimlane_heights: &swimlane_heights,
        swimlanes_start_y,
        start_x: SWIMLANE_LABEL_WIDTH,
        entity_dimensions_map: &entity_dimensions_map,
    };
    svg_content.push_str(&render_entities(&render_ctx));

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
    slice_widths: &[u32],
    start_x: u32,
    total_width: u32,
    total_height: u32,
) -> String {
    let mut svg = String::new();

    svg.push_str("  <!-- Slice headers -->\n");

    let mut current_x = start_x;

    for (index, (slice, &slice_width)) in slices.iter().zip(slice_widths.iter()).enumerate() {
        let x_position = current_x;

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

        current_x += slice_width;
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
fn render_entities(ctx: &EntityRenderContext) -> String {
    let mut svg = String::new();

    svg.push_str("  <!-- Entities -->\n");

    // Create a map of swimlane IDs to their Y positions
    let mut swimlane_y_positions = HashMap::new();
    let mut current_y = ctx.swimlanes_start_y;
    for (swimlane, &height) in ctx.swimlanes.iter().zip(ctx.swimlane_heights.iter()) {
        swimlane_y_positions.insert(&swimlane.id, current_y);
        current_y += height;
    }

    // Calculate slice X positions using the pre-calculated widths
    let mut slice_x_positions = Vec::new();
    let mut current_x = ctx.start_x;
    for &width in ctx.slice_widths {
        slice_x_positions.push(current_x);
        current_x += width;
    }

    // For now, just render views in their slices
    // First, we need to find which views appear in which slices
    let mut entities_by_slice_and_swimlane: HashMap<(usize, &yaml_types::SwimlaneId), Vec<String>> =
        HashMap::new();

    // Build a lookup map from view names to definitions for performance
    let view_lookup: HashMap<String, &yaml_types::ViewDefinition> = ctx
        .diagram
        .views()
        .iter()
        .map(|(name, def)| (name.clone().into_inner().as_str().to_string(), def))
        .collect();

    // Parse slice connections to find view positions
    for (slice_index, slice) in ctx.slices.iter().enumerate() {
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
            let slice_x = slice_x_positions[*slice_index];
            let slice_width = ctx.slice_widths[*slice_index];
            let num_entities = entity_names.len();

            // Position entities horizontally within the slice
            // Since we calculated slice width to fit all entities, we know they will fit
            for (entity_index, entity_name) in entity_names.iter().enumerate() {
                // Get entity dimensions
                let dimensions = ctx
                    .entity_dimensions_map
                    .get(entity_name)
                    .expect("Entity dimensions should have been pre-calculated");

                // Calculate entity position - entities are evenly spaced with proper margins
                let entity_x = if num_entities == 1 {
                    // Center single entity
                    slice_x + (slice_width - dimensions.width) / 2
                } else {
                    // Multiple entities - use the spacing we calculated for
                    // We need to calculate the cumulative width of previous entities
                    let mut cumulative_width = ENTITY_MARGIN;
                    for prev_entity_name in entity_names.iter().take(entity_index) {
                        let prev_dimensions = ctx
                            .entity_dimensions_map
                            .get(prev_entity_name)
                            .expect("Entity dimensions should have been pre-calculated");
                        cumulative_width += prev_dimensions.width + ENTITY_MARGIN;
                    }
                    slice_x + cumulative_width
                };

                // Get swimlane index to access height
                let swimlane_index = ctx
                    .swimlanes
                    .iter()
                    .position(|s| &s.id == *swimlane_id)
                    .unwrap();
                let swimlane_height = ctx.swimlane_heights[swimlane_index];

                // Center entity vertically in swimlane
                let entity_y = swimlane_y + (swimlane_height - dimensions.height) / 2;

                // Render view box
                svg.push_str(&render_view_box(
                    entity_x,
                    entity_y,
                    entity_name,
                    dimensions,
                ));
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

/// Wraps text into balanced lines that fit within the given width.
/// Returns the wrapped lines and the actual dimensions needed.
fn wrap_text(text: &str, max_width: u32, font_size: u32) -> (Vec<String>, u32, u32) {
    // Approximate character width (for Arial font, roughly 0.6x the font size)
    let char_width = (font_size as f32 * 0.6) as u32;
    let max_chars_per_line = max_width / char_width;

    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return (vec![text.to_string()], max_width, font_size);
    }

    // Try different line configurations to find the most balanced
    let total_length: usize = words.iter().map(|w| w.len()).sum::<usize>() + words.len() - 1;
    let ideal_lines = ((total_length as f32 / max_chars_per_line as f32)
        .sqrt()
        .ceil()) as usize;

    let mut best_lines = Vec::new();
    let mut best_score = f32::MAX;

    for target_lines in ideal_lines.saturating_sub(1)..=ideal_lines + 2 {
        if target_lines == 0 || target_lines > words.len() {
            continue;
        }

        let lines = distribute_words(&words, target_lines, max_chars_per_line as usize);
        let score = calculate_balance_score(&lines);

        if score < best_score {
            best_score = score;
            best_lines = lines;
        }
    }

    // Calculate actual dimensions needed
    let max_line_length = best_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let actual_width = (max_line_length as u32 * char_width).max(ENTITY_BOX_WIDTH);
    let line_height = (font_size as f32 * 1.2) as u32;
    let actual_height = best_lines.len() as u32 * line_height;

    (best_lines, actual_width, actual_height)
}

/// Distributes words across a target number of lines.
fn distribute_words(words: &[&str], target_lines: usize, max_chars: usize) -> Vec<String> {
    let mut lines = vec![String::new(); target_lines];
    let mut current_line = 0;

    for word in words {
        // Find the best line for this word
        let mut best_line = current_line;
        let mut min_overflow = i32::MAX;

        for (i, line) in lines.iter().enumerate().take(target_lines) {
            let new_length = if line.is_empty() {
                word.len()
            } else {
                line.len() + 1 + word.len()
            };

            let overflow = new_length as i32 - max_chars as i32;
            if overflow < min_overflow {
                min_overflow = overflow;
                best_line = i;
            }
        }

        if !lines[best_line].is_empty() {
            lines[best_line].push(' ');
        }
        lines[best_line].push_str(word);
        current_line = (best_line + 1) % target_lines;
    }

    lines.into_iter().filter(|line| !line.is_empty()).collect()
}

/// Calculates a balance score for a set of lines (lower is better).
fn calculate_balance_score(lines: &[String]) -> f32 {
    if lines.is_empty() {
        return 0.0;
    }

    let lengths: Vec<usize> = lines.iter().map(|line| line.len()).collect();
    let avg_length = lengths.iter().sum::<usize>() as f32 / lengths.len() as f32;

    // Calculate variance
    let variance: f32 = lengths
        .iter()
        .map(|&len| {
            let diff = len as f32 - avg_length;
            diff * diff
        })
        .sum::<f32>()
        / lengths.len() as f32;

    variance.sqrt()
}

/// Information about entity dimensions.
#[derive(Debug, Clone)]
struct EntityDimensions {
    width: u32,
    height: u32,
    text_lines: Vec<String>,
}

/// Context for rendering entities.
struct EntityRenderContext<'a> {
    diagram: &'a EventModelDiagram,
    swimlanes: &'a NonEmpty<yaml_types::Swimlane>,
    slices: &'a [yaml_types::Slice],
    slice_widths: &'a [u32],
    swimlane_heights: &'a [u32],
    swimlanes_start_y: u32,
    start_x: u32,
    entity_dimensions_map: &'a HashMap<String, EntityDimensions>,
}

/// Calculate dimensions needed for an entity based on its text content.
fn calculate_entity_dimensions(name: &str, _entity_type: &str) -> EntityDimensions {
    let formatted_name = format_entity_name(name);
    let (text_lines, text_width, text_height) = wrap_text(
        &formatted_name,
        ENTITY_BOX_WIDTH - 2 * ENTITY_PADDING,
        ENTITY_NAME_FONT_SIZE,
    );

    // Account for entity type label and padding
    let label_height = ENTITY_LABEL_FONT_SIZE + ENTITY_PADDING;
    let total_text_height = label_height + text_height + ENTITY_PADDING;

    // Ensure minimum dimensions
    let width = text_width.max(ENTITY_BOX_WIDTH);
    let height = total_text_height.max(ENTITY_BOX_HEIGHT);

    EntityDimensions {
        width,
        height,
        text_lines,
    }
}

/// Renders a single view box with proper text wrapping.
fn render_view_box(x: u32, y: u32, _name: &str, dimensions: &EntityDimensions) -> String {
    let mut svg = String::new();

    // Draw the box
    svg.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{}" height="{}" fill="{VIEW_BACKGROUND_COLOR}" stroke="{SWIMLANE_BORDER_COLOR}" stroke-width="1"/>
"#,
        dimensions.width, dimensions.height
    ));

    // Draw the entity type label "View"
    let label_x = x + dimensions.width / 2;
    let label_y = y + ENTITY_PADDING + ENTITY_LABEL_FONT_SIZE;
    svg.push_str(&format!(
        r#"  <text x="{label_x}" y="{label_y}" font-family="Arial, sans-serif" font-size="{ENTITY_LABEL_FONT_SIZE}" fill="{TEXT_COLOR}" text-anchor="middle">View</text>
"#
    ));

    // Draw the entity name with multiple lines
    let line_height = (ENTITY_NAME_FONT_SIZE as f32 * 1.2) as u32;
    let text_start_y =
        y + ENTITY_PADDING + ENTITY_LABEL_FONT_SIZE + ENTITY_PADDING + ENTITY_NAME_FONT_SIZE;

    for (i, line) in dimensions.text_lines.iter().enumerate() {
        let text_y = text_start_y + (i as u32 * line_height);
        svg.push_str(&format!(
            r#"  <text x="{label_x}" y="{text_y}" font-family="Arial, sans-serif" font-size="{ENTITY_NAME_FONT_SIZE}" fill="{TEXT_COLOR}" text-anchor="middle">{line}</text>
"#
        ));
    }

    svg
}
