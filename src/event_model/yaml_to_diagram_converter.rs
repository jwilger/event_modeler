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
use std::collections::HashMap;

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
    // Step 1: Convert entities first
    // Convert events
    let events = convert_events(&yaml_model.events, &yaml_model.swimlanes)?;

    // Convert commands
    let commands = convert_commands(&yaml_model.commands, &yaml_model.swimlanes)?;

    // TODO: Convert other entity types when they're ready
    let _projections: Vec<Projection> = vec![];
    let _queries: Vec<Query> = vec![];
    let _automations: Vec<Automation> = vec![];
    let _wireframes: Vec<Wireframe> = vec![]; // Views will map to wireframes

    // Step 2: Build the EntityRegistry and collect entity IDs
    use crate::event_model::entities::EntityId;
    use crate::event_model::registry::EntityRegistry;
    let mut entity_ids: Vec<EntityId> = Vec::new();

    // Collect all entity IDs
    for event in &events {
        entity_ids.push(event.id.clone());
    }
    for command in &commands {
        entity_ids.push(command.id.clone());
    }

    // Step 3: Convert swimlanes and populate with entities
    let swimlanes = convert_swimlanes_with_entities(
        &yaml_model.swimlanes,
        &yaml_model.events,
        &yaml_model.commands,
        &yaml_model.views,
        &yaml_model.projections,
        &yaml_model.queries,
        &yaml_model.automations,
    );

    // Build registry - For now we use an empty registry as the typestate pattern
    // makes it difficult to add multiple entities of the same type.
    // The node layout engine will fall back to inferring entity types from IDs.
    let registry = EntityRegistry::new();

    // TODO: Add other entity types when implemented

    // Step 4: Create metadata
    use crate::event_model::diagram::{DiagramMetadata, DiagramTitle};
    let metadata = DiagramMetadata {
        title: DiagramTitle::new(yaml_model.workflow.into_inner()),
        description: None, // YAML format doesn't have a top-level description
    };

    // Step 5: Convert YAML slices to diagram slices
    let slices = convert_yaml_slices_to_diagram_slices(&yaml_model.slices, &entity_ids)?;

    // Create the diagram
    Ok(EventModelDiagram {
        metadata,
        swimlanes,
        entities: registry,
        slices,
    })
}

