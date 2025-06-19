// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Layout engine for positioning entities in Event Model diagrams.
//!
//! This module handles the computation of positions and dimensions
//! for all visual elements in a diagram, including swimlanes, entities,
//! slices, and connections between entities.

use crate::event_model::diagram::{SliceId, SwimlaneId};
use crate::event_model::entities::EntityId;
use crate::infrastructure::types::{NonNegativeFloat, PositiveFloat, PositiveInt};
use nutype::nutype;
use std::collections::HashMap;

/// Complete layout information for a diagram.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Overall canvas dimensions and settings.
    pub canvas: Canvas,
    /// Layout information for each swimlane.
    pub swimlane_layouts: HashMap<SwimlaneId, SwimlaneLayout>,
    /// Position of each entity within its swimlane.
    pub entity_positions: HashMap<EntityId, EntityPosition>,
    /// Layout information for each slice.
    pub slice_layouts: HashMap<SliceId, SliceLayout>,
    /// Visual connections between entities.
    pub connections: Vec<Connection>,
    /// Layout information for test scenarios.
    pub test_scenario_layouts: Vec<TestScenarioLayout>,
}

/// Canvas dimensions and settings.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// Total width of the canvas.
    pub width: CanvasWidth,
    /// Total height of the canvas.
    pub height: CanvasHeight,
    /// Padding around the content.
    pub padding: Padding,
}

/// Layout information for a swimlane.
#[derive(Debug, Clone)]
pub struct SwimlaneLayout {
    /// Top-left position of the swimlane.
    pub position: Position,
    /// Width and height of the swimlane.
    pub dimensions: Dimensions,
    /// Display name of the swimlane.
    pub name: crate::infrastructure::types::NonEmptyString,
}

/// Position and size of an entity.
#[derive(Debug, Clone)]
pub struct EntityPosition {
    /// Swimlane containing this entity.
    pub swimlane_id: SwimlaneId,
    /// Position within the swimlane.
    pub position: Position,
    /// Size of the entity box.
    pub dimensions: Dimensions,
    /// Type of the entity.
    pub entity_type: crate::event_model::entities::EntityType,
    /// Name of the entity.
    pub entity_name: crate::infrastructure::types::NonEmptyString,
}

/// Layout information for a vertical slice.
#[derive(Debug, Clone)]
pub struct SliceLayout {
    /// Horizontal position of the slice.
    pub x_position: XCoordinate,
    /// Width of the slice.
    pub width: Width,
}

/// Layout information for test scenarios.
#[derive(Debug, Clone)]
pub struct TestScenarioLayout {
    /// Position of the test scenario box.
    pub position: Position,
    /// Dimensions of the test scenario box.
    pub dimensions: Dimensions,
    /// Name of the test scenario.
    pub scenario_name: crate::infrastructure::types::NonEmptyString,
    /// Command this test scenario belongs to.
    pub parent_command: EntityId,
}

/// Visual connection between two entities.
#[derive(Debug, Clone)]
pub struct Connection {
    /// Source entity.
    pub from: EntityId,
    /// Target entity.
    pub to: EntityId,
    /// Path to draw for the connection.
    pub path: ConnectionPath,
    /// Visual style for the connection.
    pub style: ConnectionStyle,
}

/// Path for drawing a connection.
#[derive(Debug, Clone)]
pub struct ConnectionPath {
    /// Points defining the path.
    pub points: Vec<Point>,
}

/// Visual style for connections.
#[derive(Debug, Clone)]
pub enum ConnectionStyle {
    /// Solid line.
    Solid,
    /// Dashed line.
    Dashed,
    /// Dotted line.
    Dotted,
}

/// 2D position in the diagram.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Horizontal coordinate.
    pub x: XCoordinate,
    /// Vertical coordinate.
    pub y: YCoordinate,
}

/// Width and height dimensions.
#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    /// Horizontal size.
    pub width: Width,
    /// Vertical size.
    pub height: Height,
}

