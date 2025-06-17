//! Layout engine for positioning entities in Event Model diagrams.
//!
//! This module handles the computation of positions and dimensions
//! for all visual elements in a diagram, including swimlanes, entities,
//! slices, and connections between entities.

use crate::event_model::diagram::{SliceId, SwimlaneId};
use crate::event_model::entities::EntityId;
use crate::infrastructure::types::{NonNegativeFloat, PositiveFloat, PositiveInt};
use nutype::nutype;
use std::collections::HashMap;

/// Complete layout information for a diagram.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Overall canvas dimensions and settings.
    pub canvas: Canvas,
    /// Layout information for each swimlane.
    pub swimlane_layouts: HashMap<SwimlaneId, SwimlaneLayout>,
    /// Position of each entity within its swimlane.
    pub entity_positions: HashMap<EntityId, EntityPosition>,
    /// Layout information for each slice.
    pub slice_layouts: HashMap<SliceId, SliceLayout>,
    /// Visual connections between entities.
    pub connections: Vec<Connection>,
}

/// Canvas dimensions and settings.
#[derive(Debug, Clone)]
pub struct Canvas {
    /// Total width of the canvas.
    pub width: CanvasWidth,
    /// Total height of the canvas.
    pub height: CanvasHeight,
    /// Padding around the content.
    pub padding: Padding,
}

/// Layout information for a swimlane.
#[derive(Debug, Clone)]
pub struct SwimlaneLayout {
    /// Top-left position of the swimlane.
    pub position: Position,
    /// Width and height of the swimlane.
    pub dimensions: Dimensions,
}

/// Position and size of an entity.
#[derive(Debug, Clone)]
pub struct EntityPosition {
    /// Swimlane containing this entity.
    pub swimlane_id: SwimlaneId,
    /// Position within the swimlane.
    pub position: Position,
    /// Size of the entity box.
    pub dimensions: Dimensions,
}

/// Layout information for a vertical slice.
#[derive(Debug, Clone)]
pub struct SliceLayout {
    /// Horizontal position of the slice.
    pub x_position: XCoordinate,
    /// Width of the slice.
    pub width: Width,
}

/// Visual connection between two entities.
#[derive(Debug, Clone)]
pub struct Connection {
    /// Source entity.
    pub from: EntityId,
    /// Target entity.
    pub to: EntityId,
    /// Path to draw for the connection.
    pub path: ConnectionPath,
    /// Visual style for the connection.
    pub style: ConnectionStyle,
}

/// Path for drawing a connection.
#[derive(Debug, Clone)]
pub struct ConnectionPath {
    /// Points defining the path.
    pub points: Vec<Point>,
}

/// Visual style for connections.
#[derive(Debug, Clone)]
pub enum ConnectionStyle {
    /// Solid line.
    Solid,
    /// Dashed line.
    Dashed,
    /// Dotted line.
    Dotted,
}

/// 2D position in the diagram.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Horizontal coordinate.
    pub x: XCoordinate,
    /// Vertical coordinate.
    pub y: YCoordinate,
}

/// Width and height dimensions.
#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    /// Horizontal size.
    pub width: Width,
    /// Vertical size.
    pub height: Height,
}

/// A point in 2D space.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// X coordinate.
    pub x: XCoordinate,
    /// Y coordinate.
    pub y: YCoordinate,
}

/// Padding values for all four sides.
#[derive(Debug, Clone, Copy)]
pub struct Padding {
    /// Top padding.
    pub top: PaddingValue,
    /// Right padding.
    pub right: PaddingValue,
    /// Bottom padding.
    pub bottom: PaddingValue,
    /// Left padding.
    pub left: PaddingValue,
}

/// Width of the canvas in pixels.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct CanvasWidth(PositiveInt);

/// Height of the canvas in pixels.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct CanvasHeight(PositiveInt);

/// Horizontal coordinate value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct XCoordinate(NonNegativeFloat);

/// Vertical coordinate value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct YCoordinate(NonNegativeFloat);

/// Width value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct Width(PositiveFloat);

/// Height value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct Height(PositiveFloat);

/// Padding value.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct PaddingValue(NonNegativeFloat);

/// Engine for computing diagram layouts.
pub struct LayoutEngine {
    /// Configuration for layout computation.
    config: LayoutConfig,
}

/// Configuration for the layout engine.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Spacing between entities.
    pub entity_spacing: EntitySpacing,
    /// Height of each swimlane.
    pub swimlane_height: SwimlaneHeight,
    /// Space between slices.
    pub slice_gutter: SliceGutter,
    /// Algorithm for routing connections.
    pub connection_routing: ConnectionRouting,
}

/// Algorithm for routing connections between entities.
#[derive(Debug, Clone)]
pub enum ConnectionRouting {
    /// Direct straight lines.
    Straight,
    /// Right-angle paths.
    Orthogonal,
    /// Smooth curved paths.
    Curved,
}

/// Spacing between entities.
#[nutype(derive(Debug, Clone, Copy))]
pub struct EntitySpacing(PositiveFloat);

/// Height of a swimlane.
#[nutype(derive(Debug, Clone, Copy))]
pub struct SwimlaneHeight(PositiveFloat);

/// Space between slices.
#[nutype(derive(Debug, Clone, Copy))]
pub struct SliceGutter(PositiveFloat);

impl LayoutEngine {
    /// Create a new layout engine with the given configuration.
    pub fn new(config: LayoutConfig) -> Self {
        Self { config }
    }

    /// Compute the layout for a diagram.
    pub fn compute_layout<W, C, E, P, Q, A>(
        &self,
        _diagram: &crate::event_model::diagram::EventModelDiagram<W, C, E, P, Q, A>,
    ) -> Result<Layout, LayoutError> {
        todo!()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
}

/// Errors that can occur during layout computation.
#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// Not enough space to place an entity.
    #[error("No space available for entity {0}")]
    NoSpaceAvailable(String),

    /// Entities have circular dependencies.
    #[error("Circular dependency detected")]
    CircularDependency,

    /// Slice boundaries are invalid.
    #[error("Invalid slice boundaries")]
    InvalidSliceBoundaries,
}
