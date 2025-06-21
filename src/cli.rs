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
    /// Optional specific output filename (if not provided, uses input filename).
    pub output_filename: Option<String>,
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
                "Usage: event_modeler <input.eventmodel> [-o <output.svg>] [--dark]".to_string(),
            ));
        }

        let input_path = &args[1];
        let mut output_path = None;
        let mut use_dark_theme = false;

        // Parse output flag
        let mut i = 2;
        while i < args.len() {
            if args[i] == "-o" && i + 1 < args.len() {
                output_path = Some(args[i + 1].clone());
                i += 2;
            } else if args[i] == "--dark" {
                use_dark_theme = true;
                i += 1;
            } else {
                i += 1;
            }
        }

        // Determine output directory, format, and filename
        let (output_dir, format, output_filename) = if let Some(path) = output_path {
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

            let filename = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().to_string());

            (dir, format, filename)
        } else {
            // Default to current directory and SVG
            (PathBuf::from("."), OutputFormat::Svg, None)
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
                style: if use_dark_theme {
                    RenderStyle::GithubDark
                } else {
                    RenderStyle::GithubLight
                },
                include_links: IncludeLinks::new(false), // Default to no links
                output_dir,
                output_filename,
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
fn execute_render(cmd: RenderCommand) -> Result<()> {
    use std::fs;
    use std::io::Write;

    // 1. Read the input file
    let input_content = fs::read_to_string(cmd.input.as_path_buf())?;

    // 2. Parse the YAML event model
    let yaml_model = crate::infrastructure::parsing::yaml_parser::parse_yaml(&input_content)
        .map_err(|e| Error::InvalidArguments(format!("YAML parse error: {}", e)))?;

    // 3. Convert YAML to domain types
    let domain_model =
        crate::infrastructure::parsing::yaml_converter::convert_yaml_to_domain(yaml_model)
            .map_err(|e| Error::InvalidArguments(format!("YAML conversion error: {}", e)))?;

    // 4. Build diagram from domain model
    let diagram = crate::diagram::build_diagram_from_domain(&domain_model)
        .map_err(|e| Error::InvalidArguments(format!("Diagram building error: {}", e)))?;

    println!(
        "Successfully converted event model: {}",
        domain_model.workflow.clone().into_inner().as_str()
    );

    // 5. Render to requested formats
    for format in cmd.options.formats.iter() {
        match format {
            OutputFormat::Svg => {
                // Render diagram to SVG
                let svg_doc = crate::diagram::render_to_svg(&diagram)
                    .map_err(|e| Error::InvalidArguments(format!("SVG rendering error: {}", e)))?;

                // Generate output filename
                let output_filename = if let Some(filename) = &cmd.options.output_filename {
                    filename.clone()
                } else {
                    let input_stem = cmd
                        .input
                        .as_path_buf()
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy();
                    format!("{}.svg", input_stem)
                };
                let output_path = cmd.options.output_dir.as_path_buf().join(&output_filename);

                // Write SVG to file
                let svg_content = svg_doc;
                let mut file = fs::File::create(&output_path)?;
                file.write_all(svg_content.as_bytes())?;

                println!("Generated SVG: {}", output_path.display());
            }
            OutputFormat::Pdf => {
                // PDF export not yet implemented
                eprintln!("Warning: PDF export not yet implemented");
            }
        }
    }

    Ok(())
}
