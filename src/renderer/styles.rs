//! Visual styling for Event Model diagram elements.
//!
//! This module defines styles, themes, and visual properties that can be
//! applied to entities, connections, swimlanes, and other diagram elements.

use nutype::nutype;
use crate::type_safety::{NonEmptyString, PositiveFloat, NonNegativeFloat, FiniteFloat};

/// Visual style for entities (wireframes, commands, events, etc.).
#[derive(Debug, Clone)]
pub struct EntityStyle {
    /// Fill style for the entity body.
    pub fill: FillStyle,
    /// Stroke style for the entity border.
    pub stroke: StrokeStyle,
    /// Optional shadow effect.
    pub shadow: Option<ShadowStyle>,
}

/// Visual style for connections between entities.
#[derive(Debug, Clone)]
pub struct ConnectionStyle {
    /// Stroke style for the connection line.
    pub stroke: StrokeStyle,
    /// Optional end marker (e.g., arrowhead).
    pub marker_end: Option<MarkerStyle>,
    /// Optional start marker.
    pub marker_start: Option<MarkerStyle>,
}

/// Fill style properties.
#[derive(Debug, Clone)]
pub struct FillStyle {
    /// Fill color.
    pub color: StyleColor,
    /// Optional opacity override.
    pub opacity: Option<StyleOpacity>,
}

/// Stroke style properties.
#[derive(Debug, Clone)]
pub struct StrokeStyle {
    /// Stroke color.
    pub color: StyleColor,
    /// Stroke width.
    pub width: StrokeWidth,
    /// Optional dash pattern.
    pub dasharray: Option<DashArray>,
    /// Optional opacity override.
    pub opacity: Option<StyleOpacity>,
}

/// Shadow effect properties.
#[derive(Debug, Clone)]
pub struct ShadowStyle {
    /// Horizontal offset.
    pub offset_x: ShadowOffset,
    /// Vertical offset.
    pub offset_y: ShadowOffset,
    /// Blur radius.
    pub blur_radius: BlurRadius,
    /// Shadow color.
    pub color: StyleColor,
    /// Optional opacity override.
    pub opacity: Option<StyleOpacity>,
}

/// Style for connection markers (arrows, etc.).
#[derive(Debug, Clone)]
pub struct MarkerStyle {
    /// Type of marker.
    pub marker_type: MarkerType,
    /// Size of the marker.
    pub size: MarkerStyleSize,
    /// Marker color.
    pub color: StyleColor,
}

/// Types of connection markers.
#[derive(Debug, Clone)]
pub enum MarkerType {
    /// Arrow head.
    Arrow,
    /// Diamond shape.
    Diamond,
    /// Circle shape.
    Circle,
    /// Square shape.
    Square,
}

/// Dash pattern for strokes.
#[derive(Debug, Clone)]
pub struct DashArray {
    /// Pattern of dash and gap lengths.
    pub pattern: Vec<DashValue>,
}

/// Complete theme defining all diagram styles.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name.
    pub name: ThemeName,
    /// Style for wireframe entities.
    pub wireframe_style: EntityStyle,
    /// Style for command entities.
    pub command_style: EntityStyle,
    /// Style for event entities.
    pub event_style: EntityStyle,
    /// Style for projection entities.
    pub projection_style: EntityStyle,
    /// Style for query entities.
    pub query_style: EntityStyle,
    /// Style for automation entities.
    pub automation_style: EntityStyle,
    /// Default connection style.
    pub connection_style: ConnectionStyle,
    /// Swimlane styling.
    pub swimlane_style: SwimlaneStyle,
    /// Slice styling.
    pub slice_style: SliceStyle,
    /// Text styles for various elements.
    pub text_style: TextStyleConfig,
}

/// Visual style for swimlanes.
#[derive(Debug, Clone)]
pub struct SwimlaneStyle {
    /// Background fill.
    pub background: FillStyle,
    /// Border stroke.
    pub border: StrokeStyle,
    /// Label text style.
    pub label_style: LabelStyle,
}

/// Visual style for slices.
#[derive(Debug, Clone)]
pub struct SliceStyle {
    /// Background fill.
    pub background: FillStyle,
    /// Border stroke.
    pub border: StrokeStyle,
    /// Style for slice gutters.
    pub gutter_style: GutterStyle,
}

/// Style for slice gutters.
#[derive(Debug, Clone)]
pub struct GutterStyle {
    /// Gutter color.
    pub color: StyleColor,
    /// Gutter pattern.
    pub pattern: GutterPattern,
}

/// Pattern for slice gutters.
#[derive(Debug, Clone)]
pub enum GutterPattern {
    /// Solid line.
    Solid,
    /// Dashed line.
    Dashed,
    /// Dotted line.
    Dotted,
}

/// Style for labels.
#[derive(Debug, Clone)]
pub struct LabelStyle {
    /// Font configuration.
    pub font: FontConfig,
    /// Text color.
    pub color: StyleColor,
    /// Text alignment.
    pub alignment: TextAlignment,
}

/// Text styles for different diagram elements.
#[derive(Debug, Clone)]
pub struct TextStyleConfig {
    /// Font for entity names.
    pub entity_name: FontConfig,
    /// Font for field names.
    pub field_name: FontConfig,
    /// Font for slice labels.
    pub slice_label: FontConfig,
    /// Font for swimlane labels.
    pub swimlane_label: FontConfig,
}

/// Font configuration.
#[derive(Debug, Clone)]
pub struct FontConfig {
    /// Font family.
    pub family: StyleFontFamily,
    /// Font size.
    pub size: StyleFontSize,
    /// Font weight.
    pub weight: StyleFontWeight,
}

/// Text alignment options.
#[derive(Debug, Clone)]
pub enum TextAlignment {
    /// Left aligned.
    Left,
    /// Center aligned.
    Center,
    /// Right aligned.
    Right,
}

/// Font weight options.
#[derive(Debug, Clone)]
pub enum StyleFontWeight {
    /// Normal weight.
    Normal,
    /// Bold weight.
    Bold,
}

/// Color value (hex, rgb, or named).
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct StyleColor(NonEmptyString);

/// Opacity value (0.0-1.0).
#[nutype(
    derive(Debug, Clone, Copy, PartialEq, PartialOrd)
)]
pub struct StyleOpacity(NonNegativeFloat);

/// Width of a stroke.
#[nutype(
    derive(Debug, Clone, Copy, PartialEq, PartialOrd)
)]
pub struct StrokeWidth(PositiveFloat);

/// Shadow offset value.
#[nutype(
    derive(Debug, Clone, Copy)
)]
pub struct ShadowOffset(FiniteFloat);

/// Blur radius for shadows.
#[nutype(
    derive(Debug, Clone, Copy)
)]
pub struct BlurRadius(NonNegativeFloat);

/// Size of a marker.
#[nutype(
    derive(Debug, Clone, Copy)
)]
pub struct MarkerStyleSize(PositiveFloat);

/// Length of a dash or gap in a dash pattern.
#[nutype(
    derive(Debug, Clone, Copy)
)]
pub struct DashValue(PositiveFloat);

/// Name of a theme.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct ThemeName(NonEmptyString);

/// Font family name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct StyleFontFamily(NonEmptyString);

/// Font size in pixels.
#[nutype(
    derive(Debug, Clone, Copy)
)]
pub struct StyleFontSize(PositiveFloat);