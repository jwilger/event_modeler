// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Entity registry with compile-time tracking of registered entities.
//!
//! This module provides a type-safe registry for Event Model entities using the typestate pattern.
//! The registry tracks which types of entities have been added at compile-time, preventing
//! access to collections that haven't been populated yet.
//!
//! # Example
//!
//! ```ignore
//! let registry = EntityRegistry::new()
//!     .add_wireframe(id1, wireframe1)
//!     .add_command(id2, command1)
//!     .add_event(id3, event1);
//!
//! // This would be a compile error if events hadn't been added:
//! let events = registry.events();
//! ```

use super::entities::{Automation, Command, EntityId, Event, Projection, Query, Wireframe};
use std::marker::PhantomData;

// Typestate markers for tracking which entities have been added

/// Marker type indicating no entities of this type have been added.
pub struct Empty;

/// Marker type indicating wireframes have been added to the registry.
pub struct HasWireframes;

/// Marker type indicating commands have been added to the registry.
pub struct HasCommands;

/// Marker type indicating events have been added to the registry.
pub struct HasEvents;

/// Marker type indicating projections have been added to the registry.
pub struct HasProjections;

/// Marker type indicating queries have been added to the registry.
pub struct HasQueries;

/// Marker type indicating automations have been added to the registry.
pub struct HasAutomations;

/// Entity registry with compile-time tracking of contents.
///
/// The registry uses phantom type parameters to track which entity types
/// have been added. This prevents calling accessor methods for entity types
/// that haven't been registered yet.
///
/// Type parameters:
/// - `W`: Wireframe state (Empty or HasWireframes)
/// - `C`: Command state (Empty or HasCommands)
/// - `E`: Event state (Empty or HasEvents)
/// - `P`: Projection state (Empty or HasProjections)
/// - `Q`: Query state (Empty or HasQueries)
/// - `A`: Automation state (Empty or HasAutomations)
#[derive(Debug, Clone)]
pub struct EntityRegistry<W, C, E, P, Q, A> {
    wireframes: Vec<(EntityId, Wireframe)>,
    commands: Vec<(EntityId, Command)>,
    events: Vec<(EntityId, Event)>,
    projections: Vec<(EntityId, Projection)>,
    queries: Vec<(EntityId, Query)>,
    automations: Vec<(EntityId, Automation)>,
    _phantom: PhantomData<(W, C, E, P, Q, A)>,
}

impl Default for EntityRegistry<Empty, Empty, Empty, Empty, Empty, Empty> {
    fn default() -> Self {
        Self::new()
    }
}

