# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Event Modeler is a CLI application that reads in a text description of an event model and renders it to SVG or PDF.

For general architecture and contribution guidelines, see [README.md](README.md).

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

See [README.md#architecture](README.md#architecture) for the core architecture principles.

### Claude-Specific Implementation Notes

- Always use `nutype` crate for domain-specific types (never raw primitives)
- When creating new types, consider if compile-time guarantees can replace runtime checks
- Prefer message passing over `Arc<Mutex<>>`
- Use `SmallVec` for small collections

### Development Process

- **User Story Driven**: All development follows user stories in `PLANNING/` directory
- **Single Story WIP**: Never more than one story in `PLANNING/doing/` at a time
- **Vertical Slices**: Each story represents complete functionality from UI to process management
- **Production Ready**: Each story must be deployable with full tests before moving to next
- Strict TDD: red-green-refactor cycle
- Refactoring includes leveraging type system to prevent test failures
- Always consider if new domain types can eliminate error cases

### Story Management for Claude

See [README.md#contributing](README.md#contributing) for the story workflow.

Claude-specific notes:
- Always check for existing work in PLANNING/doing/ before starting
- Update the "## Current Subtasks" section frequently as you work
- Use TodoWrite tool to track implementation progress

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

## Claude-Specific Git Instructions

- Always make high-quality git commits that explain the _why_ not just the how
- Commit whenever all tests are passing rather than waiting to complete a full story
- NEVER add prefixes to commit message subject lines (no "feat:", "fix:", "chore:", etc.)
- Keep commits focused and atomic
- Use the commit message format shown in README.md

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
- Use the types in `src/type_safety.rs` module
- Create parse functions that return `Result<ValidType, ParseError>`
- Never use `.unwrap()` or `.expect()` on validation

### Making Illegal States Unrepresentable
- Always check if a runtime `if` statement could be a compile-time type
- Use phantom types (see `TypedPath` in type_safety.rs)
- Use the typestate pattern (see `EntityRegistry` in model/registry.rs)

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
