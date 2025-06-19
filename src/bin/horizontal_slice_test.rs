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

    // Step 2: Render horizontal swimlanes + slice boundaries
    let svg_content = render_swimlanes_and_slices()?;

    // Write to file
    let mut file = fs::File::create(&output_path)?;
    file.write_all(svg_content.as_bytes())?;

    println!("Generated SVG: {}", output_path.display());
    Ok(())
}

fn render_swimlanes_and_slices() -> Result<String, Box<dyn std::error::Error>> {
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

    // Step 2: Render horizontal swimlanes + slice boundaries
    // Based on the gold master, we need:
    // - 3 horizontal swimlanes: UX, Commands, Events
    // - 3 vertical slices: CreateAccount, VerifyEmailAddress, (other)

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

    // Step 2: Add slice boundaries (vertical dividers)
    // Based on the example.eventmodel, we have slices: CreateAccount, VerifyEmailAddress
    let slice_names = [
        "Create Account",
        "Send Email Verification",
        "Verify Email Address",
    ];
    let slice_width = (canvas_width - 2 * padding) / slice_names.len();

    for (i, slice_name) in slice_names.iter().enumerate() {
        let slice_x = padding + (i * slice_width);

        // Draw vertical slice boundary line (except for the first one)
        if i > 0 {
            svg_content.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#d1d5da\" stroke-width=\"2\"/>",
                slice_x,
                padding,
                slice_x,
                canvas_height - padding
            ));
        }

        // Add slice header at the top
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" font-weight=\"bold\" fill=\"#24292e\">{}</text>",
            slice_x + (slice_width / 2),
            padding / 2 + 5, // Position above the swimlanes
            slice_name
        ));
    }

    svg_content.push_str("</svg>");

    Ok(svg_content)
}
