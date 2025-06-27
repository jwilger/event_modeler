//! Lead line generation for orthogonal routing.
//!
//! Lead lines are the foundation of orthogonal connector routing. They extend
//! from each side and center of entities until they hit obstacles or canvas bounds.

use super::collision::{CollisionData, CollisionDetector};
use super::{Point, Rectangle};

/// A lead line segment that extends from an entity until it hits an obstacle.
///
/// Lead lines form the edges of the routing graph.
#[derive(Debug, Clone, PartialEq)]
pub struct LeadLine {
    /// Starting point of the lead line
    pub start: Point,
    /// Ending point of the lead line (where it hits an obstacle or boundary)
    pub end: Point,
    /// The direction this line extends from its origin
    pub direction: LeadDirection,
    /// The entity this lead line originates from
    pub source_entity_id: EntityId,
}

/// Unique identifier for an entity in the routing system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(String);

impl EntityId {
    /// Creates a new entity ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Gets the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Direction a lead line extends from an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeadDirection {
    /// Extends upward from top edge
    North,
    /// Extends rightward from right edge
    East,
    /// Extends downward from bottom edge
    South,
    /// Extends leftward from left edge
    West,
}

/// Origin point on an entity where a lead line starts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeadOrigin {
    /// Center of top edge
    TopCenter,
    /// Center of right edge
    RightCenter,
    /// Center of bottom edge
    BottomCenter,
    /// Center of left edge
    LeftCenter,
    /// Center of entity extending north
    CenterNorth,
    /// Center of entity extending east
    CenterEast,
    /// Center of entity extending south
    CenterSouth,
    /// Center of entity extending west
    CenterWest,
}

impl LeadOrigin {
    /// Gets the direction this origin extends toward
    pub fn direction(&self) -> LeadDirection {
        match self {
            Self::TopCenter | Self::CenterNorth => LeadDirection::North,
            Self::RightCenter | Self::CenterEast => LeadDirection::East,
            Self::BottomCenter | Self::CenterSouth => LeadDirection::South,
            Self::LeftCenter | Self::CenterWest => LeadDirection::West,
        }
    }

    /// Checks if this origin is from the entity center
    pub fn is_center(&self) -> bool {
        matches!(
            self,
            Self::CenterNorth | Self::CenterEast | Self::CenterSouth | Self::CenterWest
        )
    }

    /// Gets all possible lead origins (only edge centers, not entity center)
    pub fn all() -> [Self; 4] {
        [
            Self::TopCenter,
            Self::RightCenter,
            Self::BottomCenter,
            Self::LeftCenter,
        ]
    }
}

/// An entity that can have lead lines generated from it
#[derive(Debug, Clone)]
pub struct RoutingEntity {
    /// Unique identifier for this entity
    pub id: EntityId,
    /// Bounding box of the entity
    pub bounds: Rectangle,
}

impl RoutingEntity {
    /// Creates a new routing entity
    pub fn new(id: EntityId, bounds: Rectangle) -> Self {
        Self { id, bounds }
    }

    /// Gets the starting point for a lead line from the given origin
    pub fn lead_line_start(&self, origin: LeadOrigin) -> Point {
        let center_x = self.bounds.x + self.bounds.width / 2;
        let center_y = self.bounds.y + self.bounds.height / 2;

        match origin {
            LeadOrigin::TopCenter => Point::new(center_x, self.bounds.y),
            LeadOrigin::RightCenter => Point::new(self.bounds.right(), center_y),
            LeadOrigin::BottomCenter => Point::new(center_x, self.bounds.bottom()),
            LeadOrigin::LeftCenter => Point::new(self.bounds.x, center_y),
            // Center lines start from the entity center
            LeadOrigin::CenterNorth => Point::new(center_x, center_y),
            LeadOrigin::CenterEast => Point::new(center_x, center_y),
            LeadOrigin::CenterSouth => Point::new(center_x, center_y),
            LeadOrigin::CenterWest => Point::new(center_x, center_y),
        }
    }
}

/// Configuration for lead line generation
#[derive(Debug, Clone)]
pub struct LeadLineConfig {
    /// Margin to maintain around entities
    pub margin: u32,
    /// Minimum distance lead lines extend from entity edges before turning
    pub min_lead_extension: u32,
    /// Canvas bounds that lead lines cannot exceed
    pub canvas_bounds: Rectangle,
}

