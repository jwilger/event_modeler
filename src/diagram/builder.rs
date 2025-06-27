//! Event model diagram builder.
//!
//! This module provides the core diagram building functionality.

use crate::event_model::yaml_types;
use crate::infrastructure::types::{NonEmpty, NonEmptyString};
use std::collections::HashMap;

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
    slices: Vec<yaml_types::Slice>,
    /// The views defined in the model.
    views: HashMap<yaml_types::ViewName, yaml_types::ViewDefinition>,
    /// The commands defined in the model.
    commands: HashMap<yaml_types::CommandName, yaml_types::CommandDefinition>,
    /// The events defined in the model.
    events: HashMap<yaml_types::EventName, yaml_types::EventDefinition>,
    /// The projections defined in the model.
    projections: HashMap<yaml_types::ProjectionName, yaml_types::ProjectionDefinition>,
    /// The queries defined in the model.
    queries: HashMap<yaml_types::QueryName, yaml_types::QueryDefinition>,
}

impl EventModelDiagram {
    /// Creates a new event model diagram from a YAML model.
    pub fn from_yaml_model(model: &yaml_types::YamlEventModel) -> Result<Self> {
        Ok(EventModelDiagram {
            workflow_title: model.workflow.clone().into_inner(),
            swimlanes: model.swimlanes.clone(),
            slices: model.slices.clone(),
            views: model.views.clone(),
            commands: model.commands.clone(),
            events: model.events.clone(),
            projections: model.projections.clone(),
            queries: model.queries.clone(),
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
    pub fn slices(&self) -> &[yaml_types::Slice] {
        &self.slices
    }

    /// Gets the views.
    pub fn views(&self) -> &HashMap<yaml_types::ViewName, yaml_types::ViewDefinition> {
        &self.views
    }

    /// Gets the commands.
    pub fn commands(&self) -> &HashMap<yaml_types::CommandName, yaml_types::CommandDefinition> {
        &self.commands
    }

    /// Gets the events.
    pub fn events(&self) -> &HashMap<yaml_types::EventName, yaml_types::EventDefinition> {
        &self.events
    }

    /// Gets the projections.
    pub fn projections(
        &self,
    ) -> &HashMap<yaml_types::ProjectionName, yaml_types::ProjectionDefinition> {
        &self.projections
    }

    /// Gets the queries.
    pub fn queries(&self) -> &HashMap<yaml_types::QueryName, yaml_types::QueryDefinition> {
        &self.queries
    }
}
