//! Safe Rust wrapper for libavoid FFI bindings.
//!
//! This module provides a safe, idiomatic Rust interface to libavoid
//! with proper error handling and memory management.

use crate::diagram::routing_types::{Point, Rectangle, RoutePath};
use crate::infrastructure::types::NonEmpty;
use thiserror::Error;

/// Errors that can occur during routing operations.
#[derive(Debug, Error)]
pub enum RoutingError {
    /// Failed to create router
    #[error("Failed to create router")]
    RouterCreation,

    /// Failed to create shape
    #[error("Failed to create shape: {0}")]
    ShapeCreation(String),

    /// Failed to create connector
    #[error("Failed to create connector: {0}")]
    ConnectorCreation(String),

    /// Failed to route connector
    #[error("Failed to route connector: {0}")]
    RoutingFailed(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
}

/// Result type for routing operations.
pub type Result<T> = std::result::Result<T, RoutingError>;

/// Opaque identifier for obstacles in the routing scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObstacleId(u32);

/// Configuration for the routing algorithm.
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// Segment penalty for orthogonal routing
    pub segment_penalty: f64,

    /// Margin around obstacles
    pub obstacle_margin: f64,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            segment_penalty: 50.0,
            obstacle_margin: 10.0,
        }
    }
}

// Mock implementation for testing
#[cfg(any(test, feature = "mock-router"))]
mod mock_impl {
    use super::*;

    /// Safe wrapper around libavoid Router (mock version).
    pub struct LibavoidRouter {
        next_obstacle_id: u32,
    }

    impl LibavoidRouter {
        /// Creates a new router instance.
        pub fn new() -> Result<Self> {
            Ok(Self {
                next_obstacle_id: 1,
            })
        }

        /// Adds a rectangular obstacle to the routing scene.
        pub fn add_obstacle(&mut self, _rect: &Rectangle) -> Result<ObstacleId> {
            let id = ObstacleId(self.next_obstacle_id);
            self.next_obstacle_id += 1;
            Ok(id)
        }

        /// Routes a connector between two points, avoiding obstacles.
        pub fn route_connector(&mut self, start: &Point, end: &Point) -> Result<RoutePath> {
            // Mock implementation: create a simple L-shaped path
            let mid_x = start.x;
            let mid_y = end.y;

            let nodes = NonEmpty::from_head_and_tail(*start, vec![Point::new(mid_x, mid_y), *end]);

            // Calculate Manhattan distance as cost
            let cost = start.manhattan_distance(&Point::new(mid_x, mid_y))
                + Point::new(mid_x, mid_y).manhattan_distance(end);

            Ok(RoutePath::new(nodes, cost))
        }

        /// Processes all pending routing operations.
        pub fn process_transaction(&mut self) -> Result<()> {
            Ok(())
        }
    }
}

// Real FFI implementation
#[cfg(not(any(test, feature = "mock-router")))]
mod ffi_impl {
    use super::*;
    use crate::routing::libavoid_ffi::{
        AvoidConnectorId, AvoidPoint, AvoidRectangle, AvoidRouter, AvoidShapeId,
        ORTHOGONAL_ROUTING, avoid_free_points, avoid_router_add_connector, avoid_router_add_shape,
        avoid_router_delete, avoid_router_delete_connector, avoid_router_delete_shape,
        avoid_router_get_route_points, avoid_router_new, avoid_router_process_transaction,
    };
    use std::collections::HashMap;
    use std::ptr;

    // Key for tracking connectors
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct ConnectorKey {
        start: (i64, i64),
        end: (i64, i64),
    }

    /// Safe wrapper around libavoid Router (FFI version).
    pub struct LibavoidRouter {
        router: AvoidRouter,
        obstacles: HashMap<ObstacleId, AvoidShapeId>,
        connectors: HashMap<ConnectorKey, AvoidConnectorId>,
        next_obstacle_id: u32,
    }

    impl LibavoidRouter {
        /// Creates a new router instance.
        pub fn new() -> Result<Self> {
            let router = unsafe { avoid_router_new(ORTHOGONAL_ROUTING) };
            if router.is_null() {
                return Err(RoutingError::RouterCreation);
            }

            Ok(Self {
                router,
                obstacles: HashMap::new(),
                connectors: HashMap::new(),
                next_obstacle_id: 1,
            })
        }

        /// Adds a rectangular obstacle to the routing scene.
        pub fn add_obstacle(&mut self, rect: &Rectangle) -> Result<ObstacleId> {
            let avoid_rect = rectangle_to_libavoid(rect);
            let shape_id = unsafe { avoid_router_add_shape(self.router, avoid_rect) };

            if shape_id == 0 {
                return Err(RoutingError::ShapeCreation("Failed to add shape".into()));
            }

            let obstacle_id = ObstacleId(self.next_obstacle_id);
            self.next_obstacle_id += 1;
            self.obstacles.insert(obstacle_id, shape_id);

            Ok(obstacle_id)
        }

