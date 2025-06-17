# Typestate Parser Design for Event Model Format

- Status: accepted
- Deciders: John Wilger, Claude
- Date: 2025-06-17
- Tags: parsing, type-safety, architecture

Technical Story: Need to encode the .eventmodel file format in the type system to maintain parse-don't-validate principle

## Context and Problem Statement

The Event Modeler needs to parse `.eventmodel` files with a specific format. How can we leverage the type system to make invalid parse states unrepresentable while maintaining good error messages and a natural parsing flow?

## Decision Drivers

- Must maintain parse-don't-validate principle
- Need clear error messages for invalid input
- Should enforce correct parsing order at compile time
- Must be understandable to contributors
- Should integrate with existing strong domain types

## Considered Options

1. Type-Level Grammar - Encode full grammar as types with nested structures
2. Typestate Builder Pattern - Use phantom types to track parser state transitions
3. Format Specification as Types - Use const generics to define format at type level

## Decision Outcome

Chosen option: "Typestate Builder Pattern", because it provides the best balance of type safety, developer experience, and pragmatism.

### Positive Consequences

- Compile-time enforcement of parsing stages (header before body)
- Natural, incremental parsing flow
- Excellent error messages at each stage
- Well-understood pattern in Rust community
- Combines well with existing domain types

### Negative Consequences

- Doesn't encode detailed format rules in types
- Still requires some runtime validation
- Multiple type parameters can be verbose

## Pros and Cons of the Options

### Type-Level Grammar

- Good, because it mirrors formal grammar specifications
- Good, because entire structure is type-checked
- Bad, because complex type signatures reduce readability
- Bad, because error messages become cryptic
- Bad, because may hit type system limitations

### Typestate Builder Pattern

- Good, because it matches natural parsing flow
- Good, because it provides clear stage transitions
- Good, because error messages are excellent
- Good, because it's a familiar Rust pattern
- Bad, because it only enforces ordering, not detailed format
- Bad, because some runtime validation still needed

### Format Specification as Types

- Good, because it could generate parser and formatter
- Good, because keywords are compile-time constants
- Bad, because it's experimental and unusual
- Bad, because heavy const generic usage
- Bad, because too clever for maintainability

## Implementation Example

```rust
// Parser moves through states: Empty -> HasHeader -> HasBody -> Complete
pub struct EventModelParser<State> {
    content: ParsedContent,
    _state: PhantomData<State>,
}

// Can only parse header when empty
impl EventModelParser<Empty> {
    pub fn parse_header(self, input: &str) -> Result<EventModelParser<HasHeader>, ParseError>
}

// Can only parse body after header
impl EventModelParser<HasHeader> {
    pub fn parse_body(self, input: &str) -> Result<EventModelParser<HasBody>, ParseError>
}

// Can only build after all sections parsed
impl EventModelParser<Complete> {
    pub fn build(self) -> EventModelDiagram
}
```

## Links

- Relates to parse-don't-validate architecture pattern
- Complements type-driven testing strategy in 20250617-type-driven-testing.md