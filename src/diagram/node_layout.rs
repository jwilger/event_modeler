// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Node-based layout engine for Event Model diagrams.
//!
//! This module implements a layout algorithm that works with diagram nodes
//! instead of entities directly. This allows entities to appear multiple
//! times in the diagram at different positions, eliminating visual clutter
//! and circular dependency issues.

use super::layout::{
    Canvas, CanvasHeight, CanvasWidth, Connection, ConnectionPath, ConnectionStyle, Dimensions,
    EntityPosition, Height, Layout, LayoutConfig, LayoutError, Padding, PaddingValue, Point,
    Position, SwimlaneLayout, Width, XCoordinate, YCoordinate,
};
use super::node::{DiagramNode, EntityReference, NodeConnection, NodeId};
use crate::event_model::diagram::{EventModelDiagram, SwimlaneId};
use crate::event_model::entities::EntityId;
use crate::event_model::registry::EntityRegistry;
use crate::infrastructure::types::{NonNegativeFloat, PositiveFloat, PositiveInt};
use std::collections::{HashMap, VecDeque};

/// Node-based layout engine that positions nodes instead of entities.
pub struct NodeLayoutEngine {
    #[allow(dead_code)]
    config: LayoutConfig,
}

/// A positioned node in the layout.
#[derive(Debug, Clone)]
pub struct PositionedNode {
    /// The diagram node.
    pub node: DiagramNode,
    /// Position of the node.
    pub position: Position,
    /// Dimensions of the node.
    pub dimensions: Dimensions,
    /// Swimlane containing this node.
    pub swimlane_id: SwimlaneId,
}

/// Result of node-based layout computation.
#[derive(Debug, Clone)]
pub struct NodeLayout {
    /// Overall canvas dimensions.
    pub canvas: Canvas,
    /// Positioned nodes.
    pub nodes: HashMap<NodeId, PositionedNode>,
    /// Connections between nodes.
    pub connections: Vec<NodeConnection>,
    /// Swimlane layouts (same as entity-based).
    pub swimlane_layouts: HashMap<SwimlaneId, SwimlaneLayout>,
}

impl NodeLayoutEngine {
    /// Create a new node-based layout engine.
    pub fn new(config: LayoutConfig) -> Self {
        Self { config }
    }

    /// Generate nodes from slices in the diagram.
    ///
    /// Each endpoint in a slice connection becomes a potential node.
    /// Multiple connections to/from the same entity create separate nodes.
    fn generate_nodes_from_slices<W, C, E, P, Q, A>(
        &self,
        diagram: &EventModelDiagram<W, C, E, P, Q, A>,
        registry: &EntityRegistry<W, C, E, P, Q, A>,
    ) -> Result<(Vec<DiagramNode>, Vec<NodeConnection>), LayoutError> {
        let mut nodes = Vec::new();
        let mut connections = Vec::new();
        let mut node_map: HashMap<(EntityId, String), NodeId> = HashMap::new();

        // Process each slice
        for slice in diagram.slices.iter() {
            // Process each connection in the slice
            for (conn_index, connection) in slice.connections.iter().enumerate() {
                let source = &connection.from;
                let target = &connection.to;

                // Generate context based on slice and connection index
                let source_context = format!("{:?}_source_{}", slice.name, conn_index);
                let target_context = format!("{:?}_target_{}", slice.name, conn_index);

                // Create or get source node
                let source_key = (source.clone(), source_context.clone());
                let source_node_id = if let Some(id) = node_map.get(&source_key) {
                    id.clone()
                } else {
                    // Look up entity type from registry
                    let entity_type = registry
                        .get_entity_type(source)
                        .ok_or_else(|| LayoutError::InvalidEntityReference(source.clone()))?;

                    let entity_ref = EntityReference {
                        entity_type,
                        entity_id: source.clone(),
                    };

                    let node_id = NodeId::new(&entity_ref, &source_context);
                    let node = DiagramNode::new(node_id.clone(), entity_ref);

                    nodes.push(node);
                    node_map.insert(source_key, node_id.clone());
                    node_id
                };

                // Create or get target node
                let target_key = (target.clone(), target_context.clone());
                let target_node_id = if let Some(id) = node_map.get(&target_key) {
                    id.clone()
                } else {
                    // Look up entity type from registry
                    let entity_type = registry
                        .get_entity_type(target)
                        .ok_or_else(|| LayoutError::InvalidEntityReference(target.clone()))?;

                    let entity_ref = EntityReference {
                        entity_type,
                        entity_id: target.clone(),
                    };

                    let node_id = NodeId::new(&entity_ref, &target_context);
                    let node = DiagramNode::new(node_id.clone(), entity_ref);

                    nodes.push(node);
                    node_map.insert(target_key, node_id.clone());
                    node_id
                };

                // Create connection between nodes
                let node_connection = NodeConnection {
                    from: source_node_id,
                    to: target_node_id,
                    label: None,
                };
                connections.push(node_connection);
            }
        }

        Ok((nodes, connections))
    }

