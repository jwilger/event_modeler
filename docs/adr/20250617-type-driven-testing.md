# Type-Driven Testing Strategy

- Status: accepted
- Deciders: John Wilger, Claude
- Date: 2025-06-17
- Tags: testing, type-safety, architecture

Technical Story: Code review revealed no tests in the codebase, leading to discussion about testing strategy with strong types

## Context and Problem Statement

Traditional testing wisdom suggests comprehensive test suites are essential for quality software. However, our codebase uses advanced type-driven design techniques (phantom types, typestate patterns, parse-don't-validate) that eliminate many categories of bugs at compile time. Should we follow traditional testing practices or leverage our type system as the primary correctness mechanism?

## Decision Drivers

- The compiler can enforce invariants that would traditionally require runtime tests
- Tests for type-checked code often become tautological (testing the compiler)
- We still need to verify behavior that can't be encoded in types (parsing, rendering)
- Future contributors need to understand our approach

## Considered Options

1. Traditional comprehensive test suite - Unit tests for all functions, high coverage targets
2. Type-driven approach with minimal tests - Rely on types, test only I/O boundaries
3. Property-based testing - Use types plus property tests for invariants

## Decision Outcome

Chosen option: "Type-driven approach with minimal tests", because our type system already provides stronger guarantees than most test suites could achieve.

### Positive Consequences

- Reduced maintenance burden (no need to update tests when types already guarantee correctness)
- Faster development cycles (no redundant test writing)
- Compiler-enforced correctness is more reliable than runtime tests
- Forces continued investment in type-driven design

### Negative Consequences

- Departure from conventional practices may surprise contributors
- Requires high discipline in type design
- Some behaviors still need testing (parsing, rendering output)

## Pros and Cons of the Options

### Traditional comprehensive test suite

- Good, because it's well-understood by all developers
- Good, because it provides a safety net for refactoring
- Bad, because many tests would be testing the compiler
- Bad, because it creates maintenance overhead for little benefit

### Type-driven approach with minimal tests

- Good, because it leverages compile-time guarantees
- Good, because it reduces redundant work
- Good, because it forces better type design
- Bad, because it's unconventional
- Bad, because some developers may feel unsafe without tests

### Property-based testing

- Good, because it complements type safety
- Good, because it can find edge cases
- Bad, because it's complex to implement well
- Bad, because many properties are already encoded in types

## Links

- Supersedes traditional TDD approach mentioned in CLAUDE.md
- Related to parse-don't-validate architecture pattern