# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Event Modeler is a CLI application that converts text-based Event Model descriptions (`.eventmodel` files) into visual diagrams (SVG/PDF format).

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

- Check implementation priorities in [ROADMAP.md](ROADMAP.md)
- Create feature branches for focused work
- Implement functionality while preserving type signatures

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
```

### Current Project Structure

```
src/
├── event_model/     # Core Event Modeling concepts
│   ├── entities.rs  # Command, Event, Projection, etc.
│   ├── diagram.rs   # EventModelDiagram
│   └── registry.rs  # EntityRegistry for compile-time tracking
├── diagram/         # Visual representation
│   ├── layout.rs    # Layout computation
│   ├── style.rs     # Visual styles
│   ├── theme.rs     # GitHub light/dark themes
│   └── svg.rs       # SVG rendering
├── export/          # Output formats
│   ├── pdf.rs       # PDF export
│   └── markdown.rs  # Markdown export
├── infrastructure/ # Technical utilities
│   ├── types.rs     # Type safety (NonEmptyString, etc.)
│   └── parsing/     # Text parsing (lexer, AST)
└── cli.rs          # Command-line interface
```

## Current Implementation Status

See [ROADMAP.md](ROADMAP.md) for current development priorities and implementation status.

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
