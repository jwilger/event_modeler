//! Domain model entities for Event Modeling.
//!
//! This module defines the core entities that make up an Event Model.
//! All types use compile-time validation to ensure invariants are
//! maintained throughout the application.

use nutype::nutype;
use crate::infrastructure::types::{TypedPath, MarkdownFile, File, MaybeExists, NonEmpty, NonEmptyString, EventName as SafeEventName, NonNegativeInt};

/// A UI wireframe showing user interface elements.
#[derive(Debug, Clone)]
pub struct Wireframe {
    /// Unique identifier for this wireframe.
    pub id: EntityId,
    /// Name of the wireframe.
    pub name: WireframeName,
    /// Input fields displayed in the UI.
    pub inputs: NonEmpty<InputField>,
    /// Output fields displayed in the UI.
    pub outputs: NonEmpty<OutputField>,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

/// A command representing a user's intention to change state.
#[derive(Debug, Clone)]
pub struct Command {
    /// Unique identifier for this command.
    pub id: EntityId,
    /// Name of the command.
    pub name: CommandName,
    /// Actor who can issue this command.
    pub actor: Actor,
    /// Data payload of the command.
    pub payload: NonEmpty<PayloadField>,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

/// An event representing a state change that has occurred.
#[derive(Debug, Clone)]
pub struct Event {
    /// Unique identifier for this event.
    pub id: EntityId,
    /// Name of the event (past tense).
    pub name: EventName,
    /// Logical timestamp for ordering.
    pub timestamp: EventTimestamp,
    /// Data recorded with the event.
    pub data: NonEmpty<EventDataField>,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

/// A projection representing a derived read model.
#[derive(Debug, Clone)]
pub struct Projection {
    /// Unique identifier for this projection.
    pub id: EntityId,
    /// Name of the projection.
    pub name: ProjectionName,
    /// Events that feed this projection.
    pub sources: NonEmpty<EventId>,
    /// Fields available in the projection.
    pub fields: NonEmpty<ProjectionField>,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

/// A query for retrieving data from a projection.
#[derive(Debug, Clone)]
pub struct Query {
    /// Unique identifier for this query.
    pub id: EntityId,
    /// Name of the query.
    pub name: QueryName,
    /// Projection this query reads from.
    pub projection: ProjectionId,
    /// Parameters for the query.
    pub parameters: NonEmpty<QueryParameter>,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

/// An automation that triggers actions based on events.
#[derive(Debug, Clone)]
pub struct Automation {
    /// Unique identifier for this automation.
    pub id: EntityId,
    /// Name of the automation.
    pub name: AutomationName,
    /// Event that triggers this automation.
    pub trigger: EventId,
    /// Command executed by this automation.
    pub action: CommandId,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

// Distinct newtype wrappers using nutype without validation
// The inner types are already validated at system boundaries

/// Unique identifier for an entity.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct EntityId(NonEmptyString);

/// Name of a wireframe.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct WireframeName(NonEmptyString);

/// Name of a command.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct CommandName(NonEmptyString);

/// Name of an event (must start with uppercase letter).
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct EventName(SafeEventName);

/// Name of a projection.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct ProjectionName(NonEmptyString);

/// Name of a query.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct QueryName(NonEmptyString);

/// Name of an automation.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct AutomationName(NonEmptyString);

/// Actor who can issue commands.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct Actor(NonEmptyString);

/// Input field name.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct InputField(NonEmptyString);

/// Output field name.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct OutputField(NonEmptyString);

/// Command payload field name.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct PayloadField(NonEmptyString);

/// Event data field name.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct EventDataField(NonEmptyString);

/// Projection field name.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct ProjectionField(NonEmptyString);

/// Query parameter name.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq)
)]
pub struct QueryParameter(NonEmptyString);

/// Logical timestamp for event ordering.
#[nutype(
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)
)]
pub struct EventTimestamp(NonNegativeInt);

/// Reference to an event by ID.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct EventId(NonEmptyString);

/// Reference to a command by ID.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct CommandId(NonEmptyString);

/// Reference to a projection by ID.
#[nutype(
    derive(Debug, Clone, PartialEq, Eq, Hash)
)]
pub struct ProjectionId(NonEmptyString);