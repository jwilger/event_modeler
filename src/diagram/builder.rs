//! Event model diagram builder.
//!
//! This module provides the core diagram building functionality.

use crate::event_model::yaml_types;
use crate::infrastructure::types::NonEmptyString;

use super::Result;

/// Represents a complete event model diagram.
///
/// This type is built from a YAML event model and contains all the
/// information needed to render the diagram.
#[derive(Debug, Clone)]
pub struct EventModelDiagram {
    /// The workflow title displayed at the top of the diagram.
    workflow_title: NonEmptyString,
}

impl EventModelDiagram {
    /// Creates a new event model diagram from a YAML model.
    pub fn from_yaml_model(model: &yaml_types::YamlEventModel) -> Result<Self> {
        Ok(EventModelDiagram {
            workflow_title: model.workflow.clone().into_inner(),
        })
    }

    /// Gets the workflow title.
    pub fn workflow_title(&self) -> &NonEmptyString {
        &self.workflow_title
    }
}
