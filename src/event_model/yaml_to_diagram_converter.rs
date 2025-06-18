// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Converts YAML event models to event model diagrams.
//!
//! This module transforms the rich YAML-based event model representation
//! into the diagram structure used for layout and rendering.

use crate::event_model::{
    diagram::EventModelDiagram,
    entities::{Automation, Command, Event, Projection, Query, Wireframe},
    yaml_types as yaml,
};

/// Converts a YAML event model into a diagram representation.
///
/// This function transforms the rich YAML format (with data schemas, test scenarios,
/// UI components, etc.) into the simplified diagram format used for layout and rendering.
///
/// # Errors
///
/// Returns an error if:
/// - Entity references in slices cannot be resolved
/// - Required entities are missing
pub fn convert_yaml_to_diagram(
    _yaml_model: yaml::YamlEventModel,
) -> Result<
    EventModelDiagram<Wireframe, Command, Event, Projection, Query, Automation>,
    ConversionError,
> {
    // TODO: Implement the conversion from YAML model to diagram
    // This will:
    // 1. Convert swimlanes
    // 2. Convert each entity type to the diagram format
    // 3. Convert slices to connectors
    // 4. Build the EntityRegistry
    // 5. Create the EventModelDiagram

    todo!("YAML to diagram conversion not yet implemented")
}

/// Errors that can occur during YAML to diagram conversion.
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    /// An entity referenced in a slice was not found.
    #[error("Unknown entity in slice: {0}")]
    UnknownEntity(String),

    /// A swimlane referenced by an entity was not found.
    #[error("Unknown swimlane: {0}")]
    UnknownSwimlane(String),

    /// Failed to parse an entity reference.
    #[error("Invalid entity reference: {0}")]
    InvalidReference(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_model::yaml_types::*;
    use crate::infrastructure::types::NonEmpty;
    use std::collections::HashMap;

    #[test]
    #[ignore = "Implementation not complete yet"]
    fn converts_minimal_yaml_model_to_diagram() {
        use crate::infrastructure::types::NonEmptyString;

        // Create a minimal YAML model
        let workflow =
            WorkflowName::new(NonEmptyString::parse("Test Workflow".to_string()).unwrap());
        let swimlane_id = SwimlaneId::new(NonEmptyString::parse("test_lane".to_string()).unwrap());
        let swimlane_name =
            SwimlaneName::new(NonEmptyString::parse("Test Lane".to_string()).unwrap());
        let swimlane = Swimlane {
            id: swimlane_id.clone(),
            name: swimlane_name,
        };

        let event_name = EventName::new(NonEmptyString::parse("TestEvent".to_string()).unwrap());
        let event_desc =
            Description::new(NonEmptyString::parse("A test event".to_string()).unwrap());
        let event = EventDefinition {
            description: event_desc,
            swimlane: swimlane_id,
            data: HashMap::new(),
        };

        let mut events = HashMap::new();
        events.insert(event_name, event);

        let yaml_model = YamlEventModel {
            version: None, // Optional version
            workflow,
            swimlanes: NonEmpty::singleton(swimlane),
            events,
            commands: HashMap::new(),
            views: HashMap::new(),
            projections: HashMap::new(),
            queries: HashMap::new(),
            automations: HashMap::new(),
            slices: HashMap::new(),
        };

        // Convert to diagram
        let result = convert_yaml_to_diagram(yaml_model);

        // For now, just check that the function is callable
        // Implementation will be added next
        assert!(result.is_err()); // Should fail with todo!()
    }

    #[test]
    #[ignore = "Implementation not complete yet"]
    fn converts_yaml_with_slices_to_connectors() {
        use crate::infrastructure::types::NonEmptyString;

        // Create YAML model with entities and slices
        let workflow = WorkflowName::new(NonEmptyString::parse("Slice Test".to_string()).unwrap());
        let swimlane_id = SwimlaneId::new(NonEmptyString::parse("test_lane".to_string()).unwrap());
        let swimlane_name =
            SwimlaneName::new(NonEmptyString::parse("Test Lane".to_string()).unwrap());
        let swimlane = Swimlane {
            id: swimlane_id.clone(),
            name: swimlane_name,
        };

        // Create a command
        let command_name =
            CommandName::new(NonEmptyString::parse("TestCommand".to_string()).unwrap());
        let command_desc =
            Description::new(NonEmptyString::parse("Test command".to_string()).unwrap());
        let command = CommandDefinition {
            description: command_desc,
            swimlane: swimlane_id.clone(),
            data: HashMap::new(),
            tests: HashMap::new(),
        };

        // Create an event
        let event_name = EventName::new(NonEmptyString::parse("TestEvent".to_string()).unwrap());
        let event_desc = Description::new(NonEmptyString::parse("Test event".to_string()).unwrap());
        let event = EventDefinition {
            description: event_desc,
            swimlane: swimlane_id,
            data: HashMap::new(),
        };

        // Create a slice connecting them
        let slice_name = SliceName::new(NonEmptyString::parse("TestFlow".to_string()).unwrap());

        // Create Connection manually since it has fields, not a newtype
        let connection = Connection {
            from: EntityReference::Command(command_name.clone()),
            to: EntityReference::Event(event_name.clone()),
        };
        let connections = NonEmpty::singleton(connection);

        let mut commands = HashMap::new();
        commands.insert(command_name, command);

        let mut events = HashMap::new();
        events.insert(event_name, event);

        let mut slices = HashMap::new();
        slices.insert(slice_name, connections);

        let yaml_model = YamlEventModel {
            version: None,
            workflow,
            swimlanes: NonEmpty::singleton(swimlane),
            events,
            commands,
            views: HashMap::new(),
            projections: HashMap::new(),
            queries: HashMap::new(),
            automations: HashMap::new(),
            slices,
        };

        // Convert to diagram
        let result = convert_yaml_to_diagram(yaml_model);

        // For now, just check that the function is callable
        // Implementation will be added next
        assert!(result.is_err()); // Should fail with todo!()
    }
}
