use nutype::nutype;
use super::entities::{EntityId, Wireframe, Command, Event, Projection, Query, Automation};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EventModelDiagram {
    pub metadata: DiagramMetadata,
    pub swimlanes: Swimlanes,
    pub entities: EntityRegistry,
    pub slices: Slices,
}

#[derive(Debug, Clone)]
pub struct DiagramMetadata {
    pub title: DiagramTitle,
    pub description: Option<DiagramDescription>,
}

#[derive(Debug, Clone)]
pub struct Swimlanes(Vec<Swimlane>);

#[derive(Debug, Clone)]
pub struct Swimlane {
    pub id: SwimlaneId,
    pub name: SwimlaneName,
    pub position: SwimlanePosition,
    pub entities: Vec<EntityId>,
}

#[derive(Debug, Clone)]
pub struct EntityRegistry {
    wireframes: HashMap<EntityId, Wireframe>,
    commands: HashMap<EntityId, Command>,
    events: HashMap<EntityId, Event>,
    projections: HashMap<EntityId, Projection>,
    queries: HashMap<EntityId, Query>,
    automations: HashMap<EntityId, Automation>,
}

#[derive(Debug, Clone)]
pub struct Slices(Vec<Slice>);

#[derive(Debug, Clone)]
pub struct Slice {
    pub id: SliceId,
    pub name: SliceName,
    pub boundaries: SliceBoundaries,
    pub entities: Vec<EntityId>,
    pub acceptance_criteria: Option<AcceptanceCriteria>,
}

#[derive(Debug, Clone)]
pub struct SliceBoundaries {
    pub start_x: HorizontalPosition,
    pub end_x: HorizontalPosition,
}

#[derive(Debug, Clone)]
pub struct AcceptanceCriteria {
    pub given: GivenCondition,
    pub when: WhenAction,
    pub then: ThenExpectations,
}

#[derive(Debug, Clone)]
pub struct ThenExpectations(Vec<ThenExpectation>);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq),
)]
pub struct DiagramTitle(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct DiagramDescription(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct SwimlaneId(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct SwimlaneName(String);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct SwimlanePosition(u32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq, Eq, Hash),
)]
pub struct SliceId(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct SliceName(String);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct HorizontalPosition(u32);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct GivenCondition(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct WhenAction(String);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone),
)]
pub struct ThenExpectation(String);

impl Swimlanes {
    pub fn new(swimlanes: Vec<Swimlane>) -> Self {
        Self(swimlanes)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Swimlane> {
        self.0.iter()
    }
}

impl EntityRegistry {
    pub fn new() -> Self {
        Self {
            wireframes: HashMap::new(),
            commands: HashMap::new(),
            events: HashMap::new(),
            projections: HashMap::new(),
            queries: HashMap::new(),
            automations: HashMap::new(),
        }
    }
    
    pub fn add_wireframe(&mut self, wireframe: Wireframe) {
        self.wireframes.insert(wireframe.id.clone(), wireframe);
    }
    
    pub fn add_command(&mut self, command: Command) {
        self.commands.insert(command.id.clone(), command);
    }
    
    pub fn add_event(&mut self, event: Event) {
        self.events.insert(event.id.clone(), event);
    }
    
    pub fn add_projection(&mut self, projection: Projection) {
        self.projections.insert(projection.id.clone(), projection);
    }
    
    pub fn add_query(&mut self, query: Query) {
        self.queries.insert(query.id.clone(), query);
    }
    
    pub fn add_automation(&mut self, automation: Automation) {
        self.automations.insert(automation.id.clone(), automation);
    }
    
    pub fn get_wireframe(&self, id: &EntityId) -> Option<&Wireframe> {
        self.wireframes.get(id)
    }
    
    pub fn get_command(&self, id: &EntityId) -> Option<&Command> {
        self.commands.get(id)
    }
    
    pub fn get_event(&self, id: &EntityId) -> Option<&Event> {
        self.events.get(id)
    }
    
    pub fn get_projection(&self, id: &EntityId) -> Option<&Projection> {
        self.projections.get(id)
    }
    
    pub fn get_query(&self, id: &EntityId) -> Option<&Query> {
        self.queries.get(id)
    }
    
    pub fn get_automation(&self, id: &EntityId) -> Option<&Automation> {
        self.automations.get(id)
    }
}

impl Slices {
    pub fn new(slices: Vec<Slice>) -> Self {
        Self(slices)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = &Slice> {
        self.0.iter()
    }
}

impl ThenExpectations {
    pub fn new(expectations: Vec<ThenExpectation>) -> Self {
        Self(expectations)
    }
    
    pub fn expectations(&self) -> &[ThenExpectation] {
        &self.0
    }
}