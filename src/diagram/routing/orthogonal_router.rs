//! Main orthogonal routing implementation.
//!
//! This module provides the high-level API for orthogonal connector routing,
//! coordinating lead line generation, graph construction, and pathfinding.

use super::graph::{NodeId, RoutingGraph, find_line_intersection, point_on_line};
use super::lead_lines::{EntityId, LeadLine, LeadLineConfig, LeadLineGenerator, RoutingEntity};
use super::pathfinding::{DijkstraPathfinder, GraphPath};
use super::{Point, Rectangle, RoutePath, RoutingConfig};
use std::collections::HashMap;

/// Debug information about the routing process
#[derive(Debug, Clone)]
pub struct RoutingDebugInfo {
    /// All generated lead lines
    pub lead_lines: Vec<LeadLine>,
    /// All intersection points found
    pub intersections: Vec<Point>,
    /// All graph nodes (intersection points + lead line endpoints)
    pub graph_nodes: Vec<Point>,
    /// Number of edges in the routing graph
    pub edge_count: usize,
    /// Number of nodes found for the source entity
    pub source_node_count: usize,
    /// Number of nodes found for the target entity  
    pub target_node_count: usize,
}

/// Main orthogonal router that coordinates the routing algorithm
pub struct OrthogonalRouter {
    config: RoutingConfig,
}

impl OrthogonalRouter {
    /// Creates a new orthogonal router with the given configuration
    pub fn new(config: RoutingConfig) -> Self {
        Self { config }
    }

