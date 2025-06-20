# Continue with Custom SVG Layout Implementation

## Status

Accepted

## Context

During the incremental rendering approach (Phase 6 of PLANNING.md), we've been developing a custom SVG layout engine in `src/bin/horizontal_slice_test.rs`. After implementing 11 steps of incremental improvements, we've achieved a working diagram that includes:

- Dynamic swimlane rendering
- All entity types (Views, Commands, Events, Projections, Queries, Automations)
- Connection arrows with proper routing
- Test scenarios with Given/When/Then structure

However, several issues remain:
- Entity sizing doesn't match the gold master
- Slice widths are fixed rather than dynamic
- Test scenario layout doesn't match the expected format
- Overall dimensions need to be fully dynamic

We investigated whether existing Rust SVG/diagramming libraries could help:

### Libraries Evaluated

1. **graphviz-rust / layout crates**
   - Pros: Automatic layout algorithms
   - Cons: No native swimlane support, limited control over exact positioning, would require significant workarounds

2. **plotters**
   - Pros: Mature SVG generation
   - Cons: Designed for charts/plots, not diagram layouts

3. **svg crate**
   - Pros: Clean SVG generation API
   - Cons: No layout capabilities, just syntax generation

4. **railroad**
   - Pros: Designed for syntax diagrams
   - Cons: Too specialized for our use case

5. **JavaScript alternatives (mermaid.js, etc.)**
   - Pros: Mature layout engines
   - Cons: Requires JS runtime or external process

## Decision

We will continue with our custom SVG layout implementation rather than adopting an external library.

## Consequences

### Positive

- **Full control** over layout algorithm tailored to event modeling diagrams
- **No new dependencies** beyond what we already have
- **Already 80% complete** - the incremental approach has proven valuable
- **Type-safe** implementation consistent with project architecture
- **Specific to our domain** - can optimize for event model visualization needs

### Negative

- **More code to maintain** - layout algorithms are complex
- **Debugging burden** - positioning issues require manual calculation
- **Reinventing the wheel** - implementing features that exist in other libraries
- **Time investment** - completing the remaining 20% still requires effort

### Mitigation

To address the negatives:
1. Keep the layout code well-isolated and testable
2. Document the layout algorithm clearly
3. Consider extracting reusable layout utilities
4. Focus on completing the minimum viable layout first

## Implementation Plan

1. Finish dynamic slice width calculation based on content
2. Implement proper test scenario layout as grouped sections
3. Reduce entity sizes to match gold master proportions
4. Add dynamic canvas sizing for any number of swimlanes/slices
5. Extract working layout code from test binary into proper modules

## Notes

This decision aligns with the project's philosophy of type-driven development and making illegal states unrepresentable. While external libraries might save time initially, our custom implementation ensures the layout algorithm precisely matches the semantics of event modeling diagrams.