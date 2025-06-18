# Event Modeler

[![CI](https://github.com/jwilger/event_modeler/workflows/CI/badge.svg)](https://github.com/jwilger/event_modeler/actions/workflows/ci.yml)

Generate Event Modeling diagrams from YAML-based event model descriptions. Write `.eventmodel` files, get professional SVG/PDF diagrams.

## Quick Start

```bash
# Install (from source for now)
git clone https://github.com/jwilger/event_modeler
cd event_modeler
cargo build --release
# Add target/release to your PATH

# Create an event model
cat > example.eventmodel << 'EOF'
workflow: Order Processing System

swimlanes:
  - customer: "Customer"
  - orders: "Orders & Inventory"

events:
  OrderPlaced:
    description: "Customer placed an order"
    swimlane: orders
    data:
      order_id: OrderId
      customer_id: CustomerId
      items: List<OrderItem>
      total: Money

commands:
  PlaceOrder:
    description: "Customer places a new order"
    swimlane: customer
    data:
      customer_id: CustomerId
      items: List<OrderItem>

projections:
  OrderList:
    description: "List of all orders"
    swimlane: orders
    fields:
      orders: List<OrderSummary>

automations:
  ProcessPayment:
    description: "Process payment when order is placed"
    swimlane: orders

slices:
  OrderFlow:
    - PlaceOrder -> OrderPlaced
    - OrderPlaced -> OrderList
    - OrderPlaced -> ProcessPayment
EOF

# Generate diagram (light theme by default)
event_modeler example.eventmodel

# Generate with dark theme
event_modeler example.eventmodel --dark

# Specify output file
event_modeler example.eventmodel -o diagram.svg
```

## Project Status

🚧 **Major Rewrite In Progress** - Transitioning from simple text format to rich YAML-based event modeling language.

**What's Ready**: 
- YAML parsing with schema versioning
- Type-safe domain model with all entity types
- Strongly-typed parsing with comprehensive validation
- Error handling with line/column information
- SVG rendering infrastructure (being extended)

**What's Coming** (Phase 3-6):
- Full entity rendering (events, commands, views, projections, queries, automations)
- Flow-based layout algorithm using slice definitions
- Test scenario sub-diagrams
- Professional visual styling with color coding
- PDF export
- Complete documentation

## Event Model YAML Format

`.eventmodel` files use a structured YAML format that captures rich event modeling concepts:

### Basic Structure

```yaml
version: 0.3.0  # Optional, defaults to current Event Modeler version
workflow: Your Workflow Name

swimlanes:
  - identifier: "Display Name"
  - another_lane: "Another Display Name"

events:
  EventName:
    description: "What happened"
    swimlane: identifier
    data:
      field_name: TypeName
      another_field: TypeName<State>

commands:
  CommandName:
    description: "What the user wants to do"
    swimlane: identifier
    data:
      field_name:
        type: TypeName
        generated: true  # Optional, for system-generated values
    tests:  # Optional test scenarios
      "Test Name":
        Given:
          - PreviousEvent:
              field: value
        When:
          - CommandName:
              field: value
        Then:
          - ExpectedEvent:
              field: value

views:
  ViewName:
    description: "UI screen or component"
    swimlane: identifier
    components:
      - SimpleComponent: ComponentType
      - FormComponent:
          type: Form
          fields:
            field_name: InputType
          actions:
            - ActionName

projections:
  ProjectionName:
    description: "Read model projection"
    swimlane: identifier
    fields:
      field_name: TypeName
      union_field: TypeA | TypeB

queries:
  QueryName:
    swimlane: identifier
    inputs:
      param_name: TypeName
    outputs:
      field_name: TypeName
      # OR for conditional outputs:
      one_of:
        case_name:
          field: Type
        other_case: ErrorType

automations:
  AutomationName:
    description: "Automated process"
    swimlane: identifier

slices:
  SliceName:
    - Source -> Target
    - Source.Component -> Target
    - Source.Component.Action -> Target
```

### Entity Types

1. **Events** - Things that happened (past tense)
   - Contain data schema with typed fields
   - Must reference a valid swimlane

2. **Commands** - User intentions (imperative mood)
   - Contain data schema with optional generated fields
   - Can include test scenarios (Given/When/Then)

3. **Views** - UI screens or components
   - Define component hierarchies
   - Support forms with fields and actions

4. **Projections** - Read models built from events
   - Define field schemas
   - Support union types (TypeA | TypeB)

5. **Queries** - Data retrieval operations
   - Define input parameters
   - Support conditional outputs (one_of)

6. **Automations** - System processes
   - Triggered by events or other entities

### Slices (Flows)

Slices define the connections between entities:
- Simple: `EntityA -> EntityB`
- Component-specific: `View.Component -> Command`
- Action-specific: `View.Form.Submit -> Command`

### Rules

- All entity names must be unique across the model
- Swimlanes must be defined before being referenced
- Collections cannot be empty
- All string values must be non-empty
- Version defaults to current Event Modeler version if not specified

See the `examples/` directory for complete examples.

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