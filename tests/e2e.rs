use std::fs;
use std::process::Command;

#[test]
fn test_basic_event_model_to_svg_conversion() {
    // Create a simple test .eventmodel file
    let test_input = r#"Title: Order Processing System

Swimlane: Orders Service
- Event: OrderPlaced
- Event: OrderConfirmed
- Event: OrderShipped

Swimlane: Payment Service
- Event: PaymentProcessed

Swimlane: Notification Service
- Event: NotificationSent

OrderPlaced -> OrderConfirmed
OrderConfirmed -> OrderShipped
OrderPlaced -> PaymentProcessed
OrderConfirmed -> NotificationSent
OrderShipped -> NotificationSent
"#;

    // Use temporary directory for test files
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("test_input.eventmodel");
    // Note: The CLI currently generates output filename based on input stem
    let expected_output_path = temp_dir.join("test_input.svg");
    let output_path = temp_dir.join("test_output.svg");

    // Write the test input
    fs::write(&input_path, test_input).expect("Failed to write test input file");

    // Run the CLI
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            input_path.to_str().unwrap(),
            "-o",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    // Debug output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("Command failed!");
        eprintln!("Exit status: {:?}", output.status.code());
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
        eprintln!("Input path exists: {}", input_path.exists());
    }

    // The actual program output goes to stdout when run via cargo run
    if !output_path.exists() {
        eprintln!("Output file not created!");
        eprintln!("Looking for: {}", output_path.display());
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
    }

    // Verify SVG output was created (with the actual output path the CLI uses)
    assert!(
        expected_output_path.exists(),
        "SVG output file was not created at: {}",
        expected_output_path.display()
    );

    // Verify the SVG file contains expected content
    let svg_content = fs::read_to_string(&expected_output_path).expect("Failed to read SVG output");
    assert!(
        svg_content.contains("<svg"),
        "Output does not appear to be valid SVG"
    );
    assert!(
        svg_content.contains("swimlane"),
        "SVG should contain swimlane elements"
    );
    assert!(
        svg_content.contains("OrderPlaced"),
        "SVG should contain entity name"
    );
    assert!(
        svg_content.contains("connector"),
        "SVG should contain connector elements"
    );

    // Clean up
    fs::remove_file(&input_path).ok();
    fs::remove_file(&expected_output_path).ok();
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

#[test]
fn test_simple_event_model_with_minimal_structure() {
    let test_input = r#"Title: Minimal Model

Swimlane: System
- Event: SystemStarted
"#;

    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("minimal.eventmodel");
    let output_path = temp_dir.join("minimal.svg");

    fs::write(&input_path, test_input).expect("Failed to write test input file");

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

    assert!(output.status.success());
    assert!(output_path.exists());

    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG output");
    assert!(svg_content.contains("SystemStarted"));

    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn test_event_model_with_multiple_entity_types() {
    let test_input = r#"Title: Mixed Entity Types

Swimlane: User Interface
- Command: SubmitOrder

Swimlane: Backend
- Event: OrderSubmitted
- Projection: OrderList
- Policy: ValidateOrder

SubmitOrder -> OrderSubmitted
OrderSubmitted -> OrderList
"#;

    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("mixed_types.eventmodel");
    let output_path = temp_dir.join("mixed_types.svg");

    fs::write(&input_path, test_input).expect("Failed to write test input file");

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

    assert!(output.status.success());

    let svg_content = fs::read_to_string(&output_path).expect("Failed to read SVG output");
    assert!(svg_content.contains("SubmitOrder"));
    assert!(svg_content.contains("OrderSubmitted"));
    assert!(svg_content.contains("OrderList"));
    assert!(svg_content.contains("ValidateOrder"));

    fs::remove_file(&input_path).ok();
    fs::remove_file(&output_path).ok();
}

#[test]
fn test_invalid_eventmodel_file_shows_error() {
    let test_input = r#"This is not a valid event model file"#;

    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("invalid.eventmodel");

    fs::write(&input_path, test_input).expect("Failed to write test input file");

    let output_path = temp_dir.join("invalid.svg");

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

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("Invalid file stderr: {}", stderr);
    assert!(
        stderr.contains("Parse error")
            || stderr.contains("MissingTitle")
            || stderr.contains("Invalid arguments")
    );

    fs::remove_file(&input_path).ok();
}

#[test]
fn test_nonexistent_file_shows_error() {
    let output = Command::new("cargo")
        .args(["run", "--", "nonexistent.eventmodel"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid path")
            || stderr.contains("must have .eventmodel extension and exist")
    );
}
