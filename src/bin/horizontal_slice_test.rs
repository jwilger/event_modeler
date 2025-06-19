//! Temporary test program for incrementally implementing horizontal slice architecture.
//!
//! This binary is used to test Step 8: Adding Automations (Green Boxes) rendering.

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

    // Step 8: Render horizontal swimlanes + slice boundaries + Views + Commands + Events + Projections + Queries + Automations
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

    // Step 8: Render horizontal swimlanes + slice boundaries + Views + Commands + Events + Projections + Queries + Automations
    // Based on the gold master, we need:
    // - 3 horizontal swimlanes: UX, Commands, Events
    // - 3 vertical slices: CreateAccount, SendEmailVerification, VerifyEmailAddress
    // - View entities (white boxes) in the UX swimlane
    // - Command entities (blue boxes) in the Commands swimlane
    // - Event entities (purple boxes) in the Events swimlane
    // - Projection entities (yellow boxes) in the Commands swimlane
    // - Query entities (blue boxes) in the Commands swimlane
    // - Automation entities (green boxes) in the UX swimlane

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

    // Step 4: Add Command entities (blue boxes) in the Commands swimlane
    // Based on example.eventmodel commands and gold master layout
    let commands = [
        ("Create\nUser Account\nCredentials", 0), // slice 0 (Create Account)
        ("Send Email\nVerification", 1),          // slice 1 (Send Email Verification)
        ("Verify\nUser Email\nAddress", 2),       // slice 2 (Verify Email Address)
    ];

    // Command styling
    let command_width = 120;
    let command_height = 80;
    let commands_swimlane_y = padding + swimlane_height; // Middle swimlane
    let command_y = commands_swimlane_y + (swimlane_height - command_height) / 2;

    for (command_name, slice_index) in &commands {
        let slice_x = padding + (slice_index * slice_width);
        let command_x = slice_x + 30; // Small offset from slice boundary

        // Draw command box (blue with border)
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#5b8def\" stroke=\"#4a6bc7\" stroke-width=\"1\" rx=\"4\"/>",
            command_x,
            command_y,
            command_width,
            command_height
        ));

        // Add command text with fitting (white text on blue background)
        let command_fitted_text =
            fit_text_to_container(command_name, command_width, command_height);
        svg_content.push_str(&command_fitted_text.render_svg_full(
            command_x + command_width / 2,
            command_y + command_height / 2,
            "normal",
            "white", // White text on blue background
            None,
        ));
    }

    // Step 5: Add Event entities (purple boxes) in the Events swimlane
    // Based on example.eventmodel events and gold master layout
    let events = [
        ("User Account\nCredentials\nCreated", 0), // slice 0 (Create Account)
        ("Email Verification\nMessage Sent", 1),   // slice 1 (Send Email Verification)
        ("Email\nAddress\nVerified", 2),           // slice 2 (Verify Email Address)
    ];

    // Event styling
    let event_width = 120;
    let event_height = 80;
    let events_swimlane_y = padding + (2 * swimlane_height); // Bottom swimlane
    let event_y = events_swimlane_y + (swimlane_height - event_height) / 2;

    for (event_name, slice_index) in &events {
        let slice_x = padding + (slice_index * slice_width);
        let event_x = slice_x + 30; // Small offset from slice boundary

        // Draw event box (purple with border)
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#8b5cf6\" stroke=\"#7c3aed\" stroke-width=\"1\" rx=\"4\"/>",
            event_x,
            event_y,
            event_width,
            event_height
        ));

        // Add event text with fitting (white text on purple background)
        let event_fitted_text = fit_text_to_container(event_name, event_width, event_height);
        svg_content.push_str(&event_fitted_text.render_svg_full(
            event_x + event_width / 2,
            event_y + event_height / 2,
            "normal",
            "white", // White text on purple background
            None,
        ));
    }

    // Step 6: Add Projection entities (yellow/orange boxes) in the Commands swimlane
    // Based on example.eventmodel projections and gold master layout
    let projections = [
        ("User\nCredentials\nProjection", 0), // slice 0 (Create Account)
        ("User Email\nVerification Token\nProjection", 1), // slice 1 (Send Email Verification)
    ];

    // Projection styling
    let projection_width = 120;
    let projection_height = 80;
    // Projections go in the middle swimlane along with commands
    let projection_y = commands_swimlane_y + (swimlane_height - projection_height) / 2;

    for (projection_name, slice_index) in &projections {
        let slice_x = padding + (slice_index * slice_width);
        // Position projections to the right of commands
        let projection_x = slice_x + 30 + command_width + 20; // Offset from command

        // Draw projection box (yellow/orange with border)
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#ffd166\" stroke=\"#f4a261\" stroke-width=\"1\" rx=\"4\"/>",
            projection_x,
            projection_y,
            projection_width,
            projection_height
        ));

        // Add projection text with fitting (dark text on yellow background)
        let projection_fitted_text =
            fit_text_to_container(projection_name, projection_width, projection_height);
        svg_content.push_str(&projection_fitted_text.render_svg_full(
            projection_x + projection_width / 2,
            projection_y + projection_height / 2,
            "normal",
            "#24292e", // Dark text on yellow background
            None,
        ));
    }

    // Step 7: Add Query entities (blue boxes) in the Commands swimlane
    // Based on example.eventmodel queries and gold master layout
    let queries = [
        ("Get Account\nID for Email\nVerification\nToken", 1), // slice 1 (Send Email Verification)
        ("Get\nUser\nProfile", 2),                             // slice 2 (Verify Email Address)
    ];

    // Query styling (same as commands - blue boxes)
    let query_width = 120;
    let query_height = 80;
    // Queries go in the middle swimlane along with commands and projections
    let query_y = commands_swimlane_y + (swimlane_height - query_height) / 2;

    for (query_name, slice_index) in &queries {
        let slice_x = padding + (slice_index * slice_width);
        // Position queries after projections in the slice
        let query_x = if *slice_index == 1 {
            // In slice 1, position after the projection
            slice_x + 30 + command_width + 20 + projection_width + 20
        } else {
            // In slice 2, we have a command but no projection, so position after command
            slice_x + 30 + command_width + 20
        };

        // Draw query box (blue like commands)
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#5b8def\" stroke=\"#4a6bc7\" stroke-width=\"1\" rx=\"4\"/>",
            query_x,
            query_y,
            query_width,
            query_height
        ));

        // Add query text with fitting (white text on blue background)
        let query_fitted_text = fit_text_to_container(query_name, query_width, query_height);
        svg_content.push_str(&query_fitted_text.render_svg_full(
            query_x + query_width / 2,
            query_y + query_height / 2,
            "normal",
            "white", // White text on blue background
            None,
        ));
    }

    // Step 8: Add Automation entities (green boxes) in the UX swimlane
    // Based on example.eventmodel automations and gold master layout
    let automations = [("Email\nVerifier", 1)]; // slice 1 (Send Email Verification)

    // Automation styling
    let automation_width = 100;
    let automation_height = 60;
    let ux_swimlane_y = padding; // Top swimlane
    let automation_y = ux_swimlane_y + (swimlane_height - automation_height) / 2;

    for (automation_name, slice_index) in &automations {
        let slice_x = padding + (slice_index * slice_width);
        // Position automation in the middle-right area of the slice in the UX swimlane
        let automation_x = slice_x + slice_width / 2;

        // Draw automation box (green with border)
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#06d6a0\" stroke=\"#04a77d\" stroke-width=\"1\" rx=\"4\"/>",
            automation_x,
            automation_y,
            automation_width,
            automation_height
        ));

        // Add automation text with fitting (white text on green background)
        let automation_fitted_text =
            fit_text_to_container(automation_name, automation_width, automation_height);
        svg_content.push_str(&automation_fitted_text.render_svg_full(
            automation_x + automation_width / 2,
            automation_y + automation_height / 2,
            "normal",
            "white", // White text on green background
            None,
        ));
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
