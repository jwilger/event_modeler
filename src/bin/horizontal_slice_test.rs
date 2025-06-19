//! Temporary test program for incrementally implementing horizontal slice architecture.
//!
//! This binary is used to test Step 1: Swimlanes Only rendering.

use event_modeler::diagram::svg::{
    DecimalPrecision, EmbedFonts, OptimizationLevel, SvgRenderConfig, SvgRenderer,
};
use event_modeler::diagram::theme::{GithubLight, ThemedRenderer};
use event_modeler::infrastructure::types::PositiveInt;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <output.svg>", args[0]);
        std::process::exit(1);
    }

    let output_path = PathBuf::from(&args[1]);

    // Step 1: Render only horizontal swimlanes
    let svg_content = render_swimlanes_only()?;

    // Write to file
    let mut file = fs::File::create(&output_path)?;
    file.write_all(svg_content.as_bytes())?;

    println!("Generated SVG: {}", output_path.display());
    Ok(())
}

fn render_swimlanes_only() -> Result<String, Box<dyn std::error::Error>> {
    // Create SVG renderer
    let svg_config = SvgRenderConfig {
        precision: DecimalPrecision::new(PositiveInt::parse(2).unwrap()),
        optimize: OptimizationLevel::Basic,
        embed_fonts: EmbedFonts::new(false),
    };

    let theme = ThemedRenderer::<GithubLight>::github_light()
        .theme()
        .clone();
    let _renderer = SvgRenderer::new(svg_config, theme);

    // Create a simple SVG showing only horizontal swimlanes
    // Based on the gold master, we need 3 horizontal bands:
    // 1. "UX, Automations" (top)
    // 2. "Commands, Projections, Queries" (middle)
    // 3. "User Account Event Stream" (bottom)

    let canvas_width = 1200;
    let canvas_height = 400;
    let swimlane_height = 120;
    let padding = 20;

    let mut svg_content = String::new();
    svg_content.push_str(&format!(
        "<svg width=\"{}\" height=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        canvas_width, canvas_height
    ));

    // Add background
    svg_content.push_str(&format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"none\"/>",
        canvas_width, canvas_height
    ));

    // Swimlane data
    let swimlanes = [
        ("UX, Automations", 0),
        ("Commands, Projections, Queries", 1),
        ("User Account Event Stream", 2),
    ];

    // Draw horizontal swimlanes
    for (name, index) in &swimlanes {
        let y = padding + (index * swimlane_height);
        let lane_y = y as f32;

        // Draw swimlane background
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#f8f9fa\" stroke=\"#e1e4e8\" stroke-width=\"1\"/>",
            padding,
            lane_y,
            canvas_width - 2 * padding,
            swimlane_height - 5 // Small gap between swimlanes
        ));

        // Add swimlane label (rotated on the left side like in the gold master)
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" transform=\"rotate(-90, {}, {})\" font-family=\"Arial, sans-serif\" font-size=\"12\" fill=\"#586069\">{}</text>",
            padding / 2,
            lane_y + (swimlane_height as f32 / 2.0),
            padding / 2,
            lane_y + (swimlane_height as f32 / 2.0),
            name
        ));
    }

    svg_content.push_str("</svg>");

    Ok(svg_content)
}
