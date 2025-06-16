use nutype::nutype;
use crate::model::diagram::EventModelDiagram;
use crate::renderer::svg::SvgDocument;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MarkdownDocument {
    pub sections: Vec<MarkdownSection>,
}

#[derive(Debug, Clone)]
pub enum MarkdownSection {
    Heading(HeadingSection),
    Paragraph(ParagraphSection),
    Image(ImageSection),
    CodeBlock(CodeBlockSection),
    Table(TableSection),
    List(ListSection),
}

#[derive(Debug, Clone)]
pub struct HeadingSection {
    pub level: HeadingLevel,
    pub content: HeadingContent,
}

#[derive(Debug, Clone)]
pub struct ParagraphSection {
    pub content: ParagraphContent,
}

#[derive(Debug, Clone)]
pub struct ImageSection {
    pub alt_text: ImageAltText,
    pub path: ImagePath,
    pub title: Option<ImageTitle>,
}

#[derive(Debug, Clone)]
pub struct CodeBlockSection {
    pub language: Option<CodeLanguage>,
    pub content: CodeContent,
}

#[derive(Debug, Clone)]
pub struct TableSection {
    pub headers: Vec<TableHeader>,
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone)]
pub struct ListSection {
    pub list_type: ListType,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone)]
pub enum ListType {
    Ordered,
    Unordered,
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: ListItemContent,
    pub sub_items: Option<Vec<ListItem>>,
}

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 6),
    derive(Debug, Clone, Copy),
)]
pub struct HeadingLevel(u8);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct HeadingContent(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ParagraphContent(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ImageAltText(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ImagePath(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ImageTitle(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct CodeLanguage(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct CodeContent(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct TableHeader(String);

#[nutype(
    derive(Debug, Clone),
)]
pub struct TableCell(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ListItemContent(String);

pub struct MarkdownExporter {
    config: MarkdownExportConfig,
}

#[derive(Debug, Clone)]
pub struct MarkdownExportConfig {
    pub flavor: MarkdownFlavor,
    pub embed_svg: EmbedSvgOption,
    pub link_style: LinkStyle,
}

#[derive(Debug, Clone)]
pub enum MarkdownFlavor {
    Github,
    CommonMark,
    Pandoc,
}

#[derive(Debug, Clone)]
pub enum EmbedSvgOption {
    Inline,
    Reference(SvgDirectory),
}

#[derive(Debug, Clone)]
pub enum LinkStyle {
    Relative,
    Absolute,
}

use crate::type_safety::{TypedPath, AnyFile, Directory, MaybeExists};

pub type SvgDirectory = TypedPath<AnyFile, Directory, MaybeExists>;

impl MarkdownExporter {
    pub fn new(config: MarkdownExportConfig) -> Self {
        Self { config }
    }
    
    pub fn export_diagram<W, C, E, P, Q, A>(&self, _diagram: &EventModelDiagram<W, C, E, P, Q, A>, _svg: &SvgDocument) -> Result<MarkdownDocument, MarkdownExportError> {
        todo!()
    }
    
    pub fn write_to_file(&self, _document: &MarkdownDocument, _path: &PathBuf) -> Result<(), MarkdownExportError> {
        todo!()
    }
    
    pub fn config(&self) -> &MarkdownExportConfig {
        &self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarkdownExportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid diagram: {0}")]
    InvalidDiagram(String),
    
    #[error("Export failed: {0}")]
    ExportFailed(String),
}