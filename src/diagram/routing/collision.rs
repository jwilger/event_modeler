//! Collision detection for lead line generation.
//!
//! This module handles detecting collisions between lead lines and entities,
//! ensuring that lead lines stop when they hit obstacles.

use super::lead_lines::{EntityId, LeadDirection};
use super::{Point, Rectangle};

/// Data structure for tracking collision detection state
#[derive(Debug, Clone)]
pub struct CollisionData {
    /// Starting point coordinate (X for horizontal, Y for vertical)
    pub start_point: u32,
    /// Start X coordinate
    pub start_x: u32,
    /// Start Y coordinate
    pub start_y: u32,
    /// End X coordinate
    pub end_x: u32,
    /// End Y coordinate
    pub end_y: u32,
    /// Maximum coordinate (canvas boundary)
    pub maximum: u32,
    /// Opposite side coordinate for center lines
    pub opposite_side: Option<u32>,
    /// Whether this is a vertical line
    pub is_vertical: bool,
}

/// A detected collision range
#[derive(Debug, Clone, Copy)]
pub struct CollisionRange {
    /// Start of the collision range
    pub start: u32,
    /// End of the collision range  
    pub end: u32,
}

impl CollisionRange {
    /// Creates a new collision range
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

/// Detects collisions between a lead line and entities
pub struct CollisionDetector {
    margin: u32,
}

impl CollisionDetector {
    /// Creates a new collision detector
    pub fn new(margin: u32) -> Self {
        Self { margin }
    }

    /// Detects all entities that would collide with a lead line
    pub fn detect_collisions(
        &self,
        direction: LeadDirection,
        start: Point,
        entities: &[(EntityId, Rectangle)],
        source_entity_id: &EntityId,
    ) -> Vec<(EntityId, CollisionRange)> {
        let mut collisions = Vec::new();

        for (entity_id, bounds) in entities {
            // Skip the source entity
            if entity_id == source_entity_id {
                continue;
            }

            // Expand bounds by margin
            let expanded = Rectangle::new(
                bounds.x.saturating_sub(self.margin),
                bounds.y.saturating_sub(self.margin),
                bounds.width + 2 * self.margin,
                bounds.height + 2 * self.margin,
            );

            // Check if this entity is in the path of the lead line
            if let Some(range) = self.get_collision_range(direction, start, &expanded) {
                collisions.push((entity_id.clone(), range));
            }
        }

        collisions
    }

    /// Gets the collision range for an entity in the path of a lead line
    fn get_collision_range(
        &self,
        direction: LeadDirection,
        start: Point,
        bounds: &Rectangle,
    ) -> Option<CollisionRange> {
        match direction {
            LeadDirection::North => {
                // Moving up - check if entity intersects our vertical line and is in our path
                let x_intersects = start.x >= bounds.x && start.x <= bounds.right();
                let in_path = bounds.y < start.y;

                if x_intersects && in_path {
                    // Entity is above us and intersects our vertical line
                    Some(CollisionRange::new(bounds.y, bounds.bottom()))
                } else {
                    None
                }
            }
            LeadDirection::South => {
                // Moving down - check if entity intersects our vertical line and is in our path
                if start.x >= bounds.x && start.x <= bounds.right() && bounds.bottom() > start.y {
                    // Entity is below us and intersects our vertical line
                    Some(CollisionRange::new(bounds.y, bounds.bottom()))
                } else {
                    None
                }
            }
            LeadDirection::East => {
                // Moving right - check if entity intersects our horizontal line and is in our path
                if start.y >= bounds.y && start.y <= bounds.bottom() && bounds.x > start.x {
                    // Entity is to our right and intersects our horizontal line
                    Some(CollisionRange::new(bounds.x, bounds.right()))
                } else {
                    None
                }
            }
            LeadDirection::West => {
                // Moving left - check if entity intersects our horizontal line and is in our path
                if start.y >= bounds.y && start.y <= bounds.bottom() && bounds.right() < start.x {
                    // Entity is to our left and intersects our horizontal line
                    Some(CollisionRange::new(bounds.x, bounds.right()))
                } else {
                    None
                }
            }
        }
    }

    /// Processes collision ranges to determine lead line segments
    pub fn process_collisions(
        &self,
        data: &CollisionData,
        collisions: &[CollisionRange],
    ) -> Vec<(Point, Point)> {
        let mut segments = Vec::new();

        if collisions.is_empty() {
            // No collisions - full line
            segments.push((
                Point::new(data.start_x, data.start_y),
                Point::new(data.end_x, data.end_y),
            ));
        } else {
            // For now, just stop at the first collision
            let closest_collision = collisions
                .iter()
                .map(|range| {
                    if data.is_vertical {
                        // For vertical lines, we care about Y coordinates
                        if data.start_y > range.end {
                            // Moving up, collision is below us
                            range.end
                        } else {
                            // Moving down, collision is above us
                            range.start
                        }
                    } else {
                        // For horizontal lines, we care about X coordinates
                        if data.start_x > range.end {
                            // Moving left, collision is to our right
                            range.end
                        } else {
                            // Moving right, collision is to our left
                            range.start
                        }
                    }
                })
                .min_by_key(|&coord| {
                    if data.is_vertical {
                        if coord > data.start_y {
                            coord - data.start_y
                        } else {
                            data.start_y - coord
                        }
                    } else if coord > data.start_x {
                        coord - data.start_x
                    } else {
                        data.start_x - coord
                    }
                });

            if let Some(collision_coord) = closest_collision {
                if data.is_vertical {
                    segments.push((
                        Point::new(data.start_x, data.start_y),
                        Point::new(data.start_x, collision_coord),
                    ));
                } else {
                    segments.push((
                        Point::new(data.start_x, data.start_y),
                        Point::new(collision_coord, data.start_y),
                    ));
                }
            } else {
                // No valid collision found, use full line
                segments.push((
                    Point::new(data.start_x, data.start_y),
                    Point::new(data.end_x, data.end_y),
                ));
            }
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_detection_horizontal() {
        let detector = CollisionDetector::new(10);
        let start = Point::new(50, 100);

        let entities = vec![(EntityId::new("obstacle1"), Rectangle::new(100, 90, 30, 20))];

        let collisions = detector.detect_collisions(
            LeadDirection::East,
            start,
            &entities,
            &EntityId::new("source"),
        );

        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].1.start, 90); // With margin
        assert_eq!(collisions[0].1.end, 140); // With margin
    }

    #[test]
    fn test_no_collision() {
        let detector = CollisionDetector::new(10);
        let start = Point::new(50, 100);

        let entities = vec![
            (EntityId::new("obstacle1"), Rectangle::new(100, 150, 30, 20)), // Below the line
        ];

        let collisions = detector.detect_collisions(
            LeadDirection::East,
            start,
            &entities,
            &EntityId::new("source"),
        );

        assert_eq!(collisions.len(), 0);
    }
}
