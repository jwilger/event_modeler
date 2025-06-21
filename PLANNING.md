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

**Last Updated**: 2025-06-21 (Phase 6 Step 2 COMPLETE - All Swimlanes)

**Latest Progress**: 
- Phase 6 (Incremental Diagram Module Rewrite) IN PROGRESS
- Step 0: Delete and Initialize (COMPLETE - PR #28 merged)
- Step 1: Canvas and Workflow Title (COMPLETE - PR #29 merged)
- Step 2: All Swimlanes (COMPLETE - PR #30 merged)
- **CRITICAL**: All code must be in the library, not in test binaries or CLI hacks

**Current Approach**: Building diagram module incrementally:
- Add one element type at a time
- Visual validation at each step
- Dynamic layout that works for ANY valid .eventmodel file

**Completed Phases**:
- Phase 1: Type System Overhaul ✅
- Phase 2: YAML Parser Implementation ✅
- Phase 3: Domain Model Extensions ✅ 
- Phase 4: Flow-Based Layout Engine ✅
- Phase 5: Rich Visual Rendering ✅ (basic implementation)

**Version Planning**: This rewrite will be released as version 0.3.0. Since we're pre-1.0, we can make breaking changes without maintaining backward compatibility. The YAML format will use this version number for its schema version.

### Existing Work to Preserve
- CLI structure and argument parsing (can be reused)
- Type-safety infrastructure (NonEmptyString, TypedPath, etc.)
- Some SVG rendering primitives (will need significant extension)
- Project structure and build configuration

## MCP Workflow Server Development

**Status**: Starting Phase 1

**Goal**: Create an MCP (Model Context Protocol) server that manages our development workflow, providing deterministic guidance and automated GitHub/Git operations. This will eventually replace the manual process rules in this document.

### Overview

The MCP Workflow Server will:
- Provide proactive monitoring of PR status, CI checks, and review feedback
- Automate PR chain management and rebasing
- Give deterministic next-step instructions
- Handle GitHub API interactions directly (no LLM interpretation)
- Run comprehensive checks with every interaction

### Development Approach

1. **Location**: Develop in `mcp-workflow-server/` subdirectory
2. **Structure**: Independent npm package for future extraction
3. **Dogfooding**: Replace manual processes incrementally as each feature is completed
4. **Process**: Follow same PR-driven workflow as main application

### Proactive Monitoring Architecture

Every MCP tool response will include:
```typescript
{
  requestedData: {...},      // What was asked for
  automaticActions: [...],   // What MCP did automatically  
  issuesFound: [...],        // Problems detected
  suggestedActions: [...],   // What LLM should do
  allPRStatus: [...]        // Always include PR overview
}
```

### Implementation Phases

#### Phase 1: Project Setup & Smart Status
**Branch**: `feature/mcp-project-setup`
**Goal**: Create foundation and intelligent status monitoring

Tasks:
- Create `mcp-workflow-server/` directory structure
- Initialize TypeScript project with MCP SDK
- Configure build system and dependencies
- Implement `workflow/status` tool that checks:
  - Current branch and git status
  - Whether current branch is based on latest main
  - All open PRs and their CI status
  - PRs needing rebase after merges
  - Uncommitted changes warning
  - Stale branch detection (branch created before recent merges)
- Add persistent state management
- **Dogfood**: Replace manual `gh pr list` checks with MCP tool

#### Phase 2: Intelligent Next Step
**Branch**: `feature/mcp-next-step`
**Goal**: Context-aware guidance for what to do next

Tasks:
- Implement `workflow/next-step` tool that:
  - Runs full status check first
  - Detects current context (PR created? Reviews pending? CI failing?)
  - Prioritizes urgent issues (failing CI, needed rebases)
  - Provides specific commands to run
- Add workflow state tracking
- Create configuration for workflow rules
- **Dogfood**: Replace "check PLANNING.md for next task" with MCP tool

#### Phase 3: Smart PR Creation
**Branch**: `feature/mcp-pr-creation`
**Goal**: Automated PR creation with monitoring

Tasks:
- Implement `workflow/create-pr` tool that:
  - Verifies PR doesn't already exist
  - Checks base branch is up-to-date
  - Creates PR with proper template
  - Immediately starts review monitoring
  - Returns PR URL and initial status
- Add PR template configuration
- **Dogfood**: Replace manual `gh pr create` commands with MCP tool

#### Phase 4: Active Review Monitoring
**Branch**: `feature/mcp-review-monitoring`
**Goal**: Proactive review and CI monitoring

Tasks:
- Implement `workflow/monitor-reviews` tool that:
  - Checks ALL open PRs (not just current)
  - Detects new reviews/comments from any reviewer
  - Formats feedback for LLM action
  - Tracks which comments need responses
  - Monitors CI status changes
- Add review comment threading support
- **Dogfood**: Replace manual review checking with MCP tool

#### Phase 5: Automated Merge & Rebase
**Branch**: `feature/mcp-merge-handling`
**Goal**: Automatic PR chain management

Tasks:
- Implement `workflow/handle-merge` tool that:
  - Detects when base PRs merge
  - Automatically rebases downstream PRs
  - Handles conflicts with clear instructions
  - Updates PR base branches via API
  - Reports all actions taken
- Add PR chain dependency tracking
- **Dogfood**: Replace manual rebase instructions with MCP tool

### Success Criteria

1. Each phase successfully replaces its manual counterpart
2. Workflow becomes more efficient with each phase
3. No workflow disruption during migration
4. 90% of process rules moved from PLANNING.md to MCP
5. Can be extracted to separate repository with minimal changes
6. All workflow decisions are deterministic and consistent

### Testing Strategy

- Mock GitHub API for unit tests
- Integration tests with real GitHub API (test repo)
- Dogfood each feature on actual Event Modeler PRs
- Document any gaps discovered during real usage

### Future Enhancements

- VS Code extension integration
- Slack/Discord notifications
- Multi-repo support
- Custom workflow definitions
- Analytics and metrics tracking

## Development Process Overview

**CRITICAL**: The VERY FIRST step when starting any work session is to create a todo list using the TodoWrite tool. Even if the only item is "Review PLANNING.md to determine next tasks", you MUST create this todo list before doing anything else. This ensures work is always tracked and organized.

The implementation will follow a PR-driven workflow with feature branch chaining:
1. Create a feature branch for each phase
2. Implement the feature with acceptance tests
3. Create a PR with auto-merge enabled
4. Branch the next feature off the previous feature branch
5. Monitor PR status and fix any CI failures
6. Handle rebasing when base branches are merged

## 🔴 CRITICAL Development Rules 🔴

These rules apply to ALL development work, not just specific phases:

### Visual Development Rule
**IMPORTANT**: When working on visual output (diagrams, UI, etc.):
1. Do NOT commit and push until you are satisfied with your comparison between the generated output and the expected result
2. Test locally first:
   - Generate the output (e.g., SVG, PNG)
   - Compare with gold standard examples
   - Refine implementation to match expected output
3. Only after you're satisfied with the visual output, THEN commit code changes and push
4. This ensures each visual element matches expectations before building on it

### Branch Management Rule
**CRITICAL**: Always update main before creating new branches:
```bash
git checkout main
git pull origin main
git checkout -b feature/your-branch-name
```

**Common Mistake Prevention**:
- NEVER reuse old branches that were created before recent merges
- If you accidentally work on a stale branch:
  1. Stash your changes: `git stash`
  2. Switch to main and update: `git checkout main && git pull origin main`
  3. Create new branch: `git checkout -b feature/new-branch-name`
  4. Apply stashed changes: `git stash pop`
- The MCP server will detect and warn about stale branches automatically

### Commit Practice Rule
**IMPORTANT**: When addressing multiple review comments or making different types of changes:
1. Commit each logical change separately for easier review
2. Each commit should address one concern or piece of feedback
3. Use clear commit messages that explain what specific issue is being addressed
4. This makes the review process clearer and allows for better discussion of individual changes

### PR Process Rule
1. Create PRs as ready (not draft) to trigger automated reviews
2. Monitor for ALL reviews (Copilot, user, or any other reviewer) using a sleep/check cycle:
   - Check PR for new reviews every 30 seconds
   - Check for ANY review comments from ANY reviewer
   - Address ALL feedback from ALL reviewers before merge
   - Continue monitoring until PR is merged
3. Before PR can be merged:
   - ALL review comments must be addressed (even if marked as "low confidence")
   - Reply directly to each review comment thread (not as top-level comments) with either:
     - How you fixed the issue (with commit reference if applicable)
     - Why you're not addressing it (with clear reasoning)
   - **IMPORTANT**: All replies must begin with `[Response from Claude Code]` to clarify attribution
   - Use GraphQL API to reply within review threads:
     ```bash
     gh api graphql -f query='
     mutation {
       addPullRequestReviewThreadReply(input: {
         pullRequestReviewThreadId: "THREAD_ID"
         body: "Your reply here"
       }) {
         comment { id }
       }
     }'
     ```
   - NEVER resolve review threads - the user will read and resolve all review comments manually
   - Always leave threads unresolved after replying, regardless of whether you made changes or provided reasoning
4. PR merge = approval to proceed to next task
5. Once PR is merged, immediately proceed to next step

### Todo List Management Rule
**CRITICAL**: The VERY LAST task on your todo list must ALWAYS be:
"Review PLANNING.md, update with current status, and await approval before proceeding"

This ensures continuous alignment with project planning and prevents drift from intended workflow.

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


### Phase 6: Incremental Diagram Module Rewrite
**Goal**: Build a proper event model diagramming library from scratch using incremental development

**Approach**: Build incrementally, adding one element type at a time with visual validation before proceeding.

**CRITICAL IMPLEMENTATION REQUIREMENTS**:

1. **ALL CODE IN THE LIBRARY**: 
   - The diagram module is part of the core library at `src/diagram/`
   - NO test binaries with implementation logic
   - NO hacks in the CLI - it should only call library functions
   - The CLI's execute_render function should convert YAML → Domain → Diagram → SVG

2. **PROPER CONVERSION FLOW**:
   ```rust
   // In the CLI or a proper conversion module:
   let yaml_model = parse_yaml(&input)?;
   let domain_model = convert_yaml_to_domain(yaml_model)?;
   let diagram = build_diagram_from_domain(domain_model)?;  // THIS IS WHAT WE'RE BUILDING
   let svg = diagram.to_svg();
   ```

3. **TESTING APPROACH**:
   - Run the actual event_modeler binary: `cargo run -- example.eventmodel -o output.svg`
   - Look at the generated SVG to verify correctness
   - Locally convert to PNG if needed: `magick output.svg output.png`
   - Compare with example.png gold standard

4. **PR WORKFLOW**:
   - Generate SVG using the actual binary
   - CI creates a new gist with the SVG
   - PR description is updated with the gist link
   - Review the actual rendered diagram in the PR

**Dynamic Requirements**: The diagram module MUST be dynamic enough to handle ANY valid .eventmodel file:
- Variable number of swimlanes (defined by the input .eventmodel file, not hardcoded)
- Swimlane names/titles come from the .eventmodel file (e.g., "UX, Automations", "Commands, Projections, Queries")
- Variable number of entities of each type
- Different slice configurations
- Different test scenario counts per command
- Dynamic layout that adapts to content
- Multiple instances of ANY entity type based on slice connections

**Visual Style Requirements**: When comparing with example.png:
- Match the STYLE and RELATIVE POSITIONING, not exact pixel dimensions
- All sizes should be DYNAMIC to best fit their contents
- Elements should have reasonable minimum dimensions when empty
- Maintain similar padding/spacing between elements and containers
- The goal is visual consistency and readability, not pixel-perfect matching

#### Implementation Steps:

**⚠️ Phase 6 Visual Development Process**
Phase 6 follows the general Visual Development Rule (see above) with these specific details:
1. Create a new feature branch for each step (e.g., `feature/diagram-step-3-slice-headers`)
2. Implement one diagram element at a time
3. Test locally:
   - Generate SVG: `cargo run -- tests/fixtures/acceptance/example.eventmodel -o test.svg`
   - Convert to PNG: `magick test.svg test.png`
   - Compare with `tests/fixtures/acceptance/example.png`
   - Refine implementation to match style as closely as possible
   - Do NOT commit generated SVG/PNG files
4. Follow the general Visual Development Rule for committing
5. Create a PR for just that element (following general PR Process Rule)
6. Once merged, immediately proceed to next step

**Branch Naming for Phase 6**:
- Branch naming: `feature/diagram-step-{number}-{element-name}`
- Example: `feature/diagram-step-3-slice-headers`
- Example: `feature/diagram-step-4-login-screen-view`
- PR title format: "Add diagram element: {element description}"

**Current Progress**:
- ✅ Step 0: Delete and Initialize (COMPLETE - PR #28 merged)
- ✅ Step 1: Canvas and Workflow Title (COMPLETE - PR #29 merged)
- ✅ Step 2: All Swimlanes (COMPLETE - PR #30 merged)
- 🔄 Step 3: Slice Headers (IN PROGRESS - PR #31 ready for merge, all feedback addressed)
- ⏸️ Step 4: View Entities (not started)

**Foundation Steps (Canvas & Structure)**:

1. **Step 0: Delete and Initialize**
   - Delete entire `src/diagram/` module
   - Create new empty module structure with proper library design
   - Set up conversion function that takes domain model and returns diagram
   - Ensure CLI can use this to generate SVGs from .eventmodel files
   - NO TEST BINARIES - test by running: `cargo run -- example.eventmodel -o test.svg`

2. **Step 1: Canvas and Workflow Title**
   - Create EventModelDiagram type in diagram module
   - Add conversion function that builds diagram from domain model
   - Render canvas with workflow title
   - Dynamic: Canvas width adjusts to content, title left-aligned

3. **Step 2: All Swimlanes**
   - Add swimlanes to diagram from domain model
   - Dynamic: Height adjusts to number of swimlanes
   - Rotated labels on left side with text wrapping
   - Dynamic width calculation for label section

4. **Step 3: Slice Headers**
   - Add slice dividers and headers
   - For testing: "Create Account", "Send Email Verification", "Verify Email Address"
   - Dynamic: Width adjusts to content in each slice
   - Headers at top of canvas above swimlanes

5. **Step 4: View Entities**
   - Implement rendering for View entity type
   - White box with "View" label and entity name
   - Position views in their assigned swimlanes and slices
   - Handle multiple views in same location

6. **Step 5: Command Entities**
   - Implement rendering for Command entity type
   - Blue box with proper styling
   - Position commands in their assigned swimlanes

7. **Step 6: Event Entities**
   - Implement rendering for Event entity type
   - Purple box with event styling
   - Position events in their assigned swimlanes

8. **Step 7: Projection Entities**
   - Implement rendering for Projection entity type
   - Yellow box styling
   - Position projections in their assigned swimlanes

9. **Step 8: Query Entities**
   - Implement rendering for Query entity type
   - Blue box styling (similar to commands)
   - Position queries in their assigned swimlanes

10. **Step 9: Automation Entities**
    - Implement rendering for Automation entity type
    - Special circular icon with automation symbol
    - Position between swimlanes as specified

11. **Step 10: Entity Connections**
    - Draw arrows between entities as defined in slice connections
    - Handle connections across swimlanes and slices
    - Proper arrow styling and routing

12. **Step 11: Layout Algorithm**
    - Implement proper entity positioning within slices
    - Handle multiple entities in same swimlane/slice
    - Ensure proper spacing and alignment
    - Support dynamic canvas sizing based on content



**Test Scenarios Section**:

24. **Step 23: Test Scenario Layout Area**
    - Add horizontal divider below main diagram
    - Create test scenario section layout
    - Three columns for the three commands

25. **Step 24: Command Test Scenario Boxes**
    - "Create User Account Credentials" scenarios
    - "Send Email Verification" scenarios  
    - "Verify Email Address" scenarios
    - Headers for each command

26. **Step 25: Given/When/Then Content**
    - Add all test entries with proper coloring
    - Error scenarios in red
    - Success scenarios match entity colors
    - Proper columnar layout

**Final Polish**:

27. **Step 26: Final Adjustments**
    - Fine-tune all positions
    - Verify arrow routing
    - Adjust text sizes
    - Ensure everything matches gold standard

**Complete Element Coverage**:
This plan covers every element visible in the gold standard:
- 1 workflow title
- 3 swimlanes  
- 3 slice headers
- 4 view instances (including duplicates - views can appear multiple times)
- 3 command instances (commands can appear multiple times)
- 3 event instances (events can appear multiple times)
- 4 projection instances (2 unique projections, each appearing twice)
- 2 query instances (queries can appear multiple times)
- 1 automation instance (automations can appear multiple times)
- All connections from slice definitions
- 6 test scenarios across 3 commands
- All styling (colors, fonts, borders, arrows)

Note: ANY entity type (views, commands, events, projections, queries, automations) can have multiple instances in the diagram. The layout engine must support showing the same logical entity in multiple visual positions.

#### Technical Approach:

- **Builder Pattern**: Use immutable builder pattern for diagram construction
- **Domain-Specific Types**: EventModelDiagram, Swimlane, View, Command, Event, etc.
- **Type-Safe Layout**: Compile-time guarantees for valid layouts
- **Functional Style**: Return new diagram instances, never mutate
- **Event Model Focus**: API specifically for event modeling, not generic SVG
- **Dynamic Layout Engine**: Must handle arbitrary numbers of elements
- **Slice-Based Positioning**: Use slice definitions to determine entity positions

Example API (conceptual):
```rust
// From parsed YAML model
let model = parse_yaml(content)?;
let domain_model = convert_yaml_to_domain(model)?;

// Build diagram dynamically from model
let mut diagram = EventModelDiagram::new(&domain_model.workflow);

// Add all swimlanes from model
for (id, label) in &domain_model.swimlanes {
    diagram = diagram.add_swimlane(Swimlane::new(id, label));
}

// Add all entities from model
for (name, view) in &domain_model.views {
    diagram = diagram.add_view(View::from_model(name, view));
}
// ... same for commands, events, etc.

// Process slices to determine layout
diagram = diagram.layout_from_slices(&domain_model.slices);
```

#### Success Criteria:
- Each incremental step produces output closer to gold standard
- Final diagram matches example.png exactly when rendering example.eventmodel:
  - Layout structure
  - Entity positioning
  - Colors and styling
  - Test scenario presentation
- **Dynamic Requirements Met**:
  - Can handle event models with 1-100+ swimlanes
  - Can handle models with any number of entities
  - Layout adapts to different slice configurations
  - Test scenarios scale with command count
  - No hardcoded positions or counts
- API is clean and domain-specific
- Could be extracted as standalone event modeling diagram crate

#### Documentation Tasks:
1. Document new diagram module API
2. Create examples showing incremental building
3. Document visual style specifications
4. Create guide for extending with new entity types

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

- Phase 1-5: ✅ COMPLETE
- Phase 6 (Incremental Diagram Rewrite): IN PROGRESS
  - Steps 0-2: ✅ COMPLETE
  - Steps 3-26: ~10 hours remaining
- Phase 7 (Acceptance Testing): 4-6 hours + 4 hours documentation
- Phase 8 (Cleanup): 2-3 hours

Remaining: ~16-21 hours of implementation + ~6 hours of documentation

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
   - **EXCEPTION for Phase 6**: Replace with "Review PLANNING.md, update with current status, and await approval before proceeding"

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

**For Phase 6 Incremental Steps**:
12. Review PLANNING.md, update with current status, and await approval before proceeding to next step

**EXCEPTION for Phase 6 Visual Development**: 
- The normal "commit after every small task" rule is SUSPENDED for Phase 6
- Instead, only commit AFTER you are satisfied with the visual comparison to example.png
- This ensures commits only contain visually correct implementations

This ensures:
1. **Visual correctness** before committing any changes
2. **Clean commits** that represent visually verified implementations
3. **Early detection** of any breaking changes
4. **Clean commit history** with each commit representing buildable code
5. **Early PR creation** for visibility and CI feedback
6. The plan stays current with actual progress
7. No steps are missed within or between phases
8. **Continuous forward momentum** - work continues seamlessly without stopping
9. Clear handoff between work sessions
10. **No pause between phases** - as soon as one phase is complete, the next begins immediately
    - **EXCEPTION**: Phase 6 requires approval between each incremental step

**Note**: Build and test checks should happen AT LEAST this frequently, if not more often. You may add additional build/test/commit steps between tasks whenever it makes sense.

## PR-Driven Development Workflow

### For Each Feature Implementation:

1. **Create Feature Branch**
   - **ALWAYS start by updating main**:
     ```bash
     git checkout main
     git pull origin main
     ```
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

4. **Create Pull Request (IMMEDIATELY after first push - NOT as draft)**
   - **CRITICAL**: This must be the VERY NEXT task in your todo list after the first push
   - Creating the PR as "ready" (not draft) triggers automatic Copilot review
   ```bash
   gh pr create \
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

5. **Monitor for Copilot Review**
   - After creating PR, immediately begin monitoring for Copilot review
   - Check every 30 seconds for up to 5 minutes:
   ```bash
   gh api repos/jwilger/event_modeler/pulls/<PR-number>/reviews \
     --jq '.[] | select(.user.login | contains("copilot")) | {state: .state, body: .body}'
   ```
   - Once Copilot review appears, read and analyze the feedback
   - **IMPORTANT**: Also check for specific code suggestions:
   ```bash
   gh api repos/jwilger/event_modeler/pulls/<PR-number>/reviews/<REVIEW-ID>/comments \
     --jq '.[] | {path: .path, line: .line, body: .body}'
   ```

6. **Address Copilot Feedback**
   - Attempt to address ALL code suggestions and issues raised by Copilot
   - **IMPORTANT**: User instructions supersede Copilot suggestions
   - If Copilot feedback contradicts user instructions, follow user instructions
   - Implement reasonable improvements (constants, performance, clarity)
   - Make necessary changes and commit/push them
   - Reply to Copilot review explaining any feedback not addressed:
   ```bash
   gh pr review <PR-number> --comment --body "Thank you for the review. [Explanation of what was addressed and why certain suggestions were not followed]"
   ```
   - Ensure ALL review comments are either implemented or explicitly dismissed with reasoning

7. **Wait for PR Merge**
   - After addressing feedback, wait for user to merge the PR
   - Use PR merge as indication of acceptance
   - Do NOT proceed to next task until PR is merged
   - Check PR status periodically:
   ```bash
   gh pr view <PR-number> --json state,merged
   ```

8. **Continue with Next Task**
   - Once PR is merged, immediately continue with next task from PLANNING.md
   - Update main branch and create new feature branch
   - The merged PR serves as user approval to proceed

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