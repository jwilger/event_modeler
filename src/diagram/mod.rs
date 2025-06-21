//! Event model diagram generation module.
//!
//! This module provides functionality to convert YAML-based event models
//! into visual SVG diagrams.

use crate::event_model::yaml_types;
use thiserror::Error;

mod builder;
mod layout_types;
mod svg;

pub use self::builder::EventModelDiagram;
pub use self::svg::render_to_svg;

/// Errors that can occur during diagram generation.
#[derive(Debug, Error)]
pub enum DiagramError {
    /// Error occurred during SVG rendering.
    #[error("SVG rendering error: {0}")]
    SvgError(String),
}

/// Result type for diagram operations.
pub type Result<T> = std::result::Result<T, DiagramError>;

/// Converts a YAML domain model into an event model diagram.
///
/// This is the main entry point for diagram generation from the parsed YAML model.
pub fn build_diagram_from_domain(model: &yaml_types::YamlEventModel) -> Result<EventModelDiagram> {
    EventModelDiagram::from_yaml_model(model)
}
