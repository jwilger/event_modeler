//! Event model diagram builder.
//!
//! Provides the main `EventModelDiagram` type and builder pattern API for
//! incrementally constructing diagrams.

use crate::infrastructure::types::NonEmptyString;

/// The main event model diagram type.
///
/// This type represents a complete event model diagram that can be rendered
/// to SVG. It is built incrementally using a builder pattern with immutable
/// updates.
#[derive(Debug, Clone)]
pub struct EventModelDiagram {
    workflow_title: NonEmptyString,
}

impl EventModelDiagram {
    /// Creates a new diagram with the specified workflow title.
    pub fn new(workflow_title: NonEmptyString) -> Self {
        Self { workflow_title }
    }

    /// Gets the workflow title.
    pub fn workflow_title(&self) -> &str {
        self.workflow_title.as_str()
    }
}
