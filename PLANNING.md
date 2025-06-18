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

**Last Updated**: 2025-06-18

**Critical Discovery**: The existing implementation was based on incorrect requirements. The actual requirements call for a rich YAML-based event modeling language with:
- Multiple entity types (events, commands, views, projections, queries, automations)
- Data schemas with type annotations
- Test scenarios (Given/When/Then)
- UI component hierarchies  
- Slice-based flow definitions
- Professional visual output with color coding and sub-diagrams

The example.eventmodel and example.jpg files represent the TRUE requirements.

**Next Step**: Begin Phase 1 of the implementation roadmap - Type System Overhaul

**Version Planning**: This rewrite will be released as version 0.3.0 (or 1.0.0 if we decide it's stable enough for a 1.0 release). The YAML format will use this version number for its schema version.

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

1. **ADR: Adopting YAML Format** (Phase 1)
   - Why we're moving from simple text to YAML
   - Benefits of structured format
   - Schema versioning strategy
   - Trade-offs considered

2. **ADR: Gold Master Testing Strategy** (Phase 1)
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

**Compatibility**:
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
- Future versions may include migration tools
- Clear documentation of changes between versions
- Consider providing a separate migration command
- Preserve original files, create migrated copies

**Implementation Notes**:
- Store version in a constant matching Cargo.toml version
- Parse version first to determine parsing strategy
- Provide clear error messages for version mismatches
- Consider future extensibility for format converters

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

#### Tasks:
1. Implement topological sort for entity positioning
2. Use slice definitions to determine flow order
3. Layout test scenarios as sub-diagrams below main flow
4. Implement smart connector routing
5. Handle multiple parallel flows
6. Ensure readable left-to-right timeline layout

#### Documentation Tasks:
1. Create ADR for flow-based layout algorithm
2. Document layout constraints and rules
3. Update diagram module documentation
4. Create layout troubleshooting guide

### Phase 5: Rich Visual Rendering
**Goal**: Produce professional diagrams matching the example output

#### Tasks:
1. Implement entity-type-specific styling:
   - Blue: Commands, Views, Queries
   - Purple: Events
   - Yellow: Projections
   - Green: Automations
   - Red: Error states
2. Render entity content:
   - Names and descriptions
   - Data schemas
   - UI component hierarchies
3. Render test scenarios:
   - Separate boxes below main flow
   - Given/When/Then sections
   - Connected to parent command
4. Professional typography:
   - Proper text sizing
   - Clear hierarchy
   - Readable spacing

#### Documentation Tasks:
1. Create visual style guide document
2. Document color scheme and rationale
3. Update theme documentation
4. Create accessibility considerations guide

### Phase 6: Acceptance Testing & Documentation
**Goal**: Ensure the implementation meets requirements and documentation is complete

#### Tasks:
1. Create test that uses example.eventmodel as input
2. Compare output structure to example.jpg
3. Add tests for all entity types
4. Add tests for error cases
5. Performance testing with large models

#### Documentation Tasks:
1. Update GitHub Pages landing page with YAML examples
2. Create migration guide from old format to YAML
3. Update all example files to use YAML format
4. Create comprehensive user guide
5. Update CONTRIBUTING.md with new development workflow
6. Create ADR summarizing the complete rewrite
7. Update all code examples in documentation
8. Create video tutorial for new format (optional)

## Timeline Estimate

- Phase 1 (Type System): 6-8 hours + 2 hours documentation
- Phase 2 (YAML Parser): 8-10 hours + 3 hours documentation
- Phase 3 (Domain Extensions): 6-8 hours + 2 hours documentation
- Phase 4 (Flow Layout): 8-10 hours + 2 hours documentation
- Phase 5 (Rich Rendering): 10-12 hours + 2 hours documentation
- Phase 6 (Acceptance Testing): 4-6 hours + 4 hours documentation

Total: ~42-54 hours of implementation + ~15 hours of documentation = ~57-69 hours

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
   
2. The **LAST item** on EVERY todo list must always be:
   - "Review PLANNING.md, update with current status, determine next tasks, and START implementing them"

### Example Todo List Structure:
1. Implement CLI argument parsing in src/cli.rs
2. Run build and tests; commit and push if passing (first push creates upstream branch)
3. Create draft PR immediately after first push
4. Implement main entry point in src/main.rs  
5. Run build and tests; commit and push if passing
6. Add error handling for invalid arguments
7. Run build and tests; commit and push if passing
8. Review PLANNING.md, update with current status, determine next tasks, and START implementing them

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

4. **Rebase Orphaned Branches**
   - When a base branch PR merges, downstream PRs need rebasing:
     ```bash
     # Check if base branch still exists
     gh pr view <PR-number> --json baseRefName
     
     # If base branch is gone (merged), rebase onto main
     git checkout feature/<orphaned-branch>
     git fetch origin
     git rebase origin/main
     git push --force-with-lease
     
     # Update PR base branch
     gh pr edit <PR-number> --base main
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

Throughout the implementation, regularly check:

1. **Every 30 minutes during active development:**
   ```bash
   # Check all open PRs
   gh pr list --author @me
   
   # Check specific PR status
   gh pr checks <PR-number>
   ```

2. **When switching between features:**
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