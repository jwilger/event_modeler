# Event Modeler

[![CI](https://github.com/jwilger/event_modeler/workflows/CI/badge.svg)](https://github.com/jwilger/event_modeler/actions/workflows/ci.yml)

Generate Event Modeling diagrams from text descriptions. Write `.eventmodel` files, get SVG/PDF diagrams.

## Quick Start

```bash
# Install (from source for now)
git clone https://github.com/jwilger/event_modeler
cd event_modeler
cargo build --release
# Add target/release to your PATH

# Create an event model
cat > example.eventmodel << 'EOF'
Title: Order Processing System

Swimlane: Customer
- Command: PlaceOrder
- Command: CancelOrder

Swimlane: Orders
- Event: OrderPlaced
- Event: OrderCancelled
- Projection: OrderList
- Policy: ProcessPayment

PlaceOrder -> OrderPlaced
OrderPlaced -> OrderList
OrderPlaced -> ProcessPayment
CancelOrder -> OrderCancelled
OrderCancelled -> OrderList
EOF

# Generate diagram (light theme by default)
event_modeler example.eventmodel

# Generate with dark theme
event_modeler example.eventmodel --dark

# Specify output file
event_modeler example.eventmodel -o diagram.svg
```

## Project Status

✅ **MVP Complete** - Full pipeline from .eventmodel files to SVG diagrams is working!

**What's Ready**: 
- Text parsing of .eventmodel files
- Layout computation with automatic sizing
- SVG rendering with GitHub light/dark themes
- Entity types: Command, Event, Projection, Policy, External System, Aggregate
- Error handling with helpful messages
- Integration tests

**What's Coming**:
- Additional entity types (UI/Wireframe, Query, Automation)
- Connector labels
- PDF export
- Markdown documentation export
- Installation via cargo install

## Event Model Syntax

`.eventmodel` files use a simple, readable syntax:

```
Title: Your Model Name

Swimlane: Actor or System Name
- Command: CommandName
- Event: EventName
- Projection: ProjectionName
- Policy: PolicyName
- External System: SystemName
- Aggregate: AggregateName

# Connections between entities
CommandName -> EventName
EventName -> ProjectionName
```

### Rules
- Title must come first
- Entity names must be unique across the entire model
- Connectors can only reference existing entities
- Use "External System" (two words) for external systems

See the `examples/` directory for more examples.

## Development Setup

```bash
# Clone and enter nix shell (includes all dependencies)
git clone https://github.com/jwilger/event_modeler
cd event_modeler
nix develop  # or use direnv

# Run tests
cargo test

# Build
cargo build

# Generate docs
cargo doc --open
```

This project is optimized for development with [Claude Code](https://claude.ai/code) - see [CLAUDE.md](CLAUDE.md) for AI pair programming guidelines.

## Architecture

### Type-Driven Design

- Heavy use of algebraic types via Rust's type system
- Domain-specific types via `nutype` crate
- Primitive types only at system boundaries
- Zero runtime validation - all invariants enforced at compile time

### Functional Core, Imperative Shell

- Pure functions for domain logic
- Side effects isolated at boundaries
- State transformations through immutable data

### Key Principles

- **Parse, Don't Validate**: Validation happens once at system boundaries
- **Make Illegal States Unrepresentable**: Use sum types and newtypes
- **Typestate Pattern**: Track state transitions in the type system

Architecture decisions are documented in the code, this README, and the [Architecture Decision Records](docs/adr/).

## Documentation

- **API Docs**: Run `cargo doc --open` for complete API documentation
- **Module Structure**: See module documentation in the codebase
- **Event Modeling**: Learn about the methodology at [eventmodeling.org](https://eventmodeling.org)
- **DSL Reference**: Coming soon (examples/ and docs/syntax.md)

## Contributing

We use feature-driven development with strict type safety:

1. Check current development priorities in [ROADMAP.md](ROADMAP.md)
2. Create feature branch for your work
3. Follow TDD: red-green-refactor
4. Maintain zero runtime validation
5. Add implementations without changing type signatures

### Development Standards

- All code must pass `cargo fmt` and `cargo clippy`
- No `#[allow(...)]` without explicit approval
- High-quality commits explaining the "why"
- No commit prefixes (no "feat:", "fix:", etc.)

## License

MIT - see [LICENSE](LICENSE)

Copyright (c) 2025 John Wilger