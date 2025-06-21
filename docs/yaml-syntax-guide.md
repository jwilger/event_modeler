# Event Modeler YAML Syntax Guide

This guide provides a comprehensive reference for the Event Modeler YAML format used in `.eventmodel` files.

## Table of Contents

- [Overview](#overview)
- [File Structure](#file-structure)
- [Version Field](#version-field)
- [Workflow](#workflow)
- [Swimlanes](#swimlanes)
- [Entity Types](#entity-types)
  - [Events](#events)
  - [Commands](#commands)
  - [Views](#views)
  - [Projections](#projections)
  - [Queries](#queries)
  - [Automations](#automations)
- [Slices (Flows)](#slices-flows)
- [Data Types](#data-types)
- [Test Scenarios](#test-scenarios)
- [Best Practices](#best-practices)
- [Common Patterns](#common-patterns)
- [Error Messages](#error-messages)

## Overview

Event Modeler uses YAML format to describe event models. The format is designed to capture the rich semantics of Event Modeling, including:

- Multiple types of entities (events, commands, views, etc.)
- Data schemas with type annotations
- UI component hierarchies
- Test scenarios using Given/When/Then
- Flow definitions using slices

## File Structure

A complete `.eventmodel` file has this structure:

```yaml
version: 0.3.0  # Optional, defaults to current Event Modeler version
workflow: Workflow Name

swimlanes:
  - identifier: "Display Name"
  # ... more swimlanes

events:
  EventName:
    # ... event definition
  # ... more events

commands:
  CommandName:
    # ... command definition
  # ... more commands

views:
  ViewName:
    # ... view definition
  # ... more views

projections:
  ProjectionName:
    # ... projection definition
  # ... more projections

queries:
  QueryName:
    # ... query definition
  # ... more queries

automations:
  AutomationName:
    # ... automation definition
  # ... more automations

slices:
  - name: Slice Name
    connections:
      - Connection1
      - Connection2
  # ... more slices
```

### Required Fields

- `workflow`: The name of your event model
- `swimlanes`: At least one swimlane must be defined

### Optional Sections

All entity sections (events, commands, views, etc.) are optional. Include only what your model needs.

## Version Field

The `version` field specifies the schema version:

```yaml
version: 0.3.0
```

- If omitted, defaults to the current Event Modeler version
- Pre-1.0: No backward compatibility guarantees
- Post-1.0: Will follow semantic versioning for compatibility

## Workflow

The workflow name identifies your event model:

```yaml
workflow: User Registration Flow
```

- Must be non-empty
- Typically describes the business process being modeled

## Swimlanes

Swimlanes organize entities by actor, system, or boundary:

```yaml
swimlanes:
  - frontend: "User Interface"
  - backend: "API & Business Logic"
  - events: "Event Store"
```

### Format Options

1. Simple format:
   ```yaml
   swimlanes:
     - identifier: "Display Name"
   ```

2. The identifier becomes the key to reference in entities

### Rules

- At least one swimlane must be defined
- Identifiers must be unique
- Display names should be descriptive

## Entity Types

### Events

Events represent things that have happened (past tense):

```yaml
events:
  UserRegistered:
    description: "A new user account was created"
    swimlane: events
    data:
      user_id: UserId
      email: Email
      registered_at: Timestamp
```

#### Event Fields

- `description` (optional): Human-readable description
- `swimlane` (required): Reference to a defined swimlane
- `data` (optional): Schema definition with typed fields

#### Data Field Formats

1. Simple type:
   ```yaml
   field_name: TypeName
   ```

2. Generic type:
   ```yaml
   email: Email<Verified>
   items: List<Item>
   ```

3. Complex field definition:
   ```yaml
   user_id:
     type: UserId
     stream-id: true
   ```

### Commands

Commands represent user intentions (imperative mood):

```yaml
commands:
  RegisterUser:
    description: "Register a new user account"
    swimlane: frontend
    data:
      email: Email
      password:
        type: Password
        generated: false
      user_id:
        type: UserId
        generated: true
    tests:
      "Successful Registration":
        Given: []
        When:
          - RegisterUser:
              email: "user@example.com"
              password: "secret123"
        Then:
          - UserRegistered:
              email: "user@example.com"
```

#### Command Fields

- `description` (optional): What the command does
- `swimlane` (required): Where the command originates
- `data` (optional): Input schema
- `tests` (optional): Test scenarios

#### Data Field Options

- `type`: The field's type
- `generated`: Whether the system generates this value
- `stream-id`: Whether this identifies the event stream

### Views

Views represent UI screens or components:

```yaml
views:
  RegistrationForm:
    description: "User registration screen"
    swimlane: frontend
    components:
      - Logo: Image
      - Title: Text
      - RegistrationFields:
          type: Form
          fields:
            email: EmailInput
            password: PasswordInput
            confirm_password: PasswordInput
          actions:
            - Submit
            - Cancel
      - LoginLink: Link
```

#### View Fields

- `description` (optional): Screen/component purpose
- `swimlane` (required): UI layer reference
- `components` (required): Component hierarchy

#### Component Formats

1. Simple component:
   ```yaml
   - ComponentName: ComponentType
   ```

2. Form component:
   ```yaml
   - FormName:
       type: Form
       fields:
         field_name: InputType
       actions:
         - ActionName
   ```

### Projections

Projections represent read models or view models:

```yaml
projections:
  UserList:
    description: "List of all registered users"
    swimlane: backend
    fields:
      users: List<UserSummary>
      total_count: Integer
      last_updated: Timestamp
```

#### Projection Fields

- `description` (optional): What data this projection provides
- `swimlane` (required): Where the projection lives
- `fields` (required): Schema of the projection

#### Field Type Options

1. Simple types:
   ```yaml
   count: Integer
   name: String
   ```

2. Union types:
   ```yaml
   status: Active | Inactive | Pending
   ```

3. Generic types:
   ```yaml
   items: List<Item>
   metadata: Map<String, Value>
   ```

### Queries

Queries represent data retrieval operations:

```yaml
queries:
  GetUserByEmail:
    swimlane: backend
    inputs:
      email: Email
    outputs:
      user: UserDetails
      
  FindUser:
    swimlane: backend
    inputs:
      search_term: String
    outputs:
      one_of:
        found:
          user: UserDetails
          match_score: Float
        not_found: UserNotFoundError
        multiple_matches:
          users: List<UserSummary>
          total: Integer
```

#### Query Fields

- `swimlane` (required): Where the query executes
- `inputs` (required): Query parameters
- `outputs` (required): Result schema

#### Output Formats

1. Simple output:
   ```yaml
   outputs:
     field_name: Type
   ```

2. Conditional output:
   ```yaml
   outputs:
     one_of:
       case_name:
         field: Type
       error_case: ErrorType
   ```

### Automations

Automations represent system processes:

```yaml
automations:
  EmailVerificationSender:
    description: "Sends verification emails when users register"
    swimlane: backend
    
  DataArchiver:
    swimlane: backend
```

#### Automation Fields

- `description` (optional): What the automation does
- `swimlane` (required): Where it runs

## Slices (Flows)

Slices define the connections between entities:

```yaml
slices:
  - name: Registration Flow
    connections:
      - LoginScreen.RegisterLink -> RegistrationForm
      - RegistrationForm.RegistrationFields.Submit -> RegisterUser
      - RegisterUser -> UserRegistered
      - UserRegistered -> UserList
      - UserRegistered -> EmailVerificationSender
      - EmailVerificationSender -> SendVerificationEmail
```

### Connection Formats

1. Simple connection:
   ```yaml
   - EntityA -> EntityB
   ```

2. Component connection:
   ```yaml
   - View.Component -> Command
   ```

3. Action connection:
   ```yaml
   - View.Form.Submit -> Command
   ```

### Connection Rules

- Source and target must be defined entities
- Components must exist in the referenced view
- Actions must be defined for the referenced form

## Data Types

### Built-in Types

Event Modeler recognizes these standard types:

- `String`: Text values
- `Integer`: Whole numbers
- `Float`: Decimal numbers
- `Boolean`: True/false values
- `Timestamp`: Date/time values
- `Date`: Date without time
- `Time`: Time without date
- `UUID`: Unique identifiers

### Generic Types

- `List<T>`: Ordered collection
- `Set<T>`: Unique collection
- `Map<K,V>`: Key-value pairs
- `Option<T>`: Optional value

### Custom Types

Define domain-specific types:

```yaml
UserId: String
Email: String
Money: { amount: Decimal, currency: Currency }
```

### Type States

Types can have states using angle brackets:

```yaml
Email<Unverified>
Email<Verified>
Password<Plain>
Password<Hashed>
```

## Test Scenarios

Commands can include test scenarios using Given/When/Then format:

```yaml
tests:
  "Test Name":
    Given:
      - EventName:
          field: value
    When:
      - CommandName:
          field: value
    Then:
      - ExpectedEvent:
          field: value
```

### Test Elements

- `Given`: Initial state (existing events)
- `When`: Action being tested (command)
- `Then`: Expected outcome (events or errors)

### Test Value Placeholders

Use single letters as value placeholders:

```yaml
Given:
  - UserRegistered:
      user_id: A
      email: B
When:
  - UpdateEmail:
      user_id: A
      new_email: C
Then:
  - EmailUpdated:
      user_id: A
      old_email: B
      new_email: C
```

## Best Practices

### Naming Conventions

1. **Events**: Past tense, describe what happened
   - ✓ `UserRegistered`, `OrderPlaced`, `PaymentProcessed`
   - ✗ `RegisterUser`, `PlaceOrder`, `ProcessPayment`

2. **Commands**: Imperative mood, describe intent
   - ✓ `RegisterUser`, `PlaceOrder`, `ProcessPayment`
   - ✗ `UserRegistered`, `OrderPlaced`, `PaymentProcessed`

3. **Views**: Descriptive screen/component names
   - ✓ `RegistrationForm`, `OrderList`, `UserProfile`
   - ✗ `Register`, `Orders`, `User`

4. **Projections**: Describe the read model
   - ✓ `UserList`, `OrderSummary`, `AccountBalance`
   - ✗ `Users`, `Orders`, `Balance`

### Organization

1. **Group related entities** in the same swimlane
2. **Use meaningful swimlane names** that reflect boundaries
3. **Keep slices focused** on specific workflows
4. **Order entities** in logical flow sequence

### Data Modeling

1. **Use domain types** instead of primitives:
   - ✓ `user_id: UserId`
   - ✗ `user_id: String`

2. **Include type states** for validation:
   - ✓ `email: Email<Verified>`
   - ✗ `email: String`

3. **Mark generated fields** explicitly:
   ```yaml
   order_id:
     type: OrderId
     generated: true
   ```

## Common Patterns

### Event Sourcing Pattern

```yaml
events:
  AccountOpened:
    swimlane: events
    data:
      account_id:
        type: AccountId
        stream-id: true
      initial_balance: Money

commands:
  OpenAccount:
    swimlane: commands
    data:
      account_id:
        type: AccountId
        generated: true
      initial_deposit: Money

slices:
  - name: Account Creation
    connections:
      - OpenAccount -> AccountOpened
```

### CQRS Pattern

```yaml
# Command side
commands:
  UpdateUserProfile:
    swimlane: write_side
    data:
      user_id: UserId
      profile: ProfileData

# Query side  
queries:
  GetUserProfile:
    swimlane: read_side
    inputs:
      user_id: UserId
    outputs:
      profile: ProfileData

projections:
  UserProfileProjection:
    swimlane: read_side
    fields:
      user_id: UserId
      profile: ProfileData
```

### Saga Pattern

```yaml
automations:
  OrderSaga:
    description: "Orchestrates order fulfillment"
    swimlane: sagas

slices:
  - name: Order Fulfillment
    connections:
      - OrderPlaced -> OrderSaga
      - OrderSaga -> ReserveInventory
      - InventoryReserved -> OrderSaga
      - OrderSaga -> ProcessPayment
      - PaymentProcessed -> OrderSaga
      - OrderSaga -> ShipOrder
```

## Error Messages

Common validation errors and their meanings:

### Empty Field Error
```
Field 'description' cannot be empty
```
**Solution**: Provide a non-empty value for the field

### Unknown Swimlane Error
```
Unknown swimlane reference: frontend
```
**Solution**: Ensure the swimlane is defined in the `swimlanes` section

### Invalid Connection Error
```
Invalid connection syntax: InvalidEntity -> Target
```
**Solution**: Check that both entities in the connection exist

### Empty Collection Error
```
Collection 'components' must not be empty
```
**Solution**: Add at least one item to the collection

### YAML Parse Error
```
YAML error at line 10, column 5: did not find expected key
```
**Solution**: Check YAML indentation and structure at the specified location

## Tips for Large Models

1. **Use clear section separators**:
   ```yaml
   # ====================
   # User Management
   # ====================
   
   events:
     UserCreated:
       # ...
   ```

2. **Group related entities**:
   ```yaml
   # Order Events
   events:
     OrderPlaced: # ...
     OrderShipped: # ...
     OrderDelivered: # ...
     
   # Payment Events  
   events:
     PaymentInitiated: # ...
     PaymentProcessed: # ...
   ```

3. **Document complex flows**:
   ```yaml
   slices:
     # This slice handles the complete order lifecycle
     # from placement through delivery
     OrderLifecycle:
       - PlaceOrder -> OrderPlaced
       # ... more connections
   ```

## Migration from Old Format

If migrating from the old simple text format:

1. Convert `Title:` to `workflow:`
2. Convert swimlane definitions to YAML list format
3. Convert entity definitions to appropriate sections
4. Convert simple arrows to slice definitions
5. Add data schemas and descriptions

### Example Migration

Old format:
```
Title: Order System

Swimlane: Customer
- Command: PlaceOrder

Swimlane: Orders  
- Event: OrderPlaced

PlaceOrder -> OrderPlaced
```

New format:
```yaml
workflow: Order System

swimlanes:
  - customer: "Customer"
  - orders: "Orders"

commands:
  PlaceOrder:
    swimlane: customer

events:
  OrderPlaced:
    swimlane: orders

slices:
  - name: Order Flow
    connections:
      - PlaceOrder -> OrderPlaced
```

## Further Reading

- [Event Modeling Methodology](https://eventmodeling.org)
- [Example Event Models](../examples/)
- [Architecture Decision Records](adr/)