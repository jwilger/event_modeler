//! Acceptance tests for the YAML-based event modeling format.
//!
//! This test file drives the implementation of the new YAML-based format
//! as specified in example.eventmodel and example.jpg.
//!
//! Uses insta for gold master/snapshot testing. To review and approve changes:
//! 1. Run tests: `cargo test`
//! 2. Review snapshots: `cargo insta review`
//! 3. For visual comparison: `./scripts/visual_compare.sh <generated.svg> <expected.svg>`

use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
#[ignore = "YAML format not yet implemented - this test drives the new implementation"]
fn test_yaml_format_acceptance() {
    // This test represents the primary acceptance criteria for the new implementation
    let input_path = Path::new("tests/fixtures/acceptance/example.eventmodel");
    let output_path = Path::new("target/test-output/yaml_acceptance.svg");

    // Ensure output directory exists
    fs::create_dir_all(output_path.parent().unwrap()).unwrap();

    // Run event_modeler with the YAML input
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute event_modeler");

    // The command should succeed
    assert!(
        output.status.success(),
        "event_modeler failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // The output file should exist
    assert!(
        output_path.exists(),
        "Expected output file {} was not created",
        output_path.display()
    );

    // Read the generated SVG
    let svg_content = fs::read_to_string(output_path).expect("Failed to read generated SVG");

    // Use insta for snapshot testing the SVG content
    // This allows us to review changes and approve new versions
    insta::assert_snapshot!("yaml_acceptance_svg", &svg_content, @"");

    // Also verify key structural elements are present
    verify_yaml_format_elements(&svg_content);

    // Note: For visual comparison, run:
    // ./scripts/visual_compare.sh target/test-output/yaml_acceptance.svg tests/fixtures/acceptance/example.jpg
}

fn verify_yaml_format_elements(svg: &str) {
    // Basic structural verification - detailed comparison is done via snapshot
    // These tests ensure critical elements exist regardless of exact formatting

    // Verify it's valid SVG
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    assert!(svg.contains("</svg>"), "Output should have closing SVG tag");

    // Verify swimlanes exist (exact text may vary based on rendering)
    assert!(
        svg.to_lowercase().contains("ux") || svg.contains("User Interface"),
        "Should contain UX/UI swimlane"
    );
    assert!(
        svg.to_lowercase().contains("command") || svg.to_lowercase().contains("projection"),
        "Should contain Commands/Projections swimlane"
    );
    assert!(
        svg.to_lowercase().contains("event") || svg.contains("Stream"),
        "Should contain Event Stream swimlane"
    );

    // The detailed comparison of all entities, styling, layout, etc.
    // is handled by the snapshot test above
}

#[test]
#[ignore = "YAML format not yet implemented"]
fn test_yaml_parsing_errors_are_helpful() {
    // Test that YAML parsing errors provide line/column information
    let invalid_yaml = r#"
workflow: Test
swimlanes:
  - invalid yaml syntax here
    should fail with clear error
"#;

    let temp_file = Path::new("target/test-output/invalid.eventmodel");
    fs::create_dir_all(temp_file.parent().unwrap()).unwrap();
    fs::write(temp_file, invalid_yaml).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", temp_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute event_modeler");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should include line/column information
    assert!(stderr.contains("line") || stderr.contains("Line"));
    assert!(stderr.contains("column") || stderr.contains("Column") || stderr.contains("col"));
}

#[test]
#[ignore = "YAML format not yet implemented"]
fn test_all_entity_types_are_supported() {
    // Test that all entity types from the YAML format are properly parsed and rendered
    let yaml_with_all_types = r#"
workflow: All Entity Types Test

swimlanes:
  - ux: "User Interface"
  - commands: "Commands"
  - events: "Events"

events:
  TestEvent:
    swimlane: events
    data:
      id: String

commands:
  TestCommand:
    swimlane: commands
    data:
      id: String
    tests:
      "Success Case":
        Given:
        When:
          - TestCommand:
              id: "123"
        Then:
          - TestEvent:
              id: "123"

views:
  TestView:
    swimlane: ux
    components:
      - TestButton: Button

projections:
  TestProjection:
    swimlane: commands
    fields:
      id: String

queries:
  TestQuery:
    swimlane: commands
    inputs:
      id: String
    outputs:
      one_of:
        found:
          data: String
        not_found: Error

automations:
  TestAutomation:
    swimlane: ux

slices:
  TestSlice:
    - TestView.TestButton -> TestCommand
    - TestCommand -> TestEvent
    - TestEvent -> TestProjection
"#;

    let temp_file = Path::new("target/test-output/all_types.eventmodel");
    fs::create_dir_all(temp_file.parent().unwrap()).unwrap();
    fs::write(temp_file, yaml_with_all_types).unwrap();

    let output_path = Path::new("target/test-output/all_types.svg");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            temp_file.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute event_modeler");

    assert!(output.status.success());

    let svg_content = fs::read_to_string(output_path).unwrap();

    // Use insta snapshot for the comprehensive test
    insta::assert_snapshot!("all_entity_types_svg", &svg_content, @"");

    // Basic verification that all entity types are present
    assert!(svg_content.contains("TestEvent"));
    assert!(svg_content.contains("TestCommand"));
    assert!(svg_content.contains("TestView"));
    assert!(svg_content.contains("TestProjection"));
    assert!(svg_content.contains("TestQuery"));
    assert!(svg_content.contains("TestAutomation"));
}

/// Helper test for generating initial gold master from example.jpg
/// This test can be run manually to help create the initial snapshot
#[test]
#[ignore = "Manual test for setting up gold master"]
fn generate_gold_master_reference() {
    println!("Gold master reference image is at:");
    println!("  tests/fixtures/acceptance/example.jpg");
    println!();
    println!("When the YAML implementation generates output, compare with:");
    println!("  ./scripts/visual_compare.sh <generated.svg> tests/fixtures/acceptance/example.jpg");
    println!();
    println!("To approve a generated SVG as the new gold master:");
    println!("  1. Run: cargo test");
    println!("  2. Run: cargo insta review");
    println!("  3. Press 'a' to accept the new snapshot");
}
