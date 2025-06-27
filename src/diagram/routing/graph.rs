//! Graph structure for orthogonal routing.
//!
//! The routing graph is built from lead line intersections and used
//! for pathfinding.

use super::Point;
use std::collections::HashMap;

/// A node in the routing graph, representing an intersection point
#[derive(Debug, Clone)]
pub struct RoutingNode {
    /// Unique identifier for this node
    pub id: NodeId,
    /// Position of this node
    pub position: Point,
}

impl RoutingNode {
    /// Creates a new routing node
    pub fn new(id: NodeId, position: Point) -> Self {
        Self { id, position }
    }
}

/// Unique identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Creates a new node ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Gets the underlying value
    pub fn value(&self) -> usize {
        self.0
    }
}

/// An edge in the routing graph connecting two nodes
#[derive(Debug, Clone)]
pub struct RoutingEdge {
    /// Source node ID
    pub source: NodeId,
    /// Destination node ID
    pub destination: NodeId,
    /// Weight of this edge (Manhattan distance)
    pub weight: u32,
}

impl RoutingEdge {
    /// Creates a new routing edge
    pub fn new(source: NodeId, destination: NodeId, weight: u32) -> Self {
        Self {
            source,
            destination,
            weight,
        }
    }
}

/// The routing graph used for pathfinding
#[derive(Debug, Clone)]
pub struct RoutingGraph {
    /// All nodes in the graph
    nodes: HashMap<NodeId, RoutingNode>,
    /// Adjacency list representation
    edges: HashMap<NodeId, Vec<RoutingEdge>>,
    /// Reverse lookup from position to node ID
    position_to_node: HashMap<Point, NodeId>,
    /// Next available node ID
    next_node_id: usize,
}

impl RoutingGraph {
    /// Creates a new empty routing graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            position_to_node: HashMap::new(),
            next_node_id: 0,
        }
    }

    /// Adds a node to the graph, returning its ID
    pub fn add_node(&mut self, position: Point) -> NodeId {
        // Check if we already have a node at this position
        if let Some(&node_id) = self.position_to_node.get(&position) {
            return node_id;
        }

        // Create new node
        let node_id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;

        let node = RoutingNode::new(node_id, position);
        self.nodes.insert(node_id, node);
        self.position_to_node.insert(position, node_id);
        self.edges.insert(node_id, Vec::new());

        node_id
    }

    /// Adds an edge between two nodes
    pub fn add_edge(&mut self, source: NodeId, destination: NodeId) -> Result<(), GraphError> {
        // Verify both nodes exist
        let source_node = self
            .nodes
            .get(&source)
            .ok_or(GraphError::NodeNotFound(source))?;
        let dest_node = self
            .nodes
            .get(&destination)
            .ok_or(GraphError::NodeNotFound(destination))?;

        // Calculate weight as Manhattan distance
        let weight = manhattan_distance(source_node.position, dest_node.position);

        // Add edge (bidirectional)
        let forward_edge = RoutingEdge::new(source, destination, weight);
        let reverse_edge = RoutingEdge::new(destination, source, weight);

        self.edges
            .get_mut(&source)
            .ok_or(GraphError::NodeNotFound(source))?
            .push(forward_edge);

        self.edges
            .get_mut(&destination)
            .ok_or(GraphError::NodeNotFound(destination))?
            .push(reverse_edge);

        Ok(())
    }

    /// Gets a node by its ID
    pub fn get_node(&self, id: NodeId) -> Option<&RoutingNode> {
        self.nodes.get(&id)
    }

    /// Gets a node by its position
    pub fn get_node_at(&self, position: Point) -> Option<&RoutingNode> {
        self.position_to_node
            .get(&position)
            .and_then(|id| self.nodes.get(id))
    }

    /// Gets all edges from a node
    pub fn get_edges(&self, node: NodeId) -> Option<&[RoutingEdge]> {
        self.edges.get(&node).map(|v| v.as_slice())
    }

    /// Gets the total number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Gets the total number of edges (counting both directions)
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum::<usize>() / 2
    }

    /// Clears the graph
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.position_to_node.clear();
        self.next_node_id = 0;
    }
}

impl Default for RoutingGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during graph operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// Node not found in graph
    NodeNotFound(NodeId),
}

/// An intersection between two lead lines
#[derive(Debug, Clone)]
pub struct Intersection {
    /// Position of the intersection
    pub position: Point,
    /// IDs of the lead lines that intersect here
    pub lead_line_ids: (usize, usize),
}

impl Intersection {
    /// Creates a new intersection
    pub fn new(position: Point, line1_id: usize, line2_id: usize) -> Self {
        Self {
            position,
            lead_line_ids: (line1_id, line2_id),
        }
    }
}