        /// Routes a connector between two points, avoiding obstacles.
        pub fn route_connector(&mut self, start: &Point, end: &Point) -> Result<RoutePath> {
            let start_point = point_to_libavoid(start);
            let end_point = point_to_libavoid(end);

            // Add connector
            let conn_id =
                unsafe { avoid_router_add_connector(self.router, start_point, end_point) };

            if conn_id == 0 {
                return Err(RoutingError::ConnectorCreation(
                    "Failed to add connector".into(),
                ));
            }

            // Store connector for cleanup
            let key = ConnectorKey {
                start: (start.x, start.y),
                end: (end.x, end.y),
            };
            self.connectors.insert(key, conn_id);

            // Process routing
            unsafe { avoid_router_process_transaction(self.router) };

            // Get route points
            let mut points_ptr: *mut AvoidPoint = ptr::null_mut();
            let num_points = unsafe {
                avoid_router_get_route_points(self.router, conn_id, &mut points_ptr as *mut _)
            };

            if num_points <= 0 || points_ptr.is_null() {
                // Clean up connector
                unsafe { avoid_router_delete_connector(self.router, conn_id) };
                self.connectors.remove(&key);
                return Err(RoutingError::RoutingFailed("No route found".into()));
            }

            // Convert points to our format
            let mut route_points = Vec::with_capacity(num_points as usize);
            for i in 0..num_points {
                let point = unsafe { *points_ptr.offset(i as isize) };
                route_points.push(point_from_libavoid(&point));
            }

            // Free the points array
            unsafe { avoid_free_points(points_ptr) };

            // Clean up connector after getting route
            unsafe { avoid_router_delete_connector(self.router, conn_id) };
            self.connectors.remove(&key);

            // Create route path
            let nodes = NonEmpty::try_from(route_points)
                .map_err(|_| RoutingError::RoutingFailed("Empty route".into()))?;

            // Calculate total cost (sum of segment lengths)
            let cost = nodes
                .windows(2)
                .map(|pair| pair[0].manhattan_distance(&pair[1]))
                .sum();

            Ok(RoutePath::new(nodes, cost))
        }

        /// Processes all pending routing operations.
        pub fn process_transaction(&mut self) -> Result<()> {
            unsafe { avoid_router_process_transaction(self.router) };
            Ok(())
        }
    }

    impl Drop for LibavoidRouter {
        fn drop(&mut self) {
            if !self.router.is_null() {
                // Clean up all obstacles
                for shape_id in self.obstacles.values() {
                    unsafe { avoid_router_delete_shape(self.router, *shape_id) };
                }

                // Clean up all connectors
                for conn_id in self.connectors.values() {
                    unsafe { avoid_router_delete_connector(self.router, *conn_id) };
                }

                // Delete the router
                unsafe { avoid_router_delete(self.router) };
            }
        }
    }

    // Send + Sync are safe because libavoid doesn't use thread-local storage
    unsafe impl Send for LibavoidRouter {}
    unsafe impl Sync for LibavoidRouter {}

    // Helper functions for converting between our types and libavoid types

    /// Converts our Point to libavoid Point.
    fn point_to_libavoid(point: &Point) -> AvoidPoint {
        AvoidPoint {
            x: point.x as f64,
            y: point.y as f64,
        }
    }

    /// Converts libavoid Point to our Point.
    fn point_from_libavoid(point: &AvoidPoint) -> Point {
        Point::new(point.x as i64, point.y as i64)
    }

    /// Converts our Rectangle to libavoid Rectangle.
    fn rectangle_to_libavoid(rect: &Rectangle) -> AvoidRectangle {
        AvoidRectangle {
            min_x: rect.x as f64,
            min_y: rect.y as f64,
            max_x: (rect.x + rect.width as i64) as f64,
            max_y: (rect.y + rect.height as i64) as f64,
        }
    }
}

// Re-export the appropriate implementation
#[cfg(any(test, feature = "mock-router"))]
pub use mock_impl::LibavoidRouter;

#[cfg(not(any(test, feature = "mock-router")))]
pub use ffi_impl::LibavoidRouter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        // With mock implementation, router creation should succeed
        let result = LibavoidRouter::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_routing() {
        let mut router = LibavoidRouter::new().unwrap();

        // Add some obstacles
        let rect1 = Rectangle::new(50, 50, 100, 100);
        let rect2 = Rectangle::new(200, 200, 100, 100);

        let obs1 = router.add_obstacle(&rect1).unwrap();
        let obs2 = router.add_obstacle(&rect2).unwrap();

        assert_ne!(obs1, obs2);

        // Route a connector
        let start = Point::new(10, 10);
        let end = Point::new(300, 300);

        let path = router.route_connector(&start, &end).unwrap();

        // Check that path has at least start and end points
        assert!(path.nodes.len() >= 2);
        assert_eq!(*path.nodes.first(), start);
        assert_eq!(*path.nodes.last(), end);

        // Verify it's an orthogonal path (mock creates L-shape)
        assert_eq!(path.nodes.len(), 3);
    }

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();
        assert_eq!(config.segment_penalty, 50.0);
        assert_eq!(config.obstacle_margin, 10.0);
    }
}
