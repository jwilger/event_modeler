//! Markdown export functionality for Event Model diagrams.
//!
//! This module handles the generation of Markdown documentation from
//! Event Model diagrams, including SVG embedding and cross-referencing.

use nutype::nutype;
use crate::model::diagram::EventModelDiagram;
use crate::renderer::svg::SvgDocument;
use crate::type_safety::{NonEmptyString, PositiveInt};
use std::path::PathBuf;

/// A complete Markdown document.
#[derive(Debug, Clone)]
pub struct MarkdownDocument {
    /// Sections in the document.
    pub sections: Vec<MarkdownSection>,
}

/// Types of sections in a Markdown document.
#[derive(Debug, Clone)]
pub enum MarkdownSection {
    /// Heading section.
    Heading(HeadingSection),
    /// Paragraph of text.
    Paragraph(ParagraphSection),
    /// Image embed.
    Image(ImageSection),
    /// Code block.
    CodeBlock(CodeBlockSection),
    /// Table.
    Table(TableSection),
    /// List (ordered or unordered).
    List(ListSection),
}

/// A heading in the Markdown document.
#[derive(Debug, Clone)]
pub struct HeadingSection {
    /// Heading level (1-6).
    pub level: HeadingLevel,
    /// Heading text.
    pub content: HeadingContent,
}

/// A paragraph of text.
#[derive(Debug, Clone)]
pub struct ParagraphSection {
    /// Paragraph content.
    pub content: ParagraphContent,
}

/// An embedded image.
#[derive(Debug, Clone)]
pub struct ImageSection {
    /// Alternative text for accessibility.
    pub alt_text: ImageAltText,
    /// Path to the image.
    pub path: ImagePath,
    /// Optional title/tooltip.
    pub title: Option<ImageTitle>,
}

/// A code block with optional syntax highlighting.
#[derive(Debug, Clone)]
pub struct CodeBlockSection {
    /// Programming language for syntax highlighting.
    pub language: Option<CodeLanguage>,
    /// Code content.
    pub content: CodeContent,
}

/// A table in Markdown format.
#[derive(Debug, Clone)]
pub struct TableSection {
    /// Table headers.
    pub headers: Vec<TableHeader>,
    /// Table rows.
    pub rows: Vec<TableRow>,
}

/// A row in a table.
#[derive(Debug, Clone)]
pub struct TableRow {
    /// Cells in the row.
    pub cells: Vec<TableCell>,
}

/// A list (ordered or unordered).
#[derive(Debug, Clone)]
pub struct ListSection {
    /// Type of list.
    pub list_type: ListType,
    /// List items.
    pub items: Vec<ListItem>,
}

/// Type of list.
#[derive(Debug, Clone)]
pub enum ListType {
    /// Numbered list.
    Ordered,
    /// Bulleted list.
    Unordered,
}

/// An item in a list.
#[derive(Debug, Clone)]
pub struct ListItem {
    /// Item content.
    pub content: ListItemContent,
    /// Nested sub-items.
    pub sub_items: Option<Vec<ListItem>>,
}

/// Heading level (1-6).
#[nutype(
    derive(Debug, Clone, Copy)
)]
pub struct HeadingLevel(PositiveInt);

/// Content of a heading.
#[nutype(
    derive(Debug, Clone)
)]
pub struct HeadingContent(NonEmptyString);

/// Content of a paragraph.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ParagraphContent(NonEmptyString);

/// Alternative text for an image.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ImageAltText(NonEmptyString);

/// Path to an image file.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ImagePath(NonEmptyString);

/// Title/tooltip for an image.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ImageTitle(NonEmptyString);

/// Programming language for syntax highlighting.
#[nutype(
    derive(Debug, Clone)
)]
pub struct CodeLanguage(NonEmptyString);

/// Content of a code block.
#[nutype(
    derive(Debug, Clone)
)]
pub struct CodeContent(NonEmptyString);

/// Header text for a table column.
#[nutype(
    derive(Debug, Clone)
)]
pub struct TableHeader(NonEmptyString);

/// Content of a table cell.
#[nutype(
    derive(Debug, Clone)
)]
pub struct TableCell(String);

/// Content of a list item.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ListItemContent(NonEmptyString);

/// Exporter for generating Markdown documentation.
pub struct MarkdownExporter {
    /// Export configuration.
    config: MarkdownExportConfig,
}

/// Configuration for Markdown export.
#[derive(Debug, Clone)]
pub struct MarkdownExportConfig {
    /// Markdown flavor to use.
    pub flavor: MarkdownFlavor,
    /// How to embed SVG diagrams.
    pub embed_svg: EmbedSvgOption,
    /// Style for links.
    pub link_style: LinkStyle,
}

/// Markdown syntax flavor.
#[derive(Debug, Clone)]
pub enum MarkdownFlavor {
    /// GitHub Flavored Markdown.
    Github,
    /// CommonMark standard.
    CommonMark,
    /// Pandoc extended Markdown.
    Pandoc,
}

/// How to embed SVG diagrams in Markdown.
#[derive(Debug, Clone)]
pub enum EmbedSvgOption {
    /// Inline SVG in the Markdown.
    Inline,
    /// Reference external SVG files.
    Reference(SvgDirectory),
}

/// Style for links in Markdown.
#[derive(Debug, Clone)]
pub enum LinkStyle {
    /// Relative paths.
    Relative,
    /// Absolute paths.
    Absolute,
}

use crate::type_safety::{TypedPath, AnyFile, Directory, MaybeExists};

/// Directory for storing SVG files.
pub type SvgDirectory = TypedPath<AnyFile, Directory, MaybeExists>;

impl MarkdownExporter {
    /// Create a new Markdown exporter.
    pub fn new(config: MarkdownExportConfig) -> Self {
        Self { config }
    }
    
    /// Export a diagram to Markdown format.
    pub fn export_diagram<W, C, E, P, Q, A>(&self, _diagram: &EventModelDiagram<W, C, E, P, Q, A>, _svg: &SvgDocument) -> Result<MarkdownDocument, MarkdownExportError> {
        todo!()
    }
    
    /// Write a Markdown document to a file.
    pub fn write_to_file(&self, _document: &MarkdownDocument, _path: &PathBuf) -> Result<(), MarkdownExportError> {
        todo!()
    }
    
    /// Get the current configuration.
    pub fn config(&self) -> &MarkdownExportConfig {
        &self.config
    }
}

/// Errors that can occur during Markdown export.
#[derive(Debug, thiserror::Error)]
pub enum MarkdownExportError {
    /// I/O error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// The diagram is invalid.
    #[error("Invalid diagram: {0}")]
    InvalidDiagram(String),
    
    /// Export failed for another reason.
    #[error("Export failed: {0}")]
    ExportFailed(String),
}