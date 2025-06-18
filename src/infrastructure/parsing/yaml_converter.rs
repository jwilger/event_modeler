// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Conversion from YAML parsing types to domain types.
//!
//! This module handles the transformation from the intermediate YAML parsing
//! representation to the strongly-typed domain model.

use crate::event_model::yaml_types as domain;
use crate::infrastructure::parsing::yaml_parser as parsing;
use crate::infrastructure::types::{NonEmpty, NonEmptyString, ParseError};
use std::collections::HashMap;

/// Helper function to convert a Vec to NonEmpty.
fn vec_to_non_empty<T>(vec: Vec<T>, name: &str) -> Result<NonEmpty<T>, ConversionError> {
    let mut iter = vec.into_iter();
    match iter.next() {
        Some(head) => {
            let tail: Vec<T> = iter.collect();
            Ok(NonEmpty::from_head_and_tail(head, tail))
        }
        None => Err(ConversionError::EmptyCollection(name.to_string())),
    }
}

/// Converts a parsed YAML model into the domain representation.
///
/// This function performs all necessary validation and transformation:
/// - Validates all entity references (swimlanes, etc.)
/// - Converts stringly-typed data to strongly-typed domain objects
/// - Ensures all invariants are met
pub fn convert_yaml_to_domain(
    yaml: parsing::YamlEventModel,
) -> Result<domain::YamlEventModel, ConversionError> {
    // Convert swimlanes
    let swimlanes = convert_swimlanes(yaml.swimlanes)?;

    // Build swimlane ID lookup for validation
    let swimlane_ids: Vec<String> = swimlanes
        .iter()
        .map(|s| s.id.clone().into_inner().into_inner())
        .collect();

    // Convert entities (with swimlane validation)
    let events = convert_events(yaml.events, &swimlane_ids)?;
    let commands = convert_commands(yaml.commands, &swimlane_ids)?;
    let views = convert_views(yaml.views, &swimlane_ids)?;
    let projections = convert_projections(yaml.projections, &swimlane_ids)?;
    let queries = convert_queries(yaml.queries, &swimlane_ids)?;
    let automations = convert_automations(yaml.automations, &swimlane_ids)?;

    // Convert slices
    let slices = convert_slices(yaml.slices)?;

    // Build the domain model
    Ok(domain::YamlEventModel {
        version: match yaml.version {
            Some(v) => {
                let non_empty = NonEmptyString::parse(v).map_err(|e| match e {
                    ParseError::EmptyString => ConversionError::EmptyField("version".to_string()),
                    _ => ConversionError::ParseError(e),
                })?;
                Some(domain::SchemaVersion::new(non_empty))
            }
            None => None,
        },
        workflow: domain::WorkflowName::new(NonEmptyString::parse(yaml.workflow).map_err(|e| {
            match e {
                ParseError::EmptyString => ConversionError::EmptyField("workflow".to_string()),
                _ => ConversionError::ParseError(e),
            }
        })?),
        swimlanes,
        events,
        commands,
        views,
        projections,
        queries,
        automations,
        slices,
    })
}

/// Converts swimlane definitions.
fn convert_swimlanes(
    swimlanes: Vec<parsing::YamlSwimlane>,
) -> Result<NonEmpty<domain::Swimlane>, ConversionError> {
    let mut result = Vec::new();

    for swimlane in swimlanes {
        match swimlane {
            parsing::YamlSwimlane::Simple(name) => {
                // For simple format, use the name as both ID and display name
                let id = domain::SwimlaneId::new(
                    NonEmptyString::parse(name.clone())
                        .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
                );
                let display_name = domain::SwimlaneName::new(
                    NonEmptyString::parse(name)
                        .map_err(|_| ConversionError::EmptyField("swimlane name".to_string()))?,
                );
                result.push(domain::Swimlane {
                    id,
                    name: display_name,
                });
            }
            parsing::YamlSwimlane::Map(map) => {
                // For map format, key is ID, value is display name
                for (id_str, name_str) in map {
                    let id = domain::SwimlaneId::new(
                        NonEmptyString::parse(id_str)
                            .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
                    );
                    let name =
                        domain::SwimlaneName::new(NonEmptyString::parse(name_str).map_err(
                            |_| ConversionError::EmptyField("swimlane name".to_string()),
                        )?);
                    result.push(domain::Swimlane { id, name });
                }
            }
        }
    }

    vec_to_non_empty(result, "swimlanes")
}

