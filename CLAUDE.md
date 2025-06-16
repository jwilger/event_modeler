# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

EventModelRenderer is a CLI application that reads in a text description of an event model and renders it to SVG or PDF

## Development Setup

**Rust Version**: Stable (latest)
**Dependency Management**: Nix flake for local development environment
**Environment**: direnv with .envrc for automatic environment loading

### Common Commands

- `nix develop` - Enter development shell with all dependencies
- `cargo test --workspace` - Run all tests
- `cargo run -- <evaluator>` - Run PrEval with evaluator
- `cargo fmt --all` - Format code
- `cargo clippy --workspace --all-targets` - Run linter
- `pre-commit install` - Install git hooks (when in nix shell)

### ADR (Architecture Decision Records) Commands

- `npm run adr:preview` - Preview ADR documentation locally (http://localhost:4004)
- `npm run adr:build` - Build static ADR documentation site
- View ADRs in `docs/adr/` directory
- Create new ADRs by copying template and following naming convention: `YYYYMMDD-title-with-dashes.md`

### Story Workflow Commands

- `ls PLANNING/todo/` - View available stories prioritized by filename
- `mv PLANNING/todo/00010-*.md PLANNING/doing/` - Start working on a story
- `mv PLANNING/doing/00010-*.md PLANNING/done/` - Mark story complete
- `ls PLANNING/doing/` - Check current work (should only be one story)

## Technical Architecture

### Core Technologies

- **Language**: Rust (latest stable)
- **TUI Framework**: Ratatui (with crossterm backend for cross-platform support)
- **Async Runtime**: Tokio for process management and concurrent operations
- **Serialization**: serde + serde_json for JSON parsing
- **CLI**: clap for argument parsing
- **Type Safety**: nutype for domain-specific types

### Key Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
thiserror = "1"
nutype = { version = "0.4", features = ["serde"] }
```

### Project Structure

TBD

## Architecture Principles

### Type-Driven Design

- Heavy use of algebraic types via Rust's type system
- Use `nutype` crate for all domain-specific types
- Primitive types only at system boundaries (terminal I/O, JSON parsing)
- Traits for converting domain types to external representations
- Make impossible scenarios impossible to represent

### Functional Core, Imperative Shell

- Purely functional core with side effects at boundaries
- Domain logic as pure functions with no side effects
- I/O operations (terminal, process spawning) isolated in shell layer
- Dependencies passed as function arguments for testability
- State transformations through immutable data structures

### Development Process

- **User Story Driven**: All development follows user stories in `PLANNING/` directory
- **Single Story WIP**: Never more than one story in `PLANNING/doing/` at a time
- **Vertical Slices**: Each story represents complete functionality from UI to process management
- **Production Ready**: Each story must be deployable with full tests before moving to next
- Strict TDD: red-green-refactor cycle
- Refactoring includes leveraging type system to prevent test failures
- Always consider if new domain types can eliminate error cases

### Story Management

- Stories are prioritized by 6-digit filename prefix (000010, 000020, etc.)
- Move stories: `mv PLANNING/todo/000010-*.md PLANNING/doing/` to start work
- Complete stories: `mv PLANNING/doing/000010-*.md PLANNING/done/` when deployed
- Each story has clear acceptance criteria and definition of done
- Stories build incrementally - complete dependencies before starting new stories

### Active Story Development

- **Subtask Tracking**: Maintain a "## Current Subtasks" section at the bottom of the active story
- Subtasks are ephemeral implementation details, not requirements
- Update subtasks frequently as work progresses:
  - Add new subtasks as discovered
  - Check off completed subtasks with `[x]`
  - Remove or modify subtasks as understanding evolves
  - Reorder subtasks based on implementation approach
- Subtasks help track progress but can change at any time
- Keep subtasks focused on current implementation work

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

## User Story Implementation

When implementing user stories from PLANNING/:

1. Move the story file from TODO/ to DOING/
2. Create a feature branch named after the story number
3. Implement incrementally, keeping the app buildable
4. Move to DONE/ when complete and merged

## Build and Release

### Local Development

```bash
# Run tests
cargo test

# Check all targets
cargo check --all-targets

# Run PrEval
cargo run -- <evaluator-command>
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

## Git and Commit Practices

- Always make high-quality git commits that explain the _why_ not just the how
- Commit whenever all tests are passing rather than waiting to complete a full story
- NEVER add prefixes to commit message subject lines (no "feat:", "fix:", "chore:", etc.)
- Keep commits focused and atomic

## Code Quality Guidelines

- Never use `#[allow(...)]` annotations to bypass compiler/linter rules without explicit permission on a case-by-case basis
- All code must pass `cargo fmt` and `cargo clippy` before committing
- Write tests for all new functionality
- Document public APIs with rustdoc comments
- Use descriptive variable and function names

## Type-Safety Techniques

When implementing new functionality, apply these type-driven design principles:

### Parse, Don't Validate

- Validation should happen once at system boundaries (I/O, JSON parsing, user input)
- Use newtypes that can only be constructed through validated constructors
- After parsing, work with validated types that maintain invariants

### Make Illegal States Unrepresentable

- Use sum types (enums) to represent mutually exclusive states
- Use newtypes with validation to prevent invalid values
- Leverage Rust's ownership system to enforce state transitions
- Example: A `CounterValue` that cannot be negative by construction

### Typestate Pattern

- Track state transitions in the type system
- Use phantom types to encode state at compile time
- Make invalid state transitions compilation errors
- Example: A builder that won't compile without required fields

### Phantom Types

- Encode compile-time guarantees without runtime cost
- Use zero-sized types to track properties
- Useful for units of measure, validation states, etc.

### Sealed Traits

- Use the sealed trait pattern to control implementations
- Ensures exhaustive matching and prevents external implementations
- Example: `MetricValue` trait that only our metric types can implement

### Test-Driven Type Refinement

1. Write tests first following TDD
2. After tests pass, refactor types to make test scenarios impossible
3. Document which compile-time guarantees replace each test
4. Remove tests that are made redundant by type constraints
5. Keep only integration tests at I/O boundaries

### Examples of Type Safety in Practice

- Required fields are enforced by the type system, not runtime checks
