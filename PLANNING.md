# Event Modeler Implementation Plan

This document outlines the complete rewrite plan for Event Modeler to support the rich YAML-based event modeling language discovered in example.eventmodel.

## 🚨 MANDATORY FIRST STEP 🚨

**ALWAYS create a todo list using TodoWrite as your VERY FIRST action when starting work.** This applies to:
- Starting a new work session
- Resuming work after a break
- Beginning implementation of any phase
- Even when you're just reviewing the plan

Your first todo item might simply be: "Review PLANNING.md to determine next tasks"

This ensures all work is tracked, organized, and nothing is missed.

## 🚨 TRACKING TODO COMMENTS 🚨

**CRITICAL**: When you discover work that needs to be done:
1. DO NOT just write TODO comments in code
2. IMMEDIATELY add the work item to PLANNING.md in the appropriate phase
3. Update your current TodoWrite list if the work affects the current phase
4. TODO comments are acceptable ONLY as temporary markers that are immediately tracked in PLANNING.md

This ensures no work is forgotten or lost in the codebase.

## Current Status

**Last Updated**: 2025-06-19 (Phase 1-5 COMPLETE, Phase 6 NEW - Horizontal Slices with Incremental Approach)

**Latest Progress**: 
- Phase 5 (Rich Visual Rendering) completed with limitations - PR #20 merged
- Node-based layout architecture completed - PR #21 merged
- Acceptance testing revealed fundamental slice architecture mismatch
- Gold master analysis completed - slices must be horizontal workflows
- PNG-based visual comparison testing implemented
- New Phase 6 added for horizontal slice architecture redesign
- Incremental rendering approach defined with 10 manual review checkpoints
- Library evaluation completed - continuing with custom implementation

**🚨 CRITICAL DISCOVERY - Node-Based Layout (2025-06-18)**: 
The example.jpg shows that entities can appear multiple times in the diagram as separate visual nodes. Each appearance is a distinct node with its own position and connections, even though they reference the same logical entity. This is essential for avoiding visual clutter and showing different relationships clearly. **This requires a fundamental shift from entity-based to node-based layout architecture.**

**Critical Discovery**: The existing implementation was based on incorrect requirements. The actual requirements call for a rich YAML-based event modeling language with:
- Multiple entity types (events, commands, views, projections, queries, automations)
- Data schemas with type annotations
- Test scenarios (Given/When/Then)
- UI component hierarchies  
- Slice-based flow definitions
- Professional visual output with color coding and sub-diagrams

The example.eventmodel and example.jpg files represent the TRUE requirements.

**Phase 1 COMPLETE**: Type System Overhaul completed with PR #15
- ✅ All entity types defined with type safety guarantees
- ✅ YAML registry for managing entities and connections
- ✅ ADRs created for YAML format and gold master testing
- ✅ Comprehensive documentation of type safety

**Phase 2 COMPLETE**: YAML Parser Implementation
- ✅ Added serde and serde_yaml dependencies
- ✅ Created ADR for schema versioning strategy
- ✅ Implemented VERSION constant for schema versioning
- ✅ Created YAML parsing types matching the format structure
- ✅ Implemented parse_yaml function with version checking
- ✅ Added EntityReference::parse method skeleton
- ✅ Created yaml_converter module with error types
- ✅ Completed conversion from parsing types to domain types
- ✅ Added comprehensive error handling with line/column numbers
- ✅ Updated README.md with YAML format specification
- ✅ Created comprehensive YAML syntax guide
- ✅ Updated CLAUDE.md with YAML-specific guidance

**Phase 3 COMPLETE**: Domain Model Extensions
- ✅ Extended Event to include data schema
- ✅ Extended Command to include data schema and test scenarios
- ✅ Implemented View with component hierarchies
- ✅ Implemented Projection with field schemas
- ✅ Implemented Query with input/output contracts
- ✅ Implemented Automation (basic support)
- ✅ Implemented Slice as first-class concept
- ✅ Updated EventModelDiagram to use slices for connections

**Phase 4 COMPLETE**: Flow-Based Layout Engine completed with PR #19
- ✅ Implemented topological sort for entity positioning based on slice connections
- ✅ Added flow-based layout algorithm that positions entities in timeline order
- ✅ Updated layout to use EntityRegistry for proper entity type and name lookup
- ✅ Maintains left-to-right timeline layout within swimlanes
- ✅ Added foundation for test scenario sub-diagram layout
- ✅ Handles circular dependency detection in entity flows
- ✅ Uses slice definitions to determine flow order