/// Converts event definitions.
fn convert_events(
    events: HashMap<String, parsing::YamlEvent>,
    swimlane_ids: &[String],
) -> Result<HashMap<domain::EventName, domain::EventDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, event) in events {
        // Validate swimlane reference
        if !swimlane_ids.contains(&event.swimlane) {
            return Err(ConversionError::UnknownSwimlane(event.swimlane));
        }

        let name = domain::EventName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("event name".to_string()))?,
        );

        let definition = domain::EventDefinition {
            description: domain::Description::new(
                NonEmptyString::parse(event.description)
                    .map_err(|_| ConversionError::EmptyField("event description".to_string()))?,
            ),
            swimlane: domain::SwimlaneId::new(
                NonEmptyString::parse(event.swimlane)
                    .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
            ),
            data: convert_field_definitions(event.data)?,
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts field definitions from parsing to domain types.
fn convert_field_definitions(
    fields: HashMap<String, parsing::YamlField>,
) -> Result<HashMap<domain::FieldName, domain::FieldDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, field) in fields {
        let name = domain::FieldName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("field name".to_string()))?,
        );

        let definition = match field {
            parsing::YamlField::Simple(type_str) => domain::FieldDefinition {
                field_type: domain::FieldType::new(
                    NonEmptyString::parse(type_str)
                        .map_err(|_| ConversionError::EmptyField("field type".to_string()))?,
                ),
                stream_id: false,
                generated: false,
            },
            parsing::YamlField::Complex {
                field_type,
                stream_id,
                generated,
            } => domain::FieldDefinition {
                field_type: domain::FieldType::new(
                    NonEmptyString::parse(field_type)
                        .map_err(|_| ConversionError::EmptyField("field type".to_string()))?,
                ),
                stream_id,
                generated,
            },
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts command definitions.
fn convert_commands(
    commands: HashMap<String, parsing::YamlCommand>,
    swimlane_ids: &[String],
) -> Result<HashMap<domain::CommandName, domain::CommandDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, command) in commands {
        // Validate swimlane reference
        if !swimlane_ids.contains(&command.swimlane) {
            return Err(ConversionError::UnknownSwimlane(command.swimlane));
        }

        let name = domain::CommandName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("command name".to_string()))?,
        );

        let definition = domain::CommandDefinition {
            description: domain::Description::new(
                NonEmptyString::parse(command.description)
                    .map_err(|_| ConversionError::EmptyField("command description".to_string()))?,
            ),
            swimlane: domain::SwimlaneId::new(
                NonEmptyString::parse(command.swimlane)
                    .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
            ),
            data: convert_field_definitions(command.data)?,
            tests: convert_test_scenarios(command.tests)?,
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts test scenarios.
fn convert_test_scenarios(
    tests: HashMap<String, parsing::YamlTestScenario>,
) -> Result<HashMap<domain::TestScenarioName, domain::TestScenario>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, scenario) in tests {
        let name = domain::TestScenarioName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("test scenario name".to_string()))?,
        );

        // Convert Given events
        let given = convert_test_events(scenario.given)?;

        // Convert When actions
        let when_actions = convert_test_actions(scenario.when)?;
        let when = vec_to_non_empty(when_actions, "when actions")?;

        // Convert Then events
        let then_events = convert_test_events(scenario.then)?;
        let then = vec_to_non_empty(then_events, "then events")?;

        let test_scenario = domain::TestScenario { given, when, then };

        result.insert(name, test_scenario);
    }

    Ok(result)
}

