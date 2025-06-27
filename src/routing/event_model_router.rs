//! Event model-specific routing implementation.
//!
//! This module provides high-level routing functionality specifically tailored
//! for event modeling diagrams, building on top of the libavoid wrapper.

use crate::diagram::EventModelDiagram;
use crate::diagram::routing_types::{Point, Rectangle, RoutePath};
use crate::event_model::yaml_types::EntityReference;
use std::collections::HashMap;

use super::{LibavoidRouter, ObstacleId, Result, RoutingConfig, RoutingError};

/// High-level router for event model diagrams.
///
/// This router understands event modeling concepts and converts them to
/// low-level routing operations using libavoid.
#[allow(dead_code)]
pub struct EventModelRouter {
    /// The underlying libavoid router.
    router: LibavoidRouter,
    /// Map from entity names to their obstacle IDs.
    obstacle_map: HashMap<String, ObstacleId>,
    /// Configuration for routing behavior.
    config: RoutingConfig,
}

impl EventModelRouter {
    /// Creates a new event model router with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(RoutingConfig::default())
    }

    /// Creates a new event model router with custom configuration.
    pub fn with_config(config: RoutingConfig) -> Result<Self> {
        let router = LibavoidRouter::new()?;
        Ok(Self {
            router,
            obstacle_map: HashMap::new(),
            config,
        })
    }

    /// Sets up the routing scene from an event model diagram.
    ///
    /// This method:
    /// - Converts all entities to obstacles
    /// - Maps swimlane boundaries
    /// - Prepares the routing environment
    pub fn setup_from_diagram(
        &mut self,
        _diagram: &EventModelDiagram,
        entity_positions: &HashMap<String, EntityPosition>,
    ) -> Result<()> {
        // Add all entities as obstacles
        for (entity_name, position) in entity_positions {
            let rect = Rectangle::new(position.x, position.y, position.width, position.height);

            let obstacle_id = self.router.add_obstacle(&rect)?;
            self.obstacle_map.insert(entity_name.clone(), obstacle_id);
        }

        // Process all routing operations
        self.router.process_transaction()?;

        Ok(())
    }

    /// Routes all connections in the diagram.
    ///
    /// Returns a list of connection identifiers with their routed paths.
    pub fn route_all_connections(
        &mut self,
        diagram: &EventModelDiagram,
        entity_positions: &HashMap<String, EntityPosition>,
    ) -> Result<Vec<(ConnectionId, RoutePath)>> {
        let mut routed_paths = Vec::new();

        // Route connections from each slice
        for (slice_index, slice) in diagram.slices().iter().enumerate() {
            for connection in slice.connections.iter() {
                let connection_id = ConnectionId {
                    slice_index,
                    from: connection.from.clone(),
                    to: connection.to.clone(),
                };

                // Find entity positions
                let from_entity = extract_entity_name(&connection.from);
                let to_entity = extract_entity_name(&connection.to);

                let from_pos = find_entity_position(&from_entity, slice_index, entity_positions)
                    .ok_or_else(|| {
                        RoutingError::InvalidParameters(format!(
                            "Cannot find position for entity: {}",
                            from_entity
                        ))
                    })?;

                let to_pos = find_entity_position(&to_entity, slice_index, entity_positions)
                    .ok_or_else(|| {
                        RoutingError::InvalidParameters(format!(
                            "Cannot find position for entity: {}",
                            to_entity
                        ))
                    })?;

                // Calculate connection points
                let (from_point, to_point) = calculate_connection_points(from_pos, to_pos);

                // Route the connection
                let path = self.router.route_connector(&from_point, &to_point)?;
                routed_paths.push((connection_id, path));
            }
        }

        // Process all routing operations
        self.router.process_transaction()?;

        Ok(routed_paths)
    }

    /// Routes a single connection between two entities.
    pub fn route_connection(
        &mut self,
        from: &EntityPosition,
        to: &EntityPosition,
    ) -> Result<RoutePath> {
        let (from_point, to_point) = calculate_connection_points(from, to);
        self.router.route_connector(&from_point, &to_point)
    }
}

/// Identifies a unique connection in the diagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionId {
    /// The slice index where this connection appears.
    pub slice_index: usize,
    /// The source entity reference.
    pub from: EntityReference,
    /// The target entity reference.
    pub to: EntityReference,
}

/// Position information for a rendered entity.
#[derive(Debug, Clone)]
pub struct EntityPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub slice_index: usize,
}

/// Extracts the entity name from an entity reference.
fn extract_entity_name(entity_ref: &EntityReference) -> String {
    match entity_ref {
        EntityReference::View(view_path) => {
            let path_string = view_path.clone().into_inner();
            let path_str = path_string.as_str();
            path_str.split('.').next().unwrap_or(path_str).to_string()
        }
        EntityReference::Command(command_name) => {
            command_name.clone().into_inner().as_str().to_string()
        }
        EntityReference::Event(event_name) => event_name.clone().into_inner().as_str().to_string(),
        EntityReference::Projection(projection_name) => {
            projection_name.clone().into_inner().as_str().to_string()
        }
        EntityReference::Query(query_name) => query_name.clone().into_inner().as_str().to_string(),
        EntityReference::Automation(automation_name) => {
            automation_name.clone().into_inner().as_str().to_string()
        }
    }
}

