# ADR: Custom Layout Implementation for Event Modeling Diagrams

**Status**: Accepted  
**Date**: 2025-06-19  
**Decision Makers**: Development Team  

## Context

Event Modeler needs to generate sophisticated diagrams that visualize event-driven systems using a specific visual language. The gold master analysis revealed that these diagrams have unique layout requirements:

1. **Horizontal workflow slices** that represent complete user flows spanning across all swimlanes
2. **Multiple swimlanes** as vertical divisions for different system layers
3. **Entity positioning** based on temporal flow within each slice
4. **Test scenario sub-sections** within slices showing different paths
5. **Custom styling** per entity type with specific colors and shapes

We evaluated existing Rust diagramming and graph layout libraries to determine if any could meet these requirements.

## Decision

We will continue with a custom layout implementation rather than adopting an existing diagramming library.

## Rationale

### Unique Layout Requirements

Event modeling diagrams have a specialized layout that no existing library supports:

1. **Horizontal Slices**: Slices are horizontal bands representing workflows, not vertical dividers. No graph layout library provides this concept.

2. **Mixed Layout Paradigm**: We need:
   - Vertical stacking of slices
   - Horizontal flow of entities within each slice
   - Swimlanes as a cross-cutting concern
   - This hybrid approach doesn't fit standard graph layout algorithms

3. **Domain-Specific Positioning**: Entity placement is determined by:
   - Workflow sequence within a slice
   - Swimlane membership
   - Test scenario grouping
   - Not by generic graph algorithms

### Library Evaluation Results

We evaluated several categories of libraries:

**Graph Layout Libraries**:
- `fdg`, `forceatlas2`, `force-graph`: Force-directed layouts create organic, spring-like positioning unsuitable for structured workflows
- No support for swimlanes or horizontal sections
- Designed for network visualization, not process flows

**Graph Data Structure Libraries**:
- `petgraph`: Excellent for graph operations but no layout capabilities
- Requires external tools (Graphviz) for positioning
- Graphviz layouts (dot, neato, etc.) don't match our needs

**Visualization Libraries**:
- `plotters`: Designed for statistical charts, not diagrams
- `simplesvg`: Just SVG primitives, similar to what we already have

### Benefits of Custom Implementation

1. **Precise Control**: We can implement exactly the layout algorithm needed for event modeling diagrams

2. **No Impedance Mismatch**: Using a general library would require:
   - Complex workarounds for our specific needs
   - Post-processing of generated layouts
   - Fighting against the library's assumptions

3. **Existing Investment**: We already have:
   - Working SVG generation
   - Node-based layout foundation
   - Theme system with proper styling
   - Just need to adapt for horizontal slices

4. **Maintainability**: A purpose-built solution is easier to understand and modify than a complex integration with a general-purpose library

## Consequences

### Positive

- Full control over layout algorithm tailored to event modeling needs
- No external dependencies for core functionality
- Can evolve the layout as requirements change
- Clear, domain-specific code

### Negative

- Must implement and maintain layout algorithms ourselves
- Cannot benefit from community improvements to layout libraries
- Potentially more code to maintain

### Mitigations

- Use `petgraph` for graph algorithms (topological sort, cycle detection) where appropriate
- Keep layout code modular and well-tested
- Document the layout algorithm clearly
- Consider extracting as a separate crate if it proves useful beyond this project

## Alternatives Considered

1. **Use Graphviz via petgraph**: Would require extensive post-processing and doesn't support our layout needs
2. **Adapt force-directed layouts**: Would produce incorrect visualizations for workflow diagrams
3. **Use a JavaScript library via WASM**: Adds complexity and doesn't provide the specific layout we need

## Implementation Notes

The custom layout implementation will:
1. Treat slices as horizontal workflow sections
2. Stack slices vertically with proper spacing
3. Position entities within slices based on workflow connections
4. Respect swimlane boundaries while laying out entities
5. Add slice headers and test scenario groupings

We may use `petgraph` for:
- Dependency analysis within slices
- Topological sorting for entity ordering
- Cycle detection in workflows

## References

- [Gold Master Analysis](../gold-master-analysis.md)
- [Diagramming Library Evaluation](../diagramming-library-evaluation.md)
- [Horizontal Slice Architecture (Phase 6)](../../PLANNING.md#phase-6-horizontal-slice-architecture-new---critical)