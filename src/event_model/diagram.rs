//! Domain model for Event Model diagrams.
//!
//! This module defines the structure of a complete Event Model diagram,
//! combining entities with layout information for rendering.

use super::entities::EntityId;
use super::registry::EntityRegistry;
use crate::infrastructure::types::{NonEmpty, NonEmptyString, NonNegativeInt};
use nutype::nutype;

/// A complete Event Model diagram with entities and layout information.
#[derive(Debug, Clone)]
pub struct EventModelDiagram<W, C, E, P, Q, A> {
    /// Diagram metadata including title and description.
    pub metadata: DiagramMetadata,
    /// Horizontal swimlanes organizing entities.
    pub swimlanes: NonEmpty<Swimlane>,
    /// Registry of all entities in the diagram.
    pub entities: EntityRegistry<W, C, E, P, Q, A>,
    /// Vertical slices defining feature boundaries.
    pub slices: NonEmpty<Slice>,
    /// Connections between entities.
    pub connectors: Vec<Connector>,
}

/// Metadata about the diagram.
#[derive(Debug, Clone)]
pub struct DiagramMetadata {
    /// Title of the diagram.
    pub title: DiagramTitle,
    /// Optional description of the diagram.
    pub description: Option<DiagramDescription>,
}

/// A horizontal swimlane grouping related entities.
#[derive(Debug, Clone)]
pub struct Swimlane {
    /// Unique identifier for the swimlane.
    pub id: SwimlaneId,
    /// Display name of the swimlane.
    pub name: SwimlaneName,
    /// Vertical position (0 = top).
    pub position: SwimlanePosition,
    /// Entity IDs contained in this swimlane.
    pub entities: Vec<EntityId>,
}

/// A vertical slice representing a feature or story.
#[derive(Debug, Clone)]
pub struct Slice {
    /// Unique identifier for the slice.
    pub id: SliceId,
    /// Display name of the slice.
    pub name: SliceName,
    /// Horizontal boundaries of the slice.
    pub boundaries: SliceBoundaries,
    /// Entity IDs contained in this slice.
    pub entities: NonEmpty<EntityId>,
    /// Optional acceptance criteria for the slice.
    pub acceptance_criteria: Option<AcceptanceCriteria>,
}

/// Horizontal boundaries of a slice.
#[derive(Debug, Clone)]
pub struct SliceBoundaries {
    /// Starting X coordinate.
    pub start_x: HorizontalPosition,
    /// Ending X coordinate.
    pub end_x: HorizontalPosition,
}

/// Acceptance criteria in Given-When-Then format.
#[derive(Debug, Clone)]
pub struct AcceptanceCriteria {
    /// Initial conditions.
    pub given: GivenCondition,
    /// Action performed.
    pub when: WhenAction,
    /// Expected outcomes.
    pub then: NonEmpty<ThenExpectation>,
}

// Distinct newtype wrappers using nutype without validation
// The inner types are already validated at system boundaries

/// Title of a diagram.
#[nutype(derive(Debug, Clone, PartialEq, Eq))]
pub struct DiagramTitle(NonEmptyString);

/// Description of a diagram.
#[nutype(derive(Debug, Clone))]
pub struct DiagramDescription(NonEmptyString);

/// Unique identifier for a swimlane.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct SwimlaneId(NonEmptyString);

/// Display name of a swimlane.
#[nutype(derive(Debug, Clone))]
pub struct SwimlaneName(NonEmptyString);

/// Vertical position of a swimlane.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct SwimlanePosition(NonNegativeInt);

/// Unique identifier for a slice.
#[nutype(derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct SliceId(NonEmptyString);

/// Display name of a slice.
#[nutype(derive(Debug, Clone))]
pub struct SliceName(NonEmptyString);

/// Horizontal position in the diagram.
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct HorizontalPosition(NonNegativeInt);

/// Given condition in acceptance criteria.
#[nutype(derive(Debug, Clone))]
pub struct GivenCondition(NonEmptyString);

/// When action in acceptance criteria.
#[nutype(derive(Debug, Clone))]
pub struct WhenAction(NonEmptyString);

/// Then expectation in acceptance criteria.
#[nutype(derive(Debug, Clone))]
pub struct ThenExpectation(NonEmptyString);

/// A connection between two entities.
#[derive(Debug, Clone)]
pub struct Connector {
    /// Source entity ID.
    pub from: EntityId,
    /// Target entity ID.
    pub to: EntityId,
    /// Optional label for the connection.
    pub label: Option<ConnectorLabel>,
}

/// Label for a connector.
#[nutype(derive(Debug, Clone))]
pub struct ConnectorLabel(NonEmptyString);