**Phase 5 COMPLETE (with limitations)**: Rich Visual Rendering (PR #20 - merging)
- ✅ Updated entity color scheme to match requirements (blue for commands/views/queries, purple for events, yellow for projections, green for automations)
- ✅ Enhanced entity text rendering with typography hierarchy (type labels + entity names)
- ✅ Added configurable entity sizing for better visual space (160x80 vs 120x60)
- ✅ Professional typography using theme font configuration
- ✅ Fixed circular dependency issue with conservative temporal filtering (temporary fix)
- ✅ Basic YAML integration working - can parse and render example.eventmodel
- ✅ Professional spacing and layout improvements
- ⚠️ **CRITICAL DISCOVERY**: Layout must support multiple visual nodes per entity (see example.jpg)
- ⚠️ Node-based layout architecture - **Required for full implementation**
- ⚠️ Rich entity content rendering (data schemas) - **Deferred: Requires node-based layout**
- ⚠️ Test scenario sub-diagrams - **Deferred: Requires node-based layout**

**Next Steps**: Continue node-based layout implementation in PR #21:
- ✅ DiagramNode type system implemented
- ✅ NodeLayoutEngine foundation created
- ✅ Node generation from slices working
- ✅ Topological sort for node positioning implemented
- ✅ SVG renderer updated to work with nodes
- ✅ Conservative temporal filtering removed
- ✅ All connection types enabled in the diagram
- ✅ Support for isolated entities added

PR #21 implementation is complete. Next step: Proceed to Phase 6 - Acceptance Testing & Documentation

**🚨 CRITICAL DISCOVERY - Horizontal Slices (2025-06-19)**: 
Acceptance testing revealed a fundamental misunderstanding of slice architecture. The gold master shows slices as **horizontal workflow sections**, not vertical dividers. Each slice represents a complete user flow that spans across all swimlanes horizontally. This is a major architectural change that affects:
- Slice interpretation and rendering
- Entity grouping and positioning
- Layout algorithms
- Visual styling and headers

See docs/gold-master-analysis.md for detailed findings.

**Architectural Impact**: This discovery means that much of the current layout implementation needs to be redesigned. The node-based layout work in PR #21 is still valuable, but it needs to work within horizontal slice boundaries rather than treating the entire diagram as one flow.

**Version Planning**: This rewrite will be released as version 0.3.0. Since we're pre-1.0, we can make breaking changes without maintaining backward compatibility. The YAML format will use this version number for its schema version.

### Existing Work to Preserve
- CLI structure and argument parsing (can be reused)
- Type-safety infrastructure (NonEmptyString, TypedPath, etc.)
- Some SVG rendering primitives (will need significant extension)
- Project structure and build configuration

## Overview

**CRITICAL**: The VERY FIRST step when starting any work session is to create a todo list using the TodoWrite tool. Even if the only item is "Review PLANNING.md to determine next tasks", you MUST create this todo list before doing anything else. This ensures work is always tracked and organized.

The implementation will follow a PR-driven workflow with feature branch chaining:
1. Create a feature branch for each phase
2. Implement the feature with acceptance tests
3. Create a PR with auto-merge enabled
4. Branch the next feature off the previous feature branch
5. Monitor PR status and fix any CI failures
6. Handle rebasing when base branches are merged

## ADRs to Create

The following Architecture Decision Records need to be created to document key decisions:

1. **ADR: Adopting YAML Format** (Phase 1) ✅ CREATED
   - Why we're moving from simple text to YAML
   - Benefits of structured format
   - Schema versioning strategy
   - Trade-offs considered

2. **ADR: Gold Master Testing Strategy** (Phase 1) ✅ CREATED
   - Why we chose insta for snapshot testing
   - Visual comparison approach
   - Benefits for iterative development

3. **ADR: Slice-Based Architecture** (Phase 3)
   - Moving from simple connectors to slices
   - Benefits for complex event flows
   - How slices drive layout

4. **ADR: Flow-Based Layout Algorithm** (Phase 4)
   - Topological sort for timeline ordering
   - Sub-diagram approach for test scenarios
   - Handling parallel flows

5. **ADR: Schema Versioning Strategy** (Phase 2)
   - Version tied to application version
   - Semantic versioning for schema changes
   - Forward/backward compatibility rules
   - Migration path planning

6. **ADR: Complete Rewrite Decision** (Phase 6)
   - Why incremental change wasn't feasible
   - Lessons learned from initial implementation
   - Future-proofing considerations

7. **ADR: Node-Based Layout Architecture** (Phase 5)
   - Why entities need multiple visual representations
   - Benefits for complex event flows without visual clutter
   - Node vs Entity separation
   - How this solves circular dependency issues

## Acceptance Test Strategy

**Primary Acceptance Test**: The implementation MUST be able to:
1. Parse `example.eventmodel` without errors
2. Produce an SVG that matches the structure of `example.jpg`
3. Include all visual elements shown in the example

**CRITICAL**: Before starting the new implementation:
1. Copy `example.eventmodel` to `tests/fixtures/acceptance/` ✅
2. Copy `example.jpg` to `tests/fixtures/acceptance/` for reference ✅
3. Create acceptance test that will drive the entire implementation ✅
4. This test will fail until the implementation is complete

## Schema Versioning Strategy

**Principle**: The .eventmodel schema version matches the Event Modeler application version.

**Semantic Versioning Rules**:
- **Major version change**: Breaking changes to schema (removing fields, changing types)
- **Minor version change**: Backward-compatible additions (new optional fields, new entity types)
- **Patch version change**: No schema changes, only implementation fixes

**Compatibility** (post-1.0 only):
- For now (pre-1.0), no backward compatibility guarantees
- After 1.0 release:
  - Parser should accept schema versions with same major version
  - Warn on minor version differences (newer features might not render)
  - Error on major version differences (incompatible schema)

**Example**:
```yaml
version: 0.3.0  # Optional, defaults to current app version
workflow: User Account Signup
# ... rest of the file
```

**Version Migration**:
- No migration needed for pre-1.0 versions
- Post-1.0: Will need migration strategy for major version changes
- For now: Clean break to new format

**Implementation Notes**:
- Store version in a constant matching Cargo.toml version
- Parse version first to determine parsing strategy
- Provide clear error messages for version mismatches
- Consider future extensibility for format converters

## Files to be Removed/Updated

### Obsolete Parser Files
- `src/infrastructure/parsing/simple_parser.rs` - Simple text parser
- `src/infrastructure/parsing/simple_lexer.rs` - Simple text lexer
- Related test files in `tests/parsing/`

### Example Files to Update
- All `.eventmodel` files in `examples/` directory
- All generated `.svg` files (will be regenerated)
- Test fixtures using old format

### Documentation to Update
- README.md examples (currently shows old format)
- Any inline code examples in Rust docs
- CLAUDE.md examples

### No Backward Compatibility Needed
- Pre-1.0 release means we can make breaking changes freely
- No need to maintain old parser or provide migration tools
- Clean break to YAML format

## Implementation Roadmap

### Phase 1: Type System Overhaul
**Goal**: Create a type-safe foundation for the rich event modeling format

#### Tasks:
1. Define types for all entity kinds:
   - Event (with data schema)
   - Command (with data schema and test scenarios)
   - View (with UI component hierarchy)
   - Projection (with field definitions)
   - Query (with input/output contracts)
   - Automation
2. Create types for:
   - Data fields with type annotations (e.g., `FieldType<State>`)
   - Test scenarios (Given/When/Then structures)
   - UI components (forms, inputs, buttons, etc.)
   - Slice definitions (connection specifications)
3. Update EntityRegistry to handle all new entity types
4. Ensure all types follow "parse, don't validate" principle

#### Documentation Tasks:
1. Create ADR for YAML format decision
2. Create ADR for gold master testing approach
3. Update all code comments to reflect new type system
4. Document type safety guarantees for each new type

### Phase 2: YAML Parser Implementation
**Goal**: Parse the rich YAML format into our type-safe domain model

#### Tasks:
1. Add serde and serde_yaml dependencies
2. Implement schema versioning:
   - Add `version` field to YAML format (matches app version)
   - Default to current version if not specified
   - Validate version compatibility on parse
   - Plan for future migration paths
3. Create parsing types that map to YAML structure
4. Implement conversion from parsing types to domain types
5. Comprehensive error handling with line/column numbers
6. Support for:
   - Nested data structures
   - Type annotations in strings
   - Component hierarchies
   - Test scenario parsing
   - Slice definition parsing

#### Documentation Tasks:
1. Update README.md with YAML format specification
2. Create comprehensive YAML syntax guide
3. Document schema versioning strategy
4. Document all parsing error types and messages
5. Update CLAUDE.md with YAML-specific guidance

### Phase 3: Domain Model Extensions
**Goal**: Extend the domain model to represent all aspects of the rich format

#### Tasks:
1. Extend Event to include data schema
2. Extend Command to include:
   - Data schema
   - Test scenarios
   - Generated field markers
3. Implement View with component hierarchies
4. Implement Projection with field schemas
5. Implement Query with input/output contracts
6. Implement Automation
7. Implement Slice as first-class concept
8. Update EventModelDiagram to use slices for connections

#### Documentation Tasks:
1. Create ADR for slice-based architecture
2. Document all new entity types with examples
3. Update module documentation for event_model
4. Create entity type reference guide

### Phase 4: Flow-Based Layout Engine
**Goal**: Layout entities based on slice-defined flows, not grid positions

**⚠️ CRITICAL UPDATE**: Phase 4 implementation must be revised to support node-based layout where entities can appear multiple times as distinct visual nodes.

#### Tasks:
1. ~~Implement topological sort for entity positioning~~ ✅ DONE (but needs revision)
2. ~~Use slice definitions to determine flow order~~ ✅ DONE (but needs revision)
3. ~~Layout test scenarios as sub-diagrams below main flow~~ ✅ Foundation added
4. ~~Implement smart connector routing~~ ✅ Basic routing done
5. ~~Handle multiple parallel flows~~ ✅ DONE
6. ~~Ensure readable left-to-right timeline layout~~ ✅ DONE

#### Required Revisions for Node-Based Layout:
1. Create distinct visual nodes for each entity reference in connections
2. Position nodes (not entities) using topological sort
3. Support multiple node instances per entity
4. Route connections between specific node instances
5. Maintain entity identity while having multiple visual representations

#### Documentation Tasks:
1. Create ADR for flow-based layout algorithm
2. **Create ADR for node-based layout architecture** 
3. Document layout constraints and rules
4. Update diagram module documentation
5. Create layout troubleshooting guide

### Phase 5: Rich Visual Rendering
**Goal**: Produce professional diagrams matching the example output

#### Completed Tasks:
1. ✅ Implement entity-type-specific styling:
   - Blue: Commands, Views, Queries
   - Purple: Events
   - Yellow: Projections
   - Green: Automations
   - Red: Error states
2. ✅ Basic entity rendering with names and type labels
3. ✅ Professional typography:
   - Proper text sizing
   - Clear hierarchy
   - Readable spacing
4. ✅ Fixed circular dependency with conservative temporal filtering

#### Remaining Tasks (Blocked on Node-Based Layout):
1. **Implement node-based layout architecture**:
   - Create `DiagramNode` type that references an entity
   - Generate nodes from slice connections (each endpoint = node)
   - Update layout engine to position nodes instead of entities
   - Support multiple visual nodes per logical entity
   - Maintain node-to-entity mapping for styling
2. Render entity content:
   - Names and descriptions ✅ DONE
   - Data schemas (requires node space allocation)
   - UI component hierarchies (requires node space allocation)
3. Render test scenarios:
   - Separate boxes below main flow
   - Given/When/Then sections
   - Connected to parent command node
4. Restore all entity types to layout:
   - Currently filtering out non-temporal connections
   - Node-based layout eliminates cycles, allowing all connections

#### Documentation Tasks:
1. Create visual style guide document
2. Document color scheme and rationale ✅ Partially done
3. Update theme documentation
4. Create accessibility considerations guide
5. Document node-based layout architecture

### Phase 6: Horizontal Slice Architecture (NEW - CRITICAL)
**Goal**: Redesign slice handling to match gold master's horizontal workflow sections

#### Context:
The gold master analysis revealed that slices should be horizontal bands representing complete workflows, not vertical dividers. This is a fundamental change that affects the entire layout system.

**Library Evaluation Results**: After evaluating Rust diagramming libraries (see docs/diagramming-library-evaluation.md), we determined that no existing library meets our specialized needs. Available libraries focus on force-directed or tree layouts, while we need horizontal workflow sections with swimlanes. We'll continue with our custom implementation, potentially using petgraph for graph algorithms.

**Incremental Rendering Approach**: To ensure we're building the right solution, we'll implement visual elements incrementally with manual review checkpoints between each element. This allows for course correction before investing too much in any particular approach.

**Implementation Strategy**: 
- Use `example.eventmodel` as the test data throughout
- Create a temporary test program that renders only the elements for the current step
- Generate PNG output using `magick` for visual comparison
- Wait for explicit user approval before proceeding to the next step
- Each step builds upon the previous, gradually increasing complexity

#### Incremental Implementation Steps:

**🚨 CRITICAL**: Each step must be completed and manually reviewed before proceeding to the next. Generate PNG output and wait for user feedback.

1. **Step 1: Swimlanes Only**
   - Render just the three swimlanes with labels
   - No entities, no slices, no connections
   - **Review Checkpoint**: User confirms swimlane layout and styling

2. **Step 2: Add Slice Boundaries**
   - Add horizontal slice divisions (CreateAccount, VerifyEmailAddress)
   - Show slice headers/labels
   - Still no entities
   - **Review Checkpoint**: User confirms slice structure

3. **Step 3: Add Views (White Boxes)**
   - Add only View entities (LoginScreen, NewAccountScreen, etc.)
   - Position within correct swimlane and slice
   - Use white/light gray styling
   - **Review Checkpoint**: User confirms view positioning and styling

4. **Step 4: Add Commands (Blue Boxes)**
   - Add Command entities
   - Position in commands swimlane within slices
   - Use medium blue styling
   - **Review Checkpoint**: User confirms command layout

5. **Step 5: Add Events (Purple Boxes)**
   - Add Event entities
   - Position in event stream swimlane
   - Use purple/lavender styling
   - **Review Checkpoint**: User confirms event positioning

6. **Step 6: Add Projections (Yellow Boxes)**
   - Add Projection entities
   - Use yellow/orange styling
   - **Review Checkpoint**: User confirms projection layout

7. **Step 7: Add Other Entity Types**
   - Add Queries, Automations
   - Apply appropriate colors
   - **Review Checkpoint**: User confirms all entity types

8. **Step 8: Add Connections**
   - Add arrows between entities following slice flows
   - Route within slice boundaries
   - **Review Checkpoint**: User confirms connection routing

9. **Step 9: Add Test Scenarios**
   - Add test scenario labels within slices
   - Group related entities
   - **Review Checkpoint**: User confirms test scenario presentation

10. **Step 10: Final Polish**
    - Adjust spacing, alignment
    - Fine-tune colors and typography
    - **Final Review**: User confirms complete diagram

#### Original Tasks (to be implemented incrementally):
1. **Redefine Slice concept**:
   - Update Slice type to represent horizontal workflow sections
   - Each slice spans across all swimlanes horizontally
   - Slices have vertical boundaries (top/bottom) not horizontal (left/right)
   - Consider using petgraph for dependency analysis within slices

2. **Update Layout Engine**:
   - Calculate vertical space for each slice based on entity count
   - Position slices as stacked horizontal bands
   - Maintain spacing between slices

3. **Entity Grouping**:
   - Group entities by their slice membership
   - Position entities within their slice's vertical band
   - Use flow-based positioning within each slice

4. **Slice Headers**:
   - Add slice name labels at the start of each slice
   - Position headers in the leftmost area of the slice
   - Style headers prominently

5. **Test Scenario Sub-sections**:
   - Within slices, show test scenarios (e.g., "Main Success", "No Such User")
   - Position these as labeled sub-flows within the slice
   - Use visual grouping to show which entities belong to which scenario

6. **Update Color Scheme**:
   - Views: White/very light gray (#f8f9fa)
   - Commands: Medium blue (#5b8def)
   - Events: Purple/lavender (#c8b6db)
   - Projections: Yellow/orange (#ffd166)
   - Automations: Green (#06d6a0)
   - Errors: Red/pink (#ef476f)

7. **Arrow Routing**:
   - Route arrows within slice boundaries
   - Follow flow paths, not just direct connections
   - Handle cross-slice references appropriately

#### Documentation Tasks:
1. Create ADR for horizontal slice architecture
2. Update all slice-related documentation
3. Document the workflow-based approach
4. Update examples to show horizontal slices

### Phase 7: Acceptance Testing & Documentation
**Goal**: Ensure the implementation meets requirements and documentation is complete

#### Tasks:
1. ✅ Create test that uses example.eventmodel as input
2. ✅ PNG-based visual comparison implemented
3. Compare output structure to example.jpg/png
4. Add tests for all entity types
5. Add tests for error cases
6. Performance testing with large models

#### Documentation Tasks:
1. Update GitHub Pages landing page with YAML examples
2. Create migration guide from old format to YAML
3. Update all example files to use YAML format
4. Create comprehensive user guide
5. Update CONTRIBUTING.md with new development workflow
6. Create ADR summarizing the complete rewrite
7. Update all code examples in documentation
8. Create video tutorial for new format (optional)

### Phase 8: Cleanup
**Goal**: Remove ALL obsolete code and prepare for release

#### Cleanup Tasks:
1. Remove simple text parser completely:
   - `src/infrastructure/parsing/simple_parser.rs`
   - `src/infrastructure/parsing/simple_lexer.rs`
   - All associated test files
2. Remove ALL old format files:
   - All `.eventmodel` files using old format
   - All example SVGs (will regenerate with new format)
3. Clean up obsolete types:
   - Remove all entities only used by old parser
   - Remove unused parsing AST types
   - Remove any compatibility shims
4. Remove obsolete tests:
   - All parser tests for simple format
   - All integration tests using old format
   - All e2e tests using old examples
5. Clean up documentation:
   - Remove ALL references to old format
   - Update ALL code examples to YAML
   - No need to archive old format (pre-1.0)

#### Release Preparation:
1. Clear release notes stating this is a breaking change
2. New examples showcasing YAML format
3. No migration path - clean break (pre-1.0)

## Timeline Estimate

- Phase 1 (Type System): ✅ COMPLETE
- Phase 2 (YAML Parser): ✅ COMPLETE
- Phase 3 (Domain Extensions): ✅ COMPLETE
- Phase 4 (Flow Layout): ✅ COMPLETE
- Phase 5 (Rich Rendering): ✅ COMPLETE (with limitations)
- Phase 6 (Horizontal Slices): 10-12 hours + 2 hours documentation (NEW)
- Phase 7 (Acceptance Testing): 4-6 hours + 4 hours documentation
- Phase 8 (Cleanup): 2-3 hours

Remaining: ~16-21 hours of implementation + ~6 hours of documentation = ~22-27 hours

**Note**: This is a complete rewrite with significantly more complexity than the original MVP. The rich format requires:
- Complex type hierarchies
- YAML parsing with nested structures
- Sophisticated layout algorithms
- Multi-layered rendering (main diagram + test scenarios)
- Professional visual design
- Comprehensive documentation updates
- GitHub Pages updates with new examples

## Future Enhancements (Post-Implementation)

### Extended Features
- Add support for edge case annotations
- Add support for external system integrations  
- Add support for saga/process manager entities
- Add support for read model projections
- Enhanced error message formatting

### Developer Experience
- VSCode extension with syntax highlighting
- Language server for auto-completion
- Live preview mode
- Integration with documentation tools
- Export to other event modeling tools

### Export Formats
- Complete PDF export implementation
- Complete Markdown documentation export
- Add PNG/JPEG export via SVG rasterization
- Add Mermaid diagram export
- Add PlantUML export
- Add draw.io export

### Distribution
- Package for cargo install
- Homebrew formula
- Docker image
- GitHub Action
- Pre-built binaries for all platforms

## Testing Strategy

Following our type-driven testing ADR:

1. **Minimal Runtime Tests**: Only test behaviors that can't be encoded in types:
   - Parsing logic (lexer and parser)
   - SVG output format
   - File I/O operations

2. **No Tests for Type-Guaranteed Behavior**:
   - Don't test that NonEmptyString is non-empty
   - Don't test that validated paths exist
   - Don't test typestate transitions

3. **Property-Based Tests** (if needed):
   - For layout algorithm invariants
   - For parser robustness

## Important: Todo List Management

**CRITICAL**: When using the TodoWrite tool to track implementation progress:

1. **EVERY OTHER TASK** in your todo list must be:
   - "Run the build (cargo build) and tests (cargo test --workspace), and if everything passes (except e2e tests), commit changes and push"
   
2. **AFTER EVERY PUSH**, the next task must be:
   - "Check PR status and CI checks after push"
   
3. The **LAST item** on EVERY todo list must always be:
   - "Review PLANNING.md, update with current status, determine next tasks, and START implementing them"

### Example Todo List Structure:
1. Implement CLI argument parsing in src/cli.rs
2. Run build and tests; commit and push if passing (first push creates upstream branch)
3. Check PR status and CI checks after push
4. Create draft PR immediately after first push
5. Check PR status after creating draft PR
6. Implement main entry point in src/main.rs  
7. Run build and tests; commit and push if passing
8. Check PR status and CI checks after push
9. Add error handling for invalid arguments
10. Run build and tests; commit and push if passing
11. Check PR status and CI checks after push
12. Review PLANNING.md, update with current status, determine next tasks, and START implementing them

This ensures:
1. **Extremely frequent verification** that code compiles and tests pass
2. **Incremental commits** capturing each small working change
3. **Early detection** of any breaking changes
4. **Clean commit history** with each commit representing buildable code
5. **Early PR creation** for visibility and CI feedback
6. The plan stays current with actual progress
7. No steps are missed within or between phases
8. **Continuous forward momentum** - work continues seamlessly without stopping
9. Clear handoff between work sessions
10. **No pause between phases** - as soon as one phase is complete, the next begins immediately

**Note**: Build and test checks should happen AT LEAST this frequently, if not more often. You may add additional build/test/commit steps between tasks whenever it makes sense.

## PR-Driven Development Workflow

### For Each Feature Implementation:

1. **Create Feature Branch**
   - First feature: `git checkout -b feature/yaml-type-system`
   - Subsequent features: `git checkout -b feature/<name> feature/<previous-feature>`
   - This creates a chain: main → yaml-type-system → yaml-parser → domain-extensions → flow-layout → rich-rendering

2. **Implement Feature**
   - Write acceptance test first
   - Fill in `todo!()` placeholders
   - Preserve all type signatures
   - Run quality checks:
     ```bash
     cargo fmt --all
     cargo clippy --workspace --all-targets
     cargo test --workspace
     ```

3. **Incremental Commits and Pushes**
   - **Commit extremely frequently**: Follow the todo list pattern where every other task is "Run build and tests; commit and push if passing"
   - This means committing after EVERY small implementation task that builds successfully
   - Write descriptive commit messages (no prefixes) explaining the "why"
   - Example commit points:
     - After implementing a single function that compiles
     - After adding a new test that passes
     - After fixing a compiler error
     - After any refactoring that maintains working code
     - After adding a type definition
     - After implementing any todo!() placeholder
   - First push: `git push -u origin feature/<name>`
   - Subsequent pushes: `git push`
   - **Remember**: The todo list structure enforces this frequency - every implementation task is followed by a build/test/commit task

4. **Create Draft Pull Request (IMMEDIATELY after first push)**
   - **CRITICAL**: This must be the VERY NEXT task in your todo list after the first push
   - Creating the draft PR early ensures visibility and CI feedback throughout development
   ```bash
   gh pr create \
     --draft \
     --title "Implement <feature description>" \
     --body "$(cat <<'EOF'
   ## Summary
   - <what this PR implements>
   - <key changes made>
   
   ## Test Plan
   - [ ] Acceptance tests pass
   - [ ] cargo clippy shows no warnings
   - [ ] cargo fmt has been run
   
   🤖 Generated with Claude Code
   EOF
   )" \
     --base <previous-feature-branch-or-main>
   ```

5. **Continue Development**
   - Keep making incremental commits as you implement
   - Push regularly to update the draft PR
   - Each push updates the PR with your latest changes

6. **Mark PR Ready and Enable Auto-Merge (when feature complete)**
   ```bash
   # Mark as ready for review
   gh pr ready <PR-number>
   
   # Enable auto-merge
   gh pr merge <PR-number> --auto --squash
   ```

7. **Start Next Feature**
   - Immediately branch off current feature branch
   - Continue implementation while previous PR runs CI

### PR Monitoring Tasks

**🚨 CRITICAL WARNING**: When any PR in a chain merges, you MUST immediately rebase all downstream PRs. Squash merging breaks PR chains and makes downstream PRs unmergeable. See section 4 below for detailed instructions.

**Check periodically throughout implementation:**

1. **Check PR Status**
   ```bash
   gh pr list --author @me
   gh pr checks <PR-number>
   ```

2. **Fix Failed Checks**
   - If a PR has failing checks:
     ```bash
     git checkout feature/<failing-branch>
     # Fix the issues
     git add -A && git commit -m "Fix CI failures"
     git push
     ```

3. **Handle Base Branch Updates**
   - If base branch has new commits:
     ```bash
     git checkout feature/<current>
     git fetch origin
     git rebase origin/feature/<base-branch>
     git push --force-with-lease
     ```

4. **🚨 CRITICAL: Rebase Orphaned Branches After Squash Merges**
   - **IMMEDIATE ACTION REQUIRED**: When a base branch PR merges, downstream PRs become orphaned
   - **Root Cause**: Squash merging creates a new commit on main, breaking the chain for downstream PRs
   - **Symptoms**: PR shows thousands of commits, impossible to merge, shows "commits that are no longer on main"
   
   **For each downstream PR, IMMEDIATELY after base PR merges:**
   ```bash
   # Check if base branch still exists
   gh pr view <PR-number> --json baseRefName
   
   # If base branch is gone (merged), rebase onto main
   git checkout feature/<orphaned-branch>
   git fetch origin
   git rebase origin/main
   
   # CONFLICT RESOLUTION: Always use --theirs to keep downstream changes
   # The conflicts happen because git sees the same changes in different commits
   git checkout --theirs <conflicted-file>
   git add <conflicted-file>
   git rebase --continue
   # Repeat for all conflicts
   
   git push --force-with-lease
   
   # Update PR base branch
   gh pr edit <PR-number> --base main
   ```
   
   **EXAMPLE**: When PR #15 merges, immediately rebase PR #17 and PR #18:
   ```bash
   # Rebase PR #17
   git checkout feature/domain-extensions
   git fetch origin && git rebase origin/main
   # Use --theirs for all conflicts
   git push --force-with-lease
   gh pr edit 17 --base main
   
   # Rebase PR #18  
   git checkout feature/flow-layout
   git rebase origin/main
   # Use --theirs for all conflicts
   git push --force-with-lease
   gh pr edit 18 --base main
   ```

### Branch Chain Example

```
main
 └── feature/yaml-type-system (PR #1)
      └── feature/yaml-parser (PR #2)
           └── feature/domain-extensions (PR #3)
                └── feature/flow-layout (PR #4)
                     └── feature/rich-rendering (PR #5)
```

As PRs merge:
1. PR #1 merges → PR #2 needs rebase to main
2. PR #2 merges → PR #3 needs rebase to main
3. And so on...

## Type System Considerations

Throughout implementation:

1. **Parse, Don't Validate**:
   - All validation at system boundaries
   - Return `Result<ValidType, ParseError>`
   - Never use `.unwrap()` or `.expect()`

2. **Leverage Existing Types**:
   - Use `NonEmptyString` for required strings
   - Use `TypedPath<F,P,E>` for file paths
   - Use validated numeric types for dimensions

3. **Maintain Type Safety**:
   - Don't bypass type constraints
   - If tempted to use `unsafe`, reconsider design
   - Let the compiler guide correct implementation

## Success Criteria

The implementation is complete when:

1. All `todo!()` placeholders are replaced
2. Can convert example.eventmodel to SVG matching example.jpg structure
3. All tests pass (minimal but comprehensive)
4. No clippy warnings
5. All public items documented
6. CI pipeline passes

## PR Monitoring Checklist

**MANDATORY**: PR monitoring must be built into every TodoWrite list:

1. **After every push** (mandatory todo item):
   ```bash
   # Check all open PRs
   gh pr list --author @me
   
   # Check specific PR status
   gh pr checks <PR-number>
   ```

2. **Additional checks every 30 minutes during active development:**
   - Run the same commands above
   - Look for any failed CI runs
   - Check if any base branches have been merged

3. **When switching between features:**
   - Ensure previous PR is green and auto-merge is enabled
   - Check if any base branches have been merged
   - Rebase if necessary

3. **Before starting new work each day:**
   - Review all open PRs
   - Check for any failed CI runs
   - Handle any necessary rebases

## Notes

- Performance is explicitly not a priority (per ADR)
- Focus on correctness through types
- Maintain the philosophy: "Parse, don't validate"
- When in doubt, encode constraints in types rather than tests

## Summary of PR Workflow

1. **Start**: Branch from main for first feature
2. **Chain**: Each subsequent feature branches from previous
3. **First Push**: Create draft PR immediately
4. **Develop**: Make frequent commits and pushes as you build
5. **Complete**: Mark PR ready and enable auto-merge when done
6. **Monitor**: Check PR status regularly
7. **Fix**: Address any CI failures on their branch
8. **Rebase**: When base branches merge, rebase downstream PRs
9. **Continue**: Keep working while PRs process in parallel

This approach allows continuous progress while maintaining clean history and ensuring each feature builds properly on its dependencies.

## Implementation Checklist

When implementing each phase:
- [ ] Create feature branch from correct base
- [ ] Write acceptance tests first
- [ ] Make first commit once tests compile (even if failing)
- [ ] Push branch with first commit
- [ ] **Create draft PR immediately after first push**
- [ ] Implement functionality preserving type signatures
- [ ] Follow the todo list pattern: every implementation task followed by build/test/commit task
- [ ] Commit after each small buildable change:
  - [ ] After implementing each function
  - [ ] After fixing compilation errors
  - [ ] After adding/updating tests
  - [ ] After adding type definitions
  - [ ] After implementing any todo!() placeholder
  - [ ] Push commits regularly to update draft PR
- [ ] Run cargo fmt, clippy, and tests before marking PR ready
- [ ] Mark PR as ready for review when feature complete
- [ ] Enable auto-merge on PR
- [ ] Update PLANNING.md status table
- [ ] Start next phase by branching from current

**Remember**: 
- Every other task in your todo list must be "Run build and tests; commit and push if passing"
- The first push must be immediately followed by creating a draft PR
- The last todo item in every TodoWrite list must be "Review PLANNING.md, update with current status, determine next tasks, and START implementing them"
- Claude should continue working until ALL phases in PLANNING.md are complete - the last todo ensures continuous progress