/// A point in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// X coordinate.
    pub x: XCoordinate,
    /// Y coordinate.
    pub y: YCoordinate,
}

/// Padding values for all four sides.
#[derive(Debug, Clone, Copy)]
pub struct Padding {
    /// Top padding.
    pub top: PaddingValue,
    /// Right padding.
    pub right: PaddingValue,
    /// Bottom padding.
    pub bottom: PaddingValue,
    /// Left padding.
    pub left: PaddingValue,
}

/// Width of the canvas in pixels.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct CanvasWidth(PositiveInt);

/// Height of the canvas in pixels.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct CanvasHeight(PositiveInt);

/// Horizontal coordinate value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct XCoordinate(NonNegativeFloat);

/// Vertical coordinate value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct YCoordinate(NonNegativeFloat);

/// Width value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct Width(PositiveFloat);

/// Height value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct Height(PositiveFloat);

/// Padding value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct PaddingValue(NonNegativeFloat);

/// Engine for computing diagram layouts.
pub struct LayoutEngine {
    /// Configuration for layout computation.
    config: LayoutConfig,
}

/// Configuration for the layout engine.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Spacing between entities.
    pub entity_spacing: EntitySpacing,
    /// Height of each swimlane.
    pub swimlane_height: SwimlaneHeight,
    /// Space between slices.
    pub slice_gutter: SliceGutter,
    /// Algorithm for routing connections.
    pub connection_routing: ConnectionRouting,
    /// Width of entity boxes.
    pub entity_width: EntityWidth,
    /// Height of entity boxes.
    pub entity_height: EntityHeight,
}

/// Algorithm for routing connections between entities.
#[derive(Debug, Clone)]
pub enum ConnectionRouting {
    /// Direct straight lines.
    Straight,
    /// Right-angle paths.
    Orthogonal,
    /// Smooth curved paths.
    Curved,
}

/// Spacing between entities.
#[nutype(derive(Debug, Clone, Copy))]
pub struct EntitySpacing(PositiveFloat);

/// Height of a swimlane.
#[nutype(derive(Debug, Clone, Copy))]
pub struct SwimlaneHeight(PositiveFloat);

/// Space between slices.
#[nutype(derive(Debug, Clone, Copy))]
pub struct SliceGutter(PositiveFloat);

/// Width of entity boxes.
#[nutype(derive(Debug, Clone, Copy))]
pub struct EntityWidth(PositiveFloat);

/// Height of entity boxes.
#[nutype(derive(Debug, Clone, Copy))]
pub struct EntityHeight(PositiveFloat);

impl LayoutEngine {
    /// Create a new layout engine with the given configuration.
    pub fn new(config: LayoutConfig) -> Self {
        Self { config }
    }

    /// Calculate the position for a swimlane based on its index.
    fn calculate_swimlane_position(&self, index: usize, padding: &Padding) -> Position {
        // Start position after top padding
        let base_y = padding.top.into_inner().value();

        // Calculate Y position based on index and swimlane height
        // No spacing between swimlanes - they share borders
        let swimlane_height = self.config.swimlane_height.into_inner().value();
        let y = base_y + (index as f32) * swimlane_height;

        Position {
            x: XCoordinate::new(
                NonNegativeFloat::parse(padding.left.into_inner().value()).unwrap(),
            ),
            y: YCoordinate::new(NonNegativeFloat::parse(y).unwrap()),
        }
    }

    /// Calculate the dimensions for a swimlane.
    fn calculate_swimlane_dimensions(&self, canvas_width: f64, padding: &Padding) -> Dimensions {
        let width = canvas_width
            - (padding.left.into_inner().value() as f64)
            - (padding.right.into_inner().value() as f64);
        let height = self.config.swimlane_height.into_inner().value();

        Dimensions {
            width: Width::new(PositiveFloat::parse(width as f32).unwrap()),
            height: Height::new(PositiveFloat::parse(height).unwrap()),
        }
    }

