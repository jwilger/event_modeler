use nutype::nutype;

#[derive(Debug, Clone)]
pub struct EntityStyle {
    pub fill: FillStyle,
    pub stroke: StrokeStyle,
    pub shadow: Option<ShadowStyle>,
}

#[derive(Debug, Clone)]
pub struct ConnectionStyle {
    pub stroke: StrokeStyle,
    pub marker_end: Option<MarkerStyle>,
    pub marker_start: Option<MarkerStyle>,
}

#[derive(Debug, Clone)]
pub struct FillStyle {
    pub color: StyleColor,
    pub opacity: Option<StyleOpacity>,
}

#[derive(Debug, Clone)]
pub struct StrokeStyle {
    pub color: StyleColor,
    pub width: StrokeWidth,
    pub dasharray: Option<DashArray>,
    pub opacity: Option<StyleOpacity>,
}

#[derive(Debug, Clone)]
pub struct ShadowStyle {
    pub offset_x: ShadowOffset,
    pub offset_y: ShadowOffset,
    pub blur_radius: BlurRadius,
    pub color: StyleColor,
    pub opacity: Option<StyleOpacity>,
}

#[derive(Debug, Clone)]
pub struct MarkerStyle {
    pub marker_type: MarkerType,
    pub size: MarkerStyleSize,
    pub color: StyleColor,
}

#[derive(Debug, Clone)]
pub enum MarkerType {
    Arrow,
    Diamond,
    Circle,
    Square,
}

#[derive(Debug, Clone)]
pub struct DashArray {
    pub pattern: Vec<DashValue>,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: ThemeName,
    pub wireframe_style: EntityStyle,
    pub command_style: EntityStyle,
    pub event_style: EntityStyle,
    pub projection_style: EntityStyle,
    pub query_style: EntityStyle,
    pub automation_style: EntityStyle,
    pub connection_style: ConnectionStyle,
    pub swimlane_style: SwimlaneStyle,
    pub slice_style: SliceStyle,
    pub text_style: TextStyleConfig,
}

#[derive(Debug, Clone)]
pub struct SwimlaneStyle {
    pub background: FillStyle,
    pub border: StrokeStyle,
    pub label_style: LabelStyle,
}

#[derive(Debug, Clone)]
pub struct SliceStyle {
    pub background: FillStyle,
    pub border: StrokeStyle,
    pub gutter_style: GutterStyle,
}

#[derive(Debug, Clone)]
pub struct GutterStyle {
    pub color: StyleColor,
    pub pattern: GutterPattern,
}

#[derive(Debug, Clone)]
pub enum GutterPattern {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone)]
pub struct LabelStyle {
    pub font: FontConfig,
    pub color: StyleColor,
    pub alignment: TextAlignment,
}

#[derive(Debug, Clone)]
pub struct TextStyleConfig {
    pub entity_name: FontConfig,
    pub field_name: FontConfig,
    pub slice_label: FontConfig,
    pub swimlane_label: FontConfig,
}

#[derive(Debug, Clone)]
pub struct FontConfig {
    pub family: StyleFontFamily,
    pub size: StyleFontSize,
    pub weight: StyleFontWeight,
}

#[derive(Debug, Clone)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub enum StyleFontWeight {
    Normal,
    Bold,
}

#[nutype(
    validate(regex = r"^#[0-9a-fA-F]{6}$|^#[0-9a-fA-F]{3}$|^rgb\(\d{1,3},\s*\d{1,3},\s*\d{1,3}\)$|^[a-zA-Z]+$"),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct StyleColor(String);

#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct StyleOpacity(f32);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy, PartialEq, PartialOrd),
)]
pub struct StrokeWidth(f32);

#[nutype(
    validate(finite),
    derive(Debug, Clone, Copy),
)]
pub struct ShadowOffset(f32);

#[nutype(
    validate(greater_or_equal = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct BlurRadius(f32);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct MarkerStyleSize(f32);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct DashValue(f32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct ThemeName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct StyleFontFamily(String);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct StyleFontSize(f32);

pub struct StyleManager {
    themes: Vec<Theme>,
    current_theme: ThemeName,
}

impl StyleManager {
    pub fn new(default_theme: Theme) -> Self {
        let theme_name = default_theme.name.clone();
        Self {
            themes: vec![default_theme],
            current_theme: theme_name,
        }
    }
    
    pub fn add_theme(&mut self, theme: Theme) {
        self.themes.push(theme);
    }
    
    pub fn set_current_theme(&mut self, name: ThemeName) -> Result<(), StyleError> {
        if self.themes.iter().any(|t| t.name == name) {
            self.current_theme = name;
            Ok(())
        } else {
            Err(StyleError::ThemeNotFound(name.into_inner()))
        }
    }
    
    pub fn current_theme(&self) -> Option<&Theme> {
        self.themes.iter().find(|t| t.name == self.current_theme)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StyleError {
    #[error("Theme not found: {0}")]
    ThemeNotFound(String),
    
    #[error("Invalid style configuration: {0}")]
    InvalidConfiguration(String),
}