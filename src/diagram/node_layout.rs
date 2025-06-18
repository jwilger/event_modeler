// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Node-based layout engine for Event Model diagrams.
//!
//! This module implements a layout algorithm that works with diagram nodes
//! instead of entities directly. This allows entities to appear multiple
//! times in the diagram at different positions, eliminating visual clutter
//! and circular dependency issues.

use super::layout::{
    Canvas, CanvasHeight, CanvasWidth, Dimensions, Layout, LayoutConfig, LayoutError, Padding,
    PaddingValue, Position,
};
use super::node::{DiagramNode, EntityReference, NodeConnection, NodeId};
use crate::event_model::diagram::{EventModelDiagram, SwimlaneId};
use crate::event_model::entities::EntityId;
use crate::event_model::registry::EntityRegistry;
use crate::infrastructure::types::{NonNegativeFloat, PositiveInt};
use std::collections::HashMap;

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
    pub swimlane_layouts: HashMap<SwimlaneId, super::layout::SwimlaneLayout>,
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

    /// Compute node-based layout for the diagram.
    pub fn compute_node_layout<W, C, E, P, Q, A>(
        &self,
        diagram: &EventModelDiagram<W, C, E, P, Q, A>,
        registry: &EntityRegistry<W, C, E, P, Q, A>,
    ) -> Result<NodeLayout, LayoutError> {
        // Generate nodes from slices
        let (_nodes, connections) = self.generate_nodes_from_slices(diagram, registry)?;

        // For now, create a simple layout
        // TODO: Implement proper topological sort and positioning
        let canvas = Canvas {
            width: CanvasWidth::new(PositiveInt::parse(1200).unwrap()),
            height: CanvasHeight::new(PositiveInt::parse(800).unwrap()),
            padding: Padding {
                top: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                right: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                bottom: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
                left: PaddingValue::new(NonNegativeFloat::parse(20.0).unwrap()),
            },
        };

        let node_layout = NodeLayout {
            canvas,
            nodes: HashMap::new(),
            connections,
            swimlane_layouts: HashMap::new(),
        };

        Ok(node_layout)
    }
}

/// Convert a node-based layout to the entity-based layout format.
///
/// This is a temporary adapter to maintain compatibility with existing
/// rendering code while we transition to node-based rendering.
pub fn adapt_node_layout_to_entity_layout(node_layout: &NodeLayout) -> Layout {
    // For now, return a minimal layout
    // TODO: Implement proper adaptation
    Layout {
        canvas: node_layout.canvas.clone(),
        swimlane_layouts: node_layout.swimlane_layouts.clone(),
        entity_positions: HashMap::new(),
        slice_layouts: HashMap::new(),
        connections: Vec::new(),
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
