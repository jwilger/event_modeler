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
const ENTITY_NAME_FONT_SIZE: u32 = 10; // Font size for entity names

// Entity colors
const VIEW_BACKGROUND_COLOR: &str = "#ffffff"; // White for views
const COMMAND_BACKGROUND_COLOR: &str = "#4a90e2"; // Blue for commands
const EVENT_BACKGROUND_COLOR: &str = "#9b59b6"; // Purple for events
const PROJECTION_BACKGROUND_COLOR: &str = "#f1c40f"; // Yellow for projections
const QUERY_BACKGROUND_COLOR: &str = "#27ae60"; // Green for queries

// Automation entity constants
const ROBOT_ICON_SIZE: u32 = 30; // Size of the robot emoji
const ICON_TEXT_SPACING: u32 = 5; // Space between icon and text

// Arrow rendering constants
const MIN_ARROW_EXTENSION: u32 = 30; // Minimum extension for arrow lead lines

/// Creates a lookup map from view names to their definitions.
fn create_view_lookup(
    views: &HashMap<yaml_types::ViewName, yaml_types::ViewDefinition>,
) -> HashMap<String, &yaml_types::ViewDefinition> {
    views
        .iter()
        .map(|(name, def)| {
            let s = name.clone().into_inner();
            (s.as_str().to_string(), def)
        })
        .collect()
}

/// Creates a lookup map from command names to their definitions.
fn create_command_lookup(
    commands: &HashMap<yaml_types::CommandName, yaml_types::CommandDefinition>,
) -> HashMap<String, &yaml_types::CommandDefinition> {
    commands
        .iter()
        .map(|(name, def)| {
            let s = name.clone().into_inner();
            (s.as_str().to_string(), def)
        })
        .collect()
}

/// Creates a lookup map from event names to their definitions.
fn create_event_lookup(
    events: &HashMap<yaml_types::EventName, yaml_types::EventDefinition>,
) -> HashMap<String, &yaml_types::EventDefinition> {
    events
        .iter()
        .map(|(name, def)| {
            let s = name.clone().into_inner();
            (s.as_str().to_string(), def)
        })
        .collect()
}

/// Creates a lookup map from projection names to their definitions.
fn create_projection_lookup(
    projections: &HashMap<yaml_types::ProjectionName, yaml_types::ProjectionDefinition>,
) -> HashMap<String, &yaml_types::ProjectionDefinition> {
    projections
        .iter()
        .map(|(name, def)| {
            let s = name.clone().into_inner();
            (s.as_str().to_string(), def)
        })
        .collect()
}

/// Creates a lookup map from query names to their definitions.
fn create_query_lookup(
    queries: &HashMap<yaml_types::QueryName, yaml_types::QueryDefinition>,
) -> HashMap<String, &yaml_types::QueryDefinition> {
    queries
        .iter()
        .map(|(name, def)| {
            let s = name.clone().into_inner();
            (s.as_str().to_string(), def)
        })
        .collect()
}

