# ADR: Node-Based Layout Architecture

## Status

Accepted (2025-01-18)

## Context

While implementing Phase 5 (Rich Visual Rendering), we discovered a fundamental mismatch between our entity-based layout system and the actual requirements shown in example.jpg. The example clearly shows that entities can appear multiple times in the diagram as separate visual nodes. For instance:

- A Command might appear once where it's triggered by a UI View
- The same Command appears again where it generates an Event
- An Event might appear multiple times - once where it's created and again where it's consumed

Our current implementation uses a one-entity-one-position model, which creates several problems:

1. **Visual Clutter**: All connections to/from an entity converge on a single visual representation, creating a "spider web" effect
2. **Circular Dependencies**: When entities reference each other in complex flows, we get cycles that are difficult to layout
3. **Poor Readability**: The timeline flow becomes unclear when entities have to connect backward to earlier positions
4. **Implementation Complexity**: We had to add "conservative temporal filtering" as a workaround, which removes valid connections from the diagram

## Decision

We will implement a node-based layout architecture where:

1. **DiagramNode**: A new type that represents a visual instance of an entity
   - Contains a reference to the entity (type + ID)
   - Has its own position in the layout
   - Maintains its own set of incoming/outgoing connections

2. **Node Generation**: Nodes are created from slice connections
   - Each source/target in a connection becomes a potential node
   - Multiple nodes can reference the same logical entity
   - Each node represents the entity in a specific context/flow

3. **Layout Algorithm**: Works with nodes instead of entities
   - Topological sort operates on nodes
   - Each node gets its own position
   - Connections route between specific node instances

4. **Entity Identity**: Preserved through node-to-entity mapping
   - Nodes inherit styling from their entity type
   - Entity data (schemas, etc.) is displayed on each node
   - Entity registry still tracks all entities, but layout uses nodes

## Consequences

### Positive

- **Eliminates Visual Clutter**: Each node has only its relevant connections
- **Removes Circular Dependencies**: Cycles exist between entities, not nodes
- **Improves Readability**: Clear left-to-right flow with entities appearing where needed
- **Simplifies Implementation**: No need for complex filtering or workarounds
- **Matches Requirements**: Directly implements the pattern shown in example.jpg

### Negative

- **Increased Complexity**: Additional abstraction layer between entities and visual representation
- **Memory Usage**: Multiple nodes per entity means more objects in memory
- **Layout Challenges**: Need to decide when to create new nodes vs reuse existing ones

### Neutral

- **Conceptual Shift**: Developers need to think in terms of nodes (visual) vs entities (logical)
- **Rendering Changes**: SVG generation needs to work with nodes instead of entities directly

## Implementation Plan

1. Create `DiagramNode` type with entity reference and position
2. Update slice processing to generate nodes from connections
3. Modify layout engine to position nodes instead of entities
4. Update SVG renderer to draw nodes with entity styling
5. Remove conservative temporal filtering workaround
6. Add tests for multi-node entity scenarios

## Example

Before (Entity-Based):
```
[Command:SignUp] ────────────────> [Event:UserCreated]
       ^                                    |
       |                                    v
[View:SignUpForm]                    [Projection:Users]
```

After (Node-Based):
```
[View:SignUpForm] ──> [Command:SignUp] ──> [Event:UserCreated] ──> [Projection:Users]
                                                      |
                                                      v
                            [Command:SendWelcome] ──> [Event:EmailSent]
                                      ^
                                      |
                              [Event:UserCreated]  // Second node for same event
```

In the node-based approach, `UserCreated` appears twice - once in the main flow and again where it triggers the welcome email automation. This matches the visual pattern in example.jpg and provides much clearer flow visualization.