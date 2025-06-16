use nutype::nutype;
use crate::model::entities::EntityId;
use crate::model::diagram::{SwimlaneId, SliceId};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Layout {
    pub canvas: Canvas,
    pub swimlane_layouts: HashMap<SwimlaneId, SwimlaneLayout>,
    pub entity_positions: HashMap<EntityId, EntityPosition>,
    pub slice_layouts: HashMap<SliceId, SliceLayout>,
    pub connections: Vec<Connection>,
}

#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: CanvasWidth,
    pub height: CanvasHeight,
    pub padding: Padding,
}

#[derive(Debug, Clone)]
pub struct SwimlaneLayout {
    pub position: Position,
    pub dimensions: Dimensions,
}

#[derive(Debug, Clone)]
pub struct EntityPosition {
    pub swimlane_id: SwimlaneId,
    pub position: Position,
    pub dimensions: Dimensions,
}

#[derive(Debug, Clone)]
pub struct SliceLayout {
    pub x_position: XCoordinate,
    pub width: Width,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub from: EntityId,
    pub to: EntityId,
    pub path: ConnectionPath,
    pub style: ConnectionStyle,
}

#[derive(Debug, Clone)]
pub struct ConnectionPath {
    pub points: Vec<Point>,
}

#[derive(Debug, Clone)]
pub enum ConnectionStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: XCoordinate,
    pub y: YCoordinate,
}

#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: Width,
    pub height: Height,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: XCoordinate,
    pub y: YCoordinate,
}

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub top: PaddingValue,
    pub right: PaddingValue,
    pub bottom: PaddingValue,
    pub left: PaddingValue,
}

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct CanvasWidth(u32);

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct CanvasHeight(u32);

#[nutype(
    validate(greater_or_equal = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct XCoordinate(f32);

#[nutype(
    validate(greater_or_equal = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct YCoordinate(f32);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct Width(f32);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct Height(f32);

#[nutype(
    validate(greater_or_equal = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct PaddingValue(f32);

pub struct LayoutEngine {
    config: LayoutConfig,
}

#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub entity_spacing: EntitySpacing,
    pub swimlane_height: SwimlaneHeight,
    pub slice_gutter: SliceGutter,
    pub connection_routing: ConnectionRouting,
}

#[derive(Debug, Clone)]
pub enum ConnectionRouting {
    Straight,
    Orthogonal,
    Curved,
}

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy),
)]
pub struct EntitySpacing(f32);

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy),
)]
pub struct SwimlaneHeight(f32);

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy),
)]
pub struct SliceGutter(f32);

impl LayoutEngine {
    pub fn new(config: LayoutConfig) -> Self {
        Self { config }
    }
    
    pub fn compute_layout<W, C, E, P, Q, A>(&self, _diagram: &crate::model::diagram::EventModelDiagram<W, C, E, P, Q, A>) -> Result<Layout, LayoutError> {
        todo!()
    }
    
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("No space available for entity {0}")]
    NoSpaceAvailable(String),
    
    #[error("Circular dependency detected")]
    CircularDependency,
    
    #[error("Invalid slice boundaries")]
    InvalidSliceBoundaries,
}