//! Converter from parsed event model to domain diagram model.
//!
//! This module handles the transformation from the parsed AST representation
//! to the strongly-typed domain model that can be used for layout and rendering.

use crate::event_model::diagram::{
    DiagramMetadata, DiagramTitle, EventModelDiagram, HorizontalPosition, Slice, SliceBoundaries,
    SliceId, SliceName, Swimlane, SwimlaneId, SwimlaneName, SwimlanePosition,
};
use crate::event_model::entities::EntityId;
use crate::event_model::registry::{Empty, EntityRegistry};
use crate::infrastructure::parsing::simple_parser::{ParsedEntity, ParsedEventModel};
use crate::infrastructure::types::{NonEmpty, NonEmptyString, NonNegativeInt};
use std::collections::HashMap;

/// Errors that can occur during conversion.
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    /// No swimlanes found in the parsed model.
    #[error("Event model must contain at least one swimlane")]
    NoSwimlanes,

    /// No entities found in the parsed model.
    #[error("Event model must contain at least one entity")]
    NoEntities,

    /// A connector references an unknown entity.
    #[error("Connector references unknown entity: {0}")]
    UnknownEntityInConnector(String),

    /// Failed to create a non-empty collection.
    #[error("Failed to create non-empty collection: {0}")]
    NonEmptyCreationFailed(String),
}

/// Convert a parsed event model into a domain diagram model.
///
/// Note: This is a simplified converter that creates a diagram structure
/// without fully populated entities. The entities in the registry are
/// placeholders as the full entity creation requires more complex validation
/// and the typestate pattern makes dynamic entity creation challenging.
pub fn convert_to_diagram(
    parsed: ParsedEventModel,
) -> Result<EventModelDiagram<Empty, Empty, Empty, Empty, Empty, Empty>, ConversionError> {
    // Check that we have at least one swimlane
    if parsed.swimlanes.is_empty() {
        return Err(ConversionError::NoSwimlanes);
    }

    // Create metadata
    let metadata = DiagramMetadata {
        title: DiagramTitle::new(parsed.title),
        description: None, // No description in simple parser yet
    };

    // Create entity lookup map and swimlane entities
    let mut entity_lookup: HashMap<String, EntityId> = HashMap::new();
    let mut swimlane_entities: HashMap<usize, Vec<EntityId>> = HashMap::new();
    let mut all_entity_ids = Vec::new();

    // Process swimlanes and create entity IDs
    for (swimlane_idx, parsed_swimlane) in parsed.swimlanes.iter().enumerate() {
        let mut lane_entities = Vec::new();

        for parsed_entity in &parsed_swimlane.entities {
            let entity_name = match parsed_entity {
                ParsedEntity::Command(name)
                | ParsedEntity::Event(name)
                | ParsedEntity::Projection(name)
                | ParsedEntity::Policy(name)
                | ParsedEntity::ExternalSystem(name)
                | ParsedEntity::Aggregate(name) => name.as_str(),
            };

            // Create a unique entity ID
            let entity_id = EntityId::new(NonEmptyString::parse(entity_name.to_string()).unwrap());
            entity_lookup.insert(entity_name.to_string(), entity_id.clone());
            lane_entities.push(entity_id.clone());
            all_entity_ids.push(entity_id);
        }

        swimlane_entities.insert(swimlane_idx, lane_entities);
    }

    // Check that we have at least one entity
    if all_entity_ids.is_empty() {
        return Err(ConversionError::NoEntities);
    }

    // Create swimlanes
    let mut swimlanes = Vec::new();
    for (idx, parsed_swimlane) in parsed.swimlanes.into_iter().enumerate() {
        let swimlane = Swimlane {
            id: SwimlaneId::new(parsed_swimlane.name.clone()),
            name: SwimlaneName::new(parsed_swimlane.name),
            position: SwimlanePosition::new(NonNegativeInt::new(idx as u32)),
            entities: swimlane_entities.get(&idx).cloned().unwrap_or_default(),
        };
        swimlanes.push(swimlane);
    }

    // Convert to NonEmpty
    let (first_swimlane, rest_swimlanes) = swimlanes
        .split_first()
        .ok_or_else(|| ConversionError::NonEmptyCreationFailed("swimlanes".to_string()))?;
    let swimlanes = NonEmpty::from_head_and_tail(first_swimlane.clone(), rest_swimlanes.to_vec());

    // Create a default slice containing all entities
    let default_slice = Slice {
        id: SliceId::new(
            NonEmptyString::parse("default_slice".to_string())
                .expect("Default slice ID is always non-empty"),
        ),
        name: SliceName::new(
            NonEmptyString::parse("Full Model".to_string())
                .expect("Default slice name is always non-empty"),
        ),
        boundaries: SliceBoundaries {
            start_x: HorizontalPosition::new(NonNegativeInt::new(0)),
            end_x: HorizontalPosition::new(NonNegativeInt::new(1000)),
        },
        entities: {
            let (first_entity, rest_entities) = all_entity_ids.split_first().ok_or_else(|| {
                ConversionError::NonEmptyCreationFailed("slice entities".to_string())
            })?;
            NonEmpty::from_head_and_tail(first_entity.clone(), rest_entities.to_vec())
        },
        acceptance_criteria: None,
    };

    let slices = NonEmpty::singleton(default_slice);

    // Create an empty registry
    // Note: In a full implementation, we would need to properly create
    // entities with all required fields and use the typestate pattern
    let entities = EntityRegistry::new();

    // Process connectors
    let mut connectors = Vec::new();
    for connector in parsed.connectors {
        // Get entity IDs
        let from_id = entity_lookup.get(connector.from.as_str()).ok_or_else(|| {
            ConversionError::UnknownEntityInConnector(connector.from.as_str().to_string())
        })?;
        let to_id = entity_lookup.get(connector.to.as_str()).ok_or_else(|| {
            ConversionError::UnknownEntityInConnector(connector.to.as_str().to_string())
        })?;

        connectors.push(crate::event_model::diagram::Connector {
            from: from_id.clone(),
            to: to_id.clone(),
            label: None, // Simple parser doesn't support connector labels yet
        });
    }

    Ok(EventModelDiagram {
        metadata,
        swimlanes,
        entities,
        slices,
        connectors,
    })
}