/// Finds the position of an entity in a specific slice.
fn find_entity_position<'a>(
    entity_name: &str,
    slice_index: usize,
    entity_positions: &'a HashMap<String, EntityPosition>,
) -> Option<&'a EntityPosition> {
    // First try exact match with slice index
    let key = format!("{}_{}", entity_name, slice_index);
    if let Some(pos) = entity_positions.get(&key) {
        return Some(pos);
    }

    // If not found, find the closest instance
    let mut closest_pos: Option<&EntityPosition> = None;
    let mut closest_distance = usize::MAX;
    let prefix = format!("{}_", entity_name);

    for (key, pos) in entity_positions {
        if key.starts_with(&prefix) {
            let distance = if pos.slice_index > slice_index {
                pos.slice_index - slice_index
            } else {
                slice_index - pos.slice_index
            };

            if distance < closest_distance {
                closest_distance = distance;
                closest_pos = Some(pos);
            }
        }
    }

    closest_pos
}

/// Calculates the optimal connection points between two entities.
fn calculate_connection_points(from: &EntityPosition, to: &EntityPosition) -> (Point, Point) {
    let from_rect = Rectangle::new(from.x, from.y, from.width, from.height);
    let to_rect = Rectangle::new(to.x, to.y, to.width, to.height);

    // Determine the best edge to connect from/to based on relative positions
    let from_center = from_rect.center();
    let to_center = to_rect.center();

    let dx = to_center.x as i32 - from_center.x as i32;
    let dy = to_center.y as i32 - from_center.y as i32;

    let from_point = if dx.abs() > dy.abs() {
        // Horizontal connection
        if dx > 0 {
            // Connect from right edge of 'from' entity
            Point::new(from.x + from.width, from.y + from.height / 2)
        } else {
            // Connect from left edge of 'from' entity
            Point::new(from.x, from.y + from.height / 2)
        }
    } else {
        // Vertical connection
        if dy > 0 {
            // Connect from bottom edge of 'from' entity
            Point::new(from.x + from.width / 2, from.y + from.height)
        } else {
            // Connect from top edge of 'from' entity
            Point::new(from.x + from.width / 2, from.y)
        }
    };

    let to_point = if dx.abs() > dy.abs() {
        // Horizontal connection
        if dx > 0 {
            // Connect to left edge of 'to' entity
            Point::new(to.x, to.y + to.height / 2)
        } else {
            // Connect to right edge of 'to' entity
            Point::new(to.x + to.width, to.y + to.height / 2)
        }
    } else {
        // Vertical connection
        if dy > 0 {
            // Connect to top edge of 'to' entity
            Point::new(to.x + to.width / 2, to.y)
        } else {
            // Connect to bottom edge of 'to' entity
            Point::new(to.x + to.width / 2, to.y + to.height)
        }
    };

    (from_point, to_point)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_entity_name() {
        use crate::event_model::yaml_types;
        use crate::infrastructure::types::NonEmptyString;

        let view_path =
            yaml_types::ViewPath::new(NonEmptyString::parse("LoginView".to_string()).unwrap());
        let view_ref = EntityReference::View(view_path);
        assert_eq!(extract_entity_name(&view_ref), "LoginView");

        let cmd_name = yaml_types::CommandName::new(
            NonEmptyString::parse("LoginCommand".to_string()).unwrap(),
        );
        let cmd_ref = EntityReference::Command(cmd_name);
        assert_eq!(extract_entity_name(&cmd_ref), "LoginCommand");
    }

    #[test]
    fn test_calculate_connection_points_horizontal() {
        let from = EntityPosition {
            x: 100,
            y: 100,
            width: 120,
            height: 60,
            slice_index: 0,
        };
        let to = EntityPosition {
            x: 300,
            y: 100,
            width: 120,
            height: 60,
            slice_index: 0,
        };

        let (from_point, to_point) = calculate_connection_points(&from, &to);

        // Should connect from right edge to left edge
        assert_eq!(from_point.x, 220); // 100 + 120
        assert_eq!(from_point.y, 130); // 100 + 60/2
        assert_eq!(to_point.x, 300);
        assert_eq!(to_point.y, 130); // 100 + 60/2
    }

    #[test]
    fn test_calculate_connection_points_vertical() {
        let from = EntityPosition {
            x: 100,
            y: 100,
            width: 120,
            height: 60,
            slice_index: 0,
        };
        let to = EntityPosition {
            x: 100,
            y: 300,
            width: 120,
            height: 60,
            slice_index: 0,
        };

        let (from_point, to_point) = calculate_connection_points(&from, &to);

        // Should connect from bottom edge to top edge
        assert_eq!(from_point.x, 160); // 100 + 120/2
        assert_eq!(from_point.y, 160); // 100 + 60
        assert_eq!(to_point.x, 160); // 100 + 120/2
        assert_eq!(to_point.y, 300);
    }

    #[test]
    fn test_find_entity_position_exact_match() {
        let mut positions = HashMap::new();
        positions.insert(
            "TestEntity_0".to_string(),
            EntityPosition {
                x: 100,
                y: 100,
                width: 120,
                height: 60,
                slice_index: 0,
            },
        );

        let result = find_entity_position("TestEntity", 0, &positions);
        assert!(result.is_some());
        assert_eq!(result.unwrap().x, 100);
    }

    #[test]
    fn test_find_entity_position_closest_match() {
        let mut positions = HashMap::new();
        positions.insert(
            "TestEntity_1".to_string(),
            EntityPosition {
                x: 100,
                y: 100,
                width: 120,
                height: 60,
                slice_index: 1,
            },
        );
        positions.insert(
            "TestEntity_3".to_string(),
            EntityPosition {
                x: 300,
                y: 100,
                width: 120,
                height: 60,
                slice_index: 3,
            },
        );

        // Looking for slice 2, should find slice 1 (distance 1) rather than slice 3 (distance 1)
        // Since they're equidistant, it depends on iteration order, but one should be found
        let result = find_entity_position("TestEntity", 2, &positions);
        assert!(result.is_some());
    }
}
