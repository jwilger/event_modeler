# Performance as Non-Priority

- Status: accepted
- Deciders: John Wilger, Claude
- Date: 2025-06-17
- Tags: performance, architecture, priorities

Technical Story: Code review suggested performance optimizations, but this is a development tool, not a high-frequency system

## Context and Problem Statement

Should we prioritize performance optimizations (like using SmallVec for small collections) in a development tool that processes relatively small event model files?

## Decision Drivers

- Event Modeler is a development-time tool, not a runtime service
- Event model files are typically small (dozens to hundreds of entities)
- Type safety and correctness are our primary goals
- Developer time is limited

## Considered Options

1. Optimize aggressively - Use SmallVec, minimize allocations, benchmark everything
2. Optimize only when needed - Profile first, optimize bottlenecks only
3. Ignore performance - Focus entirely on correctness and usability

## Decision Outcome

Chosen option: "Optimize only when needed", because premature optimization would distract from our core goals of type safety and correctness.

### Positive Consequences

- Can focus on implementing features correctly first
- Simpler code without micro-optimizations
- Can always optimize later if users report issues
- More time for type-driven design

### Negative Consequences

- Might need refactoring if performance becomes an issue
- Could give impression of not caring about performance

## Pros and Cons of the Options

### Optimize aggressively

- Good, because it shows attention to detail
- Bad, because it's premature without real usage data
- Bad, because it complicates the code
- Bad, because event models are inherently small

### Optimize only when needed

- Good, because it's pragmatic
- Good, because it keeps code simple
- Good, because we can measure real bottlenecks
- Bad, because retrofitting can be harder

### Ignore performance

- Good, because maximum simplicity
- Bad, because even dev tools should be responsive
- Bad, because it sets bad precedent

## Links

- Relates to type-safety-first architecture approach