/// Converts test events.
fn convert_test_events(
    events: Vec<parsing::YamlTestStep>,
) -> Result<Vec<domain::TestEvent>, ConversionError> {
    let mut result = Vec::new();

    for step in events {
        for (entity_name, fields) in step.step {
            let event_name = domain::EventName::new(
                NonEmptyString::parse(entity_name)
                    .map_err(|_| ConversionError::EmptyField("test event name".to_string()))?,
            );

            let mut event_fields = HashMap::new();
            for (field_name, value) in fields {
                let field = domain::FieldName::new(
                    NonEmptyString::parse(field_name)
                        .map_err(|_| ConversionError::EmptyField("test field name".to_string()))?,
                );
                let placeholder =
                    domain::PlaceholderValue::new(NonEmptyString::parse(value).map_err(|_| {
                        ConversionError::EmptyField("placeholder value".to_string())
                    })?);
                event_fields.insert(field, placeholder);
            }

            result.push(domain::TestEvent {
                name: event_name,
                fields: event_fields,
            });
        }
    }

    Ok(result)
}

/// Converts test actions (commands).
fn convert_test_actions(
    actions: Vec<parsing::YamlTestStep>,
) -> Result<Vec<domain::TestAction>, ConversionError> {
    let mut result = Vec::new();

    for step in actions {
        for (entity_name, fields) in step.step {
            let command_name = domain::CommandName::new(
                NonEmptyString::parse(entity_name)
                    .map_err(|_| ConversionError::EmptyField("test command name".to_string()))?,
            );

            let mut command_fields = HashMap::new();
            for (field_name, value) in fields {
                let field = domain::FieldName::new(
                    NonEmptyString::parse(field_name)
                        .map_err(|_| ConversionError::EmptyField("test field name".to_string()))?,
                );
                let placeholder =
                    domain::PlaceholderValue::new(NonEmptyString::parse(value).map_err(|_| {
                        ConversionError::EmptyField("placeholder value".to_string())
                    })?);
                command_fields.insert(field, placeholder);
            }

            result.push(domain::TestAction {
                name: command_name,
                fields: command_fields,
            });
        }
    }

    Ok(result)
}

