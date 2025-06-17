// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! SVG rendering for Event Model diagrams.
//!
//! This module handles the generation of SVG documents from layout information,
//! including all visual elements, styles, and optimizations.

use crate::diagram::layout::{Height, Layout, Width, XCoordinate, YCoordinate};
use crate::diagram::style::{ConnectionStyle, EntityStyle};
use crate::infrastructure::types::{
    FiniteFloat, NonEmptyString, NonNegativeFloat, Percentage as ValidatedPercentage,
    PositiveFloat, PositiveInt,
};
use nutype::nutype;

/// A complete SVG document.
#[derive(Debug, Clone)]
pub struct SvgDocument {
    /// Viewbox defining the coordinate system.
    pub viewbox: ViewBox,
    /// All visual elements in the document.
    pub elements: Vec<SvgElement>,
    /// Definitions for reusable elements.
    pub defs: SvgDefs,
}

/// SVG viewBox defining the coordinate system.
#[derive(Debug, Clone)]
pub struct ViewBox {
    /// X coordinate of the viewbox.
    pub x: XCoordinate,
    /// Y coordinate of the viewbox.
    pub y: YCoordinate,
    /// Width of the viewbox.
    pub width: Width,
    /// Height of the viewbox.
    pub height: Height,
}

/// SVG definitions section for reusable elements.
#[derive(Debug, Clone)]
pub struct SvgDefs {
    /// Pattern definitions.
    pub patterns: Vec<Pattern>,
    /// Gradient definitions.
    pub gradients: Vec<Gradient>,
    /// Marker definitions for arrows/endpoints.
    pub markers: Vec<Marker>,
}

/// Types of SVG elements.
#[derive(Debug, Clone)]
pub enum SvgElement {
    /// Group element containing other elements.
    Group(SvgGroup),
    /// Rectangle shape.
    Rectangle(SvgRectangle),
    /// Text element.
    Text(SvgText),
    /// Path element for complex shapes.
    Path(SvgPath),
    /// Image element.
    Image(SvgImage),
}

/// SVG group element.
#[derive(Debug, Clone)]
pub struct SvgGroup {
    /// Optional element ID.
    pub id: Option<ElementId>,
    /// Optional CSS class.
    pub class: Option<CssClass>,
    /// Optional transformation.
    pub transform: Option<Transform>,
    /// Child elements.
    pub children: Vec<SvgElement>,
}

/// SVG rectangle element.
#[derive(Debug, Clone)]
pub struct SvgRectangle {
    /// Optional element ID.
    pub id: Option<ElementId>,
    /// Optional CSS class.
    pub class: Option<CssClass>,
    /// X coordinate.
    pub x: XCoordinate,
    /// Y coordinate.
    pub y: YCoordinate,
    /// Width.
    pub width: Width,
    /// Height.
    pub height: Height,
    /// X-axis border radius.
    pub rx: Option<BorderRadius>,
    /// Y-axis border radius.
    pub ry: Option<BorderRadius>,
    /// Visual style.
    pub style: EntityStyle,
}

/// SVG text element.
#[derive(Debug, Clone)]
pub struct SvgText {
    /// Optional element ID.
    pub id: Option<ElementId>,
    /// Optional CSS class.
    pub class: Option<CssClass>,
    /// X coordinate.
    pub x: XCoordinate,
    /// Y coordinate.
    pub y: YCoordinate,
    /// Text to display.
    pub content: TextContent,
    /// Text style.
    pub style: TextStyle,
}

/// SVG path element.
#[derive(Debug, Clone)]
pub struct SvgPath {
    /// Optional element ID.
    pub id: Option<ElementId>,
    /// Optional CSS class.
    pub class: Option<CssClass>,
    /// Path data commands.
    pub d: PathData,
    /// Visual style.
    pub style: ConnectionStyle,
}

