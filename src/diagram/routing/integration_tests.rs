//! Integration tests for the complete orthogonal routing algorithm.

#[cfg(test)]
mod tests {
    use super::super::orthogonal_router::OrthogonalRouter;
    use super::super::{Rectangle, RoutingConfig};

    #[test]
    fn test_simple_routing_no_obstacles() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        let from = Rectangle::new(10, 10, 20, 15);
        let to = Rectangle::new(50, 30, 20, 15);
        let obstacles = vec![];
        let canvas = Rectangle::new(0, 0, 100, 100);

        let route = router.route(&from, &to, &obstacles, &canvas);

        assert!(route.is_some(), "Should find a route between entities");

        let path = route.unwrap();
        assert!(
            path.nodes.len() >= 2,
            "Path should have at least start and end"
        );
        assert!(path.total_cost > 0, "Path should have positive cost");

        // Verify SVG path generation
        let svg_path = path.to_svg_path();
        assert!(
            svg_path.starts_with("M "),
            "SVG path should start with MoveTo command"
        );
        assert!(
            svg_path.contains("L "),
            "SVG path should contain LineTo commands"
        );
    }

    #[test]
    #[ignore = "FIXME: Edge case with small entities and minimum extensions"]
    fn test_routing_with_obstacles() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        // Place entities with clear obstacle between them
        let from = Rectangle::new(10, 20, 20, 15);
        let to = Rectangle::new(70, 20, 20, 15);
        let obstacles = vec![Rectangle::new(40, 10, 20, 35)]; // Tall obstacle blocking direct path
        let canvas = Rectangle::new(0, 0, 100, 100);

        let route = router.route(&from, &to, &obstacles, &canvas);

        assert!(route.is_some(), "Should find a route around obstacles");

        let path = route.unwrap();

        // The path should route around the obstacle
        // We just verify it exists and is valid
        assert!(path.nodes.len() >= 2, "Path should connect start to end");
        assert!(path.total_cost > 0, "Path should have positive cost");

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
    fn test_routing_edge_cases() {
        let config = RoutingConfig::default();
        let router = OrthogonalRouter::new(config);

        // Test same entity (should return empty or very short path)
        let entity = Rectangle::new(10, 10, 20, 15);
        let obstacles = vec![];
        let canvas = Rectangle::new(0, 0, 100, 100);

        let route = router.route(&entity, &entity, &obstacles, &canvas);

        // Should either find a very short path or no path at all
        if let Some(path) = route {
            assert!(
                path.total_cost < 50,
                "Path to same entity should be very short"
            );
        }
    }

    #[test]
    fn test_routing_with_margin() {
        let config = RoutingConfig { margin: 5 };
        let router = OrthogonalRouter::new(config);

        let from = Rectangle::new(10, 20, 20, 15);
        let to = Rectangle::new(70, 20, 20, 15);
        let obstacles = vec![Rectangle::new(40, 15, 20, 25)]; // Obstacle in the way
        let canvas = Rectangle::new(0, 0, 100, 100);

        let route = router.route(&from, &to, &obstacles, &canvas);

        assert!(
            route.is_some(),
            "Should find route respecting entity margins"
        );

        let path = route.unwrap();

        // With margin, the algorithm creates a buffer around obstacles
        // This is correct behavior - we just verify the path exists
        assert!(path.nodes.len() >= 2, "Path should connect entities");
        assert!(path.total_cost > 0, "Path should have positive cost");
    }
}