    /// Calculate positions for entities within a swimlane.
    #[allow(dead_code)]
    fn position_entities_in_swimlane<W, C, E, P, Q, A>(
        &self,
        swimlane: &crate::event_model::diagram::Swimlane,
        swimlane_layout: &SwimlaneLayout,
        registry: &crate::event_model::registry::EntityRegistry<W, C, E, P, Q, A>,
    ) -> HashMap<EntityId, EntityPosition> {
        let mut positions = HashMap::new();
        let entity_count = swimlane.entities.len();

        if entity_count == 0 {
            return positions;
        }

        // Calculate available width for entities
        let swimlane_width = swimlane_layout.dimensions.width.into_inner().value();
        let entity_spacing = self.config.entity_spacing.into_inner().value();

        // Get entity dimensions from configuration
        let entity_width = self.config.entity_width.into_inner().value();
        let entity_height = self.config.entity_height.into_inner().value();

        // Calculate horizontal spacing between entities
        let total_entity_width = entity_count as f32 * entity_width;
        let total_spacing = (entity_count - 1).max(0) as f32 * entity_spacing;
        let content_width = total_entity_width + total_spacing;

        // Center entities horizontally within swimlane
        let start_x = swimlane_layout.position.x.into_inner().value()
            + (swimlane_width - content_width) / 2.0;

        // Vertically center entities within swimlane
        let swimlane_height = swimlane_layout.dimensions.height.into_inner().value();
        let y = swimlane_layout.position.y.into_inner().value()
            + (swimlane_height - entity_height) / 2.0;

        // Position each entity
        for (index, entity_id) in swimlane.entities.iter().enumerate() {
            let x = start_x + (index as f32) * (entity_width + entity_spacing);

            // Look up entity type from registry
            // If registry lookup fails, try to infer type from entity name
            let entity_type = registry.get_entity_type(entity_id).unwrap_or_else(|| {
                // Infer entity type from naming conventions
                let entity_name_owned = entity_id.clone().into_inner();
                let entity_name_str = entity_name_owned.as_str();

                // Events typically use past tense or end with "ed"
                if entity_name_str.ends_with("ed") || entity_name_str.ends_with("Event") {
                    crate::event_model::entities::EntityType::Event
                } else if entity_name_str.ends_with("Service") || entity_name_str.contains("System")
                {
                    // External systems and services
                    crate::event_model::entities::EntityType::Projection
                } else if entity_name_str.starts_with("Get")
                    || entity_name_str.starts_with("Find")
                    || entity_name_str.ends_with("Query")
                {
                    // Queries typically start with Get/Find or end with Query
                    crate::event_model::entities::EntityType::Query
                } else if entity_name_str.contains("Validate")
                    || entity_name_str.contains("Process")
                    || entity_name_str.ends_with("Policy")
                {
                    // Policies/automations handle validation or processing
                    crate::event_model::entities::EntityType::Automation
                } else if entity_name_str.ends_with("View")
                    || entity_name_str.ends_with("List")
                    || entity_name_str.ends_with("Details")
                {
                    // Projections are often views or lists
                    crate::event_model::entities::EntityType::Projection
                } else {
                    // Default to command for imperative names
                    crate::event_model::entities::EntityType::Command
                }
            });

            // Look up entity name from registry
            // If registry lookup fails (which it will since we don't populate the registry yet),
            // use the entity ID as the name (since IDs are created from entity names)
            let entity_name = registry.get_entity_name(entity_id).unwrap_or_else(|| {
                crate::infrastructure::types::NonEmptyString::parse(
                    entity_id.clone().into_inner().as_str().to_string(),
                )
                .unwrap()
            });

            let position = EntityPosition {
                swimlane_id: swimlane.id.clone(),
                position: Position {
                    x: XCoordinate::new(NonNegativeFloat::parse(x.max(0.0)).unwrap()),
                    y: YCoordinate::new(NonNegativeFloat::parse(y.max(0.0)).unwrap()),
                },
                dimensions: Dimensions {
                    width: Width::new(PositiveFloat::parse(entity_width.max(1.0)).unwrap()),
                    height: Height::new(PositiveFloat::parse(entity_height.max(1.0)).unwrap()),
                },
                entity_type,
                entity_name,
            };

            positions.insert(entity_id.clone(), position);
        }

        positions
    }

