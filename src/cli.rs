use nutype::nutype;
use crate::type_safety::{TypedPath, EventModelFile, Directory, File, Exists, MaybeExists, NonEmpty, AnyFile};

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
    pub input: TypedPath<EventModelFile, File, Exists>,
    pub options: RenderOptions,
}

#[derive(Debug, Clone)]
pub struct WatchCommand {
    pub directory: TypedPath<AnyFile, Directory, Exists>,
    pub serve_port: Option<ServePort>,
}

#[derive(Debug, Clone)]
pub struct ValidateCommand {
    pub input: TypedPath<EventModelFile, File, Exists>,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub formats: NonEmpty<OutputFormat>,
    pub style: RenderStyle,
    pub include_links: IncludeLinks,
    pub output_dir: TypedPath<AnyFile, Directory, MaybeExists>,
}

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
    validate(greater = 0, less_or_equal = 65535),
    derive(Debug, Clone),
)]
pub struct ServePort(u16);

#[nutype(
    derive(Debug, Clone),
)]
pub struct IncludeLinks(bool);