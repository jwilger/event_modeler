use nutype::nutype;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Cli {
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Render(RenderCommand),
    Watch(WatchCommand),
    Validate(ValidateCommand),
}

#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub input: ModelFilePath,
    pub options: RenderOptions,
}

#[derive(Debug, Clone)]
pub struct WatchCommand {
    pub directory: WatchDirectory,
    pub serve_port: Option<ServePort>,
}

#[derive(Debug, Clone)]
pub struct ValidateCommand {
    pub input: ModelFilePath,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub formats: OutputFormats,
    pub style: RenderStyle,
    pub include_links: IncludeLinks,
    pub output_dir: OutputDirectory,
}

#[nutype(
    validate(predicate = |path: &PathBuf| path.extension().map_or(false, |ext| ext == "eventmodel")),
    derive(Debug, Clone),
)]
pub struct ModelFilePath(PathBuf);

#[nutype(
    validate(predicate = |path: &PathBuf| path.is_dir()),
    derive(Debug, Clone),
)]
pub struct WatchDirectory(PathBuf);

#[nutype(
    validate(greater = 0, less_or_equal = 65535),
    derive(Debug, Clone),
)]
pub struct ServePort(u16);

#[nutype(
    validate(predicate = |formats: &Vec<OutputFormat>| !formats.is_empty()),
    derive(Debug, Clone),
)]
pub struct OutputFormats(Vec<OutputFormat>);

#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Svg,
    Pdf,
}

#[derive(Debug, Clone)]
pub enum RenderStyle {
    GithubLight,
    GithubDark,
}

#[nutype(
    derive(Debug, Clone),
)]
pub struct IncludeLinks(bool);

#[nutype(
    validate(predicate = |path: &PathBuf| {
        // Directory should exist or be creatable
        path.parent().map_or(true, |p| p.exists())
    }),
    derive(Debug, Clone),
)]
pub struct OutputDirectory(PathBuf);