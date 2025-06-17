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

🚧 **Early Development** - Type system complete, implementation in progress.

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

See [docs/adr/](docs/adr/) for architecture decisions.

## Documentation

- **DSL Reference**: See [examples/](examples/) and [docs/syntax.md](docs/syntax.md) (coming soon)
- **API Docs**: Run `cargo doc --open`
- **Event Modeling**: Learn about the methodology at [eventmodeling.org](https://eventmodeling.org)

## Contributing

We use story-driven development with strict type safety:

1. Pick a story from [PLANNING/todo/](PLANNING/todo/)
2. Move to [PLANNING/doing/](PLANNING/doing/) (only one at a time)
3. Create feature branch from story number
4. Follow TDD: red-green-refactor
5. Maintain zero runtime validation
6. Move to [PLANNING/done/](PLANNING/done/) when complete

### Development Standards

- All code must pass `cargo fmt` and `cargo clippy`
- No `#[allow(...)]` without explicit approval
- High-quality commits explaining the "why"
- No commit prefixes (no "feat:", "fix:", etc.)

## License

MIT - see [LICENSE](LICENSE)

Copyright (c) 2025 John Wilger