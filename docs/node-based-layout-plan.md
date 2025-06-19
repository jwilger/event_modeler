# Node-Based Layout Implementation Plan

## Overview

Implement a node-based layout architecture where entities can have multiple visual appearances in the diagram. This eliminates circular dependencies and matches the expected output shown in example.jpg.

## Branch Strategy

Create new feature branch: `feature/node-based-layout` based on `feature/rich-visual-rendering` (or main after PR #20 merges)

## Implementation Tasks

### 1. Core Types (2-3 hours)

```rust
// New types in diagram module
pub struct DiagramNode {
    pub id: NodeId,
    pub entity_ref: EntityReference,
    pub position: Position,
    pub dimensions: Dimensions,
    pub swimlane: SwimlaneId,
}

pub struct NodeId(String); // e.g., "view_NewAccountScreen_1"

pub struct NodeRegistry {
    nodes: HashMap<NodeId, DiagramNode>,
    entity_to_nodes: HashMap<EntityReference, Vec<NodeId>>,
}
```

### 2. Node Generation (3-4 hours)

- Analyze slice connections to identify required nodes
- Create algorithm to generate minimal set of nodes:
  - Each unique (entity, swimlane) combination gets one node initially
  - Split nodes when they would create cycles
  - Optimize for minimal crossings
- Build NodeRegistry during YAML to diagram conversion

### 3. Layout Engine Updates (4-5 hours)

- Modify `build_dependency_graph` to work with NodeIds instead of EntityIds
- Update topological sort to use nodes
- Remove conservative temporal filtering (no longer needed)
- Update `position_entities_in_timeline` to position nodes
- Ensure proper spacing between nodes in same swimlane

### 4. Connection Updates (2-3 hours)

- Update Connector to reference NodeIds instead of EntityIds
- Route connections between specific node instances
- Update SVG rendering to draw connections to/from nodes

### 5. Rendering Updates (2-3 hours)

- Render nodes using entity styling (color based on entity type)
- Ensure consistent appearance for nodes of same entity
- Add visual hints for multi-node entities (optional)

### 6. Testing (2-3 hours)

- Update existing layout tests
- Add tests for node generation
- Add tests for multi-node scenarios
- Verify circular dependency handling

## Success Criteria

1. example.eventmodel renders without circular dependency errors
2. All entity types appear in the diagram (no filtering)
3. Entities with multiple relationships appear as multiple nodes
4. Layout matches the structure shown in example.jpg
5. All tests pass

## Risks and Mitigation

1. **Risk**: Complex node generation logic
   - **Mitigation**: Start with simple approach (one node per entity reference), optimize later

2. **Risk**: Performance with many nodes
   - **Mitigation**: Not a concern per performance ADR; focus on correctness

3. **Risk**: Visual confusion with multiple nodes
   - **Mitigation**: Consistent styling, clear connections, optional visual hints

## Estimated Timeline

- Core implementation: 16-20 hours
- Testing and refinement: 4-6 hours
- Documentation: 2-3 hours
- **Total**: 22-29 hours

## Follow-up Work

After node-based layout is complete:
1. Rich entity content rendering (data schemas)
2. Test scenario sub-diagrams
3. Enhanced connection routing
4. Visual optimizations