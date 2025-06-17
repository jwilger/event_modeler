//! Command-line interface types for the Event Modeler.
//!
//! This module defines the structure of CLI commands and options using
//! type-safe constructs. All path validation happens at parse time,
//! ensuring that the rest of the application works with valid paths.

use crate::infrastructure::types::{
    AnyFile, Directory, EventModelFile, Exists, File, MaybeExists, NonEmpty, Port as ValidatedPort,
    TypedPath,
};
use nutype::nutype;

/// The main CLI structure containing the command to execute.
#[derive(Debug, Clone)]
pub struct Cli {
    /// The command to execute.
    pub command: Command,
}

/// Available commands for the Event Modeler CLI.
#[derive(Debug, Clone)]
pub enum Command {
    /// Render an event model to SVG/PDF.
    Render(RenderCommand),
    /// Watch a directory for changes and auto-render.
    Watch(WatchCommand),
    /// Validate an event model file without rendering.
    Validate(ValidateCommand),
}

/// Command to render an event model file to various output formats.
#[derive(Debug, Clone)]
pub struct RenderCommand {
    /// The input event model file (must exist with .eventmodel extension).
    pub input: TypedPath<EventModelFile, File, Exists>,
    /// Rendering options including output formats and styling.
    pub options: RenderOptions,
}

/// Command to watch a directory for event model changes.
#[derive(Debug, Clone)]
pub struct WatchCommand {
    /// The directory to watch (must exist).
    pub directory: TypedPath<AnyFile, Directory, Exists>,
    /// Optional port to serve rendered diagrams on.
    pub serve_port: Option<ServePort>,
}

/// Command to validate an event model file.
#[derive(Debug, Clone)]
pub struct ValidateCommand {
    /// The input event model file to validate (must exist with .eventmodel extension).
    pub input: TypedPath<EventModelFile, File, Exists>,
}

/// Options for rendering event models.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Output formats (at least one required).
    pub formats: NonEmpty<OutputFormat>,
    /// Visual style for rendering.
    pub style: RenderStyle,
    /// Whether to include documentation links in the output.
    pub include_links: IncludeLinks,
    /// Directory to write output files (parent must exist).
    pub output_dir: TypedPath<AnyFile, Directory, MaybeExists>,
}

/// Supported output formats for rendered diagrams.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    /// Scalable Vector Graphics format.
    Svg,
    /// Portable Document Format.
    Pdf,
}

/// Visual rendering styles optimized for different environments.
#[derive(Debug, Clone)]
pub enum RenderStyle {
    /// Light theme optimized for GitHub's light mode.
    GithubLight,
    /// Dark theme optimized for GitHub's dark mode.
    GithubDark,
}

/// Port number for serving rendered diagrams.
/// Wraps a validated port to ensure it's CLI-specific.
#[nutype(derive(Debug, Clone))]
pub struct ServePort(ValidatedPort);

/// Flag indicating whether to include documentation links in rendered output.
#[nutype(derive(Debug, Clone))]
pub struct IncludeLinks(bool);
