//! Core orthogonal connector routing algorithm implementation.
//!
//! This module implements the three-stage orthogonal routing algorithm:
//! 1. Lead line generation from entity edges and centers
//! 2. Intersection detection to create routing graph nodes  
//! 3. Dijkstra pathfinding through the routing graph

use super::{Direction, LeadLine, Point, Rectangle, RoutePath, RoutingConfig, RoutingGraph};
use crate::infrastructure::types::NonEmpty;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// The complete orthogonal connector routing algorithm.
pub struct OrthogonalRouter {
    config: RoutingConfig,
}

impl OrthogonalRouter {
    /// Creates a new orthogonal router with the given configuration.
    pub fn new(config: RoutingConfig) -> Self {
        Self { config }
    }

    /// Routes a connection between two entities, avoiding all other entities.
    ///
    /// This is the main entry point for the routing algorithm.
    pub fn route(
        &self,
        from: &Rectangle,
        to: &Rectangle,
        obstacles: &[Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Option<RoutePath> {
        // Stage 1: Generate lead lines from all entities
        let lead_lines = self.generate_lead_lines(from, to, obstacles, canvas_bounds);

        // Stage 2: Build routing graph from lead line intersections
        let graph = self.build_routing_graph(&lead_lines);

        // Stage 3: Find optimal path using Dijkstra's algorithm
        self.find_optimal_path(&graph, from, to)
    }

    /// Stage 1: Generate lead lines extending from entity edges and centers.
    ///
    /// Lead lines are horizontal and vertical lines that extend from:
    /// - Entity edge centers (top, right, bottom, left)
    /// - Entity corners
    /// - Entity centers
    ///
    /// Lines extend until they hit another entity or the canvas boundary.
    fn generate_lead_lines(
        &self,
        from: &Rectangle,
        to: &Rectangle,
        obstacles: &[Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Vec<LeadLine> {
        let mut lead_lines = Vec::new();
        let mut all_entities = vec![from, to];
        all_entities.extend(obstacles.iter());

        for entity in &all_entities {
            // Generate lines from edge centers
            lead_lines.extend(self.generate_lines_from_edge_centers(
                entity,
                &all_entities,
                canvas_bounds,
            ));

            // Generate lines from entity center
            lead_lines.extend(self.generate_lines_from_center(
                entity,
                &all_entities,
                canvas_bounds,
            ));
        }

        lead_lines
    }

    /// Generate lead lines from the four edge centers of an entity.
    fn generate_lines_from_edge_centers(
        &self,
        entity: &Rectangle,
        all_entities: &[&Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Vec<LeadLine> {
        let edge_centers = entity.edge_centers();
        let mut lines = Vec::new();

        // Top center - extend North and South
        let top_center = edge_centers[0];
        lines.extend(self.extend_line_in_directions(
            top_center,
            &[Direction::North, Direction::South],
            all_entities,
            canvas_bounds,
        ));

        // Right center - extend East and West
        let right_center = edge_centers[1];
        lines.extend(self.extend_line_in_directions(
            right_center,
            &[Direction::East, Direction::West],
            all_entities,
            canvas_bounds,
        ));

        // Bottom center - extend North and South
        let bottom_center = edge_centers[2];
        lines.extend(self.extend_line_in_directions(
            bottom_center,
            &[Direction::North, Direction::South],
            all_entities,
            canvas_bounds,
        ));

        // Left center - extend East and West
        let left_center = edge_centers[3];
        lines.extend(self.extend_line_in_directions(
            left_center,
            &[Direction::East, Direction::West],
            all_entities,
            canvas_bounds,
        ));

        lines
    }

    /// Generate lead lines from the center of an entity.
    fn generate_lines_from_center(
        &self,
        entity: &Rectangle,
        all_entities: &[&Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Vec<LeadLine> {
        let center = entity.center();
        self.extend_line_in_directions(
            center,
            &[
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ],
            all_entities,
            canvas_bounds,
        )
    }

    /// Extend lines from a point in the given directions until hitting obstacles or boundaries.
    fn extend_line_in_directions(
        &self,
        start: Point,
        directions: &[Direction],
        all_entities: &[&Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Vec<LeadLine> {
        directions
            .iter()
            .map(|&direction| {
                self.extend_line_in_direction(start, direction, all_entities, canvas_bounds)
            })
            .collect()
    }

    /// Extend a single line from a point in a direction until hitting an obstacle or boundary.
    fn extend_line_in_direction(
        &self,
        start: Point,
        direction: Direction,
        all_entities: &[&Rectangle],
        canvas_bounds: &Rectangle,
    ) -> LeadLine {
        let end = self.find_line_termination(start, direction, all_entities, canvas_bounds);
        LeadLine::new(start, direction, end)
    }

    /// Find where a line terminates due to collision with entity or canvas boundary.
    fn find_line_termination(
        &self,
        start: Point,
        direction: Direction,
        all_entities: &[&Rectangle],
        canvas_bounds: &Rectangle,
    ) -> Point {
        let max_distance = self.config.max_lead_line_length;
        let margin = self.config.entity_margin;

        // Calculate theoretical end point at maximum distance
        let theoretical_end = match direction {
            Direction::North => Point::new(start.x, start.y.saturating_sub(max_distance)),
            Direction::South => Point::new(start.x, start.y + max_distance),
            Direction::East => Point::new(start.x + max_distance, start.y),
            Direction::West => Point::new(start.x.saturating_sub(max_distance), start.y),
        };

        // Find the closest collision point
        let mut closest_collision = theoretical_end;
        let mut closest_distance = max_distance;

        // Check collisions with canvas bounds
        let canvas_collision = self.find_canvas_collision(start, direction, canvas_bounds);
        let canvas_distance = start.manhattan_distance(&canvas_collision);
        if canvas_distance < closest_distance {
            closest_distance = canvas_distance;
            closest_collision = canvas_collision;
        }

        // Check collisions with entities (including margin)
        for entity in all_entities {
            if let Some(collision) = self.find_entity_collision(start, direction, entity, margin) {
                let distance = start.manhattan_distance(&collision);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_collision = collision;
                }
            }
        }

        closest_collision
    }

    /// Find where a line collides with the canvas boundary.
    fn find_canvas_collision(
        &self,
        start: Point,
        direction: Direction,
        canvas_bounds: &Rectangle,
    ) -> Point {
        match direction {
            Direction::North => Point::new(start.x, canvas_bounds.y),
            Direction::South => Point::new(start.x, canvas_bounds.y + canvas_bounds.height),
            Direction::East => Point::new(canvas_bounds.x + canvas_bounds.width, start.y),
            Direction::West => Point::new(canvas_bounds.x, start.y),
        }
    }

    /// Find where a line collides with an entity (including margin).
    fn find_entity_collision(
        &self,
        start: Point,
        direction: Direction,
        entity: &Rectangle,
        margin: u32,
    ) -> Option<Point> {
        // Expand entity bounds by margin
        let expanded = Rectangle::new(
            entity.x.saturating_sub(margin),
            entity.y.saturating_sub(margin),
            entity.width + 2 * margin,
            entity.height + 2 * margin,
        );

        // Skip if start point is inside the entity (don't collide with self)
        if expanded.contains(&start) {
            return None;
        }

        // Check if line intersects with expanded entity
        match direction {
            Direction::North => {
                if start.x >= expanded.x
                    && start.x <= expanded.x + expanded.width
                    && start.y > expanded.y + expanded.height
                {
                    Some(Point::new(start.x, expanded.y + expanded.height))
                } else {
                    None
                }
            }
            Direction::South => {
                if start.x >= expanded.x
                    && start.x <= expanded.x + expanded.width
                    && start.y < expanded.y
                {
                    Some(Point::new(start.x, expanded.y))
                } else {
                    None
                }
            }
            Direction::East => {
                if start.y >= expanded.y
                    && start.y <= expanded.y + expanded.height
                    && start.x < expanded.x
                {
                    Some(Point::new(expanded.x, start.y))
                } else {
                    None
                }
            }
            Direction::West => {
                if start.y >= expanded.y
                    && start.y <= expanded.y + expanded.height
                    && start.x > expanded.x + expanded.width
                {
                    Some(Point::new(expanded.x + expanded.width, start.y))
                } else {
                    None
                }
            }
        }
    }

    /// Stage 2: Build routing graph from lead line intersections.
    fn build_routing_graph(&self, lead_lines: &[LeadLine]) -> RoutingGraph {
        let mut graph = RoutingGraph::new();
        let mut point_to_node: HashMap<Point, usize> = HashMap::new();

        // Find all intersection points
        let intersections = self.find_all_intersections(lead_lines);

        // Create nodes for all intersection points
        for intersection in intersections {
            point_to_node
                .entry(intersection)
                .or_insert_with(|| graph.add_node(intersection));
        }

        // Create edges between nodes that are connected along lead lines
        self.add_edges_along_lead_lines(&mut graph, lead_lines, &point_to_node);

        graph
    }

    /// Find all intersection points between lead lines.
    fn find_all_intersections(&self, lead_lines: &[LeadLine]) -> HashSet<Point> {
        let mut intersections = HashSet::new();

        // Add start and end points of all lead lines
        for line in lead_lines {
            intersections.insert(line.start);
            intersections.insert(line.end);
        }

        // Find intersection points between different lead lines
        for i in 0..lead_lines.len() {
            for j in i + 1..lead_lines.len() {
                if let Some(intersection) = lead_lines[i].intersection(&lead_lines[j]) {
                    intersections.insert(intersection);
                }
            }
        }

        intersections
    }

    /// Add edges between nodes that are directly connected along lead lines.
    fn add_edges_along_lead_lines(
        &self,
        graph: &mut RoutingGraph,
        lead_lines: &[LeadLine],
        point_to_node: &HashMap<Point, usize>,
    ) {
        for line in lead_lines {
            // Find all points that lie on this lead line
            let mut points_on_line: Vec<Point> = point_to_node
                .keys()
                .filter(|&&point| line.contains_point(&point))
                .copied()
                .collect();

            // Sort points along the line direction
            self.sort_points_along_line(&mut points_on_line, line.direction);

            // Add edges between consecutive points
            for i in 0..points_on_line.len().saturating_sub(1) {
                let from_point = points_on_line[i];
                let to_point = points_on_line[i + 1];
                let weight = from_point.manhattan_distance(&to_point);

                if let (Some(&from_id), Some(&to_id)) =
                    (point_to_node.get(&from_point), point_to_node.get(&to_point))
                {
                    graph.add_edge(from_id, to_id, weight);
                }
            }
        }
    }

    /// Sort points along a line in the direction of the line.
    fn sort_points_along_line(&self, points: &mut [Point], direction: Direction) {
        match direction {
            Direction::North => points.sort_by_key(|p| std::cmp::Reverse(p.y)),
            Direction::South => points.sort_by_key(|p| p.y),
            Direction::East => points.sort_by_key(|p| p.x),
            Direction::West => points.sort_by_key(|p| std::cmp::Reverse(p.x)),
        }
    }

    /// Stage 3: Find optimal path using Dijkstra's algorithm.
    fn find_optimal_path(
        &self,
        graph: &RoutingGraph,
        from: &Rectangle,
        to: &Rectangle,
    ) -> Option<RoutePath> {
        // Find start and end nodes on the edges of entities, not centers
        // This ensures the path starts and ends properly outside the entities
        let start_nodes = self.find_edge_nodes(graph, from);
        let end_nodes = self.find_edge_nodes(graph, to);

        if start_nodes.is_empty() || end_nodes.is_empty() {
            // If no edge nodes found, fall back to closest nodes to centers
            let start_point = from.center();
            let end_point = to.center();

            let start_node = self.find_closest_node(graph, start_point)?;
            let end_node = self.find_closest_node(graph, end_point)?;

            let path_nodes = self.dijkstra(graph, start_node, end_node)?;

            // Convert node IDs to points
            let points: Vec<Point> = path_nodes
                .iter()
                .filter_map(|&node_id| graph.nodes.get(&node_id).map(|node| node.position))
                .collect();

            if points.is_empty() {
                return None;
            }

            // Create NonEmpty from points
            let first = points[0];
            let rest = points.into_iter().skip(1).collect();
            let non_empty_points = NonEmpty::from_head_and_tail(first, rest);

            // Calculate total cost
            let total_cost = self.calculate_path_cost(graph, &path_nodes);

            return Some(RoutePath::new(non_empty_points, total_cost));
        }

        // Find the best path from any start node to any end node
        let mut best_path = None;
        let mut best_cost = u32::MAX;

        for &start_node in &start_nodes {
            for &end_node in &end_nodes {
                if let Some(path_nodes) = self.dijkstra(graph, start_node, end_node) {
                    let cost = self.calculate_path_cost(graph, &path_nodes);
                    if cost < best_cost {
                        best_cost = cost;
                        best_path = Some(path_nodes);
                    }
                }
            }
        }

        let path_nodes = best_path?;

        // Convert node IDs to points
        let points: Vec<Point> = path_nodes
            .into_iter()
            .filter_map(|node_id| graph.nodes.get(&node_id).map(|node| node.position))
            .collect();

        if points.is_empty() {
            return None;
        }

        // Create NonEmpty from points
        let first = points[0];
        let rest = points.into_iter().skip(1).collect();
        let non_empty_points = NonEmpty::from_head_and_tail(first, rest);

        // Calculate total cost
        let total_cost = non_empty_points
            .iter()
            .zip(non_empty_points.iter().skip(1))
            .map(|(a, b)| a.manhattan_distance(b))
            .sum();

        Some(RoutePath::new(non_empty_points, total_cost))
    }

    /// Find the closest graph node to a given point.
    fn find_closest_node(&self, graph: &RoutingGraph, point: Point) -> Option<usize> {
        graph
            .nodes
            .iter()
            .min_by_key(|(_, node)| node.position.manhattan_distance(&point))
            .map(|(&id, _)| id)
    }

    /// Find nodes that are on or near the edges of an entity.
    fn find_edge_nodes(&self, graph: &RoutingGraph, entity: &Rectangle) -> Vec<usize> {
        let margin = self.config.entity_margin;
        let expanded = Rectangle::new(
            entity.x.saturating_sub(margin),
            entity.y.saturating_sub(margin),
            entity.width + 2 * margin,
            entity.height + 2 * margin,
        );

        graph
            .nodes
            .iter()
            .filter_map(|(&id, node)| {
                let pos = node.position;
                // Check if node is on the perimeter of the expanded entity
                let on_edge = (pos.x == expanded.x || pos.x == expanded.x + expanded.width)
                    && pos.y >= expanded.y
                    && pos.y <= expanded.y + expanded.height
                    || (pos.y == expanded.y || pos.y == expanded.y + expanded.height)
                        && pos.x >= expanded.x
                        && pos.x <= expanded.x + expanded.width;

                if on_edge { Some(id) } else { None }
            })
            .collect()
    }

    /// Calculate the total cost of a path through the graph.
    fn calculate_path_cost(&self, graph: &RoutingGraph, path_nodes: &[usize]) -> u32 {
        path_nodes
            .windows(2)
            .map(|window| {
                let from = &graph.nodes[&window[0]];
                let to = &graph.nodes[&window[1]];
                from.position.manhattan_distance(&to.position)
            })
            .sum()
    }

    /// Dijkstra's shortest path algorithm.
    fn dijkstra(&self, graph: &RoutingGraph, start: usize, end: usize) -> Option<Vec<usize>> {
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct State {
            cost: u32,
            node: usize,
        }

        impl Ord for State {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other.cost.cmp(&self.cost) // Reverse for min-heap
            }
        }

        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap = BinaryHeap::new();
        let mut distances: HashMap<usize, u32> = HashMap::new();
        let mut predecessors: HashMap<usize, usize> = HashMap::new();

        // Initialize
        distances.insert(start, 0);
        heap.push(State {
            cost: 0,
            node: start,
        });

        while let Some(State { cost, node }) = heap.pop() {
            if node == end {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = end;
                path.push(current);

                while let Some(&pred) = predecessors.get(&current) {
                    path.push(pred);
                    current = pred;
                }

                path.reverse();
                return Some(path);
            }

            if cost > *distances.get(&node).unwrap_or(&u32::MAX) {
                continue;
            }

            // Explore neighbors
            for (neighbor, edge_weight) in graph.neighbors(node) {
                let new_cost = cost + edge_weight;
                let current_distance = *distances.get(&neighbor).unwrap_or(&u32::MAX);

                if new_cost < current_distance {
                    distances.insert(neighbor, new_cost);
                    predecessors.insert(neighbor, node);
                    heap.push(State {
                        cost: new_cost,
                        node: neighbor,
                    });
                }
            }
        }

        None // No path found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_line_generation() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        let entity = Rectangle::new(10, 10, 20, 15);
        let canvas = Rectangle::new(0, 0, 100, 100);
        let _obstacles: Vec<&Rectangle> = vec![];

        let lines = router.generate_lines_from_center(&entity, &[&entity], &canvas);

        // Should generate 4 lines (N, S, E, W) from center
        assert_eq!(lines.len(), 4);

        // All lines should start from entity center
        let center = entity.center();
        for line in &lines {
            assert_eq!(line.start, center);
        }
    }

    #[test]
    fn test_full_routing_algorithm() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        let from = Rectangle::new(10, 10, 20, 15);
        let to = Rectangle::new(50, 30, 20, 15);
        let obstacles = vec![];
        let canvas = Rectangle::new(0, 0, 100, 100);

        // Test Stage 1: Lead line generation
        let lead_lines = router.generate_lead_lines(&from, &to, &obstacles, &canvas);
        assert!(!lead_lines.is_empty(), "Should generate lead lines");

        // Test Stage 2: Graph building
        let graph = router.build_routing_graph(&lead_lines);
        assert!(!graph.nodes.is_empty(), "Graph should have nodes");

        // Test Stage 3: Pathfinding
        let route = router.find_optimal_path(&graph, &from, &to);
        assert!(route.is_some(), "Should find a path");
    }

    #[test]
    fn test_line_collision_detection() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        let start = Point::new(15, 5);
        let entity = Rectangle::new(10, 10, 20, 15);

        // Line going south should hit entity
        let collision = router.find_entity_collision(start, Direction::South, &entity, 0);
        assert_eq!(collision, Some(Point::new(15, 10)));

        // Line going north should not hit entity
        let no_collision = router.find_entity_collision(start, Direction::North, &entity, 0);
        assert_eq!(no_collision, None);
    }

    #[test]
    fn test_intersection_finding() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        let vertical = LeadLine::new(Point::new(10, 5), Direction::South, Point::new(10, 15));
        let horizontal = LeadLine::new(Point::new(5, 10), Direction::East, Point::new(15, 10));

        let intersections = router.find_all_intersections(&[vertical, horizontal]);

        // Should find intersection point plus start/end points
        assert!(intersections.contains(&Point::new(10, 10)));
        assert!(intersections.contains(&Point::new(10, 5)));
        assert!(intersections.contains(&Point::new(10, 15)));
        assert!(intersections.contains(&Point::new(5, 10)));
        assert!(intersections.contains(&Point::new(15, 10)));
    }
}
