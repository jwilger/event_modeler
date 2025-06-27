//! Orthogonal connector routing for entity connections.
//!
//! This module implements an orthogonal connector routing algorithm based on research
//! by Wybrow, Marriott, and Stuckey. The algorithm finds optimal collision-free paths
//! between entities using a three-stage approach:
//!
//! 1. Lead line generation from entity edges and centers
//! 2. Intersection detection to create routing nodes
//! 3. Graph-based pathfinding using Dijkstra's algorithm
//!
//! ## Sources
//! - "Orthogonal Connector Routing" by Wybrow, Marriott, and Stuckey
//! - OrthogonalConnectorRouting GitHub implementations
//! - yFiles documentation on edge routing algorithms

#![allow(dead_code)] // Temporary while we port the implementation

pub mod collision;
pub mod graph;
pub mod lead_lines;
pub mod orthogonal_router;
pub mod pathfinding;

#[cfg(test)]
mod integration_tests;

// Re-export main types from the root module
pub use self::orthogonal_router::OrthogonalRouter as OrthogonalRouterNew;

use crate::infrastructure::types::NonEmpty;

/// A point in 2D space for routing calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    /// Creates a new point.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Calculates the Manhattan distance to another point.
    pub fn manhattan_distance(&self, other: &Point) -> u32 {
        ((self.x as i32 - other.x as i32).abs() + (self.y as i32 - other.y as i32).abs()) as u32
    }

    /// Returns the right edge X coordinate
    pub fn right(&self) -> u32 {
        self.x
    }

    /// Returns the bottom edge Y coordinate
    pub fn bottom(&self) -> u32 {
        self.y
    }
}

/// A rectangular area representing an entity's position and dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    /// Creates a new rectangle.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Returns the right edge coordinate
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    /// Returns the bottom edge coordinate
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    /// Returns the four corner points of the rectangle.
    pub fn corners(&self) -> [Point; 4] {
        [
            Point::new(self.x, self.y),                            // top-left
            Point::new(self.x + self.width, self.y),               // top-right
            Point::new(self.x, self.y + self.height),              // bottom-left
            Point::new(self.x + self.width, self.y + self.height), // bottom-right
        ]
    }

    /// Returns the four edge center points of the rectangle.
    pub fn edge_centers(&self) -> [Point; 4] {
        [
            Point::new(self.x + self.width / 2, self.y), // top center
            Point::new(self.x + self.width, self.y + self.height / 2), // right center
            Point::new(self.x + self.width / 2, self.y + self.height), // bottom center
            Point::new(self.x, self.y + self.height / 2), // left center
        ]
    }

    /// Checks if this rectangle contains the given point.
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Checks if this rectangle intersects with another rectangle.
    pub fn intersects(&self, other: &Rectangle) -> bool {
        !(self.x + self.width < other.x
            || other.x + other.width < self.x
            || self.y + self.height < other.y
            || other.y + other.height < self.y)
    }
}

/// Direction of a lead line extending from an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

/// A complete path from source to destination through the routing graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePath {
    pub nodes: NonEmpty<Point>,
    pub total_cost: u32,
}

impl RoutePath {
    /// Creates a new route path.
    pub fn new(nodes: NonEmpty<Point>, total_cost: u32) -> Self {
        Self { nodes, total_cost }
    }

    /// Converts the path to SVG path data for rendering.
    pub fn to_svg_path(&self) -> String {
        let first = self.nodes.first();
        let mut path = format!("M {} {}", first.x, first.y);

        for point in self.nodes.iter().skip(1) {
            path.push_str(&format!(" L {} {}", point.x, point.y));
        }

        path
    }
}

/// Configuration for the routing algorithm.
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// Margin around entities to avoid placing routes too close.
    pub margin: u32,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self { margin: 10 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_manhattan_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert_eq!(p1.manhattan_distance(&p2), 7);
    }

    #[test]
    fn test_rectangle_center() {
        let rect = Rectangle::new(10, 20, 40, 30);
        assert_eq!(rect.center(), Point::new(30, 35));
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(10, 10, 20, 20);
        assert!(rect.contains(&Point::new(15, 15)));
        assert!(!rect.contains(&Point::new(5, 5)));
    }

    #[test]
    fn test_rectangle_intersects() {
        let rect1 = Rectangle::new(0, 0, 10, 10);
        let rect2 = Rectangle::new(5, 5, 10, 10);
        let rect3 = Rectangle::new(20, 20, 10, 10);

        assert!(rect1.intersects(&rect2));
        assert!(!rect1.intersects(&rect3));
    }

    #[test]
    fn test_route_path_to_svg() {
        let points = NonEmpty::from_head_and_tail(
            Point::new(0, 0),
            vec![Point::new(10, 0), Point::new(10, 10)],
        );
        let path = RoutePath::new(points, 20);
        assert_eq!(path.to_svg_path(), "M 0 0 L 10 0 L 10 10");
    }
}