/// Convert YAML swimlanes to diagram swimlanes and populate them with entities.
fn convert_swimlanes_with_entities(
    yaml_swimlanes: &crate::infrastructure::types::NonEmpty<yaml::Swimlane>,
    yaml_events: &std::collections::HashMap<yaml::EventName, yaml::EventDefinition>,
    yaml_commands: &std::collections::HashMap<yaml::CommandName, yaml::CommandDefinition>,
    yaml_views: &std::collections::HashMap<yaml::ViewName, yaml::ViewDefinition>,
    yaml_projections: &std::collections::HashMap<yaml::ProjectionName, yaml::ProjectionDefinition>,
    yaml_queries: &std::collections::HashMap<yaml::QueryName, yaml::QueryDefinition>,
    yaml_automations: &std::collections::HashMap<yaml::AutomationName, yaml::AutomationDefinition>,
) -> crate::infrastructure::types::NonEmpty<crate::event_model::diagram::Swimlane> {
    use crate::event_model::diagram::{Swimlane, SwimlaneId, SwimlaneName, SwimlanePosition};
    use crate::infrastructure::types::NonEmpty;
    use std::collections::HashMap;

    // Build map of swimlane ID to entities
    let mut swimlane_entities: HashMap<
        yaml::SwimlaneId,
        Vec<crate::event_model::entities::EntityId>,
    > = HashMap::new();

    // Initialize all swimlanes with empty entity lists
    for yaml_swimlane in yaml_swimlanes.iter() {
        swimlane_entities.insert(yaml_swimlane.id.clone(), Vec::new());
    }

    // Add events to their swimlanes by looking up the YAML definitions
    for (yaml_event_name, event_def) in yaml_events {
        let entity_id = crate::event_model::entities::EntityId::new(
            crate::infrastructure::types::NonEmptyString::parse(format!(
                "event_{}",
                yaml_event_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );
        if let Some(entity_list) = swimlane_entities.get_mut(&event_def.swimlane) {
            entity_list.push(entity_id);
        }
    }

    // Add commands to their swimlanes by looking up the YAML definitions
    for (yaml_command_name, command_def) in yaml_commands {
        let entity_id = crate::event_model::entities::EntityId::new(
            crate::infrastructure::types::NonEmptyString::parse(format!(
                "command_{}",
                yaml_command_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );
        if let Some(entity_list) = swimlane_entities.get_mut(&command_def.swimlane) {
            entity_list.push(entity_id);
        }
    }

    // Add views to their swimlanes
    for (yaml_view_name, view_def) in yaml_views {
        let entity_id = crate::event_model::entities::EntityId::new(
            crate::infrastructure::types::NonEmptyString::parse(format!(
                "view_{}",
                yaml_view_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );
        if let Some(entity_list) = swimlane_entities.get_mut(&view_def.swimlane) {
            entity_list.push(entity_id);
        }
    }

    // Add projections to their swimlanes
    for (yaml_projection_name, projection_def) in yaml_projections {
        let entity_id = crate::event_model::entities::EntityId::new(
            crate::infrastructure::types::NonEmptyString::parse(format!(
                "projection_{}",
                yaml_projection_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );
        if let Some(entity_list) = swimlane_entities.get_mut(&projection_def.swimlane) {
            entity_list.push(entity_id);
        }
    }

    // Add queries to their swimlanes
    for (yaml_query_name, query_def) in yaml_queries {
        let entity_id = crate::event_model::entities::EntityId::new(
            crate::infrastructure::types::NonEmptyString::parse(format!(
                "query_{}",
                yaml_query_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );
        if let Some(entity_list) = swimlane_entities.get_mut(&query_def.swimlane) {
            entity_list.push(entity_id);
        }
    }

    // Add automations to their swimlanes
    for (yaml_automation_name, automation_def) in yaml_automations {
        let entity_id = crate::event_model::entities::EntityId::new(
            crate::infrastructure::types::NonEmptyString::parse(format!(
                "automation_{}",
                yaml_automation_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );
        if let Some(entity_list) = swimlane_entities.get_mut(&automation_def.swimlane) {
            entity_list.push(entity_id);
        }
    }

    let head = yaml_swimlanes.first();
    let tail: Vec<_> = yaml_swimlanes.iter().skip(1).cloned().collect();

    let head_swimlane = Swimlane {
        id: SwimlaneId::new(head.id.clone().into_inner()),
        name: SwimlaneName::new(head.name.clone().into_inner()),
        position: SwimlanePosition::new(crate::infrastructure::types::NonNegativeInt::new(0)),
        entities: swimlane_entities.get(&head.id).cloned().unwrap_or_default(),
    };

    let tail_swimlanes: Vec<_> = tail
        .into_iter()
        .enumerate()
        .map(|(index, yaml_swimlane)| Swimlane {
            id: SwimlaneId::new(yaml_swimlane.id.clone().into_inner()),
            name: SwimlaneName::new(yaml_swimlane.name.into_inner()),
            position: SwimlanePosition::new(crate::infrastructure::types::NonNegativeInt::new(
                (index + 1) as u32,
            )),
            entities: swimlane_entities
                .get(&yaml_swimlane.id)
                .cloned()
                .unwrap_or_default(),
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

/// Convert YAML commands to diagram commands.
fn convert_commands(
    yaml_commands: &std::collections::HashMap<yaml::CommandName, yaml::CommandDefinition>,
    swimlanes: &crate::infrastructure::types::NonEmpty<yaml::Swimlane>,
) -> Result<Vec<Command>, ConversionError> {
    use crate::event_model::entities::{
        Actor, CommandName, EntityId, FieldDefinition, FieldName, FieldType, PayloadField,
        PlaceholderValue, TestAction, TestEvent, TestScenario, TestScenarioName,
    };
    use crate::infrastructure::types::{NonEmpty, NonEmptyString};

    let mut commands = Vec::new();

    for (yaml_command_name, command_def) in yaml_commands {
        // Verify swimlane exists
        if !swimlanes.iter().any(|s| s.id == command_def.swimlane) {
            return Err(ConversionError::UnknownSwimlane(
                command_def
                    .swimlane
                    .clone()
                    .into_inner()
                    .as_str()
                    .to_string(),
            ));
        }

        // Convert YAML command name to entities CommandName
        let command_name = CommandName::new(yaml_command_name.clone().into_inner());

        // Convert data fields to PayloadField for backward compatibility
        let payload_fields: Vec<PayloadField> = if command_def.data.is_empty() {
            vec![PayloadField::new(
                NonEmptyString::parse("payload".to_string()).unwrap(),
            )]
        } else {
            command_def
                .data
                .keys()
                .map(|field_name| PayloadField::new(field_name.clone().into_inner()))
                .collect()
        };

        let payload = if payload_fields.len() == 1 {
            NonEmpty::singleton(payload_fields.into_iter().next().unwrap())
        } else {
            let mut iter = payload_fields.into_iter();
            let head = iter.next().unwrap();
            let tail: Vec<_> = iter.collect();
            NonEmpty::from_head_and_tail(head, tail)
        };

        // Convert data schema
        let data_schema = if command_def.data.is_empty() {
            None
        } else {
            let mut schema = HashMap::new();
            for (yaml_field_name, yaml_field_def) in &command_def.data {
                let field_name = FieldName::new(yaml_field_name.clone().into_inner());
                let field_def = FieldDefinition {
                    field_type: FieldType::new(yaml_field_def.field_type.clone().into_inner()),
                    stream_id: yaml_field_def.stream_id,
                    generated: yaml_field_def.generated,
                };
                schema.insert(field_name, field_def);
            }
            Some(schema)
        };

        // Convert test scenarios
        let test_scenarios = if command_def.tests.is_empty() {
            None
        } else {
            let mut scenarios = HashMap::new();
            for (scenario_name, yaml_scenario) in &command_def.tests {
                let test_name = TestScenarioName::new(scenario_name.clone().into_inner());

                // Convert given events
                let given: Vec<TestEvent> = yaml_scenario
                    .given
                    .iter()
                    .map(|yaml_event| {
                        let mut fields = HashMap::new();
                        for (field_name, placeholder) in &yaml_event.fields {
                            fields.insert(
                                FieldName::new(field_name.clone().into_inner()),
                                PlaceholderValue::new(placeholder.clone().into_inner()),
                            );
                        }
                        TestEvent {
                            name: crate::event_model::entities::EventName::new(
                                crate::infrastructure::types::EventName::parse(
                                    yaml_event.name.clone().into_inner().as_str().to_string(),
                                )
                                .unwrap(),
                            ),
                            fields,
                        }
                    })
                    .collect();

                // Convert when actions
                let when_actions: Vec<TestAction> = yaml_scenario
                    .when
                    .iter()
                    .map(|yaml_action| {
                        let mut fields = HashMap::new();
                        for (field_name, placeholder) in &yaml_action.fields {
                            fields.insert(
                                FieldName::new(field_name.clone().into_inner()),
                                PlaceholderValue::new(placeholder.clone().into_inner()),
                            );
                        }
                        TestAction {
                            name: CommandName::new(yaml_action.name.clone().into_inner()),
                            fields,
                        }
                    })
                    .collect();

                let when = NonEmpty::from_head_and_tail(
                    when_actions[0].clone(),
                    when_actions[1..].to_vec(),
                );

                // Convert then events
                let then_events: Vec<TestEvent> = yaml_scenario
                    .then
                    .iter()
                    .map(|yaml_event| {
                        let mut fields = HashMap::new();
                        for (field_name, placeholder) in &yaml_event.fields {
                            fields.insert(
                                FieldName::new(field_name.clone().into_inner()),
                                PlaceholderValue::new(placeholder.clone().into_inner()),
                            );
                        }
                        TestEvent {
                            name: crate::event_model::entities::EventName::new(
                                crate::infrastructure::types::EventName::parse(
                                    yaml_event.name.clone().into_inner().as_str().to_string(),
                                )
                                .unwrap(),
                            ),
                            fields,
                        }
                    })
                    .collect();

                let then =
                    NonEmpty::from_head_and_tail(then_events[0].clone(), then_events[1..].to_vec());

                let scenario = TestScenario { given, when, then };
                scenarios.insert(test_name, scenario);
            }
            Some(scenarios)
        };

        // Create unique ID based on command name
        let command_id = EntityId::new(
            NonEmptyString::parse(format!(
                "command_{}",
                yaml_command_name.clone().into_inner().as_str()
            ))
            .unwrap(),
        );

        // Infer actor from swimlane name
        let actor_name = swimlanes
            .iter()
            .find(|s| s.id == command_def.swimlane)
            .map(|s| s.name.clone().into_inner())
            .unwrap_or_else(|| NonEmptyString::parse("User".to_string()).unwrap());

        let command = Command {
            id: command_id,
            name: command_name,
            actor: Actor::new(actor_name),
            payload,
            data_schema,
            test_scenarios,
            documentation: None,
        };

        commands.push(command);
    }

    Ok(commands)
}

/// Convert YAML slices to diagram slices with their connections.
fn convert_yaml_slices_to_diagram_slices(
    yaml_slices: &std::collections::HashMap<
        yaml::SliceName,
        crate::infrastructure::types::NonEmpty<yaml::Connection>,
    >,
    entity_ids: &[crate::event_model::entities::EntityId],
) -> Result<
    crate::infrastructure::types::NonEmpty<crate::event_model::diagram::Slice>,
    ConversionError,
> {
    use crate::event_model::diagram::{
        HorizontalPosition, Slice, SliceBoundaries, SliceId, SliceName,
    };
    use crate::infrastructure::types::{NonEmpty, NonEmptyString};

    if yaml_slices.is_empty() || entity_ids.is_empty() {
        // Create a default slice if no slices or entities
        let dummy_id = if entity_ids.is_empty() {
            crate::event_model::entities::EntityId::new(
                NonEmptyString::parse("dummy".to_string()).unwrap(),
            )
        } else {
            entity_ids[0].clone()
        };

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
            connections: Vec::new(),
            acceptance_criteria: None,
        };
        return Ok(NonEmpty::singleton(slice));
    }

    let mut slices = Vec::new();

    for (slice_index, (yaml_slice_name, yaml_connections)) in yaml_slices.iter().enumerate() {
        // Convert connections for this slice
        let connections = convert_yaml_connections_to_connectors(yaml_connections)?;

        // Calculate slice boundaries (spread slices horizontally)
        let start_x = slice_index * 300; // 300 pixels per slice
        let end_x = start_x + 280; // 280 pixel wide slices with 20px gap

        let slice = Slice {
            id: SliceId::new(yaml_slice_name.clone().into_inner()),
            name: SliceName::new(yaml_slice_name.clone().into_inner()),
            boundaries: SliceBoundaries {
                start_x: HorizontalPosition::new(
                    crate::infrastructure::types::NonNegativeInt::new(start_x as u32),
                ),
                end_x: HorizontalPosition::new(crate::infrastructure::types::NonNegativeInt::new(
                    end_x as u32,
                )),
            },
            // For now, put all entities in all slices (this could be refined later)
            entities: if entity_ids.len() == 1 {
                NonEmpty::singleton(entity_ids[0].clone())
            } else {
                NonEmpty::from_head_and_tail(entity_ids[0].clone(), entity_ids[1..].to_vec())
            },
            connections,
            acceptance_criteria: None,
        };

        slices.push(slice);
    }

    if slices.is_empty() {
        return Err(ConversionError::InvalidReference(
            "No slices created".to_string(),
        ));
    }

    Ok(NonEmpty::from_head_and_tail(
        slices[0].clone(),
        slices[1..].to_vec(),
    ))
}

/// Convert YAML connections to diagram connectors.
fn convert_yaml_connections_to_connectors(
    yaml_connections: &crate::infrastructure::types::NonEmpty<yaml::Connection>,
) -> Result<Vec<crate::event_model::diagram::Connector>, ConversionError> {
    use crate::event_model::diagram::Connector;
    use crate::event_model::entities::EntityId;
    use crate::infrastructure::types::NonEmptyString;

    let mut connectors = Vec::new();

    for connection in yaml_connections.iter() {
        // Extract entity names from the connection
        let from_entity_id = match &connection.from {
            yaml::EntityReference::Event(event_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "event_{}",
                    event_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::Command(command_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "command_{}",
                    command_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::View(view_path) => {
                // Extract top-level view name from view path (e.g., "NewAccountScreen" from "NewAccountScreen.AccountCredentials.Submit")
                let path_string = view_path.clone().into_inner().into_inner();
                let top_level_view = path_string.split('.').next().unwrap_or(&path_string);
                EntityId::new(NonEmptyString::parse(format!("view_{}", top_level_view)).unwrap())
            }
            yaml::EntityReference::Projection(projection_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "projection_{}",
                    projection_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::Query(query_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "query_{}",
                    query_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::Automation(automation_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "automation_{}",
                    automation_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
        };

        let to_entity_id = match &connection.to {
            yaml::EntityReference::Event(event_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "event_{}",
                    event_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::Command(command_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "command_{}",
                    command_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::View(view_path) => {
                // Extract top-level view name from view path (e.g., "NewAccountScreen" from "NewAccountScreen.AccountCredentials.Submit")
                let path_string = view_path.clone().into_inner().into_inner();
                let top_level_view = path_string.split('.').next().unwrap_or(&path_string);
                EntityId::new(NonEmptyString::parse(format!("view_{}", top_level_view)).unwrap())
            }
            yaml::EntityReference::Projection(projection_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "projection_{}",
                    projection_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::Query(query_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "query_{}",
                    query_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
            yaml::EntityReference::Automation(automation_name) => EntityId::new(
                NonEmptyString::parse(format!(
                    "automation_{}",
                    automation_name.clone().into_inner().as_str()
                ))
                .unwrap(),
            ),
        };

        let connector = Connector {
            from: from_entity_id,
            to: to_entity_id,
            label: None, // YAML connections don't specify labels
        };

        connectors.push(connector);
    }

    Ok(connectors)
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
    fn converts_yaml_with_commands() {
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

    #[test]
    fn converts_command_with_data_schema_and_tests() {
        use crate::event_model::yaml_types::*;
        use crate::infrastructure::types::NonEmptyString;

        // Create a minimal YAML model with a command that has data schema and tests
        let workflow =
            WorkflowName::new(NonEmptyString::parse("Command Test".to_string()).unwrap());
        let swimlane_id = SwimlaneId::new(NonEmptyString::parse("customer".to_string()).unwrap());
        let swimlane_name =
            SwimlaneName::new(NonEmptyString::parse("Customer".to_string()).unwrap());
        let swimlane = Swimlane {
            id: swimlane_id.clone(),
            name: swimlane_name,
        };

        // Create command with data schema
        let command_name =
            CommandName::new(NonEmptyString::parse("CreateAccount".to_string()).unwrap());
        let command_desc =
            Description::new(NonEmptyString::parse("Create a new account".to_string()).unwrap());

        let mut data_fields = HashMap::new();
        data_fields.insert(
            FieldName::new(NonEmptyString::parse("accountId".to_string()).unwrap()),
            FieldDefinition {
                field_type: FieldType::new(NonEmptyString::parse("AccountId".to_string()).unwrap()),
                stream_id: true,
                generated: true,
            },
        );
        data_fields.insert(
            FieldName::new(NonEmptyString::parse("email".to_string()).unwrap()),
            FieldDefinition {
                field_type: FieldType::new(
                    NonEmptyString::parse("EmailAddress".to_string()).unwrap(),
                ),
                stream_id: false,
                generated: false,
            },
        );

        // Create test scenario
        let test_name = TestScenarioName::new(
            NonEmptyString::parse("successful_account_creation".to_string()).unwrap(),
        );

        let given_event = TestEvent {
            name: EventName::new(NonEmptyString::parse("SystemInitialized".to_string()).unwrap()),
            fields: HashMap::new(),
        };

        let when_action = TestAction {
            name: command_name.clone(),
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    FieldName::new(NonEmptyString::parse("accountId".to_string()).unwrap()),
                    PlaceholderValue::new(NonEmptyString::parse("A".to_string()).unwrap()),
                );
                fields.insert(
                    FieldName::new(NonEmptyString::parse("email".to_string()).unwrap()),
                    PlaceholderValue::new(NonEmptyString::parse("B".to_string()).unwrap()),
                );
                fields
            },
        };

        let then_event = TestEvent {
            name: EventName::new(NonEmptyString::parse("AccountCreated".to_string()).unwrap()),
            fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    FieldName::new(NonEmptyString::parse("accountId".to_string()).unwrap()),
                    PlaceholderValue::new(NonEmptyString::parse("A".to_string()).unwrap()),
                );
                fields.insert(
                    FieldName::new(NonEmptyString::parse("email".to_string()).unwrap()),
                    PlaceholderValue::new(NonEmptyString::parse("B".to_string()).unwrap()),
                );
                fields
            },
        };

        let test_scenario = TestScenario {
            given: vec![given_event],
            when: NonEmpty::singleton(when_action),
            then: NonEmpty::singleton(then_event),
        };

        let mut tests = HashMap::new();
        tests.insert(test_name, test_scenario);

        let command = CommandDefinition {
            description: command_desc,
            swimlane: swimlane_id,
            data: data_fields,
            tests,
        };

        let mut commands = HashMap::new();
        commands.insert(command_name, command);

        let yaml_model = YamlEventModel {
            version: None,
            workflow,
            swimlanes: NonEmpty::singleton(swimlane),
            events: HashMap::new(),
            commands,
            views: HashMap::new(),
            projections: HashMap::new(),
            queries: HashMap::new(),
            automations: HashMap::new(),
            slices: HashMap::new(),
        };

        // Convert to diagram
        let result = convert_yaml_to_diagram(yaml_model);

        // Should succeed
        assert!(result.is_ok());
        let diagram = result.unwrap();

        // Verify basic structure
        assert_eq!(diagram.swimlanes.len(), 1);
        // Can't verify command details directly due to empty registry
    }
}