/// Creates a lookup map from automation names to their definitions.
fn create_automation_lookup(
    automations: &HashMap<yaml_types::AutomationName, yaml_types::AutomationDefinition>,
) -> HashMap<String, &yaml_types::AutomationDefinition> {
    automations
        .iter()
        .map(|(name, def)| {
            let s = name.clone().into_inner();
            (s.as_str().to_string(), def)
        })
        .collect()
}

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
    for command_name in diagram.commands().keys() {
        let name_string = command_name.clone().into_inner();
        let name_str = name_string.as_str();
        let dimensions = calculate_entity_dimensions(name_str, "Command");
        entity_dimensions_map.insert(name_str.to_string(), dimensions);
    }
    for event_name in diagram.events().keys() {
        let name_string = event_name.clone().into_inner();
        let name_str = name_string.as_str();
        let dimensions = calculate_entity_dimensions(name_str, "Event");
        entity_dimensions_map.insert(name_str.to_string(), dimensions);
    }
    for projection_name in diagram.projections().keys() {
        let name_string = projection_name.clone().into_inner();
        let name_str = name_string.as_str();
        let dimensions = calculate_entity_dimensions(name_str, "Projection");
        entity_dimensions_map.insert(name_str.to_string(), dimensions);
    }
    for query_name in diagram.queries().keys() {
        let name_string = query_name.clone().into_inner();
        let name_str = name_string.as_str();
        let dimensions = calculate_entity_dimensions(name_str, "Query");
        entity_dimensions_map.insert(name_str.to_string(), dimensions);
    }
    for automation_name in diagram.automations().keys() {
        let name_string = automation_name.clone().into_inner();
        let name_str = name_string.as_str();
        let dimensions = calculate_automation_dimensions(name_str);
        entity_dimensions_map.insert(name_str.to_string(), dimensions);
    }

    // Build temporary maps for entity lookups
    let lookups = EntityLookups {
        view_lookup: create_view_lookup(diagram.views()),
        command_lookup: create_command_lookup(diagram.commands()),
        event_lookup: create_event_lookup(diagram.events()),
        projection_lookup: create_projection_lookup(diagram.projections()),
        query_lookup: create_query_lookup(diagram.queries()),
        automation_lookup: create_automation_lookup(diagram.automations()),
    };

    // Analyze entities in each slice to determine required widths
    let mut slice_required_widths = vec![MIN_SLICE_WIDTH; num_slices];

    // Count entities in each slice and calculate required space
    for (slice_index, slice) in slices.iter().enumerate() {
        let mut entities_by_swimlane: HashMap<&yaml_types::SwimlaneId, Vec<String>> =
            HashMap::new();

        for connection in slice.connections.iter() {
            // Check both sides of connections for views and commands
            process_entity_for_slice(&connection.from, &lookups, &mut entities_by_swimlane);
            process_entity_for_slice(&connection.to, &lookups, &mut entities_by_swimlane);
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

    // Check views and commands in each swimlane to determine heights
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

    for (command_name, command_def) in diagram.commands() {
        if let Some(swimlane_index) = swimlanes.iter().position(|s| s.id == command_def.swimlane) {
            let name_string = command_name.clone().into_inner();
            let name_str = name_string.as_str();
            if let Some(dimensions) = entity_dimensions_map.get(name_str) {
                // Account for entity height plus margins
                swimlane_content_heights[swimlane_index] = swimlane_content_heights[swimlane_index]
                    .max(dimensions.height + 2 * ENTITY_MARGIN);
            }
        }
    }

    for (event_name, event_def) in diagram.events() {
        if let Some(swimlane_index) = swimlanes.iter().position(|s| s.id == event_def.swimlane) {
            let name_string = event_name.clone().into_inner();
            let name_str = name_string.as_str();
            if let Some(dimensions) = entity_dimensions_map.get(name_str) {
                // Account for entity height plus margins
                swimlane_content_heights[swimlane_index] = swimlane_content_heights[swimlane_index]
                    .max(dimensions.height + 2 * ENTITY_MARGIN);
            }
        }
    }

    for (projection_name, projection_def) in diagram.projections() {
        if let Some(swimlane_index) = swimlanes
            .iter()
            .position(|s| s.id == projection_def.swimlane)
        {
            let name_string = projection_name.clone().into_inner();
            let name_str = name_string.as_str();
            if let Some(dimensions) = entity_dimensions_map.get(name_str) {
                // Account for entity height plus margins
                swimlane_content_heights[swimlane_index] = swimlane_content_heights[swimlane_index]
                    .max(dimensions.height + 2 * ENTITY_MARGIN);
            }
        }
    }

    for (query_name, query_def) in diagram.queries() {
        if let Some(swimlane_index) = swimlanes.iter().position(|s| s.id == query_def.swimlane) {
            let name_string = query_name.clone().into_inner();
            let name_str = name_string.as_str();
            if let Some(dimensions) = entity_dimensions_map.get(name_str) {
                // Account for entity height plus margins
                swimlane_content_heights[swimlane_index] = swimlane_content_heights[swimlane_index]
                    .max(dimensions.height + 2 * ENTITY_MARGIN);
            }
        }
    }

    for (automation_name, automation_def) in diagram.automations() {
        if let Some(swimlane_index) = swimlanes
            .iter()
            .position(|s| s.id == automation_def.swimlane)
        {
            let name_string = automation_name.clone().into_inner();
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
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">
  <!-- Arrow marker definition -->
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
      <polygon points="0 0, 10 3.5, 0 7" fill="#333333" />
    </marker>
  </defs>
  
  <!-- Canvas background -->
  <rect x="0" y="0" width="{}" height="{}" fill="{}" stroke="none"/>
  
  <!-- Workflow title -->
  <text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" font-weight="normal" fill="{}">
    {}
  </text>
"##,
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
    let (entities_svg, entity_positions) = render_entities(&render_ctx);
    svg_content.push_str(&entities_svg);

    // Render connections (arrows between entities)
    svg_content.push_str(&render_connections(
        slices,
        &entity_positions,
        &entity_dimensions_map,
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

/// Extract entity name and swimlane from an entity reference.
fn extract_entity_info<'a>(
    entity_ref: &yaml_types::EntityReference,
    lookups: &EntityLookups<'a>,
) -> Option<(String, &'a yaml_types::SwimlaneId)> {
    match entity_ref {
        yaml_types::EntityReference::View(view_path) => {
            let view_name_string = view_path.clone().into_inner();
            let view_name_str = view_name_string.as_str();
            let base_view_name = view_name_str.split('.').next().unwrap_or(view_name_str);

            lookups
                .view_lookup
                .get(base_view_name)
                .map(|view_def| (base_view_name.to_string(), &view_def.swimlane))
        }
        yaml_types::EntityReference::Command(command_name) => {
            let command_name_string = command_name.clone().into_inner();
            let command_name_str = command_name_string.as_str();

            lookups
                .command_lookup
                .get(command_name_str)
                .map(|command_def| (command_name_str.to_string(), &command_def.swimlane))
        }
        yaml_types::EntityReference::Event(event_name) => {
            let event_name_string = event_name.clone().into_inner();
            let event_name_str = event_name_string.as_str();

            lookups
                .event_lookup
                .get(event_name_str)
                .map(|event_def| (event_name_str.to_string(), &event_def.swimlane))
        }
        yaml_types::EntityReference::Projection(projection_name) => {
            let projection_name_string = projection_name.clone().into_inner();
            let projection_name_str = projection_name_string.as_str();

            lookups
                .projection_lookup
                .get(projection_name_str)
                .map(|projection_def| (projection_name_str.to_string(), &projection_def.swimlane))
        }
        yaml_types::EntityReference::Query(query_name) => {
            let query_name_string = query_name.clone().into_inner();
            let query_name_str = query_name_string.as_str();

            lookups
                .query_lookup
                .get(query_name_str)
                .map(|query_def| (query_name_str.to_string(), &query_def.swimlane))
        }
        yaml_types::EntityReference::Automation(automation_name) => {
            let automation_name_string = automation_name.clone().into_inner();
            let automation_name_str = automation_name_string.as_str();

            lookups
                .automation_lookup
                .get(automation_name_str)
                .map(|automation_def| (automation_name_str.to_string(), &automation_def.swimlane))
        }
    }
}

/// Process an entity reference for slice width calculation.
fn process_entity_for_slice<'a>(
    entity_ref: &yaml_types::EntityReference,
    lookups: &EntityLookups<'a>,
    entities_by_swimlane: &mut HashMap<&'a yaml_types::SwimlaneId, Vec<String>>,
) {
    if let Some((entity_name, swimlane_id)) = extract_entity_info(entity_ref, lookups) {
        entities_by_swimlane
            .entry(swimlane_id)
            .or_default()
            .push(entity_name);
    }
}

/// Process an entity reference and add it to the entities_by_slice_and_swimlane map if it's a view, command, event, projection, or query.
fn process_entity_reference<'a>(
    entity_ref: &yaml_types::EntityReference,
    slice_index: usize,
    lookups: &EntityLookups<'a>,
    entities_by_slice_and_swimlane: &mut HashMap<(usize, &'a yaml_types::SwimlaneId), Vec<String>>,
) {
    if let Some((entity_name, swimlane_id)) = extract_entity_info(entity_ref, lookups) {
        let key = (slice_index, swimlane_id);
        entities_by_slice_and_swimlane
            .entry(key)
            .or_default()
            .push(entity_name);
    }
}

/// Renders all entities (views, commands, events, etc.) in their respective positions.
/// Returns the SVG string and a map of entity names to their positions.
fn render_entities(ctx: &EntityRenderContext) -> (String, HashMap<String, EntityPosition>) {
    let mut svg = String::new();
    let mut entity_positions = HashMap::new();

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

    // Render views and commands in their slices
    // First, we need to find which entities appear in which slices
    let mut entities_by_slice_and_swimlane: HashMap<(usize, &yaml_types::SwimlaneId), Vec<String>> =
        HashMap::new();

    // Build lookup maps from entity names to definitions for performance
    let lookups = EntityLookups {
        view_lookup: create_view_lookup(ctx.diagram.views()),
        command_lookup: create_command_lookup(ctx.diagram.commands()),
        event_lookup: create_event_lookup(ctx.diagram.events()),
        projection_lookup: create_projection_lookup(ctx.diagram.projections()),
        query_lookup: create_query_lookup(ctx.diagram.queries()),
        automation_lookup: create_automation_lookup(ctx.diagram.automations()),
    };

    // Parse slice connections to find view positions
    for (slice_index, slice) in ctx.slices.iter().enumerate() {
        for connection in slice.connections.iter() {
            // Process both sides of the connection
            process_entity_reference(
                &connection.from,
                slice_index,
                &lookups,
                &mut entities_by_slice_and_swimlane,
            );
            process_entity_reference(
                &connection.to,
                slice_index,
                &lookups,
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

                // Store entity position with slice index to handle multiple instances
                let position_key = format!("{}_{}", entity_name, slice_index);
                entity_positions.insert(
                    position_key,
                    EntityPosition {
                        x: entity_x,
                        y: entity_y,
                        width: dimensions.width,
                        height: dimensions.height,
                        slice_index: *slice_index,
                    },
                );

                // Determine entity type and render appropriate box
                if lookups.view_lookup.contains_key(entity_name) {
                    svg.push_str(&render_view_box(entity_x, entity_y, dimensions));
                } else if lookups.command_lookup.contains_key(entity_name) {
                    svg.push_str(&render_command_box(entity_x, entity_y, dimensions));
                } else if lookups.event_lookup.contains_key(entity_name) {
                    svg.push_str(&render_event_box(entity_x, entity_y, dimensions));
                } else if lookups.projection_lookup.contains_key(entity_name) {
                    svg.push_str(&render_projection_box(entity_x, entity_y, dimensions));
                } else if lookups.query_lookup.contains_key(entity_name) {
                    svg.push_str(&render_query_box(entity_x, entity_y, dimensions));
                } else if lookups.automation_lookup.contains_key(entity_name) {
                    svg.push_str(&render_automation(entity_x, entity_y, dimensions));
                }
            }
        }
    }

    (svg, entity_positions)
}

/// Renders connection arrows between entities based on slice definitions.
fn render_connections(
    slices: &[yaml_types::Slice],
    entity_positions: &HashMap<String, EntityPosition>,
    _entity_dimensions_map: &HashMap<String, EntityDimensions>,
) -> String {
    let mut svg = String::new();

    svg.push_str("  <!-- Connections -->\n");

    // Create the orthogonal router with better spacing configuration
    // TODO: Routing implementation will be replaced with libavoid integration

    // Process connections from each slice
    for (slice_index, slice) in slices.iter().enumerate() {
        for connection in slice.connections.iter() {
            // Extract entity names from references
            let from_name = extract_entity_name(&connection.from);
            let to_name = extract_entity_name(&connection.to);

            // Find the correct entity instances
            let from_pos = find_entity_position(&from_name, slice_index, entity_positions);
            let to_pos = find_entity_position(&to_name, slice_index, entity_positions);

            if let (Some(from_pos), Some(to_pos)) = (from_pos, to_pos) {
                // Use simple straight arrow for now (until libavoid integration)
                svg.push_str(&render_straight_arrow(from_pos, to_pos));
            }
        }
    }

    svg
}

/// Finds the position of an entity, preferring instances in the current or nearby slices.
fn find_entity_position<'a>(
    entity_name: &str,
    current_slice: usize,
    entity_positions: &'a HashMap<String, EntityPosition>,
) -> Option<&'a EntityPosition> {
    // First, try to find in current slice
    let current_key = format!("{}_{}", entity_name, current_slice);
    if let Some(pos) = entity_positions.get(&current_key) {
        return Some(pos);
    }

    // If not in current slice, find the closest instance
    let mut closest_pos: Option<&EntityPosition> = None;
    let mut closest_distance = usize::MAX;
    let prefix = format!("{}_", entity_name);

    for (key, pos) in entity_positions {
        if key.starts_with(&prefix) {
            let distance = if pos.slice_index > current_slice {
                pos.slice_index - current_slice
            } else {
                current_slice - pos.slice_index
            };

            if distance < closest_distance {
                closest_distance = distance;
                closest_pos = Some(pos);
            }
        }
    }

    closest_pos
}

/// Extracts the base entity name from an EntityReference.
fn extract_entity_name(entity_ref: &yaml_types::EntityReference) -> String {
    match entity_ref {
        yaml_types::EntityReference::View(view_path) => {
            let path_string = view_path.clone().into_inner();
            let path_str = path_string.as_str();
            path_str.split('.').next().unwrap_or(path_str).to_string()
        }
        yaml_types::EntityReference::Command(command_name) => {
            command_name.clone().into_inner().as_str().to_string()
        }
        yaml_types::EntityReference::Event(event_name) => {
            event_name.clone().into_inner().as_str().to_string()
        }
        yaml_types::EntityReference::Projection(projection_name) => {
            projection_name.clone().into_inner().as_str().to_string()
        }
        yaml_types::EntityReference::Query(query_name) => {
            query_name.clone().into_inner().as_str().to_string()
        }
        yaml_types::EntityReference::Automation(automation_name) => {
            automation_name.clone().into_inner().as_str().to_string()
        }
    }
}

/// Renders a straight arrow between two entities.
fn render_straight_arrow(from: &EntityPosition, to: &EntityPosition) -> String {
    let (from_x, from_y) = calculate_connection_point(from, to, true);
    let (to_x, to_y) = calculate_connection_point(to, from, false);

    // Add minimum lead line extensions for proper spacing
    let min_extension = MIN_ARROW_EXTENSION; // Match the routing system's minimum extension

    // Calculate extended start and end points
    let (extended_from_x, extended_from_y) =
        extend_connection_point(from_x, from_y, from, to, min_extension, true);
    let (extended_to_x, extended_to_y) =
        extend_connection_point(to_x, to_y, to, from, min_extension, false);

    // Create an orthogonal path with proper extensions
    render_orthogonal_fallback(
        extended_from_x,
        extended_from_y,
        extended_to_x,
        extended_to_y,
    )
}

/// Extends a connection point away from an entity by the specified distance.
fn extend_connection_point(
    x: u32,
    y: u32,
    entity: &EntityPosition,
    _other: &EntityPosition,
    extension: u32,
    _is_source: bool,
) -> (u32, u32) {
    // Determine which edge this connection point is on
    let on_left = x == entity.x;
    let on_right = x == entity.x + entity.width;
    let on_top = y == entity.y;
    let on_bottom = y == entity.y + entity.height;

    // Extend away from the entity
    if on_left {
        // Left edge - extend leftward
        (x.saturating_sub(extension), y)
    } else if on_right {
        // Right edge - extend rightward
        (x + extension, y)
    } else if on_top {
        // Top edge - extend upward
        (x, y.saturating_sub(extension))
    } else if on_bottom {
        // Bottom edge - extend downward
        (x, y + extension)
    } else {
        // Fallback - no extension
        (x, y)
    }
}

/// Creates a simple orthogonal path between two points as a fallback.
fn render_orthogonal_fallback(from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> String {
    // Create a simple L-shaped or Z-shaped path
    let mut path = format!("M {} {}", from_x, from_y);

    // If points are already aligned, draw a straight line
    if from_x == to_x || from_y == to_y {
        path.push_str(&format!(" L {} {}", to_x, to_y));
    } else {
        // Create an L-shaped path
        // Go horizontally first, then vertically
        let mid_x = if from_x < to_x {
            from_x + (to_x - from_x) / 2
        } else {
            to_x + (from_x - to_x) / 2
        };
        path.push_str(&format!(" L {} {}", mid_x, from_y));
        path.push_str(&format!(" L {} {}", mid_x, to_y));
        path.push_str(&format!(" L {} {}", to_x, to_y));
    }

    format!(
        r##"  <path d="{}" fill="none" stroke="#333333" stroke-width="2" marker-end="url(#arrowhead)" />
"##,
        path
    )
}

/// Renders a curved arrow using bezier curves.
#[allow(dead_code)]
fn render_curved_arrow(
    from: &EntityPosition,
    to: &EntityPosition,
    entity_positions: &HashMap<String, EntityPosition>,
) -> String {
    let (from_x, from_y) = calculate_connection_point(from, to, true);
    let (to_x, to_y) = calculate_connection_point(to, from, false);

    // Calculate centers for better curve control
    let _from_center_x = from.x + from.width / 2;
    let from_center_y = from.y + from.height / 2;
    let to_center_x = to.x + to.width / 2;
    let to_center_y = to.y + to.height / 2;

    // Calculate control points for bezier curve
    let dx = to_x as i32 - from_x as i32;
    let dy = to_y as i32 - from_y as i32;

    // Check if we need to avoid any entities
    let entities_to_avoid =
        find_entities_in_path(from_x, from_y, to_x, to_y, from, to, entity_positions);

    // Determine curve control points based on relative positions and obstacles
    let (cx1, cy1, cx2, cy2) = if !entities_to_avoid.is_empty() {
        // Complex routing to avoid entities
        calculate_avoidance_curve(from_x, from_y, to_x, to_y, from, to, &entities_to_avoid)
    } else if from.slice_index < to.slice_index {
        // Moving right across slices
        if (from_center_y as i32 - to_center_y as i32).abs() < (from.height / 3) as i32 {
            // Same row - gentle horizontal curve
            let _curve_offset = 30.min(dx.unsigned_abs() / 4);
            (
                from_x + dx.unsigned_abs() / 3,
                from_y,
                to_x - dx.unsigned_abs() / 3,
                to_y,
            )
        } else {
            // Diagonal movement
            (from_x + dx.unsigned_abs() / 2, from_y, to_x - 20, to_y)
        }
    } else if from.slice_index == to.slice_index {
        // Same slice
        if dx.abs() < 20 {
            // Vertical in same column - curve out to avoid overlap
            let curve_width = 60;
            if dy > 0 {
                (
                    from_x + curve_width,
                    from_y + 20,
                    to_x + curve_width,
                    to_y - 20,
                )
            } else {
                (
                    from_x - curve_width,
                    from_y - 20,
                    to_x - curve_width,
                    to_y + 20,
                )
            }
        } else {
            // Within same slice - create appropriate curve
            if dy.abs() > dx.abs() {
                // More vertical than horizontal
                let offset = 40.min(dx.unsigned_abs() / 2 + 20);
                if from_x < to_x {
                    // Going right and down/up
                    (from_x + offset, from_y, to_x - offset, to_y)
                } else {
                    // Going left and down/up
                    (from_x - offset, from_y, to_x + offset, to_y)
                }
            } else {
                // More horizontal - gentle curve
                (
                    from_x + dx.unsigned_abs() / 3,
                    from_y,
                    to_x - dx.unsigned_abs() / 3,
                    to_y,
                )
            }
        }
    } else {
        // Moving left (back to previous slice)
        let _mid_x = (from_x + to_x) / 2;
        let mid_y = (from_y + to_y) / 2;
        (from_x - 40, mid_y, to_x + 40, mid_y)
    };

    // Adjust the last control point to ensure arrow points at center
    // Calculate where the arrow should be pointing
    let target_angle_x = to_center_x as i32 - cx2 as i32;
    let target_angle_y = to_center_y as i32 - cy2 as i32;

    // Adjust control point to create proper approach angle
    let adjusted_cx2 = if target_angle_x.abs() > 10 {
        // Need to adjust horizontal approach
        if (to_x as i32 - to_center_x as i32).abs() < 5 {
            // Entering from top/bottom, adjust to point at center
            to_center_x
        } else {
            cx2
        }
    } else {
        cx2
    };

    let adjusted_cy2 = if target_angle_y.abs() > 10 {
        // Need to adjust vertical approach
        if (to_y as i32 - to_center_y as i32).abs() < 5 {
            // Entering from left/right, adjust to point at center
            to_center_y
        } else {
            cy2
        }
    } else {
        cy2
    };

    format!(
        r##"  <path d="M {} {} C {} {}, {} {}, {} {}" stroke="#333333" stroke-width="2" fill="none" marker-end="url(#arrowhead)" />
"##,
        from_x, from_y, cx1, cy1, adjusted_cx2, adjusted_cy2, to_x, to_y
    )
}

/// Find entities that are in the path between two points
#[allow(dead_code)]
fn find_entities_in_path<'a>(
    from_x: u32,
    from_y: u32,
    to_x: u32,
    to_y: u32,
    from_entity: &EntityPosition,
    to_entity: &EntityPosition,
    entity_positions: &'a HashMap<String, EntityPosition>,
) -> Vec<&'a EntityPosition> {
    let mut obstacles = Vec::new();

    // Create a bounding box for the path
    let min_x = from_x.min(to_x);
    let max_x = from_x.max(to_x);
    let min_y = from_y.min(to_y);
    let max_y = from_y.max(to_y);

    for pos in entity_positions.values() {
        // Skip source and target
        if (pos.x == from_entity.x && pos.y == from_entity.y)
            || (pos.x == to_entity.x && pos.y == to_entity.y)
        {
            continue;
        }

        // Check if entity overlaps with path bounding box
        if pos.x + pos.width >= min_x
            && pos.x <= max_x
            && pos.y + pos.height >= min_y
            && pos.y <= max_y
        {
            // More precise check if entity is actually in the path
            if is_entity_in_line_path(from_x, from_y, to_x, to_y, pos) {
                obstacles.push(pos);
            }
        }
    }

    obstacles
}

/// Check if an entity intersects with a line path
#[allow(dead_code)]
fn is_entity_in_line_path(x1: u32, y1: u32, x2: u32, y2: u32, entity: &EntityPosition) -> bool {
    // Simple rectangle-line intersection check
    let entity_left = entity.x;
    let entity_right = entity.x + entity.width;
    let entity_top = entity.y;
    let entity_bottom = entity.y + entity.height;

    let dx = x2 as i32 - x1 as i32;
    let dy = y2 as i32 - y1 as i32;

    // Check for zero-length line
    if dx == 0 && dy == 0 {
        return false;
    }

    // If line is mostly horizontal
    if dx.abs() > dy.abs() && dx != 0 {
        let y_at_left = y1 as i32 + ((entity_left as i32 - x1 as i32) * dy) / dx;
        let y_at_right = y1 as i32 + ((entity_right as i32 - x1 as i32) * dy) / dx;

        (y_at_left >= entity_top as i32 && y_at_left <= entity_bottom as i32)
            || (y_at_right >= entity_top as i32 && y_at_right <= entity_bottom as i32)
    } else if dy != 0 {
        // Line is mostly vertical
        let x_at_top = x1 as i32 + ((entity_top as i32 - y1 as i32) * dx) / dy;
        let x_at_bottom = x1 as i32 + ((entity_bottom as i32 - y1 as i32) * dx) / dy;

        (x_at_top >= entity_left as i32 && x_at_top <= entity_right as i32)
            || (x_at_bottom >= entity_left as i32 && x_at_bottom <= entity_right as i32)
    } else {
        false
    }
}

/// Calculate curve control points to avoid obstacles
#[allow(dead_code)]
fn calculate_avoidance_curve(
    from_x: u32,
    from_y: u32,
    to_x: u32,
    to_y: u32,
    _from_entity: &EntityPosition,
    _to_entity: &EntityPosition,
    obstacles: &[&EntityPosition],
) -> (u32, u32, u32, u32) {
    // Simple avoidance: route around the side with more space
    let dx = to_x as i32 - from_x as i32;
    let dy = to_y as i32 - from_y as i32;

    // Find the main obstacle (closest to the midpoint)
    let mid_x = (from_x + to_x) / 2;
    let mid_y = (from_y + to_y) / 2;

    let main_obstacle = obstacles.iter().min_by_key(|obs| {
        let obs_center_x = obs.x + obs.width / 2;
        let obs_center_y = obs.y + obs.height / 2;
        ((obs_center_x as i32 - mid_x as i32).pow(2) + (obs_center_y as i32 - mid_y as i32).pow(2))
            as u32
    });

    if let Some(obstacle) = main_obstacle {
        // Route around the obstacle
        let obs_center_y = obstacle.y + obstacle.height / 2;

        if dx.abs() > dy.abs() {
            // Mostly horizontal - route above or below
            if mid_y < obs_center_y {
                // Route above
                let detour_y = obstacle.y.saturating_sub(30);
                (
                    from_x + dx.unsigned_abs() / 3,
                    detour_y,
                    to_x - dx.unsigned_abs() / 3,
                    detour_y,
                )
            } else {
                // Route below
                let detour_y = obstacle.y + obstacle.height + 30;
                (
                    from_x + dx.unsigned_abs() / 3,
                    detour_y,
                    to_x - dx.unsigned_abs() / 3,
                    detour_y,
                )
            }
        } else {
            // Mostly vertical - route left or right
            let obs_center_x = obstacle.x + obstacle.width / 2;
            if mid_x < obs_center_x {
                // Route left
                let detour_x = obstacle.x.saturating_sub(30);
                (
                    detour_x,
                    from_y + dy.unsigned_abs() / 3,
                    detour_x,
                    to_y - dy.unsigned_abs() / 3,
                )
            } else {
                // Route right
                let detour_x = obstacle.x + obstacle.width + 30;
                (
                    detour_x,
                    from_y + dy.unsigned_abs() / 3,
                    detour_x,
                    to_y - dy.unsigned_abs() / 3,
                )
            }
        }
    } else {
        // Fallback to simple curve
        (
            from_x + dx.unsigned_abs() / 3,
            from_y,
            to_x - dx.unsigned_abs() / 3,
            to_y,
        )
    }
}

/// Calculates the connection point on an entity's edge.
fn calculate_connection_point(
    entity: &EntityPosition,
    other: &EntityPosition,
    is_source: bool,
) -> (u32, u32) {
    let entity_center_x = entity.x + entity.width / 2;
    let entity_center_y = entity.y + entity.height / 2;
    let other_center_x = other.x + other.width / 2;
    let other_center_y = other.y + other.height / 2;

    // Calculate angle from entity center to other center
    let dx = other_center_x as i32 - entity_center_x as i32;
    let dy = other_center_y as i32 - entity_center_y as i32;

    // Determine primary direction based on angle
    let abs_dx = dx.abs();
    let abs_dy = dy.abs();

    if is_source {
        // For source, exit toward target
        if abs_dx > abs_dy {
            // Primarily horizontal
            if dx > 0 {
                // Exit right
                (entity.x + entity.width, entity_center_y)
            } else {
                // Exit left
                (entity.x, entity_center_y)
            }
        } else {
            // Primarily vertical
            if dy > 0 {
                // Exit bottom
                (entity_center_x, entity.y + entity.height)
            } else {
                // Exit top
                (entity_center_x, entity.y)
            }
        }
    } else {
        // For target, enter from direction of source
        if abs_dx > abs_dy {
            // Primarily horizontal
            if dx > 0 {
                // Enter from left
                (entity.x, entity_center_y)
            } else {
                // Enter from right
                (entity.x + entity.width, entity_center_y)
            }
        } else {
            // Primarily vertical
            if dy > 0 {
                // Enter from top
                (entity_center_x, entity.y)
            } else {
                // Enter from bottom
                (entity_center_x, entity.y + entity.height)
            }
        }
    }
}

/// Renders a routed path as an SVG path element with an arrowhead.
#[allow(dead_code)] // Will be used once libavoid is integrated
fn render_routed_path(route: &super::routing_types::RoutePath) -> String {
    let svg_path = route.to_svg_path();
    format!(
        r##"  <path d="{}" fill="none" stroke="#333333" stroke-width="2" marker-end="url(#arrowhead)" />
"##,
        svg_path
    )
}

// TODO: Debug function removed - will be replaced with libavoid debug info

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

/// Wraps text into balanced lines, prioritizing wrapping over width expansion.
/// Returns the wrapped lines and the actual dimensions needed.
fn wrap_text(text: &str, max_width: u32, font_size: u32) -> (Vec<String>, u32, u32) {
    // Approximate character width (for Arial font, roughly 0.6x the font size)
    let char_width = (font_size as f32 * 0.6) as u32;
    let max_chars_per_line = max_width / char_width;

    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return (vec![text.to_string()], max_width, font_size);
    }

    // First, try to fit within the max width using multiple lines
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in &words {
        // Check if adding this word would exceed the line length
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{current_line} {word}")
        };

        if test_line.len() <= max_chars_per_line as usize {
            current_line = test_line;
        } else {
            // Start a new line
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // If we have lines that fit, use the standard width
    let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    let mut actual_width = ENTITY_BOX_WIDTH;

    // Only expand width if a single word is longer than the max characters
    if max_line_length > max_chars_per_line as usize {
        actual_width = (max_line_length as u32 * char_width).max(ENTITY_BOX_WIDTH);
    }

    let line_height = (font_size as f32 * 1.2) as u32;
    let actual_height = lines.len() as u32 * line_height;

    (lines, actual_width, actual_height)
}

/// Information about entity dimensions.
#[derive(Debug, Clone)]
struct EntityDimensions {
    width: u32,
    height: u32,
    text_lines: Vec<String>,
}

/// Entity lookup maps for avoiding too many function parameters.
struct EntityLookups<'a> {
    view_lookup: HashMap<String, &'a yaml_types::ViewDefinition>,
    command_lookup: HashMap<String, &'a yaml_types::CommandDefinition>,
    event_lookup: HashMap<String, &'a yaml_types::EventDefinition>,
    projection_lookup: HashMap<String, &'a yaml_types::ProjectionDefinition>,
    query_lookup: HashMap<String, &'a yaml_types::QueryDefinition>,
    automation_lookup: HashMap<String, &'a yaml_types::AutomationDefinition>,
}

