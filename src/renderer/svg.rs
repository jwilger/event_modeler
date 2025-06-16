use nutype::nutype;
use crate::renderer::layout::{Layout, XCoordinate, YCoordinate, Width, Height};
use crate::renderer::styles::{EntityStyle, ConnectionStyle};

#[derive(Debug, Clone)]
pub struct SvgDocument {
    pub viewbox: ViewBox,
    pub elements: Vec<SvgElement>,
    pub defs: SvgDefs,
}

#[derive(Debug, Clone)]
pub struct ViewBox {
    pub x: XCoordinate,
    pub y: YCoordinate,
    pub width: Width,
    pub height: Height,
}

#[derive(Debug, Clone)]
pub struct SvgDefs {
    pub patterns: Vec<Pattern>,
    pub gradients: Vec<Gradient>,
    pub markers: Vec<Marker>,
}

#[derive(Debug, Clone)]
pub enum SvgElement {
    Group(SvgGroup),
    Rectangle(SvgRectangle),
    Text(SvgText),
    Path(SvgPath),
    Image(SvgImage),
}

#[derive(Debug, Clone)]
pub struct SvgGroup {
    pub id: Option<ElementId>,
    pub class: Option<CssClass>,
    pub transform: Option<Transform>,
    pub children: Vec<SvgElement>,
}

#[derive(Debug, Clone)]
pub struct SvgRectangle {
    pub id: Option<ElementId>,
    pub class: Option<CssClass>,
    pub x: XCoordinate,
    pub y: YCoordinate,
    pub width: Width,
    pub height: Height,
    pub rx: Option<BorderRadius>,
    pub ry: Option<BorderRadius>,
    pub style: EntityStyle,
}

#[derive(Debug, Clone)]
pub struct SvgText {
    pub id: Option<ElementId>,
    pub class: Option<CssClass>,
    pub x: XCoordinate,
    pub y: YCoordinate,
    pub content: TextContent,
    pub style: TextStyle,
}

#[derive(Debug, Clone)]
pub struct SvgPath {
    pub id: Option<ElementId>,
    pub class: Option<CssClass>,
    pub d: PathData,
    pub style: ConnectionStyle,
}

#[derive(Debug, Clone)]
pub struct SvgImage {
    pub id: Option<ElementId>,
    pub x: XCoordinate,
    pub y: YCoordinate,
    pub width: Width,
    pub height: Height,
    pub href: ImageHref,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: PatternId,
    pub width: PatternSize,
    pub height: PatternSize,
    pub content: Vec<SvgElement>,
}

#[derive(Debug, Clone)]
pub struct Gradient {
    pub id: GradientId,
    pub gradient_type: GradientType,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone)]
pub enum GradientType {
    Linear(LinearGradient),
    Radial(RadialGradient),
}

#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub x1: Percentage,
    pub y1: Percentage,
    pub x2: Percentage,
    pub y2: Percentage,
}

#[derive(Debug, Clone)]
pub struct RadialGradient {
    pub cx: Percentage,
    pub cy: Percentage,
    pub r: Percentage,
}

#[derive(Debug, Clone)]
pub struct GradientStop {
    pub offset: Percentage,
    pub color: Color,
    pub opacity: Option<Opacity>,
}

#[derive(Debug, Clone)]
pub struct Marker {
    pub id: MarkerId,
    pub width: MarkerSize,
    pub height: MarkerSize,
    pub refx: MarkerRef,
    pub refy: MarkerRef,
    pub content: Vec<SvgElement>,
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: FontFamily,
    pub font_size: FontSize,
    pub font_weight: Option<FontWeight>,
    pub fill: Color,
    pub anchor: Option<TextAnchor>,
}

#[derive(Debug, Clone)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone)]
pub enum FontWeight {
    Normal,
    Bold,
    Bolder,
    Lighter,
    Weight(FontWeightValue),
}

#[derive(Debug, Clone)]
pub enum Transform {
    Translate(XCoordinate, YCoordinate),
    Scale(ScaleFactor, ScaleFactor),
    Rotate(RotationAngle, Option<XCoordinate>, Option<YCoordinate>),
    Matrix(Matrix),
}

#[derive(Debug, Clone)]
pub struct Matrix {
    pub a: MatrixValue,
    pub b: MatrixValue,
    pub c: MatrixValue,
    pub d: MatrixValue,
    pub e: MatrixValue,
    pub f: MatrixValue,
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct ElementId(String);

#[nutype(
    validate(regex = r"^[a-zA-Z_][\w-]*$"),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct CssClass(String);

#[nutype(
    validate(greater_or_equal = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct BorderRadius(f32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct TextContent(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PathData(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ImageHref(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct PatternId(String);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct PatternSize(f32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct GradientId(String);

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct Percentage(f32);

#[nutype(
    validate(regex = r"^#[0-9a-fA-F]{6}$|^#[0-9a-fA-F]{3}$|^rgb\(\d{1,3},\s*\d{1,3},\s*\d{1,3}\)$"),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct Color(String);

#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct Opacity(f32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct MarkerId(String);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct MarkerSize(f32);

#[nutype(
    validate(finite),
    derive(Debug, Clone, Copy),
)]
pub struct MarkerRef(f32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct FontFamily(String);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct FontSize(f32);

#[nutype(
    validate(greater_or_equal = 100, less_or_equal = 900),
    derive(Debug, Clone, Copy),
)]
pub struct FontWeightValue(u16);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct ScaleFactor(f32);

#[nutype(
    validate(finite),
    derive(Debug, Clone, Copy),
)]
pub struct RotationAngle(f32);

#[nutype(
    validate(finite),
    derive(Debug, Clone, Copy),
)]
pub struct MatrixValue(f32);

pub struct SvgRenderer {
    config: SvgRenderConfig,
}

#[derive(Debug, Clone)]
pub struct SvgRenderConfig {
    pub precision: DecimalPrecision,
    pub optimize: OptimizationLevel,
    pub embed_fonts: EmbedFonts,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Basic,
    Full,
}

#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 6),
    derive(Debug, Clone, Copy),
)]
pub struct DecimalPrecision(u8);

#[nutype(
    derive(Debug, Clone),
)]
pub struct EmbedFonts(bool);

impl SvgRenderer {
    pub fn new(config: SvgRenderConfig) -> Self {
        Self { config }
    }
    
    pub fn render(&self, _layout: &Layout) -> Result<SvgDocument, SvgRenderError> {
        todo!()
    }
    
    pub fn config(&self) -> &SvgRenderConfig {
        &self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SvgRenderError {
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}