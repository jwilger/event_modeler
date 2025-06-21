//! Event model diagram builder.
//!
//! This module provides the core diagram building functionality.

use crate::event_model::yaml_types;
use crate::infrastructure::types::{NonEmpty, NonEmptyString};
use indexmap::IndexMap;

use super::Result;

/// Represents a complete event model diagram.
///
/// This type is built from a YAML event model and contains all the
/// information needed to render the diagram.
#[derive(Debug, Clone)]
pub struct EventModelDiagram {
    /// The workflow title displayed at the top of the diagram.
    workflow_title: NonEmptyString,
    /// The swimlanes defined in the model.
    swimlanes: NonEmpty<yaml_types::Swimlane>,
    /// The slices defined in the model.
    slices: IndexMap<yaml_types::SliceName, NonEmpty<yaml_types::Connection>>,
}

impl EventModelDiagram {
    /// Creates a new event model diagram from a YAML model.
    pub fn from_yaml_model(model: &yaml_types::YamlEventModel) -> Result<Self> {
        Ok(EventModelDiagram {
            workflow_title: model.workflow.clone().into_inner(),
            swimlanes: model.swimlanes.clone(),
            slices: model.slices.clone(),
        })
    }

    /// Gets the workflow title.
    pub fn workflow_title(&self) -> &NonEmptyString {
        &self.workflow_title
    }

    /// Gets the swimlanes.
    pub fn swimlanes(&self) -> &NonEmpty<yaml_types::Swimlane> {
        &self.swimlanes
    }

    /// Gets the slices.
    pub fn slices(&self) -> &IndexMap<yaml_types::SliceName, NonEmpty<yaml_types::Connection>> {
        &self.slices
    }
}