/// Converts view definitions.
fn convert_views(
    views: HashMap<String, parsing::YamlView>,
    swimlane_ids: &[String],
) -> Result<HashMap<domain::ViewName, domain::ViewDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, view) in views {
        // Validate swimlane reference
        if !swimlane_ids.contains(&view.swimlane) {
            return Err(ConversionError::UnknownSwimlane(view.swimlane));
        }

        let name = domain::ViewName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("view name".to_string()))?,
        );

        let components = convert_components(view.components)?;
        let non_empty_components = vec_to_non_empty(components, "view components")?;

        let definition = domain::ViewDefinition {
            description: domain::Description::new(
                NonEmptyString::parse(view.description)
                    .map_err(|_| ConversionError::EmptyField("view description".to_string()))?,
            ),
            swimlane: domain::SwimlaneId::new(
                NonEmptyString::parse(view.swimlane)
                    .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
            ),
            components: non_empty_components,
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts UI components.
fn convert_components(
    components: Vec<parsing::YamlComponent>,
) -> Result<Vec<domain::Component>, ConversionError> {
    let mut result = Vec::new();

    for component in components {
        match component {
            parsing::YamlComponent::Simple { component } => {
                // Simple component: name -> type mapping
                for (name_str, type_str) in component {
                    let name =
                        domain::ComponentName::new(NonEmptyString::parse(name_str).map_err(
                            |_| ConversionError::EmptyField("component name".to_string()),
                        )?);
                    let component_type = domain::ComponentType::Simple(
                        domain::SimpleComponentType::new(NonEmptyString::parse(type_str).map_err(
                            |_| ConversionError::EmptyField("component type".to_string()),
                        )?),
                    );
                    result.push(domain::Component {
                        name,
                        component_type,
                    });
                }
            }
            parsing::YamlComponent::Complex { component } => {
                // Complex component: name -> { type, fields, actions }
                for (name_str, complex) in component {
                    let name =
                        domain::ComponentName::new(NonEmptyString::parse(name_str).map_err(
                            |_| ConversionError::EmptyField("component name".to_string()),
                        )?);

                    // Check if this is a form component
                    if complex.component_type.to_lowercase() == "form" {
                        // Convert form fields
                        let mut form_fields = HashMap::new();
                        for (field_name, field_type) in complex.fields {
                            let field =
                                domain::FieldName::new(NonEmptyString::parse(field_name).map_err(
                                    |_| ConversionError::EmptyField("form field name".to_string()),
                                )?);
                            let field_type = domain::SimpleComponentType::new(
                                NonEmptyString::parse(field_type).map_err(|_| {
                                    ConversionError::EmptyField("form field type".to_string())
                                })?,
                            );
                            form_fields.insert(field, field_type);
                        }

                        // Convert actions
                        let mut action_vec = Vec::new();
                        for action_str in complex.actions {
                            let action = domain::ActionName::new(
                                NonEmptyString::parse(action_str).map_err(|_| {
                                    ConversionError::EmptyField("action name".to_string())
                                })?,
                            );
                            action_vec.push(action);
                        }
                        let actions = vec_to_non_empty(action_vec, "form actions")?;

                        let component_type = domain::ComponentType::Form {
                            fields: form_fields,
                            actions,
                        };
                        result.push(domain::Component {
                            name,
                            component_type,
                        });
                    } else {
                        // Not a form, treat as simple component
                        let component_type =
                            domain::ComponentType::Simple(domain::SimpleComponentType::new(
                                NonEmptyString::parse(complex.component_type).map_err(|_| {
                                    ConversionError::EmptyField("component type".to_string())
                                })?,
                            ));
                        result.push(domain::Component {
                            name,
                            component_type,
                        });
                    }
                }
            }
        }
    }

    Ok(result)
}

/// Converts projection definitions.
fn convert_projections(
    projections: HashMap<String, parsing::YamlProjection>,
    swimlane_ids: &[String],
) -> Result<HashMap<domain::ProjectionName, domain::ProjectionDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, projection) in projections {
        // Validate swimlane reference
        if !swimlane_ids.contains(&projection.swimlane) {
            return Err(ConversionError::UnknownSwimlane(projection.swimlane));
        }

        let name = domain::ProjectionName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("projection name".to_string()))?,
        );

        let mut fields = HashMap::new();
        for (field_name, field_type) in projection.fields {
            let field =
                domain::FieldName::new(NonEmptyString::parse(field_name).map_err(|_| {
                    ConversionError::EmptyField("projection field name".to_string())
                })?);
            let ftype =
                domain::FieldType::new(NonEmptyString::parse(field_type).map_err(|_| {
                    ConversionError::EmptyField("projection field type".to_string())
                })?);
            fields.insert(field, ftype);
        }

        let definition = domain::ProjectionDefinition {
            description: domain::Description::new(
                NonEmptyString::parse(projection.description).map_err(|_| {
                    ConversionError::EmptyField("projection description".to_string())
                })?,
            ),
            swimlane: domain::SwimlaneId::new(
                NonEmptyString::parse(projection.swimlane)
                    .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
            ),
            fields,
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts query definitions.
fn convert_queries(
    queries: HashMap<String, parsing::YamlQuery>,
    swimlane_ids: &[String],
) -> Result<HashMap<domain::QueryName, domain::QueryDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, query) in queries {
        // Validate swimlane reference
        if !swimlane_ids.contains(&query.swimlane) {
            return Err(ConversionError::UnknownSwimlane(query.swimlane));
        }

        let name = domain::QueryName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("query name".to_string()))?,
        );

        // Convert inputs
        let mut inputs = HashMap::new();
        for (input_name, input_type) in query.inputs {
            let iname = domain::FieldName::new(
                NonEmptyString::parse(input_name)
                    .map_err(|_| ConversionError::EmptyField("query input name".to_string()))?,
            );
            let itype = domain::FieldType::new(
                NonEmptyString::parse(input_type)
                    .map_err(|_| ConversionError::EmptyField("query input type".to_string()))?,
            );
            inputs.insert(iname, itype);
        }

        // Convert outputs
        let outputs = convert_output_spec(query.outputs)?;

        let definition = domain::QueryDefinition {
            swimlane: domain::SwimlaneId::new(
                NonEmptyString::parse(query.swimlane)
                    .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
            ),
            inputs,
            outputs,
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts query output specifications.
fn convert_output_spec(
    output: parsing::YamlQueryOutput,
) -> Result<domain::OutputSpec, ConversionError> {
    let mut cases = HashMap::new();

    for (case_name, variant) in output.one_of {
        let case_name_domain = domain::OutputCaseName::new(
            NonEmptyString::parse(case_name)
                .map_err(|_| ConversionError::EmptyField("output case name".to_string()))?,
        );

        let case = match variant {
            parsing::YamlQueryVariant::Simple(error_type) => {
                // Simple string indicates an error type
                domain::OutputCase::Error(domain::ErrorTypeName::new(
                    NonEmptyString::parse(error_type)
                        .map_err(|_| ConversionError::EmptyField("error type name".to_string()))?,
                ))
            }
            parsing::YamlQueryVariant::Complex(fields) => {
                // Complex object with fields
                let mut field_map = HashMap::new();
                for (field_name, field_type) in fields {
                    let fname = domain::FieldName::new(NonEmptyString::parse(field_name).map_err(
                        |_| ConversionError::EmptyField("output field name".to_string()),
                    )?);
                    let ftype = domain::FieldType::new(NonEmptyString::parse(field_type).map_err(
                        |_| ConversionError::EmptyField("output field type".to_string()),
                    )?);
                    field_map.insert(fname, ftype);
                }
                domain::OutputCase::Fields(field_map)
            }
        };

        cases.insert(case_name_domain, case);
    }

    // If there's only one case that's a Fields variant, convert to Single
    match cases.len() {
        1 => {
            // Take ownership of the single item
            match cases.into_iter().next() {
                Some((_, domain::OutputCase::Fields(fields))) => {
                    Ok(domain::OutputSpec::Single(fields))
                }
                Some((k, v)) => {
                    // Not a Fields variant, recreate the map
                    let mut new_cases = HashMap::new();
                    new_cases.insert(k, v);
                    Ok(domain::OutputSpec::OneOf(new_cases))
                }
                None => unreachable!("HashMap with len 1 should have an item"),
            }
        }
        _ => Ok(domain::OutputSpec::OneOf(cases)),
    }
}

/// Converts automation definitions.
fn convert_automations(
    automations: HashMap<String, parsing::YamlAutomation>,
    swimlane_ids: &[String],
) -> Result<HashMap<domain::AutomationName, domain::AutomationDefinition>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, automation) in automations {
        // Validate swimlane reference
        if !swimlane_ids.contains(&automation.swimlane) {
            return Err(ConversionError::UnknownSwimlane(automation.swimlane));
        }

        let name = domain::AutomationName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("automation name".to_string()))?,
        );

        let definition = domain::AutomationDefinition {
            swimlane: domain::SwimlaneId::new(
                NonEmptyString::parse(automation.swimlane)
                    .map_err(|_| ConversionError::EmptyField("swimlane ID".to_string()))?,
            ),
        };

        result.insert(name, definition);
    }

    Ok(result)
}

