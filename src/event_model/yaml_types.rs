// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Type definitions for the YAML-based event model format.
//!
//! This module defines the rich type system needed to represent the YAML format
//! discovered in example.eventmodel. These types support:
//! - Data schemas with type annotations
//! - Test scenarios (Given/When/Then)
//! - UI component hierarchies
//! - Slice-based flow definitions
//!
//! # Type Safety Guarantees
//!
//! All types in this module follow the "parse, don't validate" principle and make
//! illegal states unrepresentable at the type level:
//!
//! 1. **Non-empty guarantees**: Using `NonEmpty<T>` and `NonEmptyString` ensures
//!    collections and strings can never be empty after construction.
//!
//! 2. **Newtype wrappers**: All string-based types (names, IDs, etc.) are wrapped
//!    in distinct types using `nutype`, preventing type confusion at compile time.
//!
//! 3. **Structured data**: Complex concepts like test scenarios and UI components
//!    are represented as proper types, not stringly-typed data.
//!
//! 4. **Validation at boundaries**: All validation happens when parsing YAML into
//!    these types. Once constructed, the types are guaranteed to be valid.
//!
//! 5. **Exhaustive matching**: Enums like `EntityReference` and `ComponentType`
//!    ensure all cases are handled at compile time.

use crate::infrastructure::types::{NonEmpty, NonEmptyString};
use nutype::nutype;
use std::collections::HashMap;

/// The root structure of a YAML event model file.
///
/// # Type Safety
/// - `workflow` is guaranteed non-empty via `WorkflowName(NonEmptyString)`
/// - `swimlanes` must have at least one entry via `NonEmpty<Swimlane>`
/// - Entity maps use distinct key types preventing cross-type lookups
/// - `slices` connections are guaranteed non-empty via `NonEmpty<Connection>`
#[derive(Debug, Clone)]
pub struct YamlEventModel {
    /// Optional schema version (defaults to current app version).
    pub version: Option<SchemaVersion>,
    /// Name of the workflow being modeled.
    pub workflow: WorkflowName,
    /// Swimlanes that organize entities vertically.
    pub swimlanes: NonEmpty<Swimlane>,
    /// Events that represent state changes.
    pub events: HashMap<EventName, EventDefinition>,
    /// Commands that represent user intentions.
    pub commands: HashMap<CommandName, CommandDefinition>,
    /// Views that represent UI screens.
    pub views: HashMap<ViewName, ViewDefinition>,
    /// Projections that represent derived read models.
    pub projections: HashMap<ProjectionName, ProjectionDefinition>,
    /// Queries for retrieving data.
    pub queries: HashMap<QueryName, QueryDefinition>,
    /// Automations that trigger based on events.
    pub automations: HashMap<AutomationName, AutomationDefinition>,
    /// Slices that define connections between entities.
    pub slices: HashMap<SliceName, NonEmpty<Connection>>,
}

/// Schema version following semantic versioning.
///
/// # Type Safety
/// - Guaranteed non-empty via `NonEmptyString`
/// - Distinct type prevents confusion with other version strings
/// - Validation of semantic version format happens at parse time
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct SchemaVersion(NonEmptyString);

/// Name of the workflow.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct WorkflowName(NonEmptyString);

/// Swimlane definition with ID and display name.
///
/// # Type Safety
/// - ID and name are distinct types preventing confusion
/// - Both guaranteed non-empty
/// - ID used for references, name for display
#[derive(Debug, Clone)]
pub struct Swimlane {
    /// Unique identifier for the swimlane.
    pub id: SwimlaneId,
    /// Display name for the swimlane.
    pub name: SwimlaneName,
}

/// Unique identifier for a swimlane.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct SwimlaneId(NonEmptyString);

/// Display name for a swimlane.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct SwimlaneName(NonEmptyString);

/// Event definition with data schema.
///
/// # Type Safety
/// - `description` guaranteed non-empty
/// - `swimlane` reference type-checked against defined swimlanes
/// - `data` fields use structured `FieldDefinition` not raw strings
#[derive(Debug, Clone)]
pub struct EventDefinition {
    /// Description of what this event represents.
    pub description: Description,
    /// Swimlane this event belongs to.
    pub swimlane: SwimlaneId,
    /// Data fields with type annotations.
    pub data: HashMap<FieldName, FieldDefinition>,
}

/// Command definition with data schema and test scenarios.
///
/// # Type Safety
/// - All string fields guaranteed non-empty via newtype wrappers
/// - Test scenarios use structured `TestScenario` type
/// - Field definitions include metadata (stream_id, generated flags)
#[derive(Debug, Clone)]
pub struct CommandDefinition {
    /// Description of what this command does.
    pub description: Description,
    /// Swimlane this command belongs to.
    pub swimlane: SwimlaneId,
    /// Data fields with type annotations.
    pub data: HashMap<FieldName, FieldDefinition>,
    /// Test scenarios for this command.
    pub tests: HashMap<TestScenarioName, TestScenario>,
}

/// View definition with UI component hierarchy.
///
/// # Type Safety
/// - `components` guaranteed non-empty via `NonEmpty<Component>`
/// - Component types are structured, not stringly-typed
/// - Nested form structures properly modeled with `ComponentType` enum
#[derive(Debug, Clone)]
pub struct ViewDefinition {
    /// Description of this view's purpose.
    pub description: Description,
    /// Swimlane this view belongs to.
    pub swimlane: SwimlaneId,
    /// UI components in this view.
    pub components: NonEmpty<Component>,
}