/// Calculates Manhattan distance between two points
pub fn manhattan_distance(a: Point, b: Point) -> u32 {
    let dx = if a.x > b.x { a.x - b.x } else { b.x - a.x };
    let dy = if a.y > b.y { a.y - b.y } else { b.y - a.y };
    dx + dy
}

/// Finds the intersection point between two line segments, if any
pub fn find_line_intersection(
    line1_start: Point,
    line1_end: Point,
    line2_start: Point,
    line2_end: Point,
) -> Option<Point> {
    // Convert to f64 for precision
    let x1 = line1_start.x as f64;
    let y1 = line1_start.y as f64;
    let x2 = line1_end.x as f64;
    let y2 = line1_end.y as f64;
    let x3 = line2_start.x as f64;
    let y3 = line2_start.y as f64;
    let x4 = line2_end.x as f64;
    let y4 = line2_end.y as f64;

    // Calculate determinant
    let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

    // Lines are parallel
    if denom.abs() < f64::EPSILON {
        return None;
    }

    // Calculate intersection point
    let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

    // Check if intersection is within both line segments
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        let x = x1 + t * (x2 - x1);
        let y = y1 + t * (y2 - y1);
        Some(Point::new(x.round() as u32, y.round() as u32))
    } else {
        None
    }
}

/// Checks if a point lies on a line segment
pub fn point_on_line(point: Point, line_start: Point, line_end: Point) -> bool {
    // Check if point is within bounding box
    let min_x = line_start.x.min(line_end.x);
    let max_x = line_start.x.max(line_end.x);
    let min_y = line_start.y.min(line_end.y);
    let max_y = line_start.y.max(line_end.y);

    if point.x < min_x || point.x > max_x || point.y < min_y || point.y > max_y {
        return false;
    }

    // Check if point is collinear with line
    // For orthogonal lines, this means either x or y coordinates match
    if line_start.x == line_end.x {
        // Vertical line
        point.x == line_start.x
    } else if line_start.y == line_end.y {
        // Horizontal line
        point.y == line_start.y
    } else {
        // Should not happen with orthogonal routing
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_add_node() {
        let mut graph = RoutingGraph::new();
        let pos1 = Point::new(10, 20);
        let pos2 = Point::new(30, 20);

        let id1 = graph.add_node(pos1);
        let id2 = graph.add_node(pos2);

        assert_ne!(id1, id2);
        assert_eq!(graph.node_count(), 2);

        // Adding same position returns same ID
        let id1_again = graph.add_node(pos1);
        assert_eq!(id1, id1_again);
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_graph_add_edge() {
        let mut graph = RoutingGraph::new();
        let pos1 = Point::new(10, 20);
        let pos2 = Point::new(30, 20);

        let id1 = graph.add_node(pos1);
        let id2 = graph.add_node(pos2);

        assert!(graph.add_edge(id1, id2).is_ok());
        assert_eq!(graph.edge_count(), 1);

        // Check edges exist in both directions
        let edges1 = graph.get_edges(id1).unwrap();
        assert_eq!(edges1.len(), 1);
        assert_eq!(edges1[0].destination, id2);
        assert_eq!(edges1[0].weight, 20); // Manhattan distance

        let edges2 = graph.get_edges(id2).unwrap();
        assert_eq!(edges2.len(), 1);
        assert_eq!(edges2[0].destination, id1);
    }

    #[test]
    fn test_find_line_intersection() {
        // Horizontal and vertical lines that intersect
        let h_start = Point::new(10, 20);
        let h_end = Point::new(30, 20);
        let v_start = Point::new(20, 10);
        let v_end = Point::new(20, 30);

        let intersection = find_line_intersection(h_start, h_end, v_start, v_end);
        assert_eq!(intersection, Some(Point::new(20, 20)));

        // Lines that don't intersect
        let h2_start = Point::new(10, 40);
        let h2_end = Point::new(30, 40);

        let no_intersection = find_line_intersection(h_start, h_end, h2_start, h2_end);
        assert_eq!(no_intersection, None);
    }

    #[test]
    fn test_point_on_line() {
        let line_start = Point::new(10, 20);
        let line_end = Point::new(30, 20);

        // Point on line
        assert!(point_on_line(Point::new(20, 20), line_start, line_end));
        assert!(point_on_line(Point::new(10, 20), line_start, line_end));
        assert!(point_on_line(Point::new(30, 20), line_start, line_end));

        // Point not on line
        assert!(!point_on_line(Point::new(20, 30), line_start, line_end));
        assert!(!point_on_line(Point::new(40, 20), line_start, line_end));
    }
}
