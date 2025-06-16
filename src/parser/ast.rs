use nutype::nutype;

#[derive(Debug, Clone)]
pub struct EventModel {
    pub metadata: ModelMetadata,
    pub swimlanes: Vec<Swimlane>,
    pub slices: Vec<Slice>,
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub title: Option<ModelTitle>,
    pub description: Option<ModelDescription>,
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ModelTitle(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ModelDescription(String);

#[derive(Debug, Clone)]
pub struct Swimlane {
    pub name: SwimlaneName,
    pub entities: Vec<Entity>,
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct SwimlaneName(String);

#[derive(Debug, Clone)]
pub enum Entity {
    Wireframe(Wireframe),
    Command(Command),
    Event(Event),
    Projection(Projection),
    Query(Query),
    Automation(Automation),
}

#[derive(Debug, Clone)]
pub struct Wireframe {
    pub name: EntityName,
    pub inputs: Vec<InputField>,
    pub outputs: Vec<OutputField>,
    pub link: Option<DocumentationLink>,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: EntityName,
    pub actor: Option<ActorName>,
    pub payload: Vec<PayloadField>,
    pub link: Option<DocumentationLink>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub name: EntityName,
    pub timestamp: Timestamp,
    pub data: Vec<DataField>,
    pub link: Option<DocumentationLink>,
}

#[derive(Debug, Clone)]
pub struct Projection {
    pub name: EntityName,
    pub sources: Vec<EventReference>,
    pub fields: Vec<ProjectionField>,
    pub link: Option<DocumentationLink>,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub name: EntityName,
    pub projection: ProjectionReference,
    pub parameters: Vec<QueryParameter>,
    pub link: Option<DocumentationLink>,
}

#[derive(Debug, Clone)]
pub struct Automation {
    pub name: EntityName,
    pub trigger: EventReference,
    pub action: CommandReference,
    pub link: Option<DocumentationLink>,
}

#[derive(Debug, Clone)]
pub struct Slice {
    pub name: SliceName,
    pub contains: Vec<EntityReference>,
    pub given_when_then: Option<GivenWhenThen>,
}

#[derive(Debug, Clone)]
pub struct GivenWhenThen {
    pub given: ScenarioGiven,
    pub when: ScenarioWhen,
    pub then: Vec<ScenarioThen>,
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct EntityName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ActorName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct InputField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct OutputField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct PayloadField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct DataField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ProjectionField(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct QueryParameter(String);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone),
)]
pub struct Timestamp(u32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct EventReference(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ProjectionReference(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct CommandReference(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct EntityReference(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct SliceName(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ScenarioGiven(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ScenarioWhen(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ScenarioThen(String);

#[nutype(
    validate(predicate = |path| path.ends_with(".md")),
    derive(Debug, Clone),
)]
pub struct DocumentationLink(String);