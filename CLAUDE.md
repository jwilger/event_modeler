# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Event Modeler is a CLI application that converts YAML-based event model descriptions (`.eventmodel` files) into visual diagrams (SVG/PDF format).

**Current Status**: Phase 6 (Diagram Module Rewrite) in progress - See GitHub issues with `epic` label for current work.

For general architecture and contribution guidelines, see [README.md](README.md).

## Development Setup

**Rust Version**: Stable (latest)
**Dependency Management**: Nix flake for local development environment
**Environment**: direnv with .envrc for automatic environment loading

### Common Commands

- `nix develop` - Enter development shell with all dependencies
- `cargo test --workspace` - Run all tests
- `cargo run` - Run Event Modeler CLI
- `cargo fmt --all` - Format code
- `cargo clippy --workspace --all-targets` - Run linter
- `pre-commit install` - Install git hooks (when in nix shell)

### Documentation Commands

- `cargo doc --open` - Generate and view API documentation
- View architecture decisions in README.md and module documentation

### Development Workflow

- Check GitHub issues with `epic` label for current work phases
- Find next task in sub-issues of active epics
- Create feature branches for each issue
- Follow acceptance criteria in issue descriptions

## Technical Architecture

### Core Technologies

- **Language**: Rust (latest stable)
- **Type Safety**: nutype for domain-specific types
- **Error Handling**: thiserror for structured errors
- **Validation**: Custom type-safe validation in infrastructure layer

### Key Dependencies (Cargo.toml)

```toml
[dependencies]
thiserror = "1"
nutype = { version = "0.4", features = ["serde", "regex"] }
regex = "1"
lazy_static = "1"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
indexmap = { version = "2", features = ["serde"] }
```

### Current Project Structure

```
src/
├── event_model/         # Core Event Modeling concepts
│   ├── entities.rs      # Command, Event, Projection, etc.
│   ├── diagram.rs       # EventModelDiagram
│   ├── registry.rs      # EntityRegistry for compile-time tracking
│   ├── yaml_types.rs    # YAML domain types (strongly-typed)
│   └── converter.rs     # Converts simple format to EventModelDiagram
├── diagram/             # Visual representation
│   ├── layout.rs        # Layout computation
│   ├── style.rs         # Visual styles
│   ├── theme.rs         # GitHub light/dark themes
│   └── svg.rs           # SVG rendering
├── export/              # Output formats
│   ├── pdf.rs           # PDF export
│   └── markdown.rs      # Markdown export
├── infrastructure/      # Technical utilities
│   ├── types.rs         # Type safety (NonEmptyString, etc.)
│   └── parsing/         # Parsing infrastructure
│       ├── simple_parser.rs    # Old simple text parser (to be removed)
│       ├── simple_lexer.rs     # Old simple text lexer (to be removed)
│       ├── yaml_parser.rs      # YAML parsing with version handling
│       ├── yaml_converter.rs   # YAML to domain type conversion
│       └── mod.rs              # Parsing module interface
└── cli.rs               # Command-line interface
```

## Current Implementation Status

See [ROADMAP.md](ROADMAP.md) for current development priorities and implementation status.

## Architecture Principles