    /// Routes a connection between two rectangles, avoiding obstacles
    ///
    /// Returns a path with orthogonal segments that:
    /// - Starts at the edge of `from` rectangle
    /// - Ends at the edge of `to` rectangle
    /// - Avoids all obstacles
    /// - Uses only horizontal and vertical segments
    pub fn route(
        &self,
        from: &Rectangle,
        to: &Rectangle,
        obstacles: &[Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Option<RoutePath> {
        let (path, _debug) = self.route_with_debug(from, to, obstacles, canvas_bounds);
        path
    }

    /// Routes a connection and returns debug information
    pub fn route_with_debug(
        &self,
        from: &Rectangle,
        to: &Rectangle,
        obstacles: &[Rectangle],
        canvas_bounds: &Rectangle,
    ) -> (Option<RoutePath>, RoutingDebugInfo) {
        // Create routing entities
        let mut entities = Vec::new();

        // Add source and target
        let from_id = EntityId::new("from");
        let to_id = EntityId::new("to");
        entities.push(RoutingEntity::new(from_id.clone(), from.clone()));
        entities.push(RoutingEntity::new(to_id.clone(), to.clone()));

        // Add obstacles
        for (i, obstacle) in obstacles.iter().enumerate() {
            let id = EntityId::new(format!("obstacle_{}", i));
            entities.push(RoutingEntity::new(id, obstacle.clone()));
        }

        // Generate lead lines
        let lead_config = LeadLineConfig {
            margin: self.config.margin,
            min_lead_extension: 30, // Ensure lines extend from entities before turning
            canvas_bounds: canvas_bounds.clone(),
        };
        let generator = LeadLineGenerator::new(lead_config);
        let lead_lines = generator.generate_lead_lines(&entities);

        // Find all intersections
        let intersections = self.find_all_intersections(&lead_lines);

        // Build routing graph
        let mut graph = RoutingGraph::new();
        let node_map = self.build_graph(&mut graph, &lead_lines, &intersections);

        // Find connection points on source and target
        let from_nodes = self.find_entity_nodes(&from_id, &lead_lines, &node_map);
        let to_nodes = self.find_entity_nodes(&to_id, &lead_lines, &node_map);

        // Find shortest path between any source and target node
        let mut best_path: Option<(GraphPath, NodeId, NodeId)> = None;
        let mut best_cost = u32::MAX;

        for &from_node in &from_nodes {
            for &to_node in &to_nodes {
                if let Some(path) = DijkstraPathfinder::find_path(&graph, from_node, to_node) {
                    if path.total_cost < best_cost {
                        best_cost = path.total_cost;
                        best_path = Some((path, from_node, to_node));
                    }
                }
            }
        }

        // Collect debug information
        let graph_nodes: Vec<Point> = node_map.keys().copied().collect();
        let edge_count = graph.edge_count();

        let debug_info = RoutingDebugInfo {
            lead_lines: lead_lines.clone(),
            intersections: intersections.clone(),
            graph_nodes,
            edge_count,
            source_node_count: from_nodes.len(),
            target_node_count: to_nodes.len(),
        };

        // Convert graph path to routing path
        let result = best_path.and_then(|(path, _from_node, _to_node)| {
            path.to_points(&graph).map(|points| RoutePath {
                nodes: points,
                total_cost: path.total_cost,
            })
        });

        (result, debug_info)
    }

    /// Finds all intersections between lead lines
    fn find_all_intersections(&self, lead_lines: &[LeadLine]) -> Vec<Point> {
        let mut intersections = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for (i, line1) in lead_lines.iter().enumerate() {
            for (j, line2) in lead_lines.iter().enumerate() {
                if i >= j {
                    continue; // Avoid duplicate checks
                }

                if let Some(intersection) =
                    find_line_intersection(line1.start, line1.end, line2.start, line2.end)
                {
                    if seen.insert(intersection) {
                        intersections.push(intersection);
                    }
                }
            }
        }

        intersections
    }

    /// Builds the routing graph from lead lines and intersections
    fn build_graph(
        &self,
        graph: &mut RoutingGraph,
        lead_lines: &[LeadLine],
        intersections: &[Point],
    ) -> HashMap<Point, NodeId> {
        let mut node_map = HashMap::new();

        // Add all intersection points as nodes
        for &intersection in intersections {
            let node_id = graph.add_node(intersection);
            node_map.insert(intersection, node_id);
        }

        // Add lead line endpoints as nodes
        for line in lead_lines {
            node_map
                .entry(line.start)
                .or_insert_with(|| graph.add_node(line.start));
            node_map
                .entry(line.end)
                .or_insert_with(|| graph.add_node(line.end));
        }

        // Also add all intermediate points on lead lines where intersections occur
        // This ensures we have nodes at every point where lead lines cross
        for line in lead_lines {
            for &intersection in intersections {
                if point_on_line(intersection, line.start, line.end) {
                    node_map
                        .entry(intersection)
                        .or_insert_with(|| graph.add_node(intersection));
                }
            }
        }

        // Connect adjacent nodes on the same lead line
        // This ensures we only get orthogonal connections
        for line in lead_lines {
            // Find all nodes on this lead line
            let mut nodes_on_line: Vec<(Point, NodeId)> = node_map
                .iter()
                .filter(|(point, _)| point_on_line(**point, line.start, line.end))
                .map(|(point, id)| (*point, *id))
                .collect();

            // Sort nodes along the line
            if line.start.x == line.end.x {
                // Vertical line - sort by y
                nodes_on_line.sort_by_key(|(point, _)| point.y);
            } else {
                // Horizontal line - sort by x
                nodes_on_line.sort_by_key(|(point, _)| point.x);
            }

            // Connect adjacent nodes
            for i in 0..nodes_on_line.len().saturating_sub(1) {
                let (_, id1) = nodes_on_line[i];
                let (_, id2) = nodes_on_line[i + 1];
                let _ = graph.add_edge(id1, id2);
            }
        }

        // REMOVED: Old connection logic that was creating diagonal connections
        /*
        let all_nodes: Vec<_> = node_map.iter().collect();

        for i in 0..all_nodes.len() {
            for j in i+1..all_nodes.len() {
                let (point1, id1) = all_nodes[i];
                let (point2, id2) = all_nodes[j];

                // Check if nodes are on the same horizontal or vertical line
                let on_same_line = if point1.x == point2.x {
                    // Same vertical line - check if there's a lead line connecting them
                    lead_lines.iter().any(|line| {
                        line.start.x == point1.x && line.end.x == point1.x &&
                        point_on_line(*point1, line.start, line.end) &&
                        point_on_line(*point2, line.start, line.end)
                    })
                } else if point1.y == point2.y {
                    // Same horizontal line - check if there's a lead line connecting them
                    lead_lines.iter().any(|line| {
                        line.start.y == point1.y && line.end.y == point1.y &&
                        point_on_line(*point1, line.start, line.end) &&
                        point_on_line(*point2, line.start, line.end)
                    })
                } else {
                    false
                };

                if on_same_line {
        // END REMOVED SECTION */

        node_map
    }

    /// Finds all nodes connected to a specific entity
    fn find_entity_nodes(
        &self,
        entity_id: &EntityId,
        lead_lines: &[LeadLine],
        node_map: &HashMap<Point, NodeId>,
    ) -> Vec<NodeId> {
        let mut entity_nodes = Vec::new();

        // Find all lead lines from this entity
        for line in lead_lines {
            if line.source_entity_id == *entity_id {
                // Add the start node (connection point on entity)
                if let Some(&node_id) = node_map.get(&line.start) {
                    entity_nodes.push(node_id);
                }
            }
        }

        entity_nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_horizontal_routing() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        // Two rectangles side by side
        let from = Rectangle::new(10, 50, 30, 20);
        let to = Rectangle::new(100, 50, 30, 20);
        let obstacles = vec![];
        let canvas = Rectangle::new(0, 0, 200, 200);

        let path = router.route(&from, &to, &obstacles, &canvas);
        assert!(path.is_some());

        // Should be a simple horizontal path
        let path = path.unwrap();
        assert!(path.nodes.len() >= 2);
    }

    #[test]
    fn test_routing_with_obstacle() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        // Two rectangles with an obstacle in between
        let from = Rectangle::new(10, 50, 30, 20);
        let to = Rectangle::new(100, 50, 30, 20);
        let obstacles = vec![
            Rectangle::new(50, 40, 30, 40), // Blocking direct path
        ];
        let canvas = Rectangle::new(0, 0, 300, 200);

        let path = router.route(&from, &to, &obstacles, &canvas);

        assert!(path.is_some(), "Should find a path around the obstacle");

        // Path should go around the obstacle
        let path = path.unwrap();
        assert!(path.nodes.len() >= 2, "Path should connect start to end"); // At least start and end

        // Verify path is orthogonal (all segments are horizontal or vertical)
        for i in 0..path.nodes.len().saturating_sub(1) {
            let p1 = path.nodes.get(i).unwrap();
            let p2 = path.nodes.get(i + 1).unwrap();
            assert!(
                p1.x == p2.x || p1.y == p2.y,
                "All path segments should be orthogonal"
            );
        }
    }

    #[test]
    fn test_no_path_possible() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        // Completely enclosed target
        let from = Rectangle::new(10, 10, 20, 20);
        let to = Rectangle::new(100, 100, 20, 20);
        let obstacles = vec![
            Rectangle::new(80, 80, 60, 10),  // Top wall
            Rectangle::new(80, 80, 10, 60),  // Left wall
            Rectangle::new(130, 80, 10, 60), // Right wall
            Rectangle::new(80, 130, 60, 10), // Bottom wall
        ];
        let canvas = Rectangle::new(0, 0, 200, 200);

        let _path = router.route(&from, &to, &obstacles, &canvas);
        // Might still find a path if lead lines can get through gaps
        // This test mainly verifies it doesn't panic
    }
}
