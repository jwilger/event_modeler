//! Test harness for incremental diagram development.
//!
//! This binary loads an event model and renders it incrementally,
//! allowing visual validation at each step.

use event_modeler::diagram::EventModelDiagram;
use event_modeler::infrastructure::parsing::{yaml_converter, yaml_parser};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.eventmodel> [output_dir]", args[0]);
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_dir = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(".")
    };

    // Load and parse the event model
    let input_content = fs::read_to_string(&input_path)?;
    let yaml_model = yaml_parser::parse_yaml(&input_content)?;
    let domain_model = yaml_converter::convert_yaml_to_domain(yaml_model)?;

    // Step 1: Create diagram with just workflow title
    println!("\n=== Step 1: Workflow Title Only ===");
    let workflow_title = domain_model.workflow.clone().into_inner();
    let mut diagram = EventModelDiagram::new(workflow_title);

    // Render and show
    render_and_show(&diagram, &output_dir, "step_01_title.svg")?;

    println!("\nPress Enter to continue to next step...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Step 2: Add all swimlanes
    println!("\n=== Step 2: All Swimlanes ===");

    // Add swimlanes from the domain model
    for swimlane in domain_model.swimlanes.iter() {
        let id = swimlane.id.clone();
        let label = swimlane.name.clone();

        diagram = diagram.with_swimlane(id, label);
    }

    // Render and show
    render_and_show(&diagram, &output_dir, "step_02_swimlanes.svg")?;

    println!("\nPress Enter to continue to next step...");
    std::io::stdin().read_line(&mut input)?;

    // Future steps will be added here incrementally

    Ok(())
}

fn render_and_show(
    diagram: &EventModelDiagram,
    output_dir: &Path,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate SVG
    let svg_content = diagram.to_svg();

    // Write SVG file
    let svg_path = output_dir.join(filename);
    let mut svg_file = fs::File::create(&svg_path)?;
    svg_file.write_all(svg_content.as_bytes())?;
    println!("Generated SVG: {}", svg_path.display());

    // Convert to PNG with white background
    let png_path = svg_path.with_extension("png");
    let status = Command::new("convert")
        .arg("-background")
        .arg("white")
        .arg("-alpha")
        .arg("remove")
        .arg("-alpha")
        .arg("off")
        .arg(&svg_path)
        .arg(&png_path)
        .status()?;

    if !status.success() {
        eprintln!("Failed to convert SVG to PNG. Make sure ImageMagick is installed.");
        return Ok(());
    }

    println!("Generated PNG: {}", png_path.display());

    // Open PNG for viewing
    Command::new("xdg-open").arg(&png_path).spawn()?;

    Ok(())
}
