//! Event model diagram builder.
//!
//! Provides the main `EventModelDiagram` type and builder pattern API for
//! incrementally constructing diagrams.

use crate::event_model::yaml_types::{SliceName, SwimlaneId, SwimlaneName};
use crate::infrastructure::types::NonEmptyString;

/// The main event model diagram type.
///
/// This type represents a complete event model diagram that can be rendered
/// to SVG. It is built incrementally using a builder pattern with immutable
/// updates.
#[derive(Debug, Clone)]
pub struct EventModelDiagram {
    workflow_title: NonEmptyString,
    swimlanes: Vec<Swimlane>,
    slices: Vec<Slice>,
}

/// A swimlane in the event model diagram.
#[derive(Debug, Clone)]
pub struct Swimlane {
    id: SwimlaneId,
    label: SwimlaneName,
}

/// A slice (vertical section) in the event model diagram.
#[derive(Debug, Clone)]
pub struct Slice {
    name: SliceName,
}

impl EventModelDiagram {
    /// Creates a new diagram with the specified workflow title.
    pub fn new(workflow_title: NonEmptyString) -> Self {
        Self {
            workflow_title,
            swimlanes: Vec::new(),
            slices: Vec::new(),
        }
    }

    /// Gets the workflow title.
    pub fn workflow_title(&self) -> &str {
        self.workflow_title.as_str()
    }

    /// Adds a swimlane to the diagram.
    pub fn with_swimlane(mut self, id: SwimlaneId, label: SwimlaneName) -> Self {
        self.swimlanes.push(Swimlane { id, label });
        self
    }

    /// Gets the swimlanes.
    pub fn swimlanes(&self) -> &[Swimlane] {
        &self.swimlanes
    }

    /// Adds a slice to the diagram.
    pub fn with_slice(mut self, name: SliceName) -> Self {
        self.slices.push(Slice { name });
        self
    }

    /// Gets the slices.
    pub fn slices(&self) -> &[Slice] {
        &self.slices
    }
}

impl Swimlane {
    /// Gets the swimlane ID.
    pub fn id(&self) -> String {
        self.id.clone().into_inner().into_inner()
    }

    /// Gets the swimlane label.
    pub fn label(&self) -> String {
        self.label.clone().into_inner().into_inner()
    }
}

impl Slice {
    /// Gets the slice name.
    pub fn name(&self) -> String {
        self.name.clone().into_inner().into_inner()
    }
}
