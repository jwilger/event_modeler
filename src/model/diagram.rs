use nutype::nutype;
use super::entities::EntityId;
use super::registry::EntityRegistry;
use crate::type_safety::NonEmpty;

#[derive(Debug, Clone)]
pub struct EventModelDiagram<W, C, E, P, Q, A> {
    pub metadata: DiagramMetadata,
    pub swimlanes: NonEmpty<Swimlane>,
    pub entities: EntityRegistry<W, C, E, P, Q, A>,
    pub slices: NonEmpty<Slice>,
}

#[derive(Debug, Clone)]
pub struct DiagramMetadata {
    pub title: DiagramTitle,
    pub description: Option<DiagramDescription>,
}

#[derive(Debug, Clone)]
pub struct Swimlane {
    pub id: SwimlaneId,
    pub name: SwimlaneName,
    pub position: SwimlanePosition,
    pub entities: Vec<EntityId>,
}

#[derive(Debug, Clone)]
pub struct Slice {
    pub id: SliceId,
    pub name: SliceName,
    pub boundaries: SliceBoundaries,
    pub entities: NonEmpty<EntityId>,
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
    pub then: NonEmpty<ThenExpectation>,
}

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