impl Default for LeadLineConfig {
    fn default() -> Self {
        Self {
            margin: 10,
            min_lead_extension: 30,
            canvas_bounds: Rectangle::new(0, 0, 5000, 3000),
        }
    }
}

/// Result of collision detection for a lead line
#[derive(Debug, Clone)]
pub struct CollisionResult {
    /// The point where the collision occurred
    pub collision_point: Point,
    /// The entity that was hit (if any)
    pub hit_entity: Option<EntityId>,
}

/// Generates lead lines for routing
pub struct LeadLineGenerator {
    config: LeadLineConfig,
    collision_detector: CollisionDetector,
}

impl LeadLineGenerator {
    /// Creates a new lead line generator
    pub fn new(config: LeadLineConfig) -> Self {
        let collision_detector = CollisionDetector::new(config.margin);
        Self {
            config,
            collision_detector,
        }
    }

    /// Generates all lead lines for the given entities
    pub fn generate_lead_lines(&self, entities: &[RoutingEntity]) -> Vec<LeadLine> {
        let mut lead_lines = Vec::new();

        // Convert entities to tuple format for collision detection
        let entity_bounds: Vec<(EntityId, Rectangle)> = entities
            .iter()
            .map(|e| (e.id.clone(), e.bounds.clone()))
            .collect();

        for entity in entities {
            for origin in LeadOrigin::all() {
                let lines = self.generate_lead_lines_from_origin(entity, origin, &entity_bounds);
                lead_lines.extend(lines);
            }
        }

        lead_lines
    }

    /// Generates lead lines from a specific origin on an entity
    fn generate_lead_lines_from_origin(
        &self,
        entity: &RoutingEntity,
        origin: LeadOrigin,
        all_entities: &[(EntityId, Rectangle)],
    ) -> Vec<LeadLine> {
        let mut lead_lines = Vec::new();
        let start = entity.lead_line_start(origin);
        let direction = origin.direction();

        // Apply margin offset to start point for edge origins
        // For center origins, we need to start from just outside the entity
        let adjusted_start = if !origin.is_center() {
            self.apply_margin_offset(start, direction, self.config.margin)
        } else {
            // For center lines, start from the edge of the entity + margin
            match direction {
                LeadDirection::North => {
                    Point::new(start.x, entity.bounds.y.saturating_sub(self.config.margin))
                }
                LeadDirection::South => {
                    Point::new(start.x, entity.bounds.bottom() + self.config.margin)
                }
                LeadDirection::East => {
                    Point::new(entity.bounds.right() + self.config.margin, start.y)
                }
                LeadDirection::West => {
                    Point::new(entity.bounds.x.saturating_sub(self.config.margin), start.y)
                }
            }
        };

        // Calculate minimum extension point to ensure proper separation
        let min_extension_point =
            self.apply_margin_offset(adjusted_start, direction, self.config.min_lead_extension);

        // Detect collisions starting from the minimum extension point
        // This ensures lead lines always extend at least the minimum distance
        let collisions = self.collision_detector.detect_collisions(
            direction,
            min_extension_point,
            all_entities,
            &entity.id,
        );

        // Create collision data starting from adjusted_start but considering minimum extension
        let collision_data = self.create_collision_data(entity, origin, adjusted_start);

        // Convert collisions to ranges
        let collision_ranges: Vec<_> = collisions.iter().map(|(_, range)| *range).collect();

        // Create modified collision data that starts from the minimum extension point
        let extended_collision_data = CollisionData {
            start_point: match direction {
                LeadDirection::North | LeadDirection::South => min_extension_point.y,
                LeadDirection::East | LeadDirection::West => min_extension_point.x,
            },
            start_x: min_extension_point.x,
            start_y: min_extension_point.y,
            end_x: collision_data.end_x,
            end_y: collision_data.end_y,
            maximum: collision_data.maximum,
            opposite_side: collision_data.opposite_side,
            is_vertical: collision_data.is_vertical,
        };

        // Process collisions to get line segments starting from minimum extension point
        let segments = self
            .collision_detector
            .process_collisions(&extended_collision_data, &collision_ranges);

        // Always add the minimum extension segment first
        lead_lines.push(LeadLine {
            start: adjusted_start,
            end: min_extension_point,
            direction,
            source_entity_id: entity.id.clone(),
        });

        // Add the collision-based segments (these will be continuous from min_extension_point)
        for (start_point, end_point) in segments {
            if start_point != end_point {
                lead_lines.push(LeadLine {
                    start: start_point,
                    end: end_point,
                    direction,
                    source_entity_id: entity.id.clone(),
                });
            }
        }

        lead_lines
    }

