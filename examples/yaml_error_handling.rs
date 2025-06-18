// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Example demonstrating YAML error handling with line/column information.

fn main() {
    println!("=== YAML Error Handling with Line/Column Information ===\n");

    println!(
        "This example demonstrates how to handle YAML parsing errors with location information."
    );
    println!("The event_modeler YAML parser automatically extracts line and column numbers");
    println!("from serde_yaml errors when available.\n");

    println!("To use this functionality in your code:\n");

    println!("1. Import the necessary types:");
    println!(
        "   use crate::infrastructure::parsing::yaml_parser::{{parse_yaml, YamlParseError}};\n"
    );

    println!("2. Parse YAML and handle errors:");
    println!("   match parse_yaml(yaml_content) {{");
    println!("       Ok(model) => {{");
    println!("           // Handle successful parse");
    println!("       }}");
    println!("       Err(YamlParseError::ParseError {{ line, column, message }}) => {{");
    println!("           // Handle error with location info");
    println!(
        "           eprintln!(\"Error at line {{}}, column {{}}: {{}}\", line, column, message);"
    );
    println!("       }}");
    println!("       Err(e) => {{");
    println!("           // Handle other errors (no location info available)");
    println!("           eprintln!(\"Error: {{}}\", e);");
    println!("       }}");
    println!("   }}\n");

    println!("Benefits:");
    println!("- Precise error location (line and column numbers are 1-indexed)");
    println!("- Clear error messages from serde_yaml");
    println!("- Automatic extraction of location data when available");
    println!("- Fallback to regular error handling when location is unavailable\n");

    println!("Common YAML errors that will include location information:");
    println!("- Syntax errors (unclosed quotes, missing colons, etc.)");
    println!("- Invalid indentation");
    println!("- Duplicate keys");
    println!("- Type mismatches");
    println!("- Invalid YAML structure\n");

    println!("Example error output:");
    println!("  YAML error at line 4, column 15: did not find expected key");
}