See [README.md#architecture](README.md#architecture) for the core architecture principles.

### Claude-Specific Implementation Notes

- Always use `nutype` crate for domain-specific types (never raw primitives)
- When creating new types, consider if compile-time guarantees can replace runtime checks
- Prefer message passing over `Arc<Mutex<>>`
- Performance optimizations are not a priority (see ADR 20250617-performance-non-priority.md)

### Development Process

- **Type-Driven Development**: Leverage Rust's type system to eliminate bugs at compile time (see ADR 20250617-type-driven-testing.md)
- **Parse, Don't Validate**: All validation happens at system boundaries through parse functions
- **Make Illegal States Unrepresentable**: Design types so invalid states cannot be constructed
- **Incremental Implementation**: Implement functionality while preserving type signatures
- **Testing Strategy**: Test only behaviors that cannot be encoded in types (parsing, rendering output)
- Always consider if new domain types can eliminate error cases

### Development Management for Claude

See [README.md#contributing](README.md#contributing) for the development workflow.

Claude-specific notes:
- Check [ROADMAP.md](ROADMAP.md) for current development priorities
- Use TodoWrite tool to track implementation progress
- Preserve existing type signatures when adding implementations

### Active Development

- Focus on one module implementation at a time
- Use TodoWrite tool for tracking progress within a session
- Implement functions while preserving their type signatures
- Add tests for new implementations

## Development Guidelines

### Cross-Platform Considerations

1. **Terminal Handling**: Use crossterm for consistent behavior across platforms

### Windows-Specific Notes

- Ensure ANSI color support detection
- Handle different line endings (CRLF vs LF)
- Use `cmd.exe /C` for shell commands on Windows

### Error Handling

- Use `thiserror` for domain-specific error types
- Always provide context with `.context("...")`
- Handle process termination gracefully

### Testing Approach

- Unit tests for protocol parsing and state management
- Integration tests with mock evaluators
- _ALWAYS_ make an attempt to use the type system and type-driven-development
  techniques to make it _impossible_ for a test you have written to ever fail
  (it should instead fail to compile if it represents an impossible scenario).

## Feature Implementation

When implementing new functionality:

1. Check priorities in [ROADMAP.md](ROADMAP.md)
2. Create a feature branch for your work
3. Implement incrementally, keeping the app buildable
4. Maintain existing type signatures

## Build and Release

### Local Development

```bash
# Run tests
cargo test

# Check all targets
cargo check --all-targets

# Run Event Modeler
cargo run
```

### Cross-Compilation

```bash
# Install cross
cargo install cross

# Build for Linux
cross build --release --target x86_64-unknown-linux-gnu

# Build for Windows
cross build --release --target x86_64-pc-windows-gnu
```

### Release Process

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Tag the release
4. GitHub Actions will build and upload binaries

## Performance Considerations

- Use `Arc<Mutex<>>` sparingly - prefer message passing
- Stream large outputs instead of buffering
- Use `SmallVec` for small collections

## Accessibility

- Support standard terminal screen readers
- Follow terminal color scheme preferences

## Claude-Specific Git Instructions

- Always make high-quality git commits that explain the _why_ not just the how
- Commit whenever all tests are passing rather than waiting to complete a full story
- NEVER add prefixes to commit message subject lines (no "feat:", "fix:", "chore:", etc.)
- Keep commits focused and atomic
- Use the commit message format shown in README.md

### CRITICAL: PR Chain Management

When working with chained PRs (where PR B is based on PR A), **ALWAYS** rebase downstream PRs after a base PR is merged:

#### The Problem
When we squash-merge a PR, GitHub combines all commits into a single commit. This breaks the chain for downstream PRs because:
1. PR A has commits C1, C2, C3
2. PR B is based on PR A, so it includes C1, C2, C3 + new commits C4, C5
3. When PR A is squash-merged, GitHub creates a new commit C123 on main
4. PR B still references the old commits C1, C2, C3 which no longer exist on main
5. PR B becomes impossible to merge cleanly

#### The Solution
**IMMEDIATELY** after any base PR is merged:

1. **Check for downstream PRs**:
   ```bash
   gh pr list --author @me
   ```

2. **For each downstream PR, rebase onto main**:
   ```bash
   # Check out the downstream PR branch
   git checkout feature/downstream-branch
   
   # Fetch latest changes
   git fetch origin
   
   # Rebase onto main (not the old base branch)
   git rebase origin/main
   
   # Resolve conflicts by keeping the downstream PR changes (use --theirs)
   git checkout --theirs <conflicted-file>
   git add <conflicted-file>
   git rebase --continue
   
   # Force push the rebased branch
   git push --force-with-lease
   
   # Update the PR base reference
   gh pr edit <PR-number> --base main
   ```

3. **Common Conflict Resolution**:
   - Always use `git checkout --theirs` to keep the downstream PR's changes
   - The conflicts occur because git sees the same changes in different commits
   - The downstream PR's version is always the one we want to keep

#### Example Workflow
```bash
# PR #15 just got merged, now rebase PR #17 and PR #18
git checkout feature/domain-extensions
git fetch origin
git rebase origin/main
# Resolve conflicts with --theirs
git push --force-with-lease
gh pr edit 17 --base main

git checkout feature/flow-layout  
git rebase origin/main
# Resolve conflicts with --theirs
git push --force-with-lease
gh pr edit 18 --base main
```

#### Prevention
- **Monitor PR status**: Check `gh pr list` regularly during development
- **Immediate action**: Rebase downstream PRs as soon as base PRs merge
- **Never ignore**: This issue will only get worse if not addressed immediately

## Code Quality Guidelines for Claude

See [README.md#development-standards](README.md#development-standards) for standards.

Additional Claude-specific rules:
- Never use `#[allow(...)]` without getting explicit user permission
- Always run `cargo fmt` and `cargo clippy` before suggesting commits
- Document ALL public items with rustdoc comments
- When naming things, be extremely descriptive (no single-letter variables)

## Type-Safety Implementation Guide for Claude

Core principles are in [README.md#architecture](README.md#architecture). When implementing:

### Parse, Don't Validate
- Use the types in `src/infrastructure/types.rs` module
- Create parse functions that return `Result<ValidType, ParseError>`
- Never use `.unwrap()` or `.expect()` on validation

### Making Illegal States Unrepresentable
- Always check if a runtime `if` statement could be a compile-time type
- Use phantom types (see `TypedPath` in infrastructure/types.rs)
- Use the typestate pattern (see `EntityRegistry` in event_model/registry.rs)

### Test-Driven Type Refinement Process
1. Write test first (TDD)
2. Make it pass with simple implementation
3. Ask: "Can this test failure be made into a compile error?"
4. If yes, refactor to use types and remove the test
5. Document what compile-time guarantee replaced the test

### Examples in This Codebase
- `NonEmpty<T>` - eliminates empty collection checks
- `TypedPath<F,P,E>` - compile-time path validation
- `ThemedRenderer<T>` - compile-time theme selection

## YAML Format Implementation

### Overview

Event Modeler uses a rich YAML format for `.eventmodel` files. The implementation follows a three-stage pipeline:

1. **Parse Stage** (`yaml_parser.rs`): Deserialize YAML into intermediate parsing types
2. **Convert Stage** (`yaml_converter.rs`): Transform parsing types into strongly-typed domain model
3. **Transform Stage** (Phase 3, not yet implemented): Convert YAML model to EventModelDiagram

### Key YAML Components

- **Entity Types**: Events, Commands, Views, Projections, Queries, Automations
- **Data Schemas**: Typed fields with optional type states (e.g., `Email<Verified>`)
- **Test Scenarios**: Given/When/Then format for command testing
- **UI Components**: Hierarchical view definitions with forms and actions
- **Slices**: Flow definitions connecting entities

### Implementation Notes

#### When Adding YAML Features

1. **Update Parsing Types** (`yaml_parser.rs`):
   - Add fields to the appropriate `Yaml*` struct
   - Use `Option<T>` for optional fields
   - Use `IndexMap` to preserve order
   - Add `#[serde(default)]` for collections

2. **Update Domain Types** (`yaml_types.rs`):
   - Use nutype wrappers for all strings (e.g., `EventName`, `CommandName`)
   - Use `NonEmpty<T>` for required collections
   - All types should be validated at construction

3. **Update Converter** (`yaml_converter.rs`):
   - Add conversion logic in appropriate `convert_*` function
   - Validate all constraints (non-empty strings, valid references)
   - Provide helpful error messages with field names

4. **Add Tests**:
   - Test successful conversion cases
   - Test each validation error case
   - Use minimal examples to isolate behavior

#### YAML Parsing Pipeline

```rust
// Stage 1: Parse YAML text
let parsed: YamlEventModel = parse_yaml(content)?;

// Stage 2: Convert to domain types
let domain: domain::YamlEventModel = convert_yaml_to_domain(parsed)?;

// Stage 3: Transform to diagram (TODO)
let diagram: EventModelDiagram = transform_to_diagram(domain)?;
```

#### Error Handling

- YAML syntax errors include line/column information
- Conversion errors specify which field failed validation
- All errors use the `thiserror` crate for consistency

#### Version Handling

- Schema version defaults to current app version if not specified
- Pre-1.0: Accept any version (no compatibility checks)
- Post-1.0: Will implement semantic version compatibility

### YAML Development Guidelines

1. **Always validate at parse boundaries** - never trust string input
2. **Use type states** to track validation (e.g., `Email<Unverified>` vs `Email<Verified>`)
3. **Preserve order** using `IndexMap` where sequence matters
4. **Test error cases** - ensure helpful error messages
5. **Document YAML changes** in [docs/yaml-syntax-guide.md](docs/yaml-syntax-guide.md)

### Common YAML Tasks

#### Adding a New Entity Type

1. Define the entity in `yaml_parser.rs` (parsing representation)
2. Define the domain type in `yaml_types.rs` (validated representation)
3. Add to the main `YamlEventModel` struct in both files
4. Implement conversion in `yaml_converter.rs`
5. Update the syntax guide documentation
6. Add tests for parsing and conversion

#### Adding a Field to Existing Entity

1. Add to parsing struct with appropriate type
2. Add to domain struct with validated type
3. Update conversion function to handle the field
4. Add validation if needed
5. Update documentation
6. Test both success and error cases

#### Debugging YAML Issues

- Check the examples in `tests/fixtures/` for valid YAML
- Use `parse_yaml` error messages for syntax issues
- Use `convert_yaml_to_domain` errors for validation issues
- The YAML syntax guide has comprehensive examples

### Future YAML Work (Phases 3-7)

- Transform YAML domain model to EventModelDiagram
- Implement flow-based layout using slices
- Render rich entity content (data schemas, test scenarios)
- Support sub-diagrams for test scenarios
- Complete visual styling per entity type

## Development Process Rules

### GitHub Issue-Driven Development

All work is now tracked through GitHub issues. Check issues with the `epic` label to see current phases.

### PR-Driven Workflow

1. **Always Update Main First**
   ```bash
   git checkout main
   git pull origin main
   ```

2. **Create Feature Branches**
   - Name branches descriptively: `feature/descriptive-name`
   - For Phase 6 steps: `feature/diagram-step-{number}-{description}`

3. **Commit Frequently**
   - Commit after EVERY small task that builds successfully
   - Write descriptive commit messages explaining "why"
   - No commit message prefixes (no "feat:", "fix:", etc.)

4. **Create PRs Immediately**
   - Create PR after first push (not as draft)
   - Include clear description and test plan
   - Monitor for automated reviews

5. **Handle PR Chains**
   - When a base PR merges, IMMEDIATELY rebase downstream PRs
   - Use `git rebase origin/main` and `--theirs` for conflicts
   - Update PR base with `gh pr edit <number> --base main`

### Testing Requirements

Before committing:
```bash
cargo fmt --all
cargo clippy --workspace --all-targets
cargo test --workspace
```

For Phase 6 visual development:
```bash
cargo run -- tests/fixtures/acceptance/example.eventmodel -o test.svg
# Compare with tests/fixtures/acceptance/example.png
```

### Type-Driven Development

- Use nutype for domain-specific types
- Parse, don't validate - all validation at boundaries
- Make illegal states unrepresentable
- Never use `.unwrap()` or `.expect()` on validation

### Git Commit Guidelines

- High-quality commits that explain the "why"
- Commit whenever tests pass
- Keep commits focused and atomic
- Follow the format in README.md

### Working with @claude GitHub App

When the @claude GitHub App is set up:
1. Issues should have clear acceptance criteria
2. Include "Depends on #X" for dependencies
3. Reference gold standard examples where applicable
4. @claude will add TODO checklists when starting work
