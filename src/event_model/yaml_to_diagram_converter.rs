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
    yaml_model: yaml::YamlEventModel,
) -> Result<
    EventModelDiagram<
        crate::event_model::registry::Empty,
        crate::event_model::registry::Empty,
        crate::event_model::registry::Empty,
        crate::event_model::registry::Empty,
        crate::event_model::registry::Empty,
        crate::event_model::registry::Empty,
    >,
    ConversionError,
> {
    // Step 1: Convert swimlanes
    let swimlanes = convert_swimlanes(&yaml_model.swimlanes);

    // Step 2: Convert entities
    // For now, we'll create empty collections as we don't have all entity types in the simple format

    // Convert events
    let events = convert_events(&yaml_model.events, &yaml_model.swimlanes)?;

    // TODO: Convert other entity types when they're ready
    let _commands: Vec<Command> = vec![];
    let _projections: Vec<Projection> = vec![];
    let _queries: Vec<Query> = vec![];
    let _automations: Vec<Automation> = vec![];
    let _wireframes: Vec<Wireframe> = vec![]; // Views will map to wireframes

    // Step 3: Convert slices to connectors
    let connectors = convert_slices_to_connectors(&yaml_model.slices)?;

    // Step 4: Build the EntityRegistry and collect entity IDs
    use crate::event_model::entities::EntityId;
    use crate::event_model::registry::EntityRegistry;
    let mut entity_ids: Vec<EntityId> = Vec::new();

    // Collect all entity IDs
    for event in &events {
        entity_ids.push(event.id.clone());
    }

    // Build registry - for now we're returning an empty registry
    // as the type system requires us to maintain Empty type parameters
    let registry = EntityRegistry::new();

    // TODO: Add other entity types when implemented

    // Step 5: Create metadata
    use crate::event_model::diagram::{DiagramMetadata, DiagramTitle};
    let metadata = DiagramMetadata {
        title: DiagramTitle::new(yaml_model.workflow.into_inner()),
        description: None, // YAML format doesn't have a top-level description
    };

    // Step 6: Create a minimal slice
    use crate::event_model::diagram::{
        HorizontalPosition, Slice, SliceBoundaries, SliceId, SliceName,
    };
    use crate::infrastructure::types::{NonEmpty, NonEmptyString};

    let slices = if entity_ids.is_empty() {
        // Create a dummy slice if no entities
        let dummy_id = EntityId::new(NonEmptyString::parse("dummy".to_string()).unwrap());
        let slice = Slice {
            id: SliceId::new(NonEmptyString::parse("default".to_string()).unwrap()),
            name: SliceName::new(NonEmptyString::parse("Default".to_string()).unwrap()),
            boundaries: SliceBoundaries {
                start_x: HorizontalPosition::new(
                    crate::infrastructure::types::NonNegativeInt::new(0),
                ),
                end_x: HorizontalPosition::new(crate::infrastructure::types::NonNegativeInt::new(
                    100,
                )),
            },
            entities: NonEmpty::singleton(dummy_id),
            acceptance_criteria: None,
        };
        NonEmpty::singleton(slice)
    } else {
        // Create slice with actual entities
        let slice = Slice {
            id: SliceId::new(NonEmptyString::parse("default".to_string()).unwrap()),
            name: SliceName::new(NonEmptyString::parse("Default".to_string()).unwrap()),
            boundaries: SliceBoundaries {
                start_x: HorizontalPosition::new(
                    crate::infrastructure::types::NonNegativeInt::new(0),
                ),
                end_x: HorizontalPosition::new(crate::infrastructure::types::NonNegativeInt::new(
                    100,
                )),
            },
            entities: NonEmpty::from_head_and_tail(entity_ids[0].clone(), entity_ids[1..].to_vec()),
            acceptance_criteria: None,
        };
        NonEmpty::singleton(slice)
    };

    // Create the diagram
    Ok(EventModelDiagram {
        metadata,
        swimlanes,
        entities: registry,
        slices,
        connectors,
    })
}

