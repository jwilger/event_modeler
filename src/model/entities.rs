use nutype::nutype;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Wireframe {
    pub id: EntityId,
    pub name: WireframeName,
    pub inputs: InputFields,
    pub outputs: OutputFields,
    pub documentation: Option<DocumentationPath>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub id: EntityId,
    pub name: CommandName,
    pub actor: Actor,
    pub payload: PayloadFields,
    pub documentation: Option<DocumentationPath>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: EntityId,
    pub name: EventName,
    pub timestamp: EventTimestamp,
    pub data: EventData,
    pub documentation: Option<DocumentationPath>,
}

#[derive(Debug, Clone)]
pub struct Projection {
    pub id: EntityId,
    pub name: ProjectionName,
    pub sources: EventSources,
    pub fields: ProjectionFields,
    pub documentation: Option<DocumentationPath>,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub id: EntityId,
    pub name: QueryName,
    pub projection: ProjectionId,
    pub parameters: QueryParameters,
    pub documentation: Option<DocumentationPath>,
}

#[derive(Debug, Clone)]
pub struct Automation {
    pub id: EntityId,
    pub name: AutomationName,
    pub trigger: EventId,
    pub action: CommandId,
    pub documentation: Option<DocumentationPath>,
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

#[derive(Debug, Clone)]
pub struct InputFields(Vec<InputField>);

#[derive(Debug, Clone)]
pub struct OutputFields(Vec<OutputField>);

#[derive(Debug, Clone)]
pub struct PayloadFields(Vec<PayloadField>);

#[derive(Debug, Clone)]
pub struct EventData(Vec<EventDataField>);

#[derive(Debug, Clone)]
pub struct ProjectionFields(Vec<ProjectionField>);

#[derive(Debug, Clone)]
pub struct QueryParameters(Vec<QueryParameter>);

#[derive(Debug, Clone)]
pub struct EventSources(Vec<EventId>);

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

#[nutype(
    validate(predicate = |path: &PathBuf| path.extension().map_or(false, |ext| ext == "md")),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct DocumentationPath(PathBuf);

impl InputFields {
    pub fn new(fields: Vec<InputField>) -> Self {
        Self(fields)
    }
    
    pub fn fields(&self) -> &[InputField] {
        &self.0
    }
}

impl OutputFields {
    pub fn new(fields: Vec<OutputField>) -> Self {
        Self(fields)
    }
    
    pub fn fields(&self) -> &[OutputField] {
        &self.0
    }
}

impl PayloadFields {
    pub fn new(fields: Vec<PayloadField>) -> Self {
        Self(fields)
    }
    
    pub fn fields(&self) -> &[PayloadField] {
        &self.0
    }
}

impl EventData {
    pub fn new(fields: Vec<EventDataField>) -> Self {
        Self(fields)
    }
    
    pub fn fields(&self) -> &[EventDataField] {
        &self.0
    }
}

impl ProjectionFields {
    pub fn new(fields: Vec<ProjectionField>) -> Self {
        Self(fields)
    }
    
    pub fn fields(&self) -> &[ProjectionField] {
        &self.0
    }
}

impl QueryParameters {
    pub fn new(params: Vec<QueryParameter>) -> Self {
        Self(params)
    }
    
    pub fn parameters(&self) -> &[QueryParameter] {
        &self.0
    }
}

impl EventSources {
    pub fn new(sources: Vec<EventId>) -> Self {
        Self(sources)
    }
    
    pub fn sources(&self) -> &[EventId] {
        &self.0
    }
}