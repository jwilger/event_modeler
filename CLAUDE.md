# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Critical Rules for Claude Code

1. **ALWAYS create a feature branch before making commits**
2. **ALWAYS use `nutype` for domain-specific types** (never raw primitives)
3. **NEVER use `.unwrap()` or `.expect()` on validation**
4. **ALWAYS run `cargo fmt` and `cargo clippy` before suggesting commits**
5. **NEVER use `#[allow(...)]` without explicit user permission**
6. **Check [PLANNING.md](PLANNING.md) for current work and process rules**

## Repository Overview

Event Modeler is a CLI application that converts YAML-based event model descriptions (`.eventmodel` files) into visual diagrams (SVG/PDF format).

**Current Status**: Major rewrite in progress - transitioning from simple text format to rich YAML-based event modeling language. See [PLANNING.md](PLANNING.md) for detailed implementation plan and current phase.

## Development Setup

**Environment**: Nix flake with direnv for automatic environment loading

```bash
# Enter development shell
nix develop

# Essential commands
cargo test --workspace     # Run all tests
cargo fmt --all           # Format code
cargo clippy --workspace --all-targets  # Lint
cargo run -- example.eventmodel -o output.svg  # Run CLI
```

## Technical Architecture

### Core Technologies

- **Language**: Rust (latest stable)
- **Type Safety**: nutype for domain-specific types
- **Error Handling**: thiserror for structured errors
- **Parsing**: serde_yaml for YAML format
- **Collections**: IndexMap for preserving order

### Project Structure

```
src/
├── event_model/         # Core Event Modeling concepts
├── diagram/             # Visual representation (being rewritten)
├── export/              # Output formats (SVG, PDF, Markdown)
├── infrastructure/      # Type safety utilities
│   ├── types.rs         # NonEmpty<T>, TypedPath<F,P,E>, etc.
│   └── parsing/         # YAML parsing pipeline
└── cli.rs               # Command-line interface
```

## Core Development Principles

1. **Type-Driven Development**: Make illegal states unrepresentable at compile time
2. **Parse, Don't Validate**: All validation happens at system boundaries
3. **No Performance Optimizations**: Correctness over speed (see ADR)

### Type-Safety Quick Reference

```rust
// Always use domain types
let name = EventName::new("UserCreated")?;  // ✓
let name = "UserCreated".to_string();        // ✗

// Parse at boundaries
fn parse_config(input: &str) -> Result<Config, ParseError> { ... }  // ✓
fn validate_config(config: &Config) -> bool { ... }                 // ✗

// Use types from infrastructure/types.rs
NonEmpty<T>         // For required collections
TypedPath<F,P,E>    // For compile-time path validation
Sanitized<Context>  // For context-aware string sanitization
```

## YAML Implementation Overview

Event Modeler uses a three-stage pipeline for YAML processing:

```rust
// Stage 1: Parse YAML text
let parsed: YamlEventModel = parse_yaml(content)?;

// Stage 2: Convert to domain types  
let domain: domain::YamlEventModel = convert_yaml_to_domain(parsed)?;

// Stage 3: Transform to diagram (in progress)
let diagram: EventModelDiagram = transform_to_diagram(domain)?;
```

### Key YAML Principles

1. **Always validate at parse boundaries** - never trust string input
2. **Use nutype wrappers** for all domain strings (`EventName`, `CommandName`, etc.)
3. **Preserve order** with `IndexMap` where sequence matters
4. **Provide helpful errors** with field names and context

### YAML Development Tasks

When modifying YAML support:
1. Update parsing types in `yaml_parser.rs`
2. Update domain types in `yaml_types.rs`
3. Update converter in `yaml_converter.rs`
4. Add tests for both success and error cases
5. Update [docs/yaml-syntax-guide.md](docs/yaml-syntax-guide.md)

## Git Commit Guidelines

- Make high-quality commits that explain the *why* not just the *how*
- NO commit prefixes (no "feat:", "fix:", "chore:")
- Keep commits focused and atomic
- See [README.md](README.md) for commit message format

## Code Quality Standards

- Document ALL public items with rustdoc comments
- Use descriptive names (no single-letter variables)
- Run `cargo fmt` and `cargo clippy` before every commit
- Preserve existing type signatures when implementing `todo!()`

## References

- **Current Work & Process**: [PLANNING.md](PLANNING.md)
- **Architecture & Contributing**: [README.md](README.md)
- **Development Priorities**: [ROADMAP.md](ROADMAP.md)
- **YAML Syntax**: [docs/yaml-syntax-guide.md](docs/yaml-syntax-guide.md)