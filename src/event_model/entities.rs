// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Domain model entities for Event Modeling.
//!
//! This module defines the core entities that make up an Event Model.
//! All types use compile-time validation to ensure invariants are
//! maintained throughout the application.

use crate::infrastructure::types::{
    EventName as SafeEventName, File, MarkdownFile, MaybeExists, NonEmpty, NonEmptyString,
    NonNegativeInt, TypedPath,
};
use nutype::nutype;
use std::collections::HashMap;

/// Type of entity in the event model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// UI wireframe entity.
    Wireframe,
    /// Command entity.
    Command,
    /// Event entity.
    Event,
    /// View entity (UI screen with components).
    View,
    /// Projection entity.
    Projection,
    /// Query entity.
    Query,
    /// Automation entity.
    Automation,
}

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
    /// Data schema with field definitions.
    pub data_schema: Option<HashMap<FieldName, FieldDefinition>>,
    /// Test scenarios for this command.
    pub test_scenarios: Option<HashMap<TestScenarioName, TestScenario>>,
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
    /// Fields available in the projection with type annotations.
    pub fields: HashMap<FieldName, FieldType>,
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
    /// Input parameters for the query with type annotations.
    pub inputs: HashMap<FieldName, FieldType>,
    /// Output specification (can be single or one-of multiple options).
    pub outputs: OutputSpec,
    /// Optional link to detailed documentation.
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

/// A view representing a UI screen with component hierarchy.
#[derive(Debug, Clone)]
pub struct View {
    /// Unique identifier for this view.
    pub id: EntityId,
    /// Name of the view.
    pub name: ViewName,
    /// UI components in this view.
    pub components: NonEmpty<Component>,
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
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct EntityId(NonEmptyString);

/// Name of a wireframe.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct WireframeName(NonEmptyString);

/// Name of a command.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct CommandName(NonEmptyString);

/// Name of an event (must start with uppercase letter).
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct EventName(SafeEventName);

/// Name of a projection.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ProjectionName(NonEmptyString);

/// Name of a query.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct QueryName(NonEmptyString);

/// Name of a view.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ViewName(NonEmptyString);

/// Name of an automation.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct AutomationName(NonEmptyString);

/// Actor who can issue commands.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct Actor(NonEmptyString);

/// Input field name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct InputField(NonEmptyString);

/// Output field name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct OutputField(NonEmptyString);

/// Command payload field name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct PayloadField(NonEmptyString);

/// Event data field name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct EventDataField(NonEmptyString);

/// Projection field name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ProjectionField(NonEmptyString);

/// Query parameter name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct QueryParameter(NonEmptyString);

/// Logical timestamp for event ordering.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct EventTimestamp(NonNegativeInt);

/// Reference to an event by ID.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct EventId(NonEmptyString);

/// Reference to a command by ID.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct CommandId(NonEmptyString);

/// Reference to a projection by ID.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct ProjectionId(NonEmptyString);

// Extended types for YAML support

/// Field name in a data schema.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct FieldName(NonEmptyString);

/// Field definition with type annotation and metadata.
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Type annotation for this field.
    pub field_type: FieldType,
    /// Whether this field is a stream identifier.
    pub stream_id: bool,
    /// Whether this field is generated by the system.
    pub generated: bool,
}

/// Type annotation for a field (e.g., "UserId", "EmailAddress").
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct FieldType(NonEmptyString);

/// Test scenario name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct TestScenarioName(NonEmptyString);

/// Test scenario with Given/When/Then structure.
#[derive(Debug, Clone)]
pub struct TestScenario {
    /// Given: initial state (list of events).
    pub given: Vec<TestEvent>,
    /// When: action taken (command).
    pub when: NonEmpty<TestAction>,
    /// Then: expected outcome (events).
    pub then: NonEmpty<TestEvent>,
}

/// Event reference in a test scenario.
#[derive(Debug, Clone)]
pub struct TestEvent {
    /// Name of the event.
    pub name: EventName,
    /// Field values using placeholder variables.
    pub fields: HashMap<FieldName, PlaceholderValue>,
}

/// Action in a test scenario (command execution).
#[derive(Debug, Clone)]
pub struct TestAction {
    /// Name of the command.
    pub name: CommandName,
    /// Field values using placeholder variables.
    pub fields: HashMap<FieldName, PlaceholderValue>,
}

/// Placeholder value in test scenarios (e.g., "A", "B", "C").
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct PlaceholderValue(NonEmptyString);

/// UI component definition.
#[derive(Debug, Clone)]
pub struct Component {
    /// Name of the component.
    pub name: ComponentName,
    /// Type of component or nested structure.
    pub component_type: ComponentType,
}

/// Component name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ComponentName(NonEmptyString);

/// Type of UI component.
#[derive(Debug, Clone)]
pub enum ComponentType {
    /// Simple component type (e.g., "Link", "TextInput").
    Simple(SimpleComponentType),
    /// Form component with fields and actions.
    Form {
        /// Form fields.
        fields: HashMap<FieldName, SimpleComponentType>,
        /// Form actions (e.g., Submit).
        actions: NonEmpty<ActionName>,
    },
}

/// Simple component type name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct SimpleComponentType(NonEmptyString);

/// Action name (e.g., "Submit").
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ActionName(NonEmptyString);

/// Output specification for queries.
#[derive(Debug, Clone)]
pub enum OutputSpec {
    /// Single output structure.
    Single(HashMap<FieldName, FieldType>),
    /// One of multiple possible outputs.
    OneOf(HashMap<OutputCaseName, OutputCase>),
}

/// Name of an output case.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct OutputCaseName(NonEmptyString);

/// An output case definition.
#[derive(Debug, Clone)]
pub enum OutputCase {
    /// Success case with fields.
    Fields(HashMap<FieldName, FieldType>),
    /// Error case with error type.
    Error(ErrorTypeName),
}

/// Error type name.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ErrorTypeName(NonEmptyString);