    /// Creates collision data for lead line generation
    fn create_collision_data(
        &self,
        entity: &RoutingEntity,
        origin: LeadOrigin,
        start: Point,
    ) -> CollisionData {
        let direction = origin.direction();
        let is_vertical = matches!(direction, LeadDirection::North | LeadDirection::South);

        let (start_point, _opposite_side) = if origin.is_center() {
            // For center origins, we have an opposite side
            let center = entity.bounds.center();
            match direction {
                LeadDirection::North | LeadDirection::South => {
                    // For vertical center lines, opposite side is the other vertical direction
                    (center.y, Some(center.y))
                }
                LeadDirection::East | LeadDirection::West => {
                    // For horizontal center lines, opposite side is the other horizontal direction
                    (center.x, Some(center.x))
                }
            }
        } else {
            // For edge origins, no opposite side
            match direction {
                LeadDirection::North => (entity.bounds.y, None),
                LeadDirection::South => (entity.bounds.bottom(), None),
                LeadDirection::East => (entity.bounds.right(), None),
                LeadDirection::West => (entity.bounds.x, None),
            }
        };

        // Calculate end coordinates based on canvas bounds
        let (end_x, end_y, maximum) = match direction {
            LeadDirection::North => (
                start.x,
                self.config.canvas_bounds.y,
                self.config.canvas_bounds.bottom(),
            ),
            LeadDirection::South => (
                start.x,
                self.config.canvas_bounds.bottom(),
                self.config.canvas_bounds.bottom(),
            ),
            LeadDirection::East => (
                self.config.canvas_bounds.right(),
                start.y,
                self.config.canvas_bounds.right(),
            ),
            LeadDirection::West => (
                self.config.canvas_bounds.x,
                start.y,
                self.config.canvas_bounds.right(),
            ),
        };

        CollisionData {
            start_point,
            start_x: start.x,
            start_y: start.y,
            end_x,
            end_y,
            maximum,
            opposite_side: None, // Simplified - no opposite side for now
            is_vertical,
        }
    }

    /// Applies margin offset to a point in the given direction
    fn apply_margin_offset(&self, point: Point, direction: LeadDirection, margin: u32) -> Point {
        match direction {
            LeadDirection::North => Point::new(point.x, point.y.saturating_sub(margin)),
            LeadDirection::East => Point::new(point.x + margin, point.y),
            LeadDirection::South => Point::new(point.x, point.y + margin),
            LeadDirection::West => Point::new(point.x.saturating_sub(margin), point.y),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_origin_direction() {
        assert_eq!(LeadOrigin::TopCenter.direction(), LeadDirection::North);
        assert_eq!(LeadOrigin::CenterEast.direction(), LeadDirection::East);
    }

    #[test]
    fn test_entity_lead_line_start() {
        let entity = RoutingEntity::new(EntityId::new("test"), Rectangle::new(100, 100, 50, 30));

        // Test edge centers
        assert_eq!(
            entity.lead_line_start(LeadOrigin::TopCenter),
            Point::new(125, 100)
        );
        assert_eq!(
            entity.lead_line_start(LeadOrigin::RightCenter),
            Point::new(150, 115)
        );
        assert_eq!(
            entity.lead_line_start(LeadOrigin::BottomCenter),
            Point::new(125, 130)
        );
        assert_eq!(
            entity.lead_line_start(LeadOrigin::LeftCenter),
            Point::new(100, 115)
        );

        // Test center origins (all should be entity center)
        let center = Point::new(125, 115);
        assert_eq!(entity.lead_line_start(LeadOrigin::CenterNorth), center);
        assert_eq!(entity.lead_line_start(LeadOrigin::CenterEast), center);
    }
}