/// Position information for a rendered entity.
#[derive(Debug, Clone)]
struct EntityPosition {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    slice_index: usize,
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

    // Only use padding for height calculation (no label)
    let total_text_height = text_height + 2 * ENTITY_PADDING;

    // Prefer the standard width unless text forces us wider
    let width = text_width.max(ENTITY_BOX_WIDTH);
    let height = total_text_height.max(ENTITY_BOX_HEIGHT);

    EntityDimensions {
        width,
        height,
        text_lines,
    }
}

/// Renders a box with text, using the specified colors.
fn render_box_with_text(
    x: u32,
    y: u32,
    dimensions: &EntityDimensions,
    background_color: &str,
    text_color: &str,
) -> String {
    let mut svg = String::new();

    // Draw the box
    svg.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{}" height="{}" fill="{background_color}" stroke="{SWIMLANE_BORDER_COLOR}" stroke-width="1"/>
"#,
        dimensions.width, dimensions.height
    ));

    // Draw the entity name with multiple lines
    let line_height = (ENTITY_NAME_FONT_SIZE as f32 * 1.2) as u32;
    let text_center_x = x + dimensions.width / 2;

    // Center the text vertically in the box
    let total_text_height = dimensions.text_lines.len() as u32 * line_height;
    let text_start_y = y + (dimensions.height - total_text_height) / 2 + ENTITY_NAME_FONT_SIZE;

    for (i, line) in dimensions.text_lines.iter().enumerate() {
        let text_y = text_start_y + (i as u32 * line_height);
        svg.push_str(&format!(
            r#"  <text x="{text_center_x}" y="{text_y}" font-family="Arial, sans-serif" font-size="{ENTITY_NAME_FONT_SIZE}" fill="{text_color}" text-anchor="middle">{line}</text>
"#
        ));
    }

    svg
}

