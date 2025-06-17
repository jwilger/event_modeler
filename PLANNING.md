# Event Modeler Implementation Plan

This document outlines the detailed plan for implementing the remaining functionality in Event Modeler while adhering to our type-driven development principles and architectural decisions.

## Current Status

**Last Updated**: [To be updated as work progresses]

| Phase | Branch | Status | PR # | Notes |
|-------|--------|--------|------|-------|
| CLI Foundation | `feature/cli-foundation` | Not Started | - | - |
| Text Parsing | `feature/text-parsing` | Not Started | - | - |
| Layout Engine | `feature/layout-engine` | Not Started | - | - |
| SVG Rendering | `feature/svg-rendering` | Not Started | - | - |
| Integration | `feature/integration-polish` | Not Started | - | - |

**Current Focus**: Ready to begin Phase 1 (CLI Foundation)

## Overview

The implementation will follow a PR-driven workflow with feature branch chaining:
1. Create a feature branch for each phase
2. Implement the feature with acceptance tests
3. Create a PR with auto-merge enabled
4. Branch the next feature off the previous feature branch
5. Monitor PR status and fix any CI failures
6. Handle rebasing when base branches are merged

## Implementation Phases

### Branch Structure Plan

The implementation will create these feature branches in sequence:
1. `feature/cli-foundation` - CLI argument parsing and basic structure
2. `feature/text-parsing` - Lexer and parser implementation  
3. `feature/layout-engine` - Layout computation algorithm
4. `feature/svg-rendering` - SVG generation from layout
5. `feature/integration-polish` - Final integration and polish

Each branch builds on the previous one, creating a chain of PRs.

### Phase 1: CLI Foundation & Basic E2E Test

**Branch**: `feature/cli-foundation`  
**Base**: `main`

**Goal**: Create a minimal working CLI that can process a simple .eventmodel file and produce SVG output.

#### 1.1 Create End-to-End Acceptance Test
```bash
tests/e2e/basic_conversion.rs
```
- Create a simple test .eventmodel file
- Run the CLI with this file
- Verify SVG output is created
- This test will initially fail but guide our implementation

#### 1.2 Implement CLI Argument Parsing
- Implement `src/cli.rs` functions:
  - `Cli::from_args()` - Parse command line arguments
  - `Cli::execute()` - Orchestrate the conversion process
- Preserve existing type signatures using `TypedPath<F,P,E>`
- No runtime validation - all validation through type parsing

#### 1.3 Implement Main Entry Point
- Replace `todo!()` in `src/main.rs`
- Wire up CLI to parse args and execute
- Handle errors gracefully with proper exit codes

**Acceptance Criteria**: 
- CLI can be invoked with `cargo run -- input.eventmodel -o output.svg`
- Proper error messages for invalid arguments
- Help text displays correctly
- PLANNING.md updated with completion status

### Phase 2: Text Parsing Implementation

**Branch**: `feature/text-parsing`  
**Base**: `feature/cli-foundation`

**Goal**: Parse .eventmodel files into our strongly-typed domain model.

#### 2.1 Create Parser Test Suite
```bash
tests/parsing/
├── lexer_tests.rs      # Token generation tests
├── parser_tests.rs     # AST construction tests
└── fixtures/           # Test .eventmodel files
```

#### 2.2 Implement Lexer
- Implement in `src/infrastructure/parsing/lexer.rs`:
  - `Lexer::new()` - Initialize lexer with input
  - `Lexer::next_token()` - Generate tokens from input
- Use the existing `Token` enum without modification
- Ensure position tracking for error messages

#### 2.3 Implement Parser with Typestate Pattern
- Implement in `src/infrastructure/parsing/mod.rs`:
  - `EventModelParser::parse_header()` - Parse title section
  - `EventModelParser::parse_body()` - Parse swimlanes
  - `EventModelParser::build()` - Construct final AST
- Follow the typestate pattern: `Empty -> HasHeader -> HasBody -> Complete`
- Use existing domain types from `src/event_model/entities.rs`

**Acceptance Criteria**:
- Can parse valid .eventmodel files without panics
- Invalid files produce clear error messages with line numbers
- Parser enforces correct section ordering at compile time
- PLANNING.md updated with completion status

### Phase 3: Layout Engine Implementation

**Branch**: `feature/layout-engine`  
**Base**: `feature/text-parsing`

**Goal**: Compute positions for all diagram elements.

#### 3.1 Create Layout Test Suite
```bash
tests/layout/
├── positioning_tests.rs  # Element positioning
├── sizing_tests.rs      # Size calculations
└── constraints_tests.rs # Layout constraints
```

#### 3.2 Implement Layout Algorithm
- Implement in `src/diagram/layout.rs`:
  - `LayoutEngine::compute_layout()` - Main layout computation
  - Calculate swimlane positions
  - Position entities within swimlanes
  - Route connectors between entities