/// Converts slice definitions.
fn convert_slices(
    slices: HashMap<String, Vec<String>>,
) -> Result<HashMap<domain::SliceName, NonEmpty<domain::Connection>>, ConversionError> {
    let mut result = HashMap::new();

    for (name_str, connections) in slices {
        let name = domain::SliceName::new(
            NonEmptyString::parse(name_str)
                .map_err(|_| ConversionError::EmptyField("slice name".to_string()))?,
        );

        let mut converted_connections = Vec::new();
        for conn_str in connections {
            let connection = parse_connection(&conn_str)?;
            converted_connections.push(connection);
        }

        let non_empty_connections = vec_to_non_empty(converted_connections, "slice connections")?;

        result.insert(name, non_empty_connections);
    }

    Ok(result)
}

/// Parses a connection string like "LoginScreen.CreateAccountLink -> CreateAccount".
fn parse_connection(conn_str: &str) -> Result<domain::Connection, ConversionError> {
    let parts: Vec<&str> = conn_str.split("->").map(|s| s.trim()).collect();

    if parts.len() != 2 {
        return Err(ConversionError::InvalidConnection(format!(
            "Expected 'from -> to' format, got: {}",
            conn_str
        )));
    }

    let from = parse_entity_reference(parts[0])?;
    let to = parse_entity_reference(parts[1])?;

    Ok(domain::Connection { from, to })
}

