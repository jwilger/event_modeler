use std::marker::PhantomData;
use super::entities::{
    Wireframe, Command, Event, Projection, Query, Automation,
    EntityId
};

// Typestate for tracking which entities have been added
pub struct Empty;
pub struct HasWireframes;
pub struct HasCommands;
pub struct HasEvents;
pub struct HasProjections;
pub struct HasQueries;
pub struct HasAutomations;

// Entity registry with compile-time tracking of contents
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

// Builder methods that change the typestate
impl EntityRegistry<Empty, Empty, Empty, Empty, Empty, Empty> {
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
    pub fn with_command(
        mut self,
        command: Command,
    ) -> EntityRegistry<W, HasCommands, E, P, Q, A> {
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
    pub fn with_event(
        mut self,
        event: Event,
    ) -> EntityRegistry<W, C, HasEvents, P, Q, A> {
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
    pub fn wireframes(&self) -> &[(EntityId, Wireframe)] {
        &self.wireframes
    }
}

impl<W, E, P, Q, A> EntityRegistry<W, HasCommands, E, P, Q, A> {
    pub fn commands(&self) -> &[(EntityId, Command)] {
        &self.commands
    }
}

impl<W, C, P, Q, A> EntityRegistry<W, C, HasEvents, P, Q, A> {
    pub fn events(&self) -> &[(EntityId, Event)] {
        &self.events
    }
}

// Alternative: Use const generics for compile-time entity tracking
#[derive(Debug, Clone)]
pub struct StaticEntityRegistry<const N_WIREFRAMES: usize, const N_COMMANDS: usize, const N_EVENTS: usize> {
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