    /// Route connectors between entities.
    ///
    /// This creates straight-line connections between entities.
    /// In the future, this will support more sophisticated routing algorithms.
    #[allow(dead_code)]
    fn route_connectors(
        &self,
        from_to_pairs: &[(EntityId, EntityId)],
        entity_positions: &HashMap<EntityId, EntityPosition>,
    ) -> Vec<Connection> {
        let mut connections = Vec::new();

        for (from_id, to_id) in from_to_pairs {
            // Get positions for both entities
            let from_pos = match entity_positions.get(from_id) {
                Some(pos) => pos,
                None => continue,
            };

            let to_pos = match entity_positions.get(to_id) {
                Some(pos) => pos,
                None => continue,
            };

            // Calculate connection points (center of entities for now)
            let from_center = Point {
                x: XCoordinate::new(
                    NonNegativeFloat::parse(
                        from_pos.position.x.into_inner().value()
                            + from_pos.dimensions.width.into_inner().value() / 2.0,
                    )
                    .unwrap(),
                ),
                y: YCoordinate::new(
                    NonNegativeFloat::parse(
                        from_pos.position.y.into_inner().value()
                            + from_pos.dimensions.height.into_inner().value() / 2.0,
                    )
                    .unwrap(),
                ),
            };

            let to_center = Point {
                x: XCoordinate::new(
                    NonNegativeFloat::parse(
                        to_pos.position.x.into_inner().value()
                            + to_pos.dimensions.width.into_inner().value() / 2.0,
                    )
                    .unwrap(),
                ),
                y: YCoordinate::new(
                    NonNegativeFloat::parse(
                        to_pos.position.y.into_inner().value()
                            + to_pos.dimensions.height.into_inner().value() / 2.0,
                    )
                    .unwrap(),
                ),
            };

            // Create simple straight-line path
            let path = ConnectionPath {
                points: vec![from_center, to_center],
            };

            // Create connection with appropriate style
            let connection = Connection {
                from: from_id.clone(),
                to: to_id.clone(),
                path,
                style: ConnectionStyle::Solid, // Default style for now
            };

            connections.push(connection);
        }

        connections
    }

