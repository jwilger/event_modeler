// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! PDF export functionality for Event Model diagrams.
//!
//! This module handles the conversion of SVG diagrams to PDF format,
//! including page layout, metadata, and font embedding.

// TODO: Re-enable when SvgDocument is available
// use crate::diagram::svg::SvgDocument;
use crate::infrastructure::types::{NonEmptyString, NonNegativeFloat, PositiveFloat};
use nutype::nutype;
use std::path::Path;

/// A complete PDF document.
#[derive(Debug, Clone)]
pub struct PdfDocument {
    /// Document metadata.
    pub metadata: PdfMetadata,
    /// Pages in the document.
    pub pages: Vec<PdfPage>,
}

/// PDF document metadata.
#[derive(Debug, Clone)]
pub struct PdfMetadata {
    /// Document title.
    pub title: Option<PdfTitle>,
    /// Document author.
    pub author: Option<PdfAuthor>,
    /// Document subject.
    pub subject: Option<PdfSubject>,
    /// Document keywords.
    pub keywords: Option<PdfKeywords>,
    /// Software that created the PDF.
    pub creator: PdfCreator,
    /// Creation date.
    pub creation_date: PdfDate,
}

/// A single page in a PDF document.
#[derive(Debug, Clone)]
pub struct PdfPage {
    /// Page size.
    pub size: PageSize,
    /// Page orientation.
    pub orientation: PageOrientation,
    /// Page margins.
    pub margins: PageMargins,
    /// Page content.
    pub content: PageContent,
}

/// Standard page sizes.
#[derive(Debug, Clone)]
pub enum PageSize {
    /// A4 (210mm × 297mm).
    A4,
    /// US Letter (8.5" × 11").
    Letter,
    /// US Legal (8.5" × 14").
    Legal,
    /// A3 (297mm × 420mm).
    A3,
    /// Custom size.
    Custom(PageWidth, PageHeight),
}

/// Page orientation.
#[derive(Debug, Clone)]
pub enum PageOrientation {
    /// Taller than wide.
    Portrait,
    /// Wider than tall.
    Landscape,
}

/// Margins for a PDF page.
#[derive(Debug, Clone)]
pub struct PageMargins {
    /// Top margin.
    pub top: MarginValue,
    /// Right margin.
    pub right: MarginValue,
    /// Bottom margin.
    pub bottom: MarginValue,
    /// Left margin.
    pub left: MarginValue,
}

/// Content that can be placed on a PDF page.
#[derive(Debug, Clone)]
pub enum PageContent {
    /// SVG diagram content.
    // TODO: Re-enable when SvgDocument is available
    // Svg(SvgDocument),
    Svg(String),
    /// Text content.
    Text(PdfText),
}

/// Text content for a PDF page.
#[derive(Debug, Clone)]
pub struct PdfText {
    /// The text to display.
    pub content: TextContent,
    /// Text styling.
    pub style: PdfTextStyle,
}

/// Style properties for PDF text.
#[derive(Debug, Clone)]
pub struct PdfTextStyle {
    /// Font selection.
    pub font: PdfFont,
    /// Font size.
    pub size: PdfFontSize,
    /// Text color.
    pub color: PdfColor,
}

/// Available PDF fonts.
#[derive(Debug, Clone)]
pub enum PdfFont {
    /// Helvetica (sans-serif).
    Helvetica,
    /// Times Roman (serif).
    TimesRoman,
    /// Courier (monospace).
    Courier,
    /// Custom font.
    Custom(PdfFontName),
}

/// PDF document title.
#[nutype(derive(Debug, Clone))]
pub struct PdfTitle(NonEmptyString);

/// PDF document author.
#[nutype(derive(Debug, Clone))]
pub struct PdfAuthor(NonEmptyString);

/// PDF document subject.
#[nutype(derive(Debug, Clone))]
pub struct PdfSubject(NonEmptyString);

/// PDF document keywords.
#[nutype(derive(Debug, Clone))]
pub struct PdfKeywords(NonEmptyString);

/// PDF creator software name.
#[nutype(derive(Debug, Clone))]
pub struct PdfCreator(NonEmptyString);

/// PDF creation date.
#[nutype(derive(Debug, Clone))]
pub struct PdfDate(NonEmptyString);

/// Width of a PDF page.
#[nutype(derive(Debug, Clone, Copy))]
pub struct PageWidth(PositiveFloat);

/// Height of a PDF page.
#[nutype(derive(Debug, Clone, Copy))]
pub struct PageHeight(PositiveFloat);

/// Margin value.
#[nutype(derive(Debug, Clone, Copy))]
pub struct MarginValue(NonNegativeFloat);

/// Text content for PDF.
#[nutype(derive(Debug, Clone))]
pub struct TextContent(NonEmptyString);

/// Font size for PDF text.
#[nutype(derive(Debug, Clone, Copy))]
pub struct PdfFontSize(PositiveFloat);

/// Color for PDF text.
#[nutype(derive(Debug, Clone))]
pub struct PdfColor(NonEmptyString);

/// Custom font name.
#[nutype(derive(Debug, Clone))]
pub struct PdfFontName(NonEmptyString);

/// Exporter for converting SVG to PDF.
pub struct PdfExporter {
    /// Export configuration.
    config: PdfExportConfig,
}

/// Configuration for PDF export.
#[derive(Debug, Clone)]
pub struct PdfExportConfig {
    /// Whether to compress the PDF.
    pub compress: CompressionEnabled,
    /// Whether to embed fonts.
    pub embed_fonts: EmbedPdfFonts,
    /// Color space for the PDF.
    pub color_space: ColorSpace,
}

/// PDF color space options.
#[derive(Debug, Clone)]
pub enum ColorSpace {
    /// RGB color space.
    Rgb,
    /// CMYK color space.
    Cmyk,
    /// Grayscale.
    Grayscale,
}

/// Whether PDF compression is enabled.
#[nutype(derive(Debug, Clone))]
pub struct CompressionEnabled(bool);

/// Whether to embed fonts in the PDF.
#[nutype(derive(Debug, Clone))]
pub struct EmbedPdfFonts(bool);

impl PdfExporter {
    /// Create a new PDF exporter.
    pub fn new(config: PdfExportConfig) -> Self {
        Self { config }
    }

    /// Export an SVG document to a PDF file.
    // TODO: Re-enable when SvgDocument is available
    // pub fn export(&self, _svg: &SvgDocument, _path: &Path) -> Result<(), PdfExportError> {
    pub fn export(&self, _svg: &str, _path: &Path) -> Result<(), PdfExportError> {
        todo!()
    }

    /// Export an SVG document to a PDF byte buffer.
    // TODO: Re-enable when SvgDocument is available
    // pub fn export_to_buffer(&self, _svg: &SvgDocument) -> Result<Vec<u8>, PdfExportError> {
    pub fn export_to_buffer(&self, _svg: &str) -> Result<Vec<u8>, PdfExportError> {
        todo!()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &PdfExportConfig {
        &self.config
    }
}

/// Errors that can occur during PDF export.
#[derive(Debug, thiserror::Error)]
pub enum PdfExportError {
    /// I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The SVG document is invalid.
    #[error("Invalid SVG: {0}")]
    InvalidSvg(String),

    /// A required font was not found.
    #[error("Font not found: {0}")]
    FontNotFound(String),

    /// Export failed for another reason.
    #[error("Export failed: {0}")]
    ExportFailed(String),
}