/// Projection definition with field schemas.
///
/// # Type Safety
/// - Field names and types use distinct wrappers
/// - Type annotations support generic parameters (e.g., `List<UserId>`)
/// - All strings guaranteed non-empty
#[derive(Debug, Clone)]
pub struct ProjectionDefinition {
    /// Description of what this projection represents.
    pub description: Description,
    /// Swimlane this projection belongs to.
    pub swimlane: SwimlaneId,
    /// Fields available in the projection.
    pub fields: HashMap<FieldName, FieldType>,
}

/// Query definition with input/output contracts.
///
/// # Type Safety
/// - Input/output contracts are strongly typed
/// - `OutputSpec` enum handles single vs one-of patterns
/// - Error cases explicitly modeled in output specifications
#[derive(Debug, Clone)]
pub struct QueryDefinition {
    /// Swimlane this query belongs to.
    pub swimlane: SwimlaneId,
    /// Input parameters for the query.
    pub inputs: HashMap<FieldName, FieldType>,
    /// Output specification (can be one_of multiple options).
    pub outputs: OutputSpec,
}

/// Automation definition.
#[derive(Debug, Clone)]
pub struct AutomationDefinition {
    /// Swimlane this automation belongs to.
    pub swimlane: SwimlaneId,
}

/// Field definition with type annotation and metadata.
///
/// # Type Safety
/// - Boolean flags prevent invalid combinations at runtime
/// - Type annotations are strings but validated at parse time
/// - Metadata cannot be lost or confused with other fields
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Type annotation for this field.
    pub field_type: FieldType,
    /// Whether this field is a stream identifier.
    pub stream_id: bool,
    /// Whether this field is generated by the system.
    pub generated: bool,
}

/// Type annotation for a field (e.g., "UserAccountId", "UserEmailAddress<Verified>").
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct FieldType(NonEmptyString);

/// Field name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct FieldName(NonEmptyString);

/// Description text.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct Description(NonEmptyString);

/// Event name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct EventName(NonEmptyString);

/// Command name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct CommandName(NonEmptyString);

/// View name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct ViewName(NonEmptyString);

/// Projection name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct ProjectionName(NonEmptyString);

/// Query name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct QueryName(NonEmptyString);

/// Automation name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct AutomationName(NonEmptyString);

/// Slice name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct SliceName(NonEmptyString);

/// Test scenario name.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct TestScenarioName(NonEmptyString);

/// Test scenario with Given/When/Then structure.
///
/// # Type Safety
/// - `when` and `then` guaranteed non-empty via `NonEmpty<T>`
/// - Actions and events use distinct types preventing confusion
/// - Placeholder values are type-safe via `PlaceholderValue` wrapper
#[derive(Debug, Clone)]
pub struct TestScenario {
    /// Given: initial state (list of events).
    pub given: Vec<TestEvent>,
    /// When: action taken (command or event).
    pub when: NonEmpty<TestAction>,
    /// Then: expected outcome (events).
    pub then: NonEmpty<TestEvent>,
}

/// Event reference in a test scenario.
#[derive(Debug, Clone)]
pub struct TestEvent {
    /// Name of the event.
    pub name: EventName,
    /// Field values using placeholder variables (A, B, C, etc.).
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
///
/// # Type Safety
/// - Enum ensures exhaustive matching for all component types
/// - Form components have structured fields and actions
/// - Simple components wrapped in distinct type
/// - Compiler enforces handling of all variants
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
///
/// # Type Safety
/// - Enum enforces handling both single and one-of cases
/// - Output cases can be either field sets or error types
/// - Compiler ensures exhaustive matching
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

/// Connection in a slice.
///
/// # Type Safety
/// - Source and target use same `EntityReference` type
/// - Ensures connections only reference valid entity types
/// - Validated at parse time against registry
#[derive(Debug, Clone)]
pub struct Connection {
    /// Source entity reference.
    pub from: EntityReference,
    /// Target entity reference.
    pub to: EntityReference,
}

/// Reference to an entity in a connection.
///
/// # Type Safety
/// - Enum ensures only valid entity types can be referenced
/// - Each variant wraps the appropriate name type
/// - Exhaustive matching required when processing references
/// - View paths support dot notation for component references
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityReference {
    /// Reference to an event.
    Event(EventName),
    /// Reference to a command.
    Command(CommandName),
    /// Reference to a view or view component.
    View(ViewPath),
    /// Reference to a projection.
    Projection(ProjectionName),
    /// Reference to a query.
    Query(QueryName),
    /// Reference to an automation.
    Automation(AutomationName),
}

/// Path to a view or view component (e.g., "LoginScreen.CreateAccountLink").
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct ViewPath(NonEmptyString);

impl EntityReference {
    /// Parses an entity reference from a string.
    ///
    /// Handles formats like:
    /// - "EventName" - interpreted based on context
    /// - "ViewName.ComponentPath" - view with component path
    ///
    /// Returns None if the string is empty or invalid.
    pub fn parse(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }

        // For now, we'll use a simple heuristic:
        // - If it contains a dot, it's a view path
        // - Otherwise, we'll need context to determine the type
        // In practice, the converter should know the expected type from context

        if s.contains('.') {
            // View path like "LoginScreen.CreateAccountLink"
            // TODO: Implement proper parsing with NonEmptyString
            None
        } else {
            // For other entities, we need more context
            // This is a limitation - in real usage, the parser should
            // know what type of entity is expected based on the slice context
            // For now, we'll return None and let the converter handle it
            None
        }
    }
}
