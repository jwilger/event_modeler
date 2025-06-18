# ADR: Gold Master Testing Strategy

Date: 2025-06-18

## Status

Accepted

## Context

Event Modeler produces visual outputs (SVG diagrams) that need to match specific visual requirements. Testing visual outputs presents unique challenges:

1. **Visual correctness is subjective**: Small differences may or may not matter
2. **SVG comparison is complex**: Direct string comparison is brittle due to attribute ordering, whitespace, etc.
3. **Iterative development**: We need quick feedback during implementation
4. **Regression prevention**: Must ensure changes don't break existing functionality

Traditional unit testing approaches are insufficient for validating complex visual outputs. We need a testing strategy that:
- Captures the expected visual output
- Allows for controlled updates during development
- Provides clear feedback when outputs change
- Works well with version control

## Decision

We will adopt a Gold Master (also known as Approval or Snapshot) testing approach using the `insta` crate for Rust.

Key aspects of our approach:
1. **Reference outputs**: Store example.jpg as the gold master reference
2. **Snapshot testing**: Use insta to capture and compare SVG outputs
3. **Visual review**: Manually review snapshot changes during development
4. **Structured comparison**: Compare SVG structure, not raw strings
5. **CI integration**: Fail builds when snapshots don't match

## Consequences

### Positive

1. **Rapid iteration**: Developers can quickly see the impact of changes
2. **Visual validation**: Can visually compare outputs to requirements
3. **Regression detection**: Automatically catches unintended changes
4. **Documentation**: Snapshots serve as examples of expected output
5. **Refactoring confidence**: Can refactor safely knowing outputs are tested
6. **Review friendly**: Snapshot diffs in PRs show exactly what changed

### Negative

1. **Large diffs**: Snapshot files can create large PR diffs
2. **Merge conflicts**: Concurrent changes may conflict in snapshot files
3. **False positives**: Harmless changes (like attribute reordering) may fail tests
4. **Manual review**: Requires human judgment to approve changes

### Neutral

1. **Storage overhead**: Snapshot files add to repository size
2. **Tool dependency**: Adds insta as a development dependency
3. **Workflow change**: Developers must learn snapshot review workflow

## Implementation Strategy

### Initial Setup

1. Copy example.jpg to tests/fixtures/acceptance/
2. Create acceptance test that generates SVG from example.eventmodel
3. Use insta to capture the generated SVG
4. Manually verify output matches example.jpg structure

### Development Workflow

1. Make changes to implementation
2. Run tests with `cargo test`
3. If snapshots change, review with `cargo insta review`
4. Accept changes if they're intentional improvements
5. Reject changes if they're regressions
6. Commit updated snapshots with code changes

### CI Configuration

```yaml
- name: Run tests
  run: cargo test --workspace
  
- name: Verify snapshots
  run: cargo insta test --unreferenced=reject
```

## Example Test

```rust
#[test]
fn test_yaml_format_acceptance() {
    let yaml_content = std::fs::read_to_string("tests/fixtures/acceptance/example.eventmodel")
        .expect("Failed to read example.eventmodel");
    
    let model = parse_yaml(&yaml_content).expect("Failed to parse YAML");
    let svg = render_to_svg(&model).expect("Failed to render SVG");
    
    // This creates/updates a snapshot file
    insta::assert_snapshot!(svg);
}
```

## Best Practices

1. **Semantic snapshots**: Structure tests to capture meaningful units (e.g., one entity type per snapshot)
2. **Descriptive names**: Use clear snapshot names that indicate what's being tested
3. **Minimize noise**: Normalize outputs (sort attributes, consistent formatting) before snapshotting
4. **Review carefully**: Always review snapshot changes; don't blindly accept
5. **Document changes**: Explain why snapshots changed in commit messages

## Alternatives Considered

1. **Pixel comparison**: Too brittle, fails on minor rendering differences
2. **Manual testing**: Too slow, doesn't scale, prone to human error
3. **Property-based testing**: Good for logic but not for visual outputs
4. **Custom XML comparison**: Requires significant implementation effort

Gold master testing with insta provides the best balance of developer experience, reliability, and maintainability for our visual testing needs.