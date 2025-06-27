//! Pathfinding algorithms for the routing graph.
//!
//! Implements Dijkstra's algorithm for finding shortest paths in the routing graph.

use super::Point;
use super::graph::{NodeId, RoutingGraph};
use crate::infrastructure::types::NonEmpty;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A path through the routing graph
#[derive(Debug, Clone)]
pub struct GraphPath {
    /// Ordered list of node IDs in the path
    pub nodes: NonEmpty<NodeId>,
    /// Total cost of the path
    pub total_cost: u32,
}

impl GraphPath {
    /// Creates a new path from a non-empty list of nodes
    pub fn new(nodes: NonEmpty<NodeId>, total_cost: u32) -> Self {
        Self { nodes, total_cost }
    }

    /// Converts the path to a list of points
    pub fn to_points(&self, graph: &RoutingGraph) -> Option<NonEmpty<Point>> {
        let points: Vec<Point> = self
            .nodes
            .iter()
            .filter_map(|&node_id| graph.get_node(node_id).map(|node| node.position))
            .collect();

        NonEmpty::new(points)
    }
}

/// Node state during pathfinding
#[derive(Debug, Clone, Eq, PartialEq)]
struct PathfindingNode {
    /// Node ID
    id: NodeId,
    /// Cost to reach this node
    cost: u32,
}

impl Ord for PathfindingNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.id.value().cmp(&other.id.value()))
    }
}

impl PartialOrd for PathfindingNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Dijkstra's pathfinding algorithm
pub struct DijkstraPathfinder;

impl DijkstraPathfinder {
    /// Finds the shortest path between two nodes in the graph
    pub fn find_path(graph: &RoutingGraph, start: NodeId, goal: NodeId) -> Option<GraphPath> {
        // Early exit if start equals goal
        if start == goal {
            return NonEmpty::new(vec![start]).map(|nodes| GraphPath::new(nodes, 0));
        }

        // Initialize data structures
        let mut distances: HashMap<NodeId, u32> = HashMap::new();
        let mut previous: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();

        // Start node has distance 0
        distances.insert(start, 0);
        heap.push(PathfindingNode { id: start, cost: 0 });

        // Dijkstra's algorithm
        while let Some(PathfindingNode { id: current, cost }) = heap.pop() {
            // Skip if we've already visited this node
            if !visited.insert(current) {
                continue;
            }

            // Found the goal
            if current == goal {
                return Self::reconstruct_path(start, goal, &previous, &distances);
            }

            // Skip if we've found a better path
            if cost > *distances.get(&current).unwrap_or(&u32::MAX) {
                continue;
            }

            // Check all neighbors
            if let Some(edges) = graph.get_edges(current) {
                for edge in edges {
                    let neighbor = edge.destination;
                    let new_cost = cost + edge.weight;

                    // Update if this is a better path
                    if new_cost < *distances.get(&neighbor).unwrap_or(&u32::MAX) {
                        distances.insert(neighbor, new_cost);
                        previous.insert(neighbor, current);
                        heap.push(PathfindingNode {
                            id: neighbor,
                            cost: new_cost,
                        });
                    }
                }
            }
        }

        // No path found
        None
    }

    /// Reconstructs the path from start to goal using the previous map
    fn reconstruct_path(
        start: NodeId,
        goal: NodeId,
        previous: &HashMap<NodeId, NodeId>,
        distances: &HashMap<NodeId, u32>,
    ) -> Option<GraphPath> {
        let mut path = Vec::new();
        let mut current = goal;

        // Build path backwards
        path.push(current);
        while current != start {
            current = *previous.get(&current)?;
            path.push(current);
        }

        // Reverse to get start -> goal order
        path.reverse();

        // Get total cost
        let total_cost = *distances.get(&goal)?;

        NonEmpty::new(path).map(|nodes| GraphPath::new(nodes, total_cost))
    }
}

/// A* pathfinding algorithm (for future implementation)
pub struct AStarPathfinder;

impl AStarPathfinder {
    /// Finds the shortest path using A* algorithm
    ///
    /// Currently just delegates to Dijkstra. A* requires a heuristic function
    /// which for orthogonal routing would be Manhattan distance to goal.
    pub fn find_path(graph: &RoutingGraph, start: NodeId, goal: NodeId) -> Option<GraphPath> {
        // For now, just use Dijkstra
        // TODO: Implement proper A* with Manhattan distance heuristic
        DijkstraPathfinder::find_path(graph, start, goal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagram::routing::Point;
    use crate::diagram::routing::graph::RoutingGraph;

    #[test]
    fn test_dijkstra_simple_path() {
        let mut graph = RoutingGraph::new();

        // Create a simple path: A -> B -> C
        let a = graph.add_node(Point::new(0, 0));
        let b = graph.add_node(Point::new(10, 0));
        let c = graph.add_node(Point::new(20, 0));

        graph.add_edge(a, b).unwrap();
        graph.add_edge(b, c).unwrap();

        // Find path from A to C
        let path = DijkstraPathfinder::find_path(&graph, a, c).unwrap();
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.total_cost, 20); // 10 + 10
    }

    #[test]
    fn test_dijkstra_shortest_path() {
        let mut graph = RoutingGraph::new();

        // Create a graph with multiple paths:
        //   B
        //  / \
        // A   D
        //  \ /
        //   C
        let a = graph.add_node(Point::new(0, 10));
        let b = graph.add_node(Point::new(10, 20));
        let c = graph.add_node(Point::new(10, 0));
        let d = graph.add_node(Point::new(20, 10));

        // A->B->D has cost 20+20=40
        graph.add_edge(a, b).unwrap();
        graph.add_edge(b, d).unwrap();

        // A->C->D has cost 20+20=40 (same cost)
        graph.add_edge(a, c).unwrap();
        graph.add_edge(c, d).unwrap();

        // Both paths should work
        let path = DijkstraPathfinder::find_path(&graph, a, d).unwrap();
        assert_eq!(path.nodes.len(), 3);
        assert_eq!(path.total_cost, 40);
    }

    #[test]
    fn test_dijkstra_no_path() {
        let mut graph = RoutingGraph::new();

        // Create disconnected nodes
        let a = graph.add_node(Point::new(0, 0));
        let b = graph.add_node(Point::new(10, 0));

        // No edge between them
        let path = DijkstraPathfinder::find_path(&graph, a, b);
        assert!(path.is_none());
    }

    #[test]
    fn test_dijkstra_same_start_goal() {
        let mut graph = RoutingGraph::new();
        let a = graph.add_node(Point::new(0, 0));

        let path = DijkstraPathfinder::find_path(&graph, a, a).unwrap();
        assert_eq!(path.nodes.len(), 1);
        assert_eq!(path.total_cost, 0);
    }
}