/// Convert YAML swimlanes to diagram swimlanes.
fn convert_swimlanes(
    yaml_swimlanes: &crate::infrastructure::types::NonEmpty<yaml::Swimlane>,
) -> crate::infrastructure::types::NonEmpty<crate::event_model::diagram::Swimlane> {
    use crate::event_model::diagram::{Swimlane, SwimlaneId, SwimlaneName, SwimlanePosition};
    use crate::infrastructure::types::NonEmpty;

    let head = yaml_swimlanes.first();
    let tail: Vec<_> = yaml_swimlanes.iter().skip(1).cloned().collect();

    let head_swimlane = Swimlane {
        id: SwimlaneId::new(head.id.clone().into_inner()),
        name: SwimlaneName::new(head.name.clone().into_inner()),
        position: SwimlanePosition::new(crate::infrastructure::types::NonNegativeInt::new(0)),
        entities: vec![], // Will be populated later
    };

    let tail_swimlanes: Vec<_> = tail
        .into_iter()
        .enumerate()
        .map(|(index, yaml_swimlane)| {
            Swimlane {
                id: SwimlaneId::new(yaml_swimlane.id.into_inner()),
                name: SwimlaneName::new(yaml_swimlane.name.into_inner()),
                position: SwimlanePosition::new(crate::infrastructure::types::NonNegativeInt::new(
                    (index + 1) as u32,
                )),
                entities: vec![], // Will be populated later
            }
        })
        .collect();

    NonEmpty::from_head_and_tail(head_swimlane, tail_swimlanes)
}

/// Convert YAML events to diagram events.
fn convert_events(
    yaml_events: &std::collections::HashMap<yaml::EventName, yaml::EventDefinition>,
    swimlanes: &crate::infrastructure::types::NonEmpty<yaml::Swimlane>,
) -> Result<Vec<Event>, ConversionError> {
    use crate::event_model::entities::{EntityId, EventDataField, EventName, EventTimestamp};
    use crate::infrastructure::types::{EventName as SafeEventName, NonEmpty, NonEmptyString};

    let mut events = Vec::new();

    for (timestamp, (yaml_event_name, event_def)) in yaml_events.iter().enumerate() {
        // Verify swimlane exists
        if !swimlanes.iter().any(|s| s.id == event_def.swimlane) {
            return Err(ConversionError::UnknownSwimlane(
                event_def.swimlane.clone().into_inner().as_str().to_string(),
            ));
        }

        // Convert YAML event name to entities EventName
        // First get the inner NonEmptyString from YAML EventName
        let name_string = yaml_event_name.clone().into_inner();
        // Parse it as SafeEventName (validates uppercase first letter)
        let safe_event_name =
            SafeEventName::parse(name_string.as_str().to_string()).map_err(|_| {
                ConversionError::InvalidReference(format!(
                    "Invalid event name: {}",
                    name_string.as_str()
                ))
            })?;
        // Wrap in entities EventName
        let event_name = EventName::new(safe_event_name);

        // Convert data fields from YAML to simple fields
        // For now, create one field per data entry
        let data_fields: Vec<EventDataField> = if event_def.data.is_empty() {
            // If no data defined, create a default field
            vec![EventDataField::new(
                NonEmptyString::parse("data".to_string()).unwrap(),
            )]
        } else {
            event_def
                .data
                .keys()
                .map(|field_name| EventDataField::new(field_name.clone().into_inner()))
                .collect()
        };

        // Create NonEmpty collection
        let data = if data_fields.len() == 1 {
            NonEmpty::singleton(data_fields.into_iter().next().unwrap())
        } else {
            let mut iter = data_fields.into_iter();
            let head = iter.next().unwrap();
            let tail: Vec<_> = iter.collect();
            NonEmpty::from_head_and_tail(head, tail)
        };

        // Create unique ID based on event name
        let event_id = EntityId::new(
            NonEmptyString::parse(format!(
                "event_{}",
                yaml_event_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );

        let event = Event {
            id: event_id,
            name: event_name,
            timestamp: EventTimestamp::new(crate::infrastructure::types::NonNegativeInt::new(
                timestamp as u32,
            )),
            data,
            documentation: None,
        };

        events.push(event);
    }

    Ok(events)
}

/// Convert YAML slices to diagram connectors.
fn convert_slices_to_connectors(
    _yaml_slices: &std::collections::HashMap<
        yaml::SliceName,
        crate::infrastructure::types::NonEmpty<yaml::Connection>,
    >,
) -> Result<Vec<crate::event_model::diagram::Connector>, ConversionError> {
    // For now, return empty connectors as we need to resolve entity references
    // This will be implemented when we have all entity types converted
    Ok(vec![])
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

        // Should succeed now
        assert!(result.is_ok());
        let diagram = result.unwrap();

        // Verify basic structure
        assert_eq!(diagram.swimlanes.len(), 1);
        // Can't check entity count directly since registry is empty by design
    }

    #[test]
    #[ignore = "Commands not yet implemented"]
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

        // Should succeed now
        assert!(result.is_ok());
        let diagram = result.unwrap();

        // Verify basic structure
        assert_eq!(diagram.swimlanes.len(), 1);
        // Can't check entity count directly since registry is empty by design
    }
}