/// SVG image element.
#[derive(Debug, Clone)]
pub struct SvgImage {
    /// Optional element ID.
    pub id: Option<ElementId>,
    /// X coordinate.
    pub x: XCoordinate,
    /// Y coordinate.
    pub y: YCoordinate,
    /// Width.
    pub width: Width,
    /// Height.
    pub height: Height,
    /// Image source URL.
    pub href: ImageHref,
}

/// SVG pattern definition.
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Unique pattern ID.
    pub id: PatternId,
    /// Pattern width.
    pub width: PatternSize,
    /// Pattern height.
    pub height: PatternSize,
    /// Pattern content.
    pub content: Vec<SvgElement>,
}

/// SVG gradient definition.
#[derive(Debug, Clone)]
pub struct Gradient {
    /// Unique gradient ID.
    pub id: GradientId,
    /// Type of gradient.
    pub gradient_type: GradientType,
    /// Color stops.
    pub stops: Vec<GradientStop>,
}

/// Type of gradient.
#[derive(Debug, Clone)]
pub enum GradientType {
    /// Linear gradient.
    Linear(LinearGradient),
    /// Radial gradient.
    Radial(RadialGradient),
}

/// Linear gradient parameters.
#[derive(Debug, Clone)]
pub struct LinearGradient {
    /// Start X position.
    pub x1: Percentage,
    /// Start Y position.
    pub y1: Percentage,
    /// End X position.
    pub x2: Percentage,
    /// End Y position.
    pub y2: Percentage,
}

/// Radial gradient parameters.
#[derive(Debug, Clone)]
pub struct RadialGradient {
    /// Center X position.
    pub cx: Percentage,
    /// Center Y position.
    pub cy: Percentage,
    /// Radius.
    pub r: Percentage,
}

/// Color stop in a gradient.
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// Position along gradient (0-100%).
    pub offset: Percentage,
    /// Color at this stop.
    pub color: Color,
    /// Optional opacity.
    pub opacity: Option<Opacity>,
}

/// SVG marker for arrow heads and endpoints.
#[derive(Debug, Clone)]
pub struct Marker {
    /// Unique marker ID.
    pub id: MarkerId,
    /// Marker width.
    pub width: MarkerSize,
    /// Marker height.
    pub height: MarkerSize,
    /// X reference point.
    pub refx: MarkerRef,
    /// Y reference point.
    pub refy: MarkerRef,
    /// Marker content.
    pub content: Vec<SvgElement>,
}

/// Style properties for text.
#[derive(Debug, Clone)]
pub struct TextStyle {
    /// Font family.
    pub font_family: FontFamily,
    /// Font size.
    pub font_size: FontSize,
    /// Font weight.
    pub font_weight: Option<FontWeight>,
    /// Text color.
    pub fill: Color,
    /// Text alignment.
    pub anchor: Option<TextAnchor>,
}

/// Text alignment anchor point.
#[derive(Debug, Clone)]
pub enum TextAnchor {
    /// Align to start (left for LTR).
    Start,
    /// Center alignment.
    Middle,
    /// Align to end (right for LTR).
    End,
}

/// Font weight options.
#[derive(Debug, Clone)]
pub enum FontWeight {
    /// Normal weight.
    Normal,
    /// Bold weight.
    Bold,
    /// Bolder than parent.
    Bolder,
    /// Lighter than parent.
    Lighter,
    /// Specific numeric weight.
    Weight(FontWeightValue),
}

/// SVG transformation types.
#[derive(Debug, Clone)]
pub enum Transform {
    /// Translation by x,y.
    Translate(XCoordinate, YCoordinate),
    /// Scaling by x,y factors.
    Scale(ScaleFactor, ScaleFactor),
    /// Rotation with optional center point.
    Rotate(RotationAngle, Option<XCoordinate>, Option<YCoordinate>),
    /// Full transformation matrix.
    Matrix(Matrix),
}

/// 2D transformation matrix.
#[derive(Debug, Clone)]
pub struct Matrix {
    /// Scale X.
    pub a: MatrixValue,
    /// Skew Y.
    pub b: MatrixValue,
    /// Skew X.
    pub c: MatrixValue,
    /// Scale Y.
    pub d: MatrixValue,
    /// Translate X.
    pub e: MatrixValue,
    /// Translate Y.
    pub f: MatrixValue,
}