    /// Perform topological sort on nodes to determine their horizontal positions.
    ///
    /// Returns a vector of node layers, where each layer contains nodes that
    /// can be positioned at the same horizontal position.
    fn topological_sort(
        nodes: &[DiagramNode],
        connections: &[NodeConnection],
    ) -> Result<Vec<Vec<NodeId>>, LayoutError> {
        // Build adjacency list and in-degree map
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();

        // Initialize all nodes with 0 in-degree
        for node in nodes {
            in_degree.insert(node.id().clone(), 0);
            adjacency.insert(node.id().clone(), Vec::new());
        }

        // Build graph from connections
        for conn in connections {
            adjacency.get_mut(&conn.from).unwrap().push(conn.to.clone());
            *in_degree.get_mut(&conn.to).unwrap() += 1;
        }

        // Find all nodes with no incoming edges
        let mut queue: VecDeque<NodeId> = VecDeque::new();
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        // Process nodes in topological order, grouping by layers
        let mut layers: Vec<Vec<NodeId>> = Vec::new();
        let mut processed_count = 0;

        while !queue.is_empty() {
            let layer_size = queue.len();
            let mut current_layer = Vec::new();

            // Process all nodes at the current level
            for _ in 0..layer_size {
                let node_id = queue.pop_front().unwrap();
                current_layer.push(node_id.clone());
                processed_count += 1;

                // Reduce in-degree for all neighbors
                if let Some(neighbors) = adjacency.get(&node_id) {
                    for neighbor in neighbors {
                        let degree = in_degree.get_mut(neighbor).unwrap();
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }

            layers.push(current_layer);
        }

        // Check for cycles
        if processed_count != nodes.len() {
            return Err(LayoutError::CircularDependency);
        }

        Ok(layers)
    }

    /// Position nodes based on topological layers and swimlane assignments.
    fn position_nodes(
        &self,
        nodes: &[DiagramNode],
        connections: &[NodeConnection],
        swimlane_assignments: &HashMap<NodeId, SwimlaneId>,
        swimlane_layouts: &HashMap<SwimlaneId, SwimlaneLayout>,
    ) -> Result<HashMap<NodeId, PositionedNode>, LayoutError> {
        let layers = Self::topological_sort(nodes, connections)?;
        let mut positioned_nodes = HashMap::new();

        // Calculate horizontal position for each layer
        let padding = 20.0;
        let node_width = self.config.entity_width.into_inner().value();
        let node_height = self.config.entity_height.into_inner().value();
        let layer_spacing = self.config.entity_spacing.into_inner().value() * 2.0;

        for (layer_index, layer_nodes) in layers.iter().enumerate() {
            let x = padding + (layer_index as f64) * (node_width as f64 + layer_spacing as f64);

            // Group nodes by swimlane for vertical positioning
            let mut swimlane_nodes: HashMap<SwimlaneId, Vec<&NodeId>> = HashMap::new();
            for node_id in layer_nodes {
                if let Some(swimlane_id) = swimlane_assignments.get(node_id) {
                    swimlane_nodes
                        .entry(swimlane_id.clone())
                        .or_default()
                        .push(node_id);
                }
            }

            // Position nodes within each swimlane
            for (swimlane_id, node_ids) in swimlane_nodes {
                if let Some(swimlane_layout) = swimlane_layouts.get(&swimlane_id) {
                    let swimlane_height = swimlane_layout.dimensions.height.into_inner().value();
                    let swimlane_y = swimlane_layout.position.y.into_inner().value();

                    // Vertically center nodes within swimlane
                    let total_height = node_ids.len() as f64 * node_height as f64;
                    let vertical_spacing = if node_ids.len() > 1 {
                        (swimlane_height as f64 - total_height) / (node_ids.len() - 1) as f64
                    } else {
                        0.0
                    };

                    let start_y = swimlane_y as f64
                        + (swimlane_height as f64
                            - total_height
                            - vertical_spacing * (node_ids.len() - 1) as f64)
                            / 2.0;

                    for (i, node_id) in node_ids.iter().enumerate() {
                        let y = start_y + (i as f64) * (node_height as f64 + vertical_spacing);

                        // Find the actual node
                        if let Some(node) = nodes.iter().find(|n| n.id() == *node_id) {
                            let position = Position {
                                x: XCoordinate::new(NonNegativeFloat::parse(x as f32).unwrap()),
                                y: YCoordinate::new(NonNegativeFloat::parse(y as f32).unwrap()),
                            };

                            let dimensions = Dimensions {
                                width: Width::new(PositiveFloat::parse(node_width).unwrap()),
                                height: Height::new(PositiveFloat::parse(node_height).unwrap()),
                            };

                            let mut positioned_node =
                                DiagramNode::new(node.id().clone(), node.entity_ref().clone());
                            positioned_node.set_position(super::node::Position { x, y });

                            positioned_nodes.insert(
                                node.id().clone(),
                                PositionedNode {
                                    node: positioned_node,
                                    position,
                                    dimensions,
                                    swimlane_id: swimlane_id.clone(),
                                },
                            );
                        }
                    }
                }
            }
        }

        Ok(positioned_nodes)
    }

    /// Compute node-based layout for the diagram.
    pub fn compute_node_layout<W, C, E, P, Q, A>(
        &self,
        diagram: &EventModelDiagram<W, C, E, P, Q, A>,
        registry: &EntityRegistry<W, C, E, P, Q, A>,
    ) -> Result<NodeLayout, LayoutError> {
        // Generate nodes from slices
        let (nodes, connections) = self.generate_nodes_from_slices(diagram, registry)?;

        // Create swimlane layouts
        let mut swimlane_layouts = HashMap::new();
        let padding = Padding {
            top: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            right: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            bottom: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            left: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
        };

        for (index, swimlane) in diagram.swimlanes.iter().enumerate() {
            let position = Position {
                x: XCoordinate::new(
                    NonNegativeFloat::parse(padding.left.into_inner().value()).unwrap(),
                ),
                y: YCoordinate::new(
                    NonNegativeFloat::parse(
                        padding.top.into_inner().value()
                            + (index as f32)
                                * (self.config.swimlane_height.into_inner().value()
                                    + self.config.entity_spacing.into_inner().value()),
                    )
                    .unwrap(),
                ),
            };

            let dimensions = Dimensions {
                width: Width::new(PositiveFloat::parse(1160.0).unwrap()), // canvas width - padding
                height: Height::new(self.config.swimlane_height.into_inner()),
            };

            swimlane_layouts.insert(
                swimlane.id.clone(),
                SwimlaneLayout {
                    position,
                    dimensions,
                    name: swimlane.name.clone().into_inner(),
                },
            );
        }

        // Assign nodes to swimlanes based on their entity
        let mut swimlane_assignments: HashMap<NodeId, SwimlaneId> = HashMap::new();
        for node in &nodes {
            // Find which swimlane contains this entity
            for swimlane in diagram.swimlanes.iter() {
                if swimlane
                    .entities
                    .iter()
                    .any(|e| e == &node.entity_ref().entity_id)
                {
                    swimlane_assignments.insert(node.id().clone(), swimlane.id.clone());
                    break;
                }
            }
        }

        // Position nodes
        let positioned_nodes = self.position_nodes(
            &nodes,
            &connections,
            &swimlane_assignments,
            &swimlane_layouts,
        )?;

        // Calculate canvas size based on positioned nodes
        let max_x = positioned_nodes
            .values()
            .map(|pn| pn.position.x.into_inner().value() + pn.dimensions.width.into_inner().value())
            .fold(0.0f32, |a, b| a.max(b));
        let max_y = positioned_nodes
            .values()
            .map(|pn| {
                pn.position.y.into_inner().value() + pn.dimensions.height.into_inner().value()
            })
            .fold(0.0f32, |a, b| a.max(b));

        let canvas = Canvas {
            width: CanvasWidth::new(
                PositiveInt::parse((max_x + padding.right.into_inner().value()) as u32).unwrap(),
            ),
            height: CanvasHeight::new(
                PositiveInt::parse((max_y + padding.bottom.into_inner().value()) as u32).unwrap(),
            ),
            padding,
        };

        let node_layout = NodeLayout {
            canvas,
            nodes: positioned_nodes,
            connections,
            swimlane_layouts,
        };

        Ok(node_layout)
    }
}

/// Convert a node-based layout to the entity-based layout format.
///
/// This is a temporary adapter to maintain compatibility with existing
/// rendering code while we transition to node-based rendering.
pub fn adapt_node_layout_to_entity_layout(node_layout: &NodeLayout) -> Layout {
    let mut entity_positions = HashMap::new();
    let mut connections = Vec::new();

    // Convert positioned nodes to entity positions
    // For now, we'll use the first occurrence of each entity
    let mut seen_entities: HashMap<EntityId, ()> = HashMap::new();

    for positioned_node in node_layout.nodes.values() {
        let entity_id = positioned_node.node.entity_ref().entity_id.clone();

        // Only include the first occurrence of each entity
        if !seen_entities.contains_key(&entity_id) {
            seen_entities.insert(entity_id.clone(), ());

            let entity_position = EntityPosition {
                swimlane_id: positioned_node.swimlane_id.clone(),
                position: positioned_node.position,
                dimensions: positioned_node.dimensions,
                entity_type: positioned_node.node.entity_ref().entity_type,
                entity_name: entity_id.clone().into_inner(),
            };

            entity_positions.insert(entity_id, entity_position);
        }
    }

    // Convert node connections to entity connections
    // This is lossy - we lose the multiple node information
    for node_conn in &node_layout.connections {
        // Find the entities for the source and target nodes
        if let (Some(source_node), Some(target_node)) = (
            node_layout.nodes.get(&node_conn.from),
            node_layout.nodes.get(&node_conn.to),
        ) {
            let from_entity = source_node.node.entity_ref().entity_id.clone();
            let to_entity = target_node.node.entity_ref().entity_id.clone();

            // Create a simple straight-line connection path
            let from_pos = &source_node.position;
            let to_pos = &target_node.position;
            let from_dims = &source_node.dimensions;
            let to_dims = &target_node.dimensions;

            let start_x = from_pos.x.into_inner().value() + from_dims.width.into_inner().value();
            let start_y =
                from_pos.y.into_inner().value() + from_dims.height.into_inner().value() / 2.0;
            let end_x = to_pos.x.into_inner().value();
            let end_y = to_pos.y.into_inner().value() + to_dims.height.into_inner().value() / 2.0;

            let connection = Connection {
                from: from_entity,
                to: to_entity,
                path: ConnectionPath {
                    points: vec![
                        Point {
                            x: XCoordinate::new(NonNegativeFloat::parse(start_x).unwrap()),
                            y: YCoordinate::new(NonNegativeFloat::parse(start_y).unwrap()),
                        },
                        Point {
                            x: XCoordinate::new(NonNegativeFloat::parse(end_x).unwrap()),
                            y: YCoordinate::new(NonNegativeFloat::parse(end_y).unwrap()),
                        },
                    ],
                },
                style: ConnectionStyle::Solid,
            };

            connections.push(connection);
        }
    }

    Layout {
        canvas: node_layout.canvas.clone(),
        swimlane_layouts: node_layout.swimlane_layouts.clone(),
        entity_positions,
        slice_layouts: HashMap::new(),
        connections,
        test_scenario_layouts: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_layout_engine_creation() {
        use crate::diagram::layout::{
            ConnectionRouting, EntityHeight, EntitySpacing, EntityWidth, SliceGutter,
            SwimlaneHeight,
        };
        use crate::infrastructure::types::PositiveFloat;

        let config = LayoutConfig {
            entity_spacing: EntitySpacing::new(PositiveFloat::parse(20.0).unwrap()),
            swimlane_height: SwimlaneHeight::new(PositiveFloat::parse(100.0).unwrap()),
            slice_gutter: SliceGutter::new(PositiveFloat::parse(10.0).unwrap()),
            connection_routing: ConnectionRouting::Straight,
            entity_width: EntityWidth::new(PositiveFloat::parse(160.0).unwrap()),
            entity_height: EntityHeight::new(PositiveFloat::parse(80.0).unwrap()),
        };
        let engine = NodeLayoutEngine::new(config);
        assert_eq!(engine.config.entity_width.into_inner().value(), 160.0);
    }

    #[test]
    fn test_node_generation_from_slices() {
        // TODO: Add comprehensive tests for node generation
        // This will be implemented after we integrate with the layout engine
    }
}
