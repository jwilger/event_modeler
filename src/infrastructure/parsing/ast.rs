//! Abstract Syntax Tree (AST) for Event Model DSL.
//!
//! This module defines the structure of parsed Event Models. The AST
//! represents the logical structure of an event model without concern
//! for the concrete syntax used in the source file.
//!
//! # Structure
//!
//! An `EventModel` consists of:
//! - Metadata (title, description)
//! - Swimlanes containing entities
//! - Slices defining feature boundaries

use nutype::nutype;
use crate::infrastructure::types::{NonEmptyString, NonNegativeInt};

/// The root AST node representing a complete Event Model.
#[derive(Debug, Clone)]
pub struct EventModel {
    /// Model-level metadata.
    pub metadata: ModelMetadata,
    /// Horizontal swimlanes organizing entities by actor/system.
    pub swimlanes: Vec<Swimlane>,
    /// Vertical slices defining feature boundaries.
    pub slices: Vec<Slice>,
}

/// Metadata about the Event Model.
#[derive(Debug, Clone)]
pub struct ModelMetadata {
    /// Optional title for the model.
    pub title: Option<ModelTitle>,
    /// Optional description of the model.
    pub description: Option<ModelDescription>,
}

/// Title of an Event Model.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ModelTitle(NonEmptyString);

/// Description of an Event Model.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ModelDescription(NonEmptyString);

/// A horizontal swimlane grouping related entities.
///
/// Swimlanes typically represent actors, systems, or departments.
#[derive(Debug, Clone)]
pub struct Swimlane {
    /// Name of the swimlane.
    pub name: SwimlaneName,
    /// Entities contained in this swimlane.
    pub entities: Vec<Entity>,
}

/// Name of a swimlane.
#[nutype(
    derive(Debug, Clone)
)]
pub struct SwimlaneName(NonEmptyString);

/// An entity in the Event Model.
///
/// Entities are the core building blocks of an Event Model,
/// representing different aspects of the system.
#[derive(Debug, Clone)]
pub enum Entity {
    /// UI mockup showing inputs and outputs.
    Wireframe(Wireframe),
    /// User intention to change state.
    Command(Command),
    /// Record of a state change.
    Event(Event),
    /// Derived read model.
    Projection(Projection),
    /// Data retrieval operation.
    Query(Query),
    /// System-triggered action.
    Automation(Automation),
}

/// A UI wireframe showing user interface elements.
#[derive(Debug, Clone)]
pub struct Wireframe {
    /// Name of the wireframe.
    pub name: EntityName,
    /// Input fields displayed in the UI.
    pub inputs: Vec<InputField>,
    /// Output fields displayed in the UI.
    pub outputs: Vec<OutputField>,
    /// Optional link to detailed documentation.
    pub link: Option<DocumentationLink>,
}

/// A command representing a user's intention to change state.
#[derive(Debug, Clone)]
pub struct Command {
    /// Name of the command.
    pub name: EntityName,
    /// Actor who can issue this command.
    pub actor: Option<ActorName>,
    /// Data payload of the command.
    pub payload: Vec<PayloadField>,
    /// Optional link to detailed documentation.
    pub link: Option<DocumentationLink>,
}

/// An event representing a state change that has occurred.
#[derive(Debug, Clone)]
pub struct Event {
    /// Name of the event (past tense).
    pub name: EntityName,
    /// Logical timestamp for ordering.
    pub timestamp: Timestamp,
    /// Data recorded with the event.
    pub data: Vec<DataField>,
    /// Optional link to detailed documentation.
    pub link: Option<DocumentationLink>,
}

/// A projection representing a derived read model.
#[derive(Debug, Clone)]
pub struct Projection {
    /// Name of the projection.
    pub name: EntityName,
    /// Events that feed this projection.
    pub sources: Vec<EventReference>,
    /// Fields available in the projection.
    pub fields: Vec<ProjectionField>,
    /// Optional link to detailed documentation.
    pub link: Option<DocumentationLink>,
}

/// A query for retrieving data from a projection.
#[derive(Debug, Clone)]
pub struct Query {
    /// Name of the query.
    pub name: EntityName,
    /// Projection this query reads from.
    pub projection: ProjectionReference,
    /// Parameters for the query.
    pub parameters: Vec<QueryParameter>,
    /// Optional link to detailed documentation.
    pub link: Option<DocumentationLink>,
}

/// An automation that triggers actions based on events.
#[derive(Debug, Clone)]
pub struct Automation {
    /// Name of the automation.
    pub name: EntityName,
    /// Event that triggers this automation.
    pub trigger: EventReference,
    /// Command executed by this automation.
    pub action: CommandReference,
    /// Optional link to detailed documentation.
    pub link: Option<DocumentationLink>,
}

/// A vertical slice representing a feature boundary.
#[derive(Debug, Clone)]
pub struct Slice {
    /// Name of the slice.
    pub name: SliceName,
    /// Entity names contained in this slice.
    pub contains: Vec<EntityReference>,
    /// Optional acceptance criteria.
    pub given_when_then: Option<GivenWhenThen>,
}

/// Acceptance criteria in Given-When-Then format.
#[derive(Debug, Clone)]
pub struct GivenWhenThen {
    /// Preconditions (Given).
    pub given: ScenarioGiven,
    /// Action taken (When).
    pub when: ScenarioWhen,
    /// Expected outcomes (Then).
    pub then: Vec<ScenarioThen>,
}

// AST-specific newtype wrappers to ensure type safety
// These wrap already-validated types from type_safety module

/// Name of an entity.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct EntityName(NonEmptyString);

/// Name of an actor.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ActorName(NonEmptyString);

/// Input field name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct InputField(NonEmptyString);

/// Output field name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct OutputField(NonEmptyString);

/// Command payload field name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct PayloadField(NonEmptyString);

/// Event data field name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct DataField(NonEmptyString);

/// Projection field name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ProjectionField(NonEmptyString);

/// Query parameter name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct QueryParameter(NonEmptyString);

/// Logical timestamp for event ordering.
#[nutype(
    derive(Debug, Clone)
)]
pub struct Timestamp(NonNegativeInt);

/// Reference to an event by name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct EventReference(NonEmptyString);

/// Reference to a projection by name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ProjectionReference(NonEmptyString);

/// Reference to a command by name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct CommandReference(NonEmptyString);

/// Reference to any entity by name.
#[nutype(
    derive(Debug, Clone)
)]
pub struct EntityReference(NonEmptyString);

/// Name of a slice.
#[nutype(
    derive(Debug, Clone)
)]
pub struct SliceName(NonEmptyString);

/// Given clause in acceptance criteria.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ScenarioGiven(NonEmptyString);

/// When clause in acceptance criteria.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ScenarioWhen(NonEmptyString);

/// Then clause in acceptance criteria.
#[nutype(
    derive(Debug, Clone)
)]
pub struct ScenarioThen(NonEmptyString);

/// Link to external documentation.
#[nutype(
    derive(Debug, Clone)
)]
pub struct DocumentationLink(NonEmptyString);