# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Critical Rules for Claude Code

1. **ALWAYS use MCP workflow tools** (`workflow_next`, `workflow_status`, etc.) for project workflow
2. **ALWAYS create a feature branch before making commits**
3. **ALWAYS use `nutype` for domain-specific types** (never raw primitives)
4. **NEVER use `.unwrap()` or `.expect()` on validation**
5. **ALWAYS run `cargo fmt` and `cargo clippy` before suggesting commits**
6. **NEVER use `#[allow(...)]` without explicit user permission**
7. **Check GitHub issues and epics for current work** (see Project Workflow section)

## Collaboration Style

Act as a peer programmer, not an assistant:

- **Be direct**: Skip excessive agreement and enthusiasm. No need for "Excellent!", "You're absolutely right!", etc.
- **Think critically**: Question approaches that seem suboptimal. Offer alternatives when you see better solutions.
- **Seek clarity**: If reasoning isn't clear, ask why before proceeding.
- **Give honest feedback**: Only agree when you genuinely think something is correct or well-designed.
- **Professional tone**: Respond like a colleague, not a cheerleader. Focus on the code and problem-solving.

## Conversation Compaction

When the conversation is compacted (manual or auto-compact), ensure [CLAUDE.md](CLAUDE.md) is reviewed and check GitHub issues to maintain awareness of:

- Critical development rules and architecture principles
- Current work status via GitHub issues and epics
- Active PRs and their status

## Repository Overview

Event Modeler is a CLI application that converts YAML-based event model descriptions (`.eventmodel` files) into visual diagrams (SVG/PDF format).

**Current Status**: Major rewrite in progress - transitioning from simple text format to rich YAML-based event modeling language. Work is tracked via GitHub issues with epics for major phases.

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

- Make high-quality commits that explain the _why_ not just the _how_
- NO commit prefixes (no "feat:", "fix:", "chore:")
- Keep commits focused and atomic
- See [README.md](README.md) for commit message format

## Code Quality Standards

- Document ALL public items with rustdoc comments
- Use descriptive names (no single-letter variables)
- Run `cargo fmt` and `cargo clippy` before every commit
- Preserve existing type signatures when implementing `todo!()`

## Project Workflow

### GitHub Issues & Epics

Development work is tracked using GitHub issues with the following structure:

1. **Epics**: Major phases labeled with "epic"

   - Track overall progress for large features
   - Contain multiple sub-issues
   - Examples: "Phase 6: Diagram Module Rewrite", "MCP Workflow Server Development"

2. **Sub-Issues**: Individual work items

   - Linked to epics using GitHub's sub-issue feature
   - Include acceptance criteria, dependencies, and implementation notes
   - Use "Depends on #X" pattern for dependencies

3. **Finding Work**:
   - ALWAYS use MCP `workflow_next` tool to determine what to work on
   - Use MCP `workflow_status` tool to check current state
   - Use MCP `workflow_create_pr` tool to create pull requests
   - Use MCP `workflow_monitor_reviews` to check PR review status
   - The workflow tools enforce proper PR review discipline

### PR-Driven Development

1. Create feature branches from issues
2. Make commits frequently with clear messages
3. Create PRs early (not as draft)
4. Monitor and address all reviews
5. Merge triggers next work item

## References

- **Architecture & Contributing**: [README.md](README.md)
- **Development Process**: [DEVELOPMENT_PROCESS.md](DEVELOPMENT_PROCESS.md)
- **Roadmap & Vision**: [ROADMAP.md](ROADMAP.md)
- **YAML Syntax**: [docs/yaml-syntax-guide.md](docs/yaml-syntax-guide.md)

## REMEMBER

- ALWAYS use prefer MCP tools available to you instead of Bash commands.
- If an MCP tool call fails, make sure you are calling it properly rather than immediately falling back to a Bash command.
- ALWAYS use our workflow MCP server to determine your next task and follow the suggested next actions its responses provide.
