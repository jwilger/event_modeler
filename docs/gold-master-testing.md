# Gold Master Testing Guide

Event Modeler uses gold master testing (snapshot testing) to verify the visual output of generated diagrams. This approach allows us to:

1. Detect any changes to the generated output
2. Review changes visually before accepting them
3. Maintain a "gold master" reference that represents the expected output

## Tools Used

- **insta**: Rust snapshot testing library for comparing SVG text content
- **cargo-insta**: CLI tool for reviewing and approving snapshot changes
- **visual_compare.sh**: Basic script for side-by-side visual comparison
- **gold_master_compare.sh**: Interactive tool for choosing between generated and gold master versions
- **test_gold_master.sh**: Workflow automation script for running tests and updating gold masters

## Workflow

### Running Tests

```bash
# Run all tests including acceptance tests
cargo test

# Run only the acceptance tests
cargo test yaml_acceptance
```

### Reviewing Changes

When the generated output differs from the expected output, you have several options:

#### Quick Visual Gold Master Update (Recommended)

Use the interactive gold master comparison tool:

```bash
# Run tests and interactively update gold masters
./scripts/test_gold_master.sh

# Or run a specific test
./scripts/test_gold_master.sh yaml_acceptance
```

This will:
1. Run the acceptance test(s)
2. Open a side-by-side comparison in your browser
3. Let you choose which version to keep as the gold master

#### Manual Comparison

For manual comparison of specific files:

```bash
# Interactive comparison with choice
./scripts/gold_master_compare.sh generated.svg goldmaster.svg

# Simple side-by-side viewing
./scripts/visual_compare.sh generated.svg goldmaster.svg
```

#### Text-Based Snapshot Review

For reviewing the SVG text differences:

```bash
cargo insta review
```
- Press `a` to accept the new version
- Press `r` to reject and keep the old version
- Press `s` to skip

### Accepting New Gold Masters

When you're satisfied with the generated output:

1. Use `./scripts/test_gold_master.sh` to interactively choose the new gold master
2. OR use `./scripts/gold_master_compare.sh` to update specific files
3. Run `cargo insta review` to update text snapshots
4. Commit both the gold master SVG files and `.snap` files

## Test Structure

### Primary Acceptance Test

`tests/yaml_acceptance.rs::test_yaml_format_acceptance`
- Uses `example.eventmodel` as input
- Compares generated SVG against snapshot
- Reference image: `tests/fixtures/acceptance/example.png`

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