- Use existing `LayoutConfig` and dimension types
- Ensure all positions use validated numeric types

**Acceptance Criteria**:
- Elements positioned without overlaps
- Connectors route cleanly between entities
- Layout respects configured spacing and margins
- PLANNING.md updated with completion status

### Phase 4: SVG Rendering Implementation

**Branch**: `feature/svg-rendering`  
**Base**: `feature/layout-engine`

**Goal**: Generate valid SVG output from layout data.

#### 4.1 Create Rendering Test Suite
```bash
tests/rendering/
├── svg_output_tests.rs   # SVG structure tests
├── theme_tests.rs        # Theme application tests
└── expected/             # Expected SVG outputs
```

#### 4.2 Implement SVG Generation
- Implement in `src/diagram/svg.rs`:
  - `SvgRenderer::render()` - Generate complete SVG
  - Render swimlanes with labels
  - Render entities with appropriate shapes
  - Render connectors with arrows
- Use the strongly-typed SVG element builders
- Apply theme styles from `src/diagram/theme.rs`

**Acceptance Criteria**:
- Generated SVG is valid XML
- All elements properly styled according to theme
- SVG renders correctly in browsers
- PLANNING.md updated with completion status

### Phase 5: Integration & Polish

**Branch**: `feature/integration-polish`  
**Base**: `feature/svg-rendering`

**Goal**: Ensure all components work together seamlessly.

#### 5.1 Integration Testing
- Create comprehensive integration tests
- Test various .eventmodel file formats
- Verify error handling across the pipeline
- Test both light and dark themes

#### 5.2 Documentation Generation
- Ensure all public items have rustdoc comments
- Run `cargo doc` to verify documentation builds
- Update README with usage examples

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

**CRITICAL**: When using the TodoWrite tool to track implementation progress, the LAST item on EVERY todo list must always be:
- "Review PLANNING.md, update with current status, and determine next tasks"

This ensures:
1. The plan stays current with actual progress
2. No steps are missed within or between phases
3. Continuous forward momentum
4. Clear handoff between work sessions
5. Flexibility to continue current phase work or move to next phase as appropriate

## PR-Driven Development Workflow

### For Each Feature Implementation:

1. **Create Feature Branch**
   - First feature: `git checkout -b feature/cli-foundation`
   - Subsequent features: `git checkout -b feature/<name> feature/<previous-feature>`
   - This creates a chain: main → cli-foundation → parsing → layout → svg-rendering

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

3. **Commit and Push**
   - Write descriptive commit messages (no prefixes)
   - Push branch: `git push -u origin feature/<name>`

4. **Create Pull Request**
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

5. **Enable Auto-Merge**
   ```bash
   gh pr merge <PR-number> --auto --squash
   ```

6. **Start Next Feature**
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
 └── feature/cli-foundation (PR #1)
      └── feature/text-parsing (PR #2)
           └── feature/layout-engine (PR #3)
                └── feature/svg-rendering (PR #4)
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
2. Can convert .eventmodel files to SVG/PDF
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

## Timeline Estimate

- Phase 1 (CLI): 2-3 hours
- Phase 2 (Parsing): 4-6 hours  
- Phase 3 (Layout): 3-4 hours
- Phase 4 (SVG): 3-4 hours
- Phase 5 (Integration): 2 hours

Total: ~14-19 hours of implementation

**Note**: Additional time may be needed for:
- Fixing CI failures
- Rebasing branches as PRs merge
- Addressing any review feedback (though with auto-merge, this should be minimal)

## Notes

- Performance is explicitly not a priority (per ADR)
- Focus on correctness through types
- Maintain the philosophy: "Parse, don't validate"
- When in doubt, encode constraints in types rather than tests

## Summary of PR Workflow

1. **Start**: Branch from main for first feature
2. **Chain**: Each subsequent feature branches from previous
3. **Push**: Create PR with auto-merge enabled
4. **Monitor**: Check PR status regularly
5. **Fix**: Address any CI failures on their branch
6. **Rebase**: When base branches merge, rebase downstream PRs
7. **Continue**: Keep working while PRs process in parallel

This approach allows continuous progress while maintaining clean history and ensuring each feature builds properly on its dependencies.

## Implementation Checklist

When implementing each phase:
- [ ] Create feature branch from correct base
- [ ] Write acceptance tests first
- [ ] Implement functionality preserving type signatures
- [ ] Run cargo fmt, clippy, and tests
- [ ] Commit with descriptive message
- [ ] Push branch and create PR
- [ ] Enable auto-merge on PR
- [ ] Update PLANNING.md status table
- [ ] Start next phase by branching from current

**Remember**: The last todo item in every TodoWrite list must be "Review PLANNING.md, update with current status, and determine next tasks"