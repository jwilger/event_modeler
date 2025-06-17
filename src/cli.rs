//! Command-line interface types for the Event Modeler.
//!
//! This module defines the structure of CLI commands and options using
//! type-safe constructs. All path validation happens at parse time,
//! ensuring that the rest of the application works with valid paths.

use crate::infrastructure::types::{
    AnyFile, Directory, EventModelFile, Exists, File, MaybeExists, NonEmpty, PathBuilder,
    Port as ValidatedPort, TypedPath,
};
use nutype::nutype;
use std::env;
use std::path::PathBuf;

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
#[derive(Debug, Clone)]
pub struct IncludeLinks(bool);

impl IncludeLinks {
    /// Create a new IncludeLinks flag.
    pub fn new(value: bool) -> Self {
        Self(value)
    }

    /// Get the inner boolean value.
    pub fn as_bool(&self) -> bool {
        self.0
    }
}

/// Result type for CLI operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during CLI parsing or execution.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid command line arguments.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Failed to parse a path.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl Cli {
    /// Parse command line arguments into a CLI structure.
    pub fn from_args() -> Result<Self> {
        let args: Vec<String> = env::args().collect();

        // Basic argument parsing - for now just support: event_modeler input.eventmodel -o output.svg
        if args.len() < 2 {
            return Err(Error::InvalidArguments(
                "Usage: event_modeler <input.eventmodel> [-o <output.svg>]".to_string(),
            ));
        }

        let input_path = &args[1];
        let mut output_path = None;

        // Parse output flag
        let mut i = 2;
        while i < args.len() {
            if args[i] == "-o" && i + 1 < args.len() {
                output_path = Some(args[i + 1].clone());
                i += 2;
            } else {
                i += 1;
            }
        }

        // Determine output directory and format
        let (output_dir, format) = if let Some(path) = output_path {
            let path_buf = PathBuf::from(&path);
            let dir = path_buf
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));

            let format = if path.ends_with(".svg") {
                OutputFormat::Svg
            } else if path.ends_with(".pdf") {
                OutputFormat::Pdf
            } else {
                OutputFormat::Svg // Default to SVG
            };

            (dir, format)
        } else {
            // Default to current directory and SVG
            (PathBuf::from("."), OutputFormat::Svg)
        };

        // Parse the input file path
        let input = PathBuilder::parse_event_model_file(PathBuf::from(input_path))
            .map_err(|e| Error::InvalidPath(format!("Input file error: {}", e)))?;

        // Parse the output directory
        let output_dir = PathBuilder::parse_output_directory(output_dir)
            .map_err(|e| Error::InvalidPath(format!("Output directory error: {}", e)))?;

        // Create formats list with the determined format
        let formats = NonEmpty::singleton(format);

        let command = Command::Render(RenderCommand {
            input,
            options: RenderOptions {
                formats,
                style: RenderStyle::GithubLight, // Default style
                include_links: IncludeLinks::new(false), // Default to no links
                output_dir,
            },
        });

        Ok(Cli { command })
    }

    /// Execute the CLI command.
    pub fn execute(self) -> Result<()> {
        match self.command {
            Command::Render(cmd) => execute_render(cmd),
            Command::Watch(_) => todo!("Watch command not implemented"),
            Command::Validate(_) => todo!("Validate command not implemented"),
        }
    }
}

/// Execute a render command.
fn execute_render(_cmd: RenderCommand) -> Result<()> {
    // This is where we'll orchestrate the full pipeline:
    // 1. Parse the input file
    // 2. Build the event model
    // 3. Compute layout
    // 4. Render to requested formats
    // For now, just a placeholder
    todo!("Render command implementation")
}