/// Information about entities in the parsed model.
///
/// This is used to provide entity counts and types for display purposes
/// without needing to fully construct domain entities.
pub struct ParsedEntityInfo {
    /// Number of wireframes.
    pub wireframe_count: usize,
    /// Number of commands.
    pub command_count: usize,
    /// Number of events.
    pub event_count: usize,
    /// Number of projections.
    pub projection_count: usize,
    /// Number of queries.
    pub query_count: usize,
    /// Number of automations.
    pub automation_count: usize,
}

/// Count entities by type in a parsed event model.
pub fn count_entities(parsed: &ParsedEventModel) -> ParsedEntityInfo {
    let mut info = ParsedEntityInfo {
        wireframe_count: 0,
        command_count: 0,
        event_count: 0,
        projection_count: 0,
        query_count: 0,
        automation_count: 0,
    };

    for swimlane in &parsed.swimlanes {
        for entity in &swimlane.entities {
            match entity {
                ParsedEntity::Command(_) => info.command_count += 1,
                ParsedEntity::Event(_) => info.event_count += 1,
                ParsedEntity::Projection(_) => info.projection_count += 1,
                ParsedEntity::Policy(_) => info.automation_count += 1,
                ParsedEntity::ExternalSystem(_) => info.projection_count += 1,
                ParsedEntity::Aggregate(_) => info.projection_count += 1,
            }
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::parsing::simple_parser::{ParsedConnector, ParsedSwimlane};
    use crate::infrastructure::types::Identifier;

    #[test]
    fn converts_minimal_parsed_model() {
        let parsed = ParsedEventModel {
            title: NonEmptyString::parse("Test Model".to_string()).unwrap(),
            swimlanes: vec![ParsedSwimlane {
                name: NonEmptyString::parse("System".to_string()).unwrap(),
                entities: vec![ParsedEntity::Event(
                    NonEmptyString::parse("TestEvent".to_string()).unwrap(),
                )],
            }],
            connectors: vec![],
        };

        let result = convert_to_diagram(parsed);
        assert!(result.is_ok());

        let diagram = result.unwrap();
        assert_eq!(diagram.metadata.title.into_inner().as_str(), "Test Model");
        assert_eq!(diagram.swimlanes.len(), 1);
        assert_eq!(diagram.slices.len(), 1);
    }

    #[test]
    fn counts_entities_correctly() {
        let parsed = ParsedEventModel {
            title: NonEmptyString::parse("Complex Model".to_string()).unwrap(),
            swimlanes: vec![
                ParsedSwimlane {
                    name: NonEmptyString::parse("Customer".to_string()).unwrap(),
                    entities: vec![
                        ParsedEntity::Command(
                            NonEmptyString::parse("PlaceOrder".to_string()).unwrap(),
                        ),
                        ParsedEntity::Event(
                            NonEmptyString::parse("OrderPlaced".to_string()).unwrap(),
                        ),
                    ],
                },
                ParsedSwimlane {
                    name: NonEmptyString::parse("System".to_string()).unwrap(),
                    entities: vec![
                        ParsedEntity::Projection(
                            NonEmptyString::parse("OrderList".to_string()).unwrap(),
                        ),
                        ParsedEntity::Policy(
                            NonEmptyString::parse("ProcessPayment".to_string()).unwrap(),
                        ),
                    ],
                },
            ],
            connectors: vec![],
        };

        let info = count_entities(&parsed);
        assert_eq!(info.command_count, 1);
        assert_eq!(info.event_count, 1);
        assert_eq!(info.projection_count, 1);
        assert_eq!(info.automation_count, 1);
    }

    #[test]
    fn errors_on_empty_swimlanes() {
        let parsed = ParsedEventModel {
            title: NonEmptyString::parse("Empty Model".to_string()).unwrap(),
            swimlanes: vec![],
            connectors: vec![],
        };

        let result = convert_to_diagram(parsed);
        assert!(matches!(result, Err(ConversionError::NoSwimlanes)));
    }

    #[test]
    fn errors_on_unknown_entity_in_connector() {
        let parsed = ParsedEventModel {
            title: NonEmptyString::parse("Test Model".to_string()).unwrap(),
            swimlanes: vec![ParsedSwimlane {
                name: NonEmptyString::parse("System".to_string()).unwrap(),
                entities: vec![ParsedEntity::Event(
                    NonEmptyString::parse("TestEvent".to_string()).unwrap(),
                )],
            }],
            connectors: vec![ParsedConnector {
                from: Identifier::parse("UnknownEntity".to_string()).unwrap(),
                to: Identifier::parse("TestEvent".to_string()).unwrap(),
            }],
        };

        let result = convert_to_diagram(parsed);
        assert!(matches!(
            result,
            Err(ConversionError::UnknownEntityInConnector(_))
        ));
    }
}