/// Unique identifier for an SVG element.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ElementId(NonEmptyString);

/// CSS class name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct CssClass(NonEmptyString);

/// Border radius for rounded corners.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct BorderRadius(NonNegativeFloat);

/// Text content to display.
#[nutype(derive(Debug, Clone))]
pub struct TextContent(NonEmptyString);

/// SVG path data string.
#[nutype(derive(Debug, Clone))]
pub struct PathData(NonEmptyString);

/// Image URL or data URI.
#[nutype(derive(Debug, Clone))]
pub struct ImageHref(NonEmptyString);

/// Unique identifier for a pattern.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct PatternId(NonEmptyString);

/// Size of a pattern.
#[nutype(derive(Debug, Clone, Copy))]
pub struct PatternSize(PositiveFloat);

/// Unique identifier for a gradient.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct GradientId(NonEmptyString);

/// Percentage value (0-100).
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct Percentage(ValidatedPercentage);

/// Color value (hex or rgb).
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct Color(NonEmptyString);

/// Opacity value (0.0-1.0).
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct Opacity(NonNegativeFloat);

/// Unique identifier for a marker.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct MarkerId(NonEmptyString);

/// Size of a marker.
#[nutype(derive(Debug, Clone, Copy))]
pub struct MarkerSize(PositiveFloat);

/// Reference point for a marker.
#[nutype(derive(Debug, Clone, Copy))]
pub struct MarkerRef(FiniteFloat);

/// Font family name.
#[nutype(derive(Debug, Clone))]
pub struct FontFamily(NonEmptyString);

/// Font size in pixels.
#[nutype(derive(Debug, Clone, Copy))]
pub struct FontSize(PositiveFloat);

/// Font weight value (100-900).
#[nutype(derive(Debug, Clone, Copy))]
pub struct FontWeightValue(PositiveInt);

/// Scale transformation factor.
#[nutype(derive(Debug, Clone, Copy, PartialEq, PartialOrd))]
pub struct ScaleFactor(PositiveFloat);

/// Rotation angle in degrees.
#[nutype(derive(Debug, Clone, Copy))]
pub struct RotationAngle(FiniteFloat);

/// Value in a transformation matrix.
#[nutype(derive(Debug, Clone, Copy))]
pub struct MatrixValue(FiniteFloat);

/// Renderer for converting layouts to SVG documents.
pub struct SvgRenderer {
    /// Configuration for rendering.
    config: SvgRenderConfig,
}

/// Configuration for SVG rendering.
#[derive(Debug, Clone)]
pub struct SvgRenderConfig {
    /// Decimal precision for coordinates.
    pub precision: DecimalPrecision,
    /// Optimization level for output.
    pub optimize: OptimizationLevel,
    /// Whether to embed fonts.
    pub embed_fonts: EmbedFonts,
}

/// Level of SVG optimization.
#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    /// No optimization.
    None,
    /// Basic optimization (remove comments, whitespace).
    Basic,
    /// Full optimization (merge paths, simplify).
    Full,
}

/// Decimal precision for coordinates (0-6).
#[nutype(derive(Debug, Clone, Copy))]
pub struct DecimalPrecision(PositiveInt);

/// Whether to embed fonts in the SVG.
#[nutype(derive(Debug, Clone))]
pub struct EmbedFonts(bool);

impl SvgRenderer {
    /// Create a new SVG renderer.
    pub fn new(config: SvgRenderConfig) -> Self {
        Self { config }
    }

    /// Render a layout to an SVG document.
    pub fn render(&self, layout: &Layout) -> Result<SvgDocument, SvgRenderError> {
        // Create viewbox from canvas dimensions
        let viewbox = ViewBox {
            x: XCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(0.0).unwrap()),
            width: Width::new(
                PositiveFloat::parse(layout.canvas.width.into_inner().value() as f32).unwrap(),
            ),
            height: Height::new(
                PositiveFloat::parse(layout.canvas.height.into_inner().value() as f32).unwrap(),
            ),
        };

