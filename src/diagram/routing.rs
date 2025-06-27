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

#![allow(dead_code)] // Foundational types - not all used yet

use crate::infrastructure::types::NonEmpty;
use std::collections::HashMap;

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

/// A line extending from an entity edge or center in a cardinal direction.
/// Lead lines form the basis of the routing graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeadLine {
    /// Starting point of the line (on entity edge or center).
    pub start: Point,
    /// Direction the line extends.
    pub direction: Direction,
    /// End point where the line terminates (due to collision or boundary).
    pub end: Point,
}

impl LeadLine {
    /// Creates a new lead line.
    pub fn new(start: Point, direction: Direction, end: Point) -> Self {
        Self {
            start,
            direction,
            end,
        }
    }

    /// Checks if this lead line intersects with another lead line.
    /// Returns the intersection point if they intersect.
    pub fn intersection(&self, other: &LeadLine) -> Option<Point> {
        // Only perpendicular lines can intersect in an orthogonal system
        match (self.direction, other.direction) {
            (Direction::North | Direction::South, Direction::East | Direction::West)
            | (Direction::East | Direction::West, Direction::North | Direction::South) => {
                self.find_intersection_point(other)
            }
            _ => None, // Parallel lines don't intersect
        }
    }

    /// Finds the actual intersection point between two perpendicular lead lines.
    fn find_intersection_point(&self, other: &LeadLine) -> Option<Point> {
        let (vertical_line, horizontal_line) = match self.direction {
            Direction::North | Direction::South => (self, other),
            Direction::East | Direction::West => (other, self),
        };

        let intersection_x = vertical_line.start.x;
        let intersection_y = horizontal_line.start.y;
        let intersection = Point::new(intersection_x, intersection_y);

        // Check if intersection point lies within both line segments
        if vertical_line.contains_point(&intersection)
            && horizontal_line.contains_point(&intersection)
        {
            Some(intersection)
        } else {
            None
        }
    }

    /// Checks if a point lies on this lead line segment.
    fn contains_point(&self, point: &Point) -> bool {
        match self.direction {
            Direction::North => {
                point.x == self.start.x && point.y >= self.end.y && point.y <= self.start.y
            }
            Direction::South => {
                point.x == self.start.x && point.y >= self.start.y && point.y <= self.end.y
            }
            Direction::East => {
                point.y == self.start.y && point.x >= self.start.x && point.x <= self.end.x
            }
            Direction::West => {
                point.y == self.start.y && point.x >= self.end.x && point.x <= self.start.x
            }
        }
    }
}

/// A node in the routing graph, typically at the intersection of lead lines.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutingNode {
    pub position: Point,
    pub node_id: usize,
}

impl RoutingNode {
    /// Creates a new routing node.
    pub fn new(position: Point, node_id: usize) -> Self {
        Self { position, node_id }
    }
}

/// An edge in the routing graph connecting two routing nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingEdge {
    pub from: usize,
    pub to: usize,
    pub weight: u32,
}

impl RoutingEdge {
    /// Creates a new routing edge with weight based on Manhattan distance.
    pub fn new(from: usize, to: usize, weight: u32) -> Self {
        Self { from, to, weight }
    }
}

/// A complete routing graph containing nodes and edges for pathfinding.
#[derive(Debug, Clone)]
pub struct RoutingGraph {
    pub nodes: HashMap<usize, RoutingNode>,
    pub edges: Vec<RoutingEdge>,
}

impl RoutingGraph {
    /// Creates a new empty routing graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Adds a node to the graph and returns its ID.
    pub fn add_node(&mut self, position: Point) -> usize {
        let node_id = self.nodes.len();
        let node = RoutingNode::new(position, node_id);
        self.nodes.insert(node_id, node);
        node_id
    }

    /// Adds an edge between two nodes with the given weight.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: u32) {
        self.edges.push(RoutingEdge::new(from, to, weight));
    }

    /// Gets the neighbors of a node for pathfinding algorithms.
    pub fn neighbors(&self, node_id: usize) -> Vec<(usize, u32)> {
        self.edges
            .iter()
            .filter_map(|edge| {
                if edge.from == node_id {
                    Some((edge.to, edge.weight))
                } else if edge.to == node_id {
                    Some((edge.from, edge.weight))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for RoutingGraph {
    fn default() -> Self {
        Self::new()
    }
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
    pub entity_margin: u32,
    /// Maximum distance to extend lead lines.
    pub max_lead_line_length: u32,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            entity_margin: 10,
            max_lead_line_length: 1000,
        }
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
    fn test_lead_line_intersection() {
        let vertical = LeadLine::new(Point::new(10, 5), Direction::South, Point::new(10, 15));
        let horizontal = LeadLine::new(Point::new(5, 10), Direction::East, Point::new(15, 10));

        assert_eq!(vertical.intersection(&horizontal), Some(Point::new(10, 10)));
    }

    #[test]
    fn test_routing_graph_add_node() {
        let mut graph = RoutingGraph::new();
        let node_id = graph.add_node(Point::new(10, 10));
        assert_eq!(node_id, 0);
        assert!(graph.nodes.contains_key(&node_id));
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