    /// Compute the layout for a diagram.
    pub fn compute_layout<W, C, E, P, Q, A>(
        &self,
        diagram: &crate::event_model::diagram::EventModelDiagram<W, C, E, P, Q, A>,
    ) -> Result<Layout, LayoutError> {
        // Calculate canvas dimensions based on content
        let padding = Padding {
            top: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            right: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            bottom: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            left: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
        };

        // Calculate needed height based on number of swimlanes
        let swimlane_height = self.config.swimlane_height.into_inner().value();
        let spacing = self.config.entity_spacing.into_inner().value();
        let num_swimlanes = diagram.swimlanes.len() as f32;
        // No spacing between swimlanes - they share borders
        let content_height = num_swimlanes * swimlane_height;
        let total_height =
            content_height + padding.top.into_inner().value() + padding.bottom.into_inner().value();

        // Calculate needed width based on max entities in any swimlane
        let max_entities = diagram
            .swimlanes
            .iter()
            .map(|s| s.entities.len())
            .max()
            .unwrap_or(1) as f32;
        let entity_width = self.config.entity_width.into_inner().value();
        let content_width = max_entities * (entity_width + spacing) + spacing;
        let total_width =
            content_width + padding.left.into_inner().value() + padding.right.into_inner().value();

        let canvas = Canvas {
            width: CanvasWidth::new(PositiveInt::parse(total_width.max(800.0) as u32).unwrap()),
            height: CanvasHeight::new(PositiveInt::parse(total_height.max(600.0) as u32).unwrap()),
            padding,
        };

        let canvas_width = canvas.width.into_inner().value() as f64;

        // Position swimlanes
        let mut swimlane_layouts = HashMap::new();
        for (index, swimlane) in diagram.swimlanes.iter().enumerate() {
            let position = self.calculate_swimlane_position(index, &canvas.padding);
            let dimensions = self.calculate_swimlane_dimensions(canvas_width, &canvas.padding);

            swimlane_layouts.insert(
                swimlane.id.clone(),
                SwimlaneLayout {
                    position,
                    dimensions,
                    name: swimlane.name.clone().into_inner(),
                },
            );
        }

        // Position entities using flow-based layout with topological sorting
        let entity_positions = self.compute_flow_based_positions(diagram, &swimlane_layouts)?;

        // Route connections between entities from all slices
        let connector_pairs: Vec<(EntityId, EntityId)> = diagram
            .slices
            .iter()
            .flat_map(|slice| slice.connections.iter())
            .map(|conn| (conn.from.clone(), conn.to.clone()))
            .collect();
        let connections = self.route_connectors(&connector_pairs, &entity_positions);

        // Compute test scenario layouts
        let test_scenario_layouts = self.compute_test_scenario_layouts(diagram, &entity_positions);

        Ok(Layout {
            canvas,
            swimlane_layouts,
            entity_positions,
            slice_layouts: HashMap::new(),
            connections,
            test_scenario_layouts,
        })
    }

    /// Compute entity positions using flow-based layout with topological sorting.
    ///
    /// This method uses the connections in slices to determine the temporal order
    /// of entities and positions them in a left-to-right timeline layout.
    fn compute_flow_based_positions<W, C, E, P, Q, A>(
        &self,
        diagram: &crate::event_model::diagram::EventModelDiagram<W, C, E, P, Q, A>,
        swimlane_layouts: &HashMap<SwimlaneId, SwimlaneLayout>,
    ) -> Result<HashMap<EntityId, EntityPosition>, LayoutError> {
        // Build dependency graph from slice connections
        let (graph, entity_to_swimlane) = self.build_dependency_graph(diagram);

        // Perform topological sort to determine temporal order
        let timeline_order = self.topological_sort(&graph)?;

        // Position entities based on timeline order and swimlanes
        let positions = self.position_entities_in_timeline(
            &timeline_order,
            &entity_to_swimlane,
            swimlane_layouts,
            &diagram.entities,
        );

        Ok(positions)
    }

    /// Build a dependency graph from slice connections.
    ///
    /// Returns a graph where each entity points to its dependencies,
    /// and a mapping from entity to swimlane.
    fn build_dependency_graph<W, C, E, P, Q, A>(
        &self,
        diagram: &crate::event_model::diagram::EventModelDiagram<W, C, E, P, Q, A>,
    ) -> (
        HashMap<EntityId, Vec<EntityId>>,
        HashMap<EntityId, SwimlaneId>,
    ) {
        let mut graph: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
        let mut entity_to_swimlane: HashMap<EntityId, SwimlaneId> = HashMap::new();

        // Initialize graph with all entities
        for swimlane in diagram.swimlanes.iter() {
            for entity_id in &swimlane.entities {
                graph.insert(entity_id.clone(), Vec::new());
                entity_to_swimlane.insert(entity_id.clone(), swimlane.id.clone());
            }
        }

        // Add dependencies from slice connections
        // Only consider temporal dependencies for layout, not notification/update flows
        for slice in diagram.slices.iter() {
            for connection in &slice.connections {
                // Determine if this connection represents temporal flow (for layout)
                let is_temporal = Self::is_temporal_connection(&connection.from, &connection.to);

                if is_temporal {
                    // 'to' entity depends on 'from' entity (from happens before to in timeline)
                    if let Some(dependencies) = graph.get_mut(&connection.to) {
                        dependencies.push(connection.from.clone());
                    }
                }
            }
        }

        (graph, entity_to_swimlane)
    }

