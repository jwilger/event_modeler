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

    // Step 3: Render horizontal swimlanes + slice boundaries + Views
    // Based on the gold master, we need:
    // - 3 horizontal swimlanes: UX, Commands, Events
    // - 3 vertical slices: CreateAccount, SendEmailVerification, VerifyEmailAddress
    // - View entities (white boxes) in the UX swimlane

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

        // Add swimlane label with text fitting (rotated on the left side)
        let swimlane_fitted_text = fit_text_to_container(name, swimlane_height, padding);
        svg_content.push_str(&swimlane_fitted_text.render_svg_rotated(
            padding / 2,
            lane_y + (swimlane_height as f32 / 2.0),
            -90.0,     // Rotate -90 degrees
            "#586069", // Gray color for swimlane labels
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

        // Add slice header at the top with text fitting
        let header_fitted_text = fit_text_to_container(slice_name, slice_width, padding);
        svg_content.push_str(&header_fitted_text.render_svg_with_style(
            slice_x + (slice_width / 2),
            padding / 2 + 5,
            "bold",
        ));
    }

    // Step 3: Add View entities (white boxes) in the UX swimlane
    // Based on example.eventmodel views and gold master layout
    let views = [
        ("Login\nScreen", 0),                 // slice 0 (Create Account)
        ("New\nAccount\nScreen", 0),          // slice 0 (Create Account)
        ("Verify Email\nAddress\nScreen", 2), // slice 2 (Verify Email Address)
        ("User\nProfile\nScreen", 2),         // slice 2 (Verify Email Address)
    ];

    // View styling
    let view_width = 100;
    let view_height = 60;
    let ux_swimlane_y = padding; // Top swimlane
    let view_y = ux_swimlane_y + (swimlane_height - view_height) / 2;

    for (i, (view_name, slice_index)) in views.iter().enumerate() {
        let slice_x = padding + (slice_index * slice_width);
        let view_x = slice_x + 20 + (i % 2) * (view_width + 10); // Offset multiple views in same slice

        // Draw view box (white with border)
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\" stroke=\"#d1d5da\" stroke-width=\"1\" rx=\"4\"/>",
            view_x,
            view_y,
            view_width,
            view_height
        ));

        // Add view text with proper text fitting
        let fitted_text = fit_text_to_container(view_name, view_width, view_height);
        svg_content
            .push_str(&fitted_text.render_svg(view_x + view_width / 2, view_y + view_height / 2));
    }

    svg_content.push_str("</svg>");

    Ok(svg_content)
}

/// Represents fitted text with appropriate font size and line breaks
struct FittedText {
    lines: Vec<String>,
    font_size: f32,
}

impl FittedText {
    fn render_svg(&self, center_x: usize, center_y: usize) -> String {
        self.render_svg_with_style(center_x, center_y, "normal")
    }

    fn render_svg_with_style(&self, center_x: usize, center_y: usize, font_weight: &str) -> String {
        self.render_svg_full(center_x, center_y, font_weight, "#24292e", None)
    }

    fn render_svg_rotated(
        &self,
        center_x: usize,
        center_y: f32,
        rotation: f32,
        color: &str,
    ) -> String {
        self.render_svg_full(center_x, center_y as usize, "normal", color, Some(rotation))
    }

    fn render_svg_full(
        &self,
        center_x: usize,
        center_y: usize,
        font_weight: &str,
        color: &str,
        rotation: Option<f32>,
    ) -> String {
        let mut svg = String::new();
        let line_height = self.font_size * 1.2;
        let total_height = line_height * self.lines.len() as f32;
        let start_y = center_y as f32 - (total_height - line_height) / 2.0;

        for (i, line) in self.lines.iter().enumerate() {
            let y = start_y + (i as f32 * line_height);

            let transform = if let Some(rot) = rotation {
                format!(" transform=\"rotate({}, {}, {})\"", rot, center_x, y)
            } else {
                String::new()
            };

            svg.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"{}\" font-weight=\"{}\" fill=\"{}\"{}>{}</text>",
                center_x,
                y,
                self.font_size,
                font_weight,
                color,
                transform,
                line
            ));
        }
        svg
    }
}

/// Fits text to a container by adjusting font size and wrapping lines
fn fit_text_to_container(
    text: &str,
    container_width: usize,
    container_height: usize,
) -> FittedText {
    // Convert newlines to line breaks for manual line breaks
    let initial_lines: Vec<&str> = text.split('\n').collect();

    // Start with a reasonable font size and scale down if needed
    let mut font_size = 12.0;
    let _max_font_size = 16.0;
    let min_font_size = 6.0;

    // Approximate character width (varies by font, but this is a reasonable estimate)
    let char_width_ratio = 0.6; // chars per pixel at font size 1

    loop {
        let max_chars_per_line =
            ((container_width as f32 - 10.0) / (font_size * char_width_ratio)) as usize;
        let line_height = font_size * 1.2;

        // Wrap text based on character limits
        let mut wrapped_lines = Vec::new();
        for line in &initial_lines {
            if line.len() <= max_chars_per_line {
                wrapped_lines.push(line.to_string());
            } else {
                // Simple word wrapping
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut current_line = String::new();

                for word in words {
                    if current_line.is_empty() {
                        current_line = word.to_string();
                    } else if current_line.len() + 1 + word.len() <= max_chars_per_line {
                        current_line.push(' ');
                        current_line.push_str(word);
                    } else {
                        wrapped_lines.push(current_line);
                        current_line = word.to_string();
                    }
                }
                if !current_line.is_empty() {
                    wrapped_lines.push(current_line);
                }
            }
        }

        // Check if all lines fit vertically
        let total_height = line_height * wrapped_lines.len() as f32;
        if total_height <= (container_height as f32 - 10.0) || font_size <= min_font_size {
            return FittedText {
                lines: wrapped_lines,
                font_size,
            };
        }

        // Reduce font size and try again
        font_size = (font_size - 1.0).max(min_font_size);
    }
}
