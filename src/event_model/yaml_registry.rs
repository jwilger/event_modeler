// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Entity registry for YAML-based event models.
//!
//! This module provides a type-safe registry for YAML event model entities.
//! Unlike the legacy registry that uses the typestate pattern, this registry
//! works with the rich YAML format that includes data schemas, test scenarios,
//! UI components, and slice-based connections.

use super::yaml_types::{
    AutomationDefinition, AutomationName, CommandDefinition, CommandName, Connection,
    EntityReference, EventDefinition, EventName, ProjectionDefinition, ProjectionName,
    QueryDefinition, QueryName, SliceName, ViewDefinition, ViewName, YamlEventModel,
};
use crate::infrastructure::types::NonEmpty;
use std::collections::HashMap;

/// Registry for YAML event model entities.
///
/// This registry provides centralized access to all entities defined in a YAML
/// event model, including their relationships through slices.
#[derive(Debug, Clone)]
pub struct YamlEntityRegistry {
    /// Events indexed by name.
    pub events: HashMap<EventName, EventDefinition>,
    /// Commands indexed by name.
    pub commands: HashMap<CommandName, CommandDefinition>,
    /// Views indexed by name.
    pub views: HashMap<ViewName, ViewDefinition>,
    /// Projections indexed by name.
    pub projections: HashMap<ProjectionName, ProjectionDefinition>,
    /// Queries indexed by name.
    pub queries: HashMap<QueryName, QueryDefinition>,
    /// Automations indexed by name.
    pub automations: HashMap<AutomationName, AutomationDefinition>,
    /// Slices defining connections between entities.
    pub slices: HashMap<SliceName, NonEmpty<Connection>>,
}

impl YamlEntityRegistry {
    /// Creates a new registry from a YAML event model.
    pub fn from_model(model: YamlEventModel) -> Self {
        Self {
            events: model.events,
            commands: model.commands,
            views: model.views,
            projections: model.projections,
            queries: model.queries,
            automations: model.automations,
            slices: model.slices,
        }
    }

    /// Finds all entities that are sources in connections.
    pub fn find_source_entities(&self) -> Vec<EntityReference> {
        let mut sources = Vec::new();
        for connections in self.slices.values() {
            for connection in connections.iter() {
                if !sources.contains(&connection.from) {
                    sources.push(connection.from.clone());
                }
            }
        }
        sources
    }

    /// Finds all entities that are targets in connections.
    pub fn find_target_entities(&self) -> Vec<EntityReference> {
        let mut targets = Vec::new();
        for connections in self.slices.values() {
            for connection in connections.iter() {
                if !targets.contains(&connection.to) {
                    targets.push(connection.to.clone());
                }
            }
        }
        targets
    }

    /// Finds all connections from a specific entity.
    pub fn find_connections_from(&self, entity: &EntityReference) -> Vec<Connection> {
        let mut result = Vec::new();
        for connections in self.slices.values() {
            for connection in connections.iter() {
                if &connection.from == entity {
                    result.push(connection.clone());
                }
            }
        }
        result
    }

    /// Finds all connections to a specific entity.
    pub fn find_connections_to(&self, entity: &EntityReference) -> Vec<Connection> {
        let mut result = Vec::new();
        for connections in self.slices.values() {
            for connection in connections.iter() {
                if &connection.to == entity {
                    result.push(connection.clone());
                }
            }
        }
        result
    }

    /// Validates that all entity references in connections exist.
    pub fn validate_connections(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        for (slice_name, connections) in &self.slices {
            for connection in connections.iter() {
                if let Err(e) = self.validate_entity_reference(&connection.from) {
                    errors.push(ValidationError::InvalidSource {
                        slice: slice_name.clone(),
                        reference: connection.from.clone(),
                        reason: e,
                    });
                }
                if let Err(e) = self.validate_entity_reference(&connection.to) {
                    errors.push(ValidationError::InvalidTarget {
                        slice: slice_name.clone(),
                        reference: connection.to.clone(),
                        reason: e,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validates that an entity reference exists in the registry.
    fn validate_entity_reference(&self, reference: &EntityReference) -> Result<(), String> {
        match reference {
            EntityReference::Event(name) => {
                if self.events.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "Event '{}' not found",
                        name.clone().into_inner().as_str()
                    ))
                }
            }
            EntityReference::Command(name) => {
                if self.commands.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "Command '{}' not found",
                        name.clone().into_inner().as_str()
                    ))
                }
            }
            EntityReference::View(path) => {
                // For now, we just check if the view exists
                // TODO: Validate full path including components
                let path_str = path.clone().into_inner();
                let view_name = path_str.as_str().split('.').next().unwrap();
                if self.views.keys().any(|n| {
                    let n_str = n.clone().into_inner();
                    n_str.as_str() == view_name
                }) {
                    Ok(())
                } else {
                    Err(format!("View '{}' not found", view_name))
                }
            }
            EntityReference::Projection(name) => {
                if self.projections.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "Projection '{}' not found",
                        name.clone().into_inner().as_str()
                    ))
                }
            }
            EntityReference::Query(name) => {
                if self.queries.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "Query '{}' not found",
                        name.clone().into_inner().as_str()
                    ))
                }
            }
            EntityReference::Automation(name) => {
                if self.automations.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "Automation '{}' not found",
                        name.clone().into_inner().as_str()
                    ))
                }
            }
        }
    }

    /// Counts the total number of entities across all types.
    pub fn total_entity_count(&self) -> usize {
        self.events.len()
            + self.commands.len()
            + self.views.len()
            + self.projections.len()
            + self.queries.len()
            + self.automations.len()
    }

    /// Gets all entity names grouped by type.
    pub fn all_entity_names(&self) -> EntityNamesByType {
        EntityNamesByType {
            events: self.events.keys().cloned().collect(),
            commands: self.commands.keys().cloned().collect(),
            views: self.views.keys().cloned().collect(),
            projections: self.projections.keys().cloned().collect(),
            queries: self.queries.keys().cloned().collect(),
            automations: self.automations.keys().cloned().collect(),
        }
    }
}

/// Entity names grouped by type.
#[derive(Debug, Clone)]
pub struct EntityNamesByType {
    /// Event names.
    pub events: Vec<EventName>,
    /// Command names.
    pub commands: Vec<CommandName>,
    /// View names.
    pub views: Vec<ViewName>,
    /// Projection names.
    pub projections: Vec<ProjectionName>,
    /// Query names.
    pub queries: Vec<QueryName>,
    /// Automation names.
    pub automations: Vec<AutomationName>,
}

/// Validation errors for entity references.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Invalid source entity in connection.
    InvalidSource {
        /// The slice containing the invalid connection.
        slice: SliceName,
        /// The invalid entity reference.
        reference: EntityReference,
        /// Reason for the error.
        reason: String,
    },
    /// Invalid target entity in connection.
    InvalidTarget {
        /// The slice containing the invalid connection.
        slice: SliceName,
        /// The invalid entity reference.
        reference: EntityReference,
        /// Reason for the error.
        reason: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidSource {
                slice,
                reference: _,
                reason,
            } => write!(
                f,
                "Invalid source in slice '{}': {}",
                slice.clone().into_inner().as_str(),
                reason
            ),
            ValidationError::InvalidTarget {
                slice,
                reference: _,
                reason,
            } => write!(
                f,
                "Invalid target in slice '{}': {}",
                slice.clone().into_inner().as_str(),
                reason
            ),
        }
    }
}

impl std::error::Error for ValidationError {}