/// Parses an entity reference, determining its type from context.
fn parse_entity_reference(ref_str: &str) -> Result<domain::EntityReference, ConversionError> {
    if ref_str.is_empty() {
        return Err(ConversionError::EmptyField("entity reference".to_string()));
    }

    // Handle view paths (contain dots)
    if ref_str.contains('.') {
        let path = domain::ViewPath::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("view path".to_string()))?,
        );
        return Ok(domain::EntityReference::View(path));
    }

    // For other entity types, we need context to determine the type
    // This is a limitation of the current approach - we're guessing based on naming conventions
    // In a real implementation, we'd need to look up the entity in the registry

    // Try to guess based on common naming patterns
    let lower = ref_str.to_lowercase();

    if lower.ends_with("event") || lower.ends_with("ed") || lower.ends_with("sent") {
        // Likely an event (past tense or event-like ending)
        let name = domain::EventName::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("event name".to_string()))?,
        );
        Ok(domain::EntityReference::Event(name))
    } else if lower.ends_with("command")
        || lower.starts_with("create")
        || lower.starts_with("update")
        || lower.starts_with("delete")
    {
        // Likely a command
        let name = domain::CommandName::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("command name".to_string()))?,
        );
        Ok(domain::EntityReference::Command(name))
    } else if lower.ends_with("projection") {
        // Likely a projection
        let name = domain::ProjectionName::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("projection name".to_string()))?,
        );
        Ok(domain::EntityReference::Projection(name))
    } else if lower.ends_with("screen") || lower.ends_with("view") || lower.ends_with("page") {
        // Likely a view
        let path = domain::ViewPath::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("view path".to_string()))?,
        );
        Ok(domain::EntityReference::View(path))
    } else if lower.ends_with("query") || lower.starts_with("get") || lower.starts_with("find") {
        // Likely a query
        let name = domain::QueryName::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("query name".to_string()))?,
        );
        Ok(domain::EntityReference::Query(name))
    } else if lower.ends_with("automation") || lower.contains("process") {
        // Likely an automation
        let name = domain::AutomationName::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("automation name".to_string()))?,
        );
        Ok(domain::EntityReference::Automation(name))
    } else {
        // Default to command if we can't determine
        let name = domain::CommandName::new(
            NonEmptyString::parse(ref_str.to_string())
                .map_err(|_| ConversionError::EmptyField("command name".to_string()))?,
        );
        Ok(domain::EntityReference::Command(name))
    }
}