/// Renders a single view box with proper text wrapping.
fn render_view_box(x: u32, y: u32, dimensions: &EntityDimensions) -> String {
    render_box_with_text(x, y, dimensions, VIEW_BACKGROUND_COLOR, TEXT_COLOR)
}

/// Renders a single command box with proper text wrapping.
fn render_command_box(x: u32, y: u32, dimensions: &EntityDimensions) -> String {
    render_box_with_text(x, y, dimensions, COMMAND_BACKGROUND_COLOR, "#ffffff")
}

/// Renders a single event box with proper text wrapping.
fn render_event_box(x: u32, y: u32, dimensions: &EntityDimensions) -> String {
    render_box_with_text(x, y, dimensions, EVENT_BACKGROUND_COLOR, "#ffffff")
}

/// Renders a single projection box with proper text wrapping.
fn render_projection_box(x: u32, y: u32, dimensions: &EntityDimensions) -> String {
    render_box_with_text(x, y, dimensions, PROJECTION_BACKGROUND_COLOR, TEXT_COLOR)
}

/// Renders a single query box with proper text wrapping.
fn render_query_box(x: u32, y: u32, dimensions: &EntityDimensions) -> String {
    render_box_with_text(x, y, dimensions, QUERY_BACKGROUND_COLOR, "#ffffff")
}