// Builder methods that change the typestate
impl EntityRegistry<Empty, Empty, Empty, Empty, Empty, Empty> {
    /// Creates a new empty entity registry.
    ///
    /// The registry starts with all entity types in the `Empty` state,
    /// meaning no accessor methods are available until entities are added.
    pub fn new() -> Self {
        Self {
            wireframes: Vec::new(),
            commands: Vec::new(),
            events: Vec::new(),
            projections: Vec::new(),
            queries: Vec::new(),
            automations: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<C, E, P, Q, A> EntityRegistry<Empty, C, E, P, Q, A> {
    /// Adds a wireframe to the registry.
    ///
    /// This method transitions the wireframe state from `Empty` to `HasWireframes`,
    /// enabling the `wireframes()` accessor method.
    pub fn with_wireframe(
        mut self,
        wireframe: Wireframe,
    ) -> EntityRegistry<HasWireframes, C, E, P, Q, A> {
        let id = wireframe.id.clone();
        self.wireframes.push((id, wireframe));
        EntityRegistry {
            wireframes: self.wireframes,
            commands: self.commands,
            events: self.events,
            projections: self.projections,
            queries: self.queries,
            automations: self.automations,
            _phantom: PhantomData,
        }
    }
}

impl<W, E, P, Q, A> EntityRegistry<W, Empty, E, P, Q, A> {
    /// Adds a command to the registry.
    ///
    /// This method transitions the command state from `Empty` to `HasCommands`,
    /// enabling the `commands()` accessor method.
    pub fn with_command(mut self, command: Command) -> EntityRegistry<W, HasCommands, E, P, Q, A> {
        let id = command.id.clone();
        self.commands.push((id, command));
        EntityRegistry {
            wireframes: self.wireframes,
            commands: self.commands,
            events: self.events,
            projections: self.projections,
            queries: self.queries,
            automations: self.automations,
            _phantom: PhantomData,
        }
    }
}

impl<W, C, P, Q, A> EntityRegistry<W, C, Empty, P, Q, A> {
    /// Adds an event to the registry.
    ///
    /// This method transitions the event state from `Empty` to `HasEvents`,
    /// enabling the `events()` accessor method.
    pub fn with_event(mut self, event: Event) -> EntityRegistry<W, C, HasEvents, P, Q, A> {
        let id = event.id.clone();
        self.events.push((id, event));
        EntityRegistry {
            wireframes: self.wireframes,
            commands: self.commands,
            events: self.events,
            projections: self.projections,
            queries: self.queries,
            automations: self.automations,
            _phantom: PhantomData,
        }
    }
}

// Methods available only when entities exist
impl<C, E, P, Q, A> EntityRegistry<HasWireframes, C, E, P, Q, A> {
    /// Returns a slice of all wireframes in the registry.
    ///
    /// This method is only available when the registry is in the `HasWireframes` state,
    /// which occurs after at least one wireframe has been added via `with_wireframe()`.
    pub fn wireframes(&self) -> &[(EntityId, Wireframe)] {
        &self.wireframes
    }
}

impl<W, E, P, Q, A> EntityRegistry<W, HasCommands, E, P, Q, A> {
    /// Returns a slice of all commands in the registry.
    ///
    /// This method is only available when the registry is in the `HasCommands` state,
    /// which occurs after at least one command has been added via `with_command()`.
    pub fn commands(&self) -> &[(EntityId, Command)] {
        &self.commands
    }
}

impl<W, C, P, Q, A> EntityRegistry<W, C, HasEvents, P, Q, A> {
    /// Returns a slice of all events in the registry.
    ///
    /// This method is only available when the registry is in the `HasEvents` state,
    /// which occurs after at least one event has been added via `with_event()`.
    pub fn events(&self) -> &[(EntityId, Event)] {
        &self.events
    }
}

impl<W, C, E, Q, A> EntityRegistry<W, C, E, Empty, Q, A> {
    /// Adds a projection to the registry.
    ///
    /// This method transitions the projection state from `Empty` to `HasProjections`,
    /// enabling the `projections()` accessor method.
    pub fn with_projection(
        mut self,
        projection: Projection,
    ) -> EntityRegistry<W, C, E, HasProjections, Q, A> {
        let id = projection.id.clone();
        self.projections.push((id, projection));
        EntityRegistry {
            wireframes: self.wireframes,
            commands: self.commands,
            events: self.events,
            projections: self.projections,
            queries: self.queries,
            automations: self.automations,
            _phantom: PhantomData,
        }
    }
}

impl<W, C, E, P, A> EntityRegistry<W, C, E, P, Empty, A> {
    /// Adds a query to the registry.
    ///
    /// This method transitions the query state from `Empty` to `HasQueries`,
    /// enabling the `queries()` accessor method.
    pub fn with_query(mut self, query: Query) -> EntityRegistry<W, C, E, P, HasQueries, A> {
        let id = query.id.clone();
        self.queries.push((id, query));
        EntityRegistry {
            wireframes: self.wireframes,
            commands: self.commands,
            events: self.events,
            projections: self.projections,
            queries: self.queries,
            automations: self.automations,
            _phantom: PhantomData,
        }
    }
}

impl<W, C, E, P, Q> EntityRegistry<W, C, E, P, Q, Empty> {
    /// Adds an automation to the registry.
    ///
    /// This method transitions the automation state from `Empty` to `HasAutomations`,
    /// enabling the `automations()` accessor method.
    pub fn with_automation(
        mut self,
        automation: Automation,
    ) -> EntityRegistry<W, C, E, P, Q, HasAutomations> {
        let id = automation.id.clone();
        self.automations.push((id, automation));
        EntityRegistry {
            wireframes: self.wireframes,
            commands: self.commands,
            events: self.events,
            projections: self.projections,
            queries: self.queries,
            automations: self.automations,
            _phantom: PhantomData,
        }
    }
}

impl<W, C, E, Q, A> EntityRegistry<W, C, E, HasProjections, Q, A> {
    /// Returns a slice of all projections in the registry.
    ///
    /// This method is only available when the registry is in the `HasProjections` state,
    /// which occurs after at least one projection has been added via `with_projection()`.
    pub fn projections(&self) -> &[(EntityId, Projection)] {
        &self.projections
    }
}

impl<W, C, E, P, A> EntityRegistry<W, C, E, P, HasQueries, A> {
    /// Returns a slice of all queries in the registry.
    ///
    /// This method is only available when the registry is in the `HasQueries` state,
    /// which occurs after at least one query has been added via `with_query()`.
    pub fn queries(&self) -> &[(EntityId, Query)] {
        &self.queries
    }
}

impl<W, C, E, P, Q> EntityRegistry<W, C, E, P, Q, HasAutomations> {
    /// Returns a slice of all automations in the registry.
    ///
    /// This method is only available when the registry is in the `HasAutomations` state,
    /// which occurs after at least one automation has been added via `with_automation()`.
    pub fn automations(&self) -> &[(EntityId, Automation)] {
        &self.automations
    }
}

// Alternative: Use const generics for compile-time entity tracking
#[derive(Debug, Clone)]
pub struct StaticEntityRegistry<
    const N_WIREFRAMES: usize,
    const N_COMMANDS: usize,
    const N_EVENTS: usize,
> {
    wireframes: [(EntityId, Wireframe); N_WIREFRAMES],
    commands: [(EntityId, Command); N_COMMANDS],
    events: [(EntityId, Event); N_EVENTS],
}

// Entity references with compile-time guarantees
#[derive(Debug, Clone)]
pub struct EntityRef<T> {
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T> EntityRef<T> {
    /// Creates a new entity reference with the given index.
    ///
    /// The index should correspond to the position of the entity in its collection.
    /// This is a const function, allowing entity references to be created at compile time.
    pub const fn new(index: usize) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}

// Compile-time entity lookup
pub trait EntityLookup<T> {
    fn get(&self, reference: EntityRef<T>) -> &T;
}

impl<const N: usize, const M: usize, const L: usize> EntityLookup<Wireframe>
    for StaticEntityRegistry<N, M, L>
{
    fn get(&self, reference: EntityRef<Wireframe>) -> &Wireframe {
        &self.wireframes[reference.index].1
    }
}

impl<const N: usize, const M: usize, const L: usize> EntityLookup<Command>
    for StaticEntityRegistry<N, M, L>
{
    fn get(&self, reference: EntityRef<Command>) -> &Command {
        &self.commands[reference.index].1
    }
}

impl<const N: usize, const M: usize, const L: usize> EntityLookup<Event>
    for StaticEntityRegistry<N, M, L>
{
    fn get(&self, reference: EntityRef<Event>) -> &Event {
        &self.events[reference.index].1
    }
}
