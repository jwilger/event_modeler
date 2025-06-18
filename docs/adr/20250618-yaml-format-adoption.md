# ADR: Adopting YAML Format for Event Models

Date: 2025-06-18

## Status

Accepted

## Context

The initial implementation of Event Modeler used a simple text-based format for defining event models. This format was designed to be minimal and easy to parse, but it had significant limitations:

1. **Limited expressiveness**: Could not represent data schemas, field types, or test scenarios
2. **No structured data**: Everything was plain text without type annotations
3. **Simplistic connections**: Only basic "A -> B" style connections were supported
4. **No UI component modeling**: Could not represent view hierarchies or form structures
5. **No test scenario support**: Could not express Given/When/Then test cases

Upon reviewing the actual requirements (example.eventmodel and example.jpg), we discovered that the system needs to support:
- Rich data schemas with type annotations (e.g., `UserEmailAddress<Verified>`)
- Test scenarios using Given/When/Then format
- UI component hierarchies with forms, inputs, and actions
- Complex slice-based flow definitions
- Professional visual output with entity-specific styling

## Decision

We will adopt a YAML-based format for event model definitions, completely replacing the simple text format.

The YAML format will:
1. Use semantic indentation for structure
2. Support nested data structures naturally
3. Allow type annotations within strings
4. Enable test scenario definitions
5. Support schema versioning

## Consequences

### Positive

1. **Rich expressiveness**: Can represent all required concepts including data schemas, UI components, and test scenarios
2. **Industry standard**: YAML is well-understood and has excellent tooling support
3. **Type safety**: Can validate structure at parse time using serde
4. **Extensibility**: Easy to add new fields and concepts without breaking existing files
5. **Human readable**: Maintains readability while adding structure
6. **Better error messages**: YAML parsers provide line/column numbers for errors

### Negative

1. **Breaking change**: Existing event model files will not work with the new parser
2. **Increased complexity**: YAML parsing is more complex than simple text parsing
3. **Dependency**: Adds serde_yaml as a dependency
4. **Indentation sensitivity**: Users must be careful with whitespace

### Neutral

1. **File size**: YAML files will be larger but more expressive
2. **Learning curve**: Users familiar with YAML will adapt quickly; others may need time

## Implementation Notes

Since we're pre-1.0 (version 0.2.0), we can make this breaking change without maintaining backward compatibility. The new format will be released as version 0.3.0.

Key implementation decisions:
- Schema version will match application version
- No migration tools needed for pre-1.0 versions
- Clean break from old format
- All examples will be updated to use YAML

## Example

Old format:
```
Event Modeling: User Registration

= User Interface
= Commands  
= Events

User Interface:
  RegistrationForm [wireframe]
  
Commands:
  RegisterUser [command]
  
Events:
  UserRegistered [event]
  
RegisterUser -> UserRegistered
```

New YAML format:
```yaml
workflow: User Registration

swimlanes:
  - ui: "User Interface"
  - commands: "Commands"
  - events: "Events"

views:
  RegistrationForm:
    swimlane: ui
    components:
      - RegistrationForm:
          type: Form
          fields:
            email: TextInput
            password: PasswordInput
          actions:
            - Submit

commands:
  RegisterUser:
    swimlane: commands
    data:
      email: EmailAddress
      password: Password<Hashed>
    tests:
      "Success Case":
        Given: []
        When:
          - RegisterUser:
              email: test@example.com
              password: hashedpassword
        Then:
          - UserRegistered:
              email: test@example.com

events:
  UserRegistered:
    swimlane: events
    data:
      email: EmailAddress
      timestamp: DateTime

slices:
  Registration:
    - RegistrationForm.Submit -> RegisterUser
    - RegisterUser -> UserRegistered
```

The YAML format is significantly more expressive while remaining readable and maintainable.