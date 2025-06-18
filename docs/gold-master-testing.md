# Gold Master Testing Guide

Event Modeler uses gold master testing (snapshot testing) to verify the visual output of generated diagrams. This approach allows us to:

1. Detect any changes to the generated output
2. Review changes visually before accepting them
3. Maintain a "gold master" reference that represents the expected output

## Tools Used

- **insta**: Rust snapshot testing library for comparing SVG text content
- **cargo-insta**: CLI tool for reviewing and approving snapshot changes
- **visual_compare.sh**: Custom script for side-by-side visual comparison

## Workflow

### Running Tests

```bash
# Run all tests including acceptance tests
cargo test

# Run only the acceptance tests
cargo test yaml_acceptance
```

### Reviewing Changes

When the generated output differs from the snapshot:

1. **Review text differences**:
   ```bash
   cargo insta review
   ```
   - Press `a` to accept the new version
   - Press `r` to reject and keep the old version
   - Press `s` to skip

2. **Visual comparison**:
   ```bash
   # Compare generated SVG with the reference
   ./scripts/visual_compare.sh target/test-output/yaml_acceptance.svg tests/fixtures/acceptance/example.jpg
   
   # Compare two SVG files side-by-side
   ./scripts/visual_compare.sh old.svg new.svg
   ```

### Accepting New Gold Masters

When you're satisfied with the generated output:

1. Run `cargo insta review`
2. Press `a` to accept the new snapshot
3. Commit the updated snapshot files (`.snap` files)

## Test Structure

### Primary Acceptance Test

`tests/yaml_acceptance.rs::test_yaml_format_acceptance`
- Uses `example.eventmodel` as input
- Compares generated SVG against snapshot
- Reference image: `tests/fixtures/acceptance/example.jpg`

### Key Points

- The test doesn't require pixel-perfect matching
- Focus is on correct structure, layout, and visual hierarchy
- Small rendering differences (fonts, exact spacing) are acceptable
- The snapshot captures the SVG text, allowing detailed diff review

## Adding New Gold Master Tests

```rust
#[test]
fn test_new_feature() {
    // Generate output
    let svg_content = generate_svg();
    
    // Assert against snapshot
    insta::assert_snapshot!("feature_name", svg_content);
}
```

## Tips

- Always visually review changes before accepting
- Document why you're accepting changes in commit messages
- Keep reference images (`.jpg`, `.png`) for visual comparison
- Use meaningful snapshot names for easy identification