use std::fs;
use std::process::Command;

#[test]
#[ignore = "Render command not yet implemented"]
fn test_basic_event_model_to_svg_conversion() {
    // Create a simple test .eventmodel file
    let test_input = r#"# Order Processing System

[Orders Service]
Order Placed -> Order Confirmed
Order Confirmed -> Order Shipped

[Payment Service]
Order Placed -> Payment Processed

[Notification Service]
Order Confirmed -> Notification Sent
Order Shipped -> Notification Sent
"#;

    // Use temporary directory for test files
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_input.eventmodel");
    let output_path = temp_dir.join("test_output.svg");

    // Clean up any existing files
    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&output_path);

    // Write the test input
    fs::write(&input_path, test_input).expect("Failed to write test input file");

    // Run the CLI
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Check that the command succeeded
    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify SVG output was created
    assert!(output_path.exists(), "SVG output file was not created");

    // Verify the SVG file contains expected content
    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG output");
    assert!(
        svg_content.contains("<svg"),
        "Output does not appear to be valid SVG"
    );
    assert!(
        svg_content.contains("Orders Service"),
        "SVG should contain swimlane label"
    );
    assert!(
        svg_content.contains("Order Placed"),
        "SVG should contain entity name"
    );

    // Clean up
    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn test_cli_shows_usage_without_args() {
    let output = Command::new("cargo")
        .args(["run", "--"])
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "CLI should fail without arguments"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: event_modeler"),
        "Should show usage message"
    );
}
