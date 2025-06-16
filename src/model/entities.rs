use nutype::nutype;
use crate::type_safety::{TypedPath, MarkdownFile, File, MaybeExists, NonEmpty};

#[derive(Debug, Clone)]
pub struct Wireframe {
    pub id: EntityId,
    pub name: WireframeName,
    pub inputs: NonEmpty<InputField>,
    pub outputs: NonEmpty<OutputField>,
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub id: EntityId,
    pub name: CommandName,
    pub actor: Actor,
    pub payload: NonEmpty<PayloadField>,
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EntityId,
    pub name: EventName,
    pub timestamp: EventTimestamp,
    pub data: NonEmpty<EventDataField>,
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

#[derive(Debug, Clone)]
pub struct Projection {
    pub id: EntityId,
    pub name: ProjectionName,
    pub sources: NonEmpty<EventId>,
    pub fields: NonEmpty<ProjectionField>,
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub id: EntityId,
    pub name: QueryName,
    pub projection: ProjectionId,
    pub parameters: NonEmpty<QueryParameter>,
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

#[derive(Debug, Clone)]
pub struct Automation {
    pub id: EntityId,
    pub name: AutomationName,
    pub trigger: EventId,
    pub action: CommandId,
    pub documentation: Option<TypedPath<MarkdownFile, File, MaybeExists>>,
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct EntityId(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct WireframeName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct CommandName(String);

#[nutype(
    validate(regex = r"^[A-Z][a-zA-Z]*$"),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct EventName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct ProjectionName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct QueryName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct AutomationName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct Actor(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct InputField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct OutputField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct PayloadField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct EventDataField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct ProjectionField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct QueryParameter(String);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct EventTimestamp(u32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct EventId(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct CommandId(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct ProjectionId(String);