/// Errors that can occur during conversion.
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    /// A required field was empty.
    #[error("Field '{0}' cannot be empty")]
    EmptyField(String),

    /// An unknown swimlane was referenced.
    #[error("Unknown swimlane reference: {0}")]
    UnknownSwimlane(String),

    /// A slice connection was invalid.
    #[error("Invalid connection syntax: {0}")]
    InvalidConnection(String),

    /// A collection that must be non-empty was empty.
    #[error("Collection '{0}' must not be empty")]
    EmptyCollection(String),

    /// A parse error occurred.
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::parsing::yaml_parser;

    #[test]
    fn converts_minimal_event_model() {
        let yaml = r#"
workflow: Test Workflow
swimlanes:
  - user: "User Interface"
  - backend: "Backend System"
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.workflow.into_inner().into_inner(), "Test Workflow");
        assert_eq!(model.swimlanes.len(), 2);
    }

    #[test]
    fn converts_events_with_validation() {
        let yaml = r#"
workflow: Test
swimlanes:
  - backend: "Backend"
events:
  UserCreated:
    description: "A new user was created"
    swimlane: backend
    data:
      userId:
        type: UserId
        stream-id: true
      email: EmailAddress
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.events.len(), 1);

        let event = model.events.iter().next().unwrap();
        assert_eq!(event.0.clone().into_inner().into_inner(), "UserCreated");
        assert_eq!(
            event.1.description.clone().into_inner().into_inner(),
            "A new user was created"
        );
        assert_eq!(event.1.data.len(), 2);
    }

    #[test]
    fn rejects_unknown_swimlane() {
        let yaml = r#"
workflow: Test
swimlanes:
  - backend: "Backend"
events:
  UserCreated:
    description: "A new user was created"
    swimlane: unknown
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::UnknownSwimlane(s) => assert_eq!(s, "unknown"),
            _ => panic!("Expected UnknownSwimlane error"),
        }
    }

    #[test]
    fn converts_commands_with_tests() {
        let yaml = r#"
workflow: Test
swimlanes:
  - backend: "Backend"
commands:
  CreateUser:
    description: "Create a new user"
    swimlane: backend
    data:
      email: EmailAddress
    tests:
      happy_path:
        Given: []
        When:
          - CreateUser:
              email: A
        Then:
          - UserCreated:
              email: A
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.commands.len(), 1);

        let command = model.commands.iter().next().unwrap();
        assert_eq!(command.1.tests.len(), 1);

        let test = command.1.tests.iter().next().unwrap();
        assert_eq!(test.0.clone().into_inner().into_inner(), "happy_path");
        assert_eq!(test.1.given.len(), 0);
        assert_eq!(test.1.when.len(), 1);
        assert_eq!(test.1.then.len(), 1);
    }

    #[test]
    fn converts_view_components() {
        let yaml = r#"
workflow: Test
swimlanes:
  - ui: "UI"
views:
  LoginScreen:
    description: "User login screen"
    swimlane: ui
    components:
      - Title: Label
      - LoginForm:
          type: Form
          fields:
            email: TextInput
            password: PasswordInput
          actions:
            - Submit
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.views.len(), 1);

        let view = model.views.iter().next().unwrap();
        assert_eq!(view.1.components.len(), 2);
    }

    #[test]
    fn converts_query_with_one_of_outputs() {
        let yaml = r#"
workflow: Test
swimlanes:
  - backend: "Backend"
queries:
  GetUser:
    swimlane: backend
    inputs:
      userId: UserId
    outputs:
      one_of:
        success:
          user: UserData
        notFound: NotFoundError
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_ok());
        let model = result.unwrap();
        let query = model.queries.iter().next().unwrap();

        match &query.1.outputs {
            domain::OutputSpec::OneOf(cases) => {
                assert_eq!(cases.len(), 2);
            }
            _ => panic!("Expected OneOf output spec"),
        }
    }

    #[test]
    fn converts_slices_with_connections() {
        let yaml = r#"
workflow: Test
swimlanes:
  - ui: "UI"
slices:
  UserRegistration:
    - "LoginScreen.CreateAccountLink -> CreateAccount"
    - "CreateAccount -> UserCreated"
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_ok());
        let model = result.unwrap();
        assert_eq!(model.slices.len(), 1);

        let slice = model.slices.iter().next().unwrap();
        assert_eq!(
            slice.0.clone().into_inner().into_inner(),
            "UserRegistration"
        );
        assert_eq!(slice.1.len(), 2);
    }

    #[test]
    fn rejects_empty_collections() {
        let yaml = r#"
workflow: Test
swimlanes: []
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::EmptyCollection(s) => assert_eq!(s, "swimlanes"),
            _ => panic!("Expected EmptyCollection error"),
        }
    }

    #[test]
    fn rejects_empty_strings() {
        let yaml = r#"
workflow: ""
swimlanes:
  - test: "Test"
"#;
        let parsed = yaml_parser::parse_yaml(yaml).unwrap();
        let result = convert_yaml_to_domain(parsed);

        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::EmptyField(s) => assert_eq!(s, "workflow"),
            _ => panic!("Expected EmptyField error"),
        }
    }
}