    /// Determines if a connection represents temporal flow (for layout) vs notification flow.
    ///
    /// Temporal connections (used for layout):
    /// - View -> Command (user action triggers command)  
    /// - Command -> Event (command produces event)
    /// - Event -> Projection (event builds projection over time)
    /// - Query -> View (query provides data to view)
    ///
    /// Non-temporal connections (notifications, not used for layout):
    /// - Event -> View (event notifies view to update)
    /// - Projection -> View (projection provides current state to view)
    /// - Automation -> Command (automation triggers command - but this could be temporal)
    fn is_temporal_connection(from: &EntityId, to: &EntityId) -> bool {
        // Extract entity types from prefixed IDs (e.g., "command_CreateUser" -> "command")
        let from_type = Self::extract_entity_type(from);
        let to_type = Self::extract_entity_type(to);

        match (from_type.as_str(), to_type.as_str()) {
            // Temporal flows (used for layout) - only the core event modeling flow
            ("command", "event") => true,    // Command produces event
            ("event", "projection") => true, // Event builds projection

            // Non-temporal flows (not used for layout)
            ("view", "command") => false, // View triggers command (creates cycles with event->view)
            ("view", "view") => false,    // View navigation (not temporal)
            ("event", "view") => false,   // Event notifies view (not temporal)
            ("projection", "view") => false, // Projection shows current state (not temporal)
            ("query", "view") => false,   // Query provides data to view (not temporal)
            ("automation", "command") => false, // Automation triggers command (could create cycles)

            // Be conservative: default to non-temporal to avoid cycles
            _ => false,
        }
    }

    /// Extracts the entity type from a prefixed EntityId (e.g., "command_CreateUser" -> "command")
    fn extract_entity_type(entity_id: &EntityId) -> String {
        let id_string = entity_id.clone().into_inner().into_inner();
        if let Some(underscore_pos) = id_string.find('_') {
            id_string[..underscore_pos].to_string()
        } else {
            // Fallback if no prefix
            "unknown".to_string()
        }
    }

    /// Perform topological sort on the dependency graph.
    ///
    /// Returns entities in temporal order (entities with no dependencies first).
    fn topological_sort(
        &self,
        graph: &HashMap<EntityId, Vec<EntityId>>,
    ) -> Result<Vec<EntityId>, LayoutError> {
        // Handle circular dependencies by breaking cycles
        let result = Self::topological_sort_impl(graph);
        if let Err(LayoutError::CircularDependency) = result {
            // Break cycles and retry
            let mut cycle_broken_graph = graph.clone();
            Self::break_cycles(&mut cycle_broken_graph);
            Self::topological_sort_impl(&cycle_broken_graph)
        } else {
            result
        }
    }