/// Calculate dimensions for automation entities (robot icon + text below).
fn calculate_automation_dimensions(name: &str) -> EntityDimensions {
    let formatted_name = format_entity_name(name);
    let (text_lines, text_width, text_height) = wrap_text(
        &formatted_name,
        ENTITY_BOX_WIDTH - 2 * ENTITY_PADDING,
        ENTITY_NAME_FONT_SIZE,
    );

    // Width is the max of icon size or text width
    let width = ROBOT_ICON_SIZE.max(text_width) + 2 * ENTITY_PADDING;
    // Height is icon + spacing + text + padding
    let height = ROBOT_ICON_SIZE + ICON_TEXT_SPACING + text_height + 2 * ENTITY_PADDING;

    EntityDimensions {
        width,
        height,
        text_lines,
    }
}

/// Renders an automation entity with robot icon and text below.
fn render_automation(x: u32, y: u32, dimensions: &EntityDimensions) -> String {
    let mut svg = String::new();

    // Center the robot icon horizontally
    let icon_x = x + dimensions.width / 2;
    let icon_y = y + ENTITY_PADDING + 15; // 15 is half the icon size for vertical centering

    // Render automation icon (gear emoji for a friendlier appearance)
    svg.push_str(&format!(
        r#"  <text x="{icon_x}" y="{icon_y}" font-family="Arial, sans-serif" font-size="30" text-anchor="middle">⚙️</text>
"#
    ));

    // Render automation name below the icon
    let text_start_y =
        y + ENTITY_PADDING + ROBOT_ICON_SIZE + ICON_TEXT_SPACING + ENTITY_NAME_FONT_SIZE;
    let text_center_x = x + dimensions.width / 2;

    let line_height = (ENTITY_NAME_FONT_SIZE as f32 * 1.2) as u32;
    for (i, line) in dimensions.text_lines.iter().enumerate() {
        let text_y = text_start_y + (i as u32 * line_height);
        svg.push_str(&format!(
            r#"  <text x="{text_center_x}" y="{text_y}" font-family="Arial, sans-serif" font-size="{ENTITY_NAME_FONT_SIZE}" fill="{TEXT_COLOR}" text-anchor="middle">{line}</text>
"#
        ));
    }

    svg
}
