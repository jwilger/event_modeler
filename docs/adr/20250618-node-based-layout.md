# ADR: Node-Based Layout Architecture

## Status
Proposed

## Context

During the implementation of Phase 5 (Rich Visual Rendering), we discovered that the example.jpg reference image shows entities appearing multiple times in the diagram. For instance, `NewAccountScreen` appears in multiple locations with different connections. This is a common pattern in event modeling diagrams to:

1. Reduce visual clutter from long crossing connection lines
2. Show different relationships and contexts clearly
3. Group related flows together
4. Make the diagram more readable

Our current implementation uses an entity-based layout where each entity (Command, Event, View, etc.) can only appear once in the diagram. This creates circular dependencies when trying to lay out entities that have bidirectional relationships (e.g., View → Command → Event → View).

## Decision

We will implement a **node-based layout architecture** where:

1. **Entities** are the logical domain objects (Commands, Events, Views, etc.)
2. **Nodes** are visual representations of entities in the diagram
3. Each entity can have multiple nodes (visual appearances)
4. Each connection endpoint creates or references a specific node
5. Layout algorithms work with nodes, not entities
6. Nodes maintain references to their underlying entities for styling and content

## Consequences

### Positive

1. **Eliminates circular dependencies**: Since each appearance is a separate node, there are no cycles in the layout graph
2. **Matches expected output**: Aligns with the example.jpg showing multiple entity appearances
3. **Improves readability**: Reduces long connection lines across the diagram
4. **Enables richer layouts**: Allows entities to appear in different contexts (main flow, test scenarios, etc.)
5. **Simplifies layout algorithm**: No need for complex cycle-breaking logic

### Negative

1. **Increased complexity**: Need to manage node-to-entity mapping
2. **Layout decisions**: Need heuristics to decide when to create new nodes vs reuse existing ones
3. **Potential confusion**: Users might not immediately recognize that multiple nodes represent the same entity

### Neutral

1. **Memory usage**: More nodes than entities, but negligible for typical diagrams
2. **Rendering performance**: More elements to render, but still well within acceptable limits

## Implementation Strategy

### Phase 1: Core Types
```rust
pub struct DiagramNode {
    pub id: NodeId,
    pub entity_ref: EntityReference,
    pub position: Position,
    pub dimensions: Dimensions,
}

pub struct NodeId(String); // e.g., "view_NewAccountScreen_1"
```

### Phase 2: Node Generation
- Analyze slice connections to determine required nodes
- Create nodes for each unique entity appearance in connections
- Maintain node registry for lookup

### Phase 3: Layout Update
- Modify layout engine to work with nodes instead of entities
- Update topological sort to use node dependencies
- Position nodes within swimlanes

### Phase 4: Rendering Update
- Render nodes with entity styling
- Route connections between specific nodes
- Maintain visual consistency for same-entity nodes

## Alternatives Considered

1. **Force-directed layout**: Would naturally handle cycles but gives less control over timeline flow
2. **Hierarchical layout with back-edges**: Complex to implement and may not match expected output
3. **Single entity with connection routing**: Would require complex routing algorithms to avoid overlaps
4. **Manual cycle breaking**: Current approach, but doesn't match the expected multi-appearance pattern

## References

- Example.jpg showing multiple appearances of entities
- Common practice in ER diagrams and data flow diagrams
- Event Modeling visual conventions