        // Create SVG elements
        let mut elements = Vec::new();

        // Render swimlanes
        for (swimlane_id, swimlane_layout) in &layout.swimlane_layouts {
            let swimlane_group = self.render_swimlane(swimlane_id, swimlane_layout)?;
            elements.push(SvgElement::Group(swimlane_group));
        }

        // TODO: Add entity rendering
        // TODO: Add connector rendering

        // Create empty defs for now
        let defs = SvgDefs {
            patterns: vec![],
            gradients: vec![],
            markers: vec![],
        };

        Ok(SvgDocument {
            viewbox,
            elements,
            defs,
        })
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SvgRenderConfig {
        &self.config
    }

    /// Render a swimlane as an SVG group.
    fn render_swimlane(
        &self,
        swimlane_id: &crate::event_model::diagram::SwimlaneId,
        layout: &crate::diagram::layout::SwimlaneLayout,
    ) -> Result<SvgGroup, SvgRenderError> {
        let mut children = Vec::new();

        // Create swimlane background rectangle
        let background = SvgRectangle {
            id: Some(ElementId::new(
                NonEmptyString::parse(format!(
                    "swimlane-{}",
                    swimlane_id.clone().into_inner().as_str()
                ))
                .unwrap(),
            )),
            class: Some(CssClass::new(
                NonEmptyString::parse("swimlane".to_string()).unwrap(),
            )),
            x: layout.position.x,
            y: layout.position.y,
            width: layout.dimensions.width,
            height: layout.dimensions.height,
            rx: None,
            ry: None,
            style: crate::diagram::style::EntityStyle {
                fill: crate::diagram::style::FillStyle {
                    color: crate::diagram::style::StyleColor::new(
                        NonEmptyString::parse("#f6f8fa".to_string()).unwrap(),
                    ),
                    opacity: None,
                },
                stroke: crate::diagram::style::StrokeStyle {
                    color: crate::diagram::style::StyleColor::new(
                        NonEmptyString::parse("#d1d5db".to_string()).unwrap(),
                    ),
                    width: crate::diagram::style::StrokeWidth::new(
                        PositiveFloat::parse(1.0).unwrap(),
                    ),
                    dasharray: None,
                    opacity: None,
                },
                shadow: None,
            },
        };
        children.push(SvgElement::Rectangle(background));

        // Create swimlane label
        let label_x = layout.position.x.into_inner().value() + 10.0;
        let label_y = layout.position.y.into_inner().value() + 25.0;

        let label = SvgText {
            id: None,
            class: Some(CssClass::new(
                NonEmptyString::parse("swimlane-label".to_string()).unwrap(),
            )),
            x: XCoordinate::new(NonNegativeFloat::parse(label_x).unwrap()),
            y: YCoordinate::new(NonNegativeFloat::parse(label_y).unwrap()),
            content: TextContent::new(
                NonEmptyString::parse(swimlane_id.clone().into_inner().as_str().to_string())
                    .unwrap(),
            ),
            style: TextStyle {
                font_family: FontFamily::new(
                    NonEmptyString::parse("Arial, sans-serif".to_string()).unwrap(),
                ),
                font_size: FontSize::new(PositiveFloat::parse(14.0).unwrap()),
                font_weight: Some(FontWeight::Bold),
                fill: Color::new(NonEmptyString::parse("#24292e".to_string()).unwrap()),
                anchor: Some(TextAnchor::Start),
            },
        };
        children.push(SvgElement::Text(label));

        Ok(SvgGroup {
            id: None,
            class: None,
            transform: None,
            children,
        })
    }
}

/// Errors that can occur during SVG rendering.
#[derive(Debug, thiserror::Error)]
pub enum SvgRenderError {
    /// The layout is invalid for SVG generation.
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),

    /// A required resource was not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

#[cfg(test)]
#[path = "svg_tests.rs"]
mod tests;