    /// Internal implementation of topological sort.
    fn topological_sort_impl(
        graph: &HashMap<EntityId, Vec<EntityId>>,
    ) -> Result<Vec<EntityId>, LayoutError> {
        let mut in_degree: HashMap<EntityId, usize> = HashMap::new();
        let mut adj_list: HashMap<EntityId, Vec<EntityId>> = HashMap::new();

        // Initialize in-degree count and adjacency list
        for (entity, dependencies) in graph {
            in_degree.insert(entity.clone(), dependencies.len());

            // Build reverse adjacency list (from dependencies to dependents)
            for dependency in dependencies {
                adj_list
                    .entry(dependency.clone())
                    .or_default()
                    .push(entity.clone());
            }
        }

        // Find entities with no dependencies (in-degree 0)
        let mut queue: Vec<EntityId> = in_degree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(entity, _)| entity.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(entity) = queue.pop() {
            result.push(entity.clone());

            // Reduce in-degree for all dependents
            if let Some(dependents) = adj_list.get(&entity) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check for circular dependencies
        if result.len() != graph.len() {
            return Err(LayoutError::CircularDependency);
        }

        Ok(result)
    }

    /// Breaks cycles in the dependency graph by removing edges.
    /// Uses DFS to detect cycles and removes one edge from each cycle.
    fn break_cycles(graph: &mut HashMap<EntityId, Vec<EntityId>>) {
        use std::collections::HashSet;

        // Track visited nodes and nodes in current path (for cycle detection)
        let mut visited = HashSet::new();
        let mut path = HashSet::new();
        let mut edges_to_remove = Vec::new();

        // Helper function for DFS cycle detection
        fn dfs_find_cycles(
            node: &EntityId,
            graph: &HashMap<EntityId, Vec<EntityId>>,
            visited: &mut HashSet<EntityId>,
            path: &mut HashSet<EntityId>,
            edges_to_remove: &mut Vec<(EntityId, EntityId)>,
        ) {
            if path.contains(node) {
                // Found a cycle - don't traverse further to avoid infinite recursion
                return;
            }

            if visited.contains(node) {
                return; // Already processed this subtree
            }

            visited.insert(node.clone());
            path.insert(node.clone());

            if let Some(dependencies) = graph.get(node) {
                for dep in dependencies {
                    if path.contains(dep) {
                        // Found a back edge (cycle) - mark it for removal
                        edges_to_remove.push((node.clone(), dep.clone()));
                    } else {
                        dfs_find_cycles(dep, graph, visited, path, edges_to_remove);
                    }
                }
            }

            path.remove(node);
        }

        // Find all nodes to start DFS from
        let all_nodes: Vec<EntityId> = graph.keys().cloned().collect();

        // Run DFS from each node to find cycles
        for node in &all_nodes {
            if !visited.contains(node) {
                dfs_find_cycles(node, graph, &mut visited, &mut path, &mut edges_to_remove);
            }
        }

        // Remove the problematic edges
        for (from, to) in edges_to_remove {
            if let Some(deps) = graph.get_mut(&from) {
                deps.retain(|dep| dep != &to);
            }
        }
    }

    /// Position entities in timeline order within their respective swimlanes.
    fn position_entities_in_timeline<W, C, E, P, Q, A>(
        &self,
        timeline_order: &[EntityId],
        entity_to_swimlane: &HashMap<EntityId, SwimlaneId>,
        swimlane_layouts: &HashMap<SwimlaneId, SwimlaneLayout>,
        entities: &crate::event_model::registry::EntityRegistry<W, C, E, P, Q, A>,
    ) -> HashMap<EntityId, EntityPosition> {
        let mut positions = HashMap::new();
        let mut swimlane_x_positions: HashMap<SwimlaneId, f32> = HashMap::new();

        let entity_width = self.config.entity_width.into_inner().value();
        let entity_height = self.config.entity_height.into_inner().value();
        let spacing = self.config.entity_spacing.into_inner().value();

        // Initialize starting X positions for each swimlane
        for swimlane_id in swimlane_layouts.keys() {
            swimlane_x_positions.insert(swimlane_id.clone(), spacing);
        }

        // Position entities in timeline order
        for entity_id in timeline_order {
            if let Some(swimlane_id) = entity_to_swimlane.get(entity_id) {
                if let Some(swimlane_layout) = swimlane_layouts.get(swimlane_id) {
                    let current_x = swimlane_x_positions.get(swimlane_id).unwrap_or(&spacing);

                    // Look up entity type and name from registry
                    let entity_type = entities.get_entity_type(entity_id).unwrap_or_else(|| {
                        // Fallback to inferring type from entity name if not in registry
                        let entity_name_owned = entity_id.clone().into_inner();
                        let entity_name_str = entity_name_owned.as_str();

                        if entity_name_str.ends_with("ed") || entity_name_str.ends_with("Event") {
                            crate::event_model::entities::EntityType::Event
                        } else if entity_name_str.ends_with("Service")
                            || entity_name_str.contains("System")
                        {
                            crate::event_model::entities::EntityType::Projection
                        } else if entity_name_str.starts_with("Get")
                            || entity_name_str.starts_with("Find")
                            || entity_name_str.ends_with("Query")
                        {
                            crate::event_model::entities::EntityType::Query
                        } else if entity_name_str.contains("Validate")
                            || entity_name_str.contains("Process")
                            || entity_name_str.ends_with("Policy")
                        {
                            crate::event_model::entities::EntityType::Automation
                        } else if entity_name_str.ends_with("View")
                            || entity_name_str.ends_with("List")
                            || entity_name_str.ends_with("Details")
                        {
                            crate::event_model::entities::EntityType::Wireframe
                        } else {
                            crate::event_model::entities::EntityType::Command
                        }
                    });

                    let entity_name = entities.get_entity_name(entity_id).unwrap_or_else(|| {
                        // Fallback to using the entity ID as the name
                        crate::infrastructure::types::NonEmptyString::parse(
                            entity_id.clone().into_inner().as_str().to_string(),
                        )
                        .unwrap()
                    });

                    let position = EntityPosition {
                        position: Position {
                            x: XCoordinate::new(
                                NonNegativeFloat::parse(
                                    swimlane_layout.position.x.into_inner().value() + current_x,
                                )
                                .unwrap(),
                            ),
                            y: YCoordinate::new(
                                NonNegativeFloat::parse(
                                    swimlane_layout.position.y.into_inner().value() + 10.0,
                                )
                                .unwrap(),
                            ),
                        },
                        dimensions: Dimensions {
                            width: Width::new(PositiveFloat::parse(entity_width).unwrap()),
                            height: Height::new(PositiveFloat::parse(entity_height).unwrap()),
                        },
                        entity_type,
                        entity_name,
                        swimlane_id: swimlane_id.clone(),
                    };

                    positions.insert(entity_id.clone(), position);

                    // Update X position for next entity in this swimlane
                    swimlane_x_positions
                        .insert(swimlane_id.clone(), current_x + entity_width + spacing);
                }
            }
        }

        positions
    }

    /// Compute layout positions for test scenarios.
    ///
    /// Test scenarios are positioned below their parent commands as sub-diagrams.
    /// Each test scenario gets its own box with Given/When/Then sections.
    fn compute_test_scenario_layouts<W, C, E, P, Q, A>(
        &self,
        _diagram: &crate::event_model::diagram::EventModelDiagram<W, C, E, P, Q, A>,
        _entity_positions: &HashMap<EntityId, EntityPosition>,
    ) -> Vec<TestScenarioLayout> {
        // First, we need to collect commands with test scenarios from the entities registry
        // This is a placeholder implementation that would need access to command test data
        // For now, return empty vector since we don't have direct access to YAML test scenarios
        // in the current diagram structure

        // TODO: Implement test scenario access from YAML domain model
        // The EventModelDiagram would need to include test scenario information
        // or we'd need to pass the YAML domain model separately

        Vec::new()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
}

/// Errors that can occur during layout computation.
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// Not enough space to place an entity.
    #[error("No space available for entity {0}")]
    NoSpaceAvailable(String),

    /// Entities have circular dependencies.
    #[error("Circular dependency detected")]
    CircularDependency,

    /// Slice boundaries are invalid.
    #[error("Invalid slice boundaries")]
    InvalidSliceBoundaries,

    /// Entity reference not found in registry.
    #[error("Invalid entity reference: {0:?}")]
    InvalidEntityReference(EntityId),
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
