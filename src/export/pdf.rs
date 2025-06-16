use nutype::nutype;
use crate::renderer::svg::SvgDocument;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PdfDocument {
    pub metadata: PdfMetadata,
    pub pages: Vec<PdfPage>,
}

#[derive(Debug, Clone)]
pub struct PdfMetadata {
    pub title: Option<PdfTitle>,
    pub author: Option<PdfAuthor>,
    pub subject: Option<PdfSubject>,
    pub keywords: Option<PdfKeywords>,
    pub creator: PdfCreator,
    pub creation_date: PdfDate,
}

#[derive(Debug, Clone)]
pub struct PdfPage {
    pub size: PageSize,
    pub orientation: PageOrientation,
    pub margins: PageMargins,
    pub content: PageContent,
}

#[derive(Debug, Clone)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    A3,
    Custom(PageWidth, PageHeight),
}

#[derive(Debug, Clone)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone)]
pub struct PageMargins {
    pub top: MarginValue,
    pub right: MarginValue,
    pub bottom: MarginValue,
    pub left: MarginValue,
}

#[derive(Debug, Clone)]
pub enum PageContent {
    Svg(SvgDocument),
    Text(PdfText),
}

#[derive(Debug, Clone)]
pub struct PdfText {
    pub content: TextContent,
    pub style: PdfTextStyle,
}

#[derive(Debug, Clone)]
pub struct PdfTextStyle {
    pub font: PdfFont,
    pub size: PdfFontSize,
    pub color: PdfColor,
}

#[derive(Debug, Clone)]
pub enum PdfFont {
    Helvetica,
    TimesRoman,
    Courier,
    Custom(PdfFontName),
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfTitle(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfAuthor(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfSubject(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfKeywords(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfCreator(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfDate(String);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct PageWidth(f32);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct PageHeight(f32);

#[nutype(
    validate(greater_or_equal = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct MarginValue(f32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct TextContent(String);

#[nutype(
    validate(greater = 0, finite),
    derive(Debug, Clone, Copy),
)]
pub struct PdfFontSize(f32);

#[nutype(
    validate(regex = r"^#[0-9a-fA-F]{6}$|^#[0-9a-fA-F]{3}$"),
    derive(Debug, Clone),
)]
pub struct PdfColor(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PdfFontName(String);

pub struct PdfExporter {
    config: PdfExportConfig,
}

#[derive(Debug, Clone)]
pub struct PdfExportConfig {
    pub compress: CompressionEnabled,
    pub embed_fonts: EmbedPdfFonts,
    pub color_space: ColorSpace,
}

#[derive(Debug, Clone)]
pub enum ColorSpace {
    Rgb,
    Cmyk,
    Grayscale,
}

#[nutype(
    derive(Debug, Clone),
)]
pub struct CompressionEnabled(bool);

#[nutype(
    derive(Debug, Clone),
)]
pub struct EmbedPdfFonts(bool);

impl PdfExporter {
    pub fn new(config: PdfExportConfig) -> Self {
        Self { config }
    }
    
    pub fn export(&self, _svg: &SvgDocument, _path: &PathBuf) -> Result<(), PdfExportError> {
        todo!()
    }
    
    pub fn export_to_buffer(&self, _svg: &SvgDocument) -> Result<Vec<u8>, PdfExportError> {
        todo!()
    }
    
    pub fn config(&self) -> &PdfExportConfig {
        &self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PdfExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid SVG: {0}")]
    InvalidSvg(String),
    
    #[error("Font not found: {0}")]
    FontNotFound(String),
    
    #[error("Export failed: {0}")]
    ExportFailed(String),
}