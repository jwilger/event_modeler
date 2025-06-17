# Event Modeler

Generate Event Modeling diagrams from text descriptions. Write `.eventmodel` files, get SVG/PDF diagrams.

## Quick Start

```bash
# Install
cargo install event_modeler

# Create an event model
cat > example.eventmodel << 'EOF'
title: Order Processing System

swimlane Customer:
  wireframe "Place Order" -> command "Submit Order"
  
swimlane Orders:
  command "Submit Order" -> event "Order Placed"
  event "Order Placed" -> projection "Order List"
  query "Get Orders" <- projection "Order List"
EOF

# Generate diagram
event_modeler render example.eventmodel
```

## Project Status

🚧 **Early Development** - Module structure and type system complete, core functionality in progress.

**What's Ready**: Complete type-safe domain model, module organization, documentation
**What's Next**: CLI parsing, text processing, layout computation, SVG rendering

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