// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Node-based layout system for Event Model diagrams.
//!
//! This module implements a node-based approach where entities can appear
//! multiple times in a diagram as separate visual nodes. This eliminates
//! visual clutter and circular dependency issues while providing clearer
//! flow visualization.

use crate::event_model::entities::{EntityId, EntityType};

/// A visual node in the diagram representing an instance of an entity.
///
/// Each node has its own position and connections, even though multiple
/// nodes may reference the same logical entity. This allows entities to
/// appear multiple times in different contexts within the diagram.
#[derive(Debug, Clone, PartialEq)]
pub struct DiagramNode {
    /// Unique identifier for this node instance
    id: NodeId,
    /// The entity this node represents
    entity_ref: EntityReference,
    /// Position of this node in the layout
    position: Option<Position>,
}

/// Unique identifier for a diagram node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    /// Create a new node ID from entity reference and context.
    ///
    /// The context helps differentiate multiple nodes for the same entity.
    pub fn new(entity_ref: &EntityReference, context: &str) -> Self {
        let id = format!(
            "{:?}:{}:{}",
            entity_ref.entity_type,
            entity_ref.entity_id.clone().into_inner().into_inner(),
            context
        );
        Self(id)
    }

    /// Get the string representation of the node ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Reference to an entity in the model.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityReference {
    /// Type of the entity
    pub entity_type: EntityType,
    /// ID of the entity
    pub entity_id: EntityId,
}

/// Position of a node in 2D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// X coordinate
    pub x: f64,
    /// Y coordinate  
    pub y: f64,
}

/// A connection between two nodes in the diagram.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeConnection {
    /// Source node ID
    pub from: NodeId,
    /// Target node ID
    pub to: NodeId,
    /// Optional label for the connection
    pub label: Option<String>,
}

impl DiagramNode {
    /// Create a new diagram node.
    pub fn new(id: NodeId, entity_ref: EntityReference) -> Self {
        Self {
            id,
            entity_ref,
            position: None,
        }
    }

    /// Get the node's ID.
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    /// Get the entity reference.
    pub fn entity_ref(&self) -> &EntityReference {
        &self.entity_ref
    }

    /// Get the node's position if set.
    pub fn position(&self) -> Option<&Position> {
        self.position.as_ref()
    }

    /// Set the node's position.
    pub fn set_position(&mut self, position: Position) {
        self.position = Some(position);
    }
}

/// Generates diagram nodes from entity connections.
///
/// This is responsible for analyzing slices and connections to determine
/// where nodes should be created in the diagram.
pub struct NodeGenerator {
    nodes: Vec<DiagramNode>,
    connections: Vec<NodeConnection>,
}

impl NodeGenerator {
    /// Create a new node generator.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
        }
    }

    /// Generate nodes from entity connections.
    ///
    /// For now, this is a placeholder that will be implemented
    /// once we update the layout engine integration.
    pub fn generate_from_slices(&mut self) {
        todo!("Implement node generation from slices")
    }

    /// Get the generated nodes.
    pub fn nodes(&self) -> &[DiagramNode] {
        &self.nodes
    }

    /// Get the generated connections.
    pub fn connections(&self) -> &[NodeConnection] {
        &self.connections
    }
}

impl Default for NodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_generation() {
        use crate::infrastructure::types::NonEmptyString;

        let entity_ref = EntityReference {
            entity_type: EntityType::Command,
            entity_id: EntityId::new(NonEmptyString::parse("SignUp".to_string()).unwrap()),
        };

        let node_id = NodeId::new(&entity_ref, "source");
        assert_eq!(node_id.as_str(), "Command:SignUp:source");
    }

    #[test]
    fn test_diagram_node_creation() {
        use crate::infrastructure::types::NonEmptyString;

        let entity_ref = EntityReference {
            entity_type: EntityType::Event,
            entity_id: EntityId::new(NonEmptyString::parse("UserCreated".to_string()).unwrap()),
        };

        let node_id = NodeId::new(&entity_ref, "target");
        let mut node = DiagramNode::new(node_id.clone(), entity_ref.clone());

        assert_eq!(node.id(), &node_id);
        assert_eq!(node.entity_ref(), &entity_ref);
        assert_eq!(node.position(), None);

        let pos = Position { x: 100.0, y: 200.0 };
        node.set_position(pos);
        assert_eq!(node.position(), Some(&pos));
    }
}
