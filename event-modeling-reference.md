# Event Modeling Reference Guide

## Overview

Event Modeling is a method for designing information systems created by Adam Dymitruk. It provides a blueprint that shows how information flows through a system over time, similar to a movie storyboard. Unlike traditional modeling approaches that create multiple disconnected artifacts, Event Modeling produces a single, comprehensive visual representation.

## Core Philosophy

### The 7 Year Old Test

If a 7-year-old cannot understand the model with minimal explanation, it's too complex. This ensures the model remains accessible to all stakeholders, not just technical team members.

### Information Completeness

Every piece of information shown on the right side of the timeline must be traceable to its origin on the left. This creates a complete story with no "plot holes" where data appears mysteriously.

### Forward-Only Time

Information flows left-to-right chronologically. No time travel or backward references allowed. This matches how we naturally understand stories and prevents circular dependencies.

## Visual Language

### Color Coding

- **Orange/Yellow**: Events (things that happened)
- **Blue**: Commands (intentions to change state)
- **Green**: Read Models/Projections (views of current state)
- **Purple/Gray**: Automations (system-triggered actions)
- **Black/White**: UI Wireframes and mockups

### Spatial Organization

- **Horizontal Swimlanes**: Separate concerns by actor, system, or department
- **Vertical Slices**: Feature boundaries for independent development
- **Left-to-Right Timeline**: Chronological flow of information

## The 4 Patterns

### 1. Command Pattern

```
UI/Actor → Command → Event(s)
```

- User initiates action through interface
- Command captures intent with validation
- Events record what actually happened

### 2. View/Query Pattern

```
Event(s) → Projection → View/UI
```

- Events update read models (projections)
- Views query projections for display
- Optimized for specific UI needs

### 3. Automation Pattern

```
Event → Automation → Command → Event
```

- System reacts to events automatically
- Triggers new commands without user intervention
- Examples: notifications, scheduled tasks, policies

### 4. Translation Pattern

```
External Event → Translation → Internal Event
```

- Adapts external system events to internal format
- Maintains system boundaries
- Anti-corruption layer implementation

## Modeling Process

### Step 1: Brain Storming

- Identify all events in the system (orange sticky notes)
- Use past tense: "OrderPlaced", "PaymentReceived"
- No judgment, capture everything

### Step 2: The Plot

- Arrange events chronologically left-to-right
- Group related events into swimlanes
- Identify missing events to complete the story

### Step 3: Story Board

- Add UI wireframes above the timeline
- Show what users see and do
- Connect UI elements to commands/views

### Step 4: Identify Inputs

- Add commands (blue boxes) that cause events
- Include all required data/parameters
- Show validation rules

### Step 5: Identify Outputs

- Add read models (green boxes) that events update
- Show queries that feed the UI
- Ensure all displayed data has a source

### Step 6: Apply Conway's Law

- Draw vertical slices through the model
- Each slice = autonomous feature/team
- Minimize cross-slice dependencies

### Step 7: Elaborate Scenarios

- Add Given-When-Then acceptance criteria
- Cover happy paths and edge cases
- Make each slice independently testable

## Best Practices

### Event Design

- Events are immutable facts in past tense
- Include all necessary data (denormalized)
- Events never fail - they already happened
- Small, focused events over large "god events"

### Command Design

- Commands can fail with validation
- Include actor/user context
- One command may produce multiple events
- Commands are intentions, not implementations

### Projection Design

- Optimized for specific read needs
- Can combine multiple event streams
- Eventually consistent with events
- Disposable and rebuildable

### Swimlane Organization

- Top lane: UI/User interactions
- Middle lanes: Business logic (commands/events/projections)
- Bottom lanes: External systems/integrations
- Keep related concepts vertically aligned

### Slice Definition

- Each slice is independently deployable
- Contains complete feature from UI to storage
- Minimal dependencies on other slices
- Clear ownership boundaries

## Common Patterns

### Registration Flow

```
Registration Form → Register Command → UserRegistered Event
UserRegistered → User Profile Projection → Profile View
UserRegistered → Send Welcome Email Automation
```

### Order Processing

```
Order Form → Place Order Command → OrderPlaced Event
OrderPlaced → Inventory Projection → Updated Stock View
OrderPlaced → Process Payment Automation → PaymentProcessed Event
PaymentProcessed → Order Status Projection → Order Confirmation View
```

### Integration Pattern

```
External Payment Gateway Event → Payment Translation → PaymentReceived Event
PaymentReceived → Update Order Status → OrderPaid Event
```

## Anti-Patterns to Avoid

### 1. Chatty Events

Creating too many fine-grained events that should be combined

### 2. Command-Query Mixing

Returning data from commands instead of using separate queries

### 3. Backward References

Events depending on future state or circular dependencies

### 4. Missing Actors

Not showing who/what initiates commands

### 5. Projection Overload

Creating too many specialized projections instead of reusable ones

### 6. Slice Coupling

Strong dependencies between vertical slices

## Acceptance Criteria Format

Each slice should include Given-When-Then scenarios:

```
Given: [Current state/preconditions]
When: [Action taken/command issued]
Then: [Expected outcomes/events produced]
```

Example:

```
Given: Customer has items in shopping cart
When: Customer clicks "Place Order" with valid payment
Then:
  - OrderPlaced event is created
  - Payment is processed
  - Order confirmation is displayed
  - Inventory is updated
```

## Linking and Documentation

### Link Types

- Commands: Link to API documentation
- Events: Link to schema definitions
- Projections: Link to query interfaces
- UI Elements: Link to design mockups
- Slices: Link to feature specifications

### Documentation Structure

```
/docs
  /ui           - Wireframe details and mockups
  /commands     - Command handlers and validation
  /events       - Event schemas and meanings
  /projections  - Read model structures
  /queries      - Available query operations
  /slices       - Feature specifications
```

## Model Evolution

### Adding Features

1. Start with new UI mockup
2. Add new commands/events
3. Update affected projections
4. Define new slice with acceptance criteria

### Refactoring

1. Events are immutable - never change existing
2. Add new events for new requirements
3. Migrate projections incrementally
4. Deprecate old patterns gradually

### Versioning

- Event schemas are versioned
- New fields are added, never removed
- Projections handle multiple event versions
- Commands can evolve with backward compatibility

## Metrics and Analysis

### Model Health Indicators

- Events per slice (aim for 3-7)
- Cross-slice dependencies (minimize)
- Projection complexity (keep focused)
- Command failure scenarios covered

### Team Indicators

- Slices per team (balanced distribution)
- Slice completion rate
- Model coverage (UI to storage)
- Acceptance criteria completeness

## Tool Integration

### Version Control

- Models stored as text for diffing
- Branching for model experiments
- Pull requests for model reviews
- Tagged releases for model versions

### CI/CD Pipeline

- Validate model syntax
- Generate diagrams automatically
- Update documentation
- Deploy model changes

### Team Collaboration

- Shared understanding through visual model
- Clear ownership boundaries
- Parallel development enablement
- Reduced integration conflicts

This reference guide provides the foundational knowledge needed to create accurate Event Modeling diagrams that follow the methodology's principles and best practices.
