//! Temporary test program for implementing dynamic horizontal slice architecture.
//!
//! This binary creates a properly sized diagram with dynamic slice widths and test scenarios.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct Entity {
    id: String,
    entity_type: EntityType,
    name: String,
    slice: usize,
    swimlane: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntityType {
    View,
    Command,
    Event,
    Projection,
    Query,
    Automation,
}

impl EntityType {
    fn label(&self) -> &'static str {
        match self {
            EntityType::View => "View",
            EntityType::Command => "Command",
            EntityType::Event => "Event",
            EntityType::Projection => "Projection",
            EntityType::Query => "Query",
            EntityType::Automation => "Automation",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            EntityType::View => "#f8f9fa",
            EntityType::Command => "#5b8def",
            EntityType::Event => "#8b5cf6",
            EntityType::Projection => "#ffd166",
            EntityType::Query => "#5b8def",
            EntityType::Automation => "#06d6a0",
        }
    }

    fn text_color(&self) -> &'static str {
        match self {
            EntityType::View => "#24292e",
            _ => "white",
        }
    }

    fn border_color(&self) -> &'static str {
        match self {
            EntityType::View => "#d1d5da",
            EntityType::Command | EntityType::Query => "#4a6bc7",
            EntityType::Event => "#7c3aed",
            EntityType::Projection => "#f4a261",
            EntityType::Automation => "#04a77d",
        }
    }
}

#[derive(Debug, Clone)]
struct Connection {
    from: String,
    to: String,
}

#[derive(Debug, Clone)]
struct TestScenario {
    command: String,
    name: String,
    given: Vec<TestEntry>,
    when: Vec<TestEntry>,
    then: Vec<TestEntry>,
}

#[derive(Debug, Clone)]
struct TestEntry {
    _entity_type: String,
    text: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <output.svg>", args[0]);
        std::process::exit(1);
    }

    let output_path = PathBuf::from(&args[1]);

    // Render dynamic diagram with all components
    let svg_content = render_dynamic_diagram()?;

    // Write to file
    let mut file = fs::File::create(&output_path)?;
    file.write_all(svg_content.as_bytes())?;

    println!("Generated SVG: {}", output_path.display());
    Ok(())
}

fn render_dynamic_diagram() -> Result<String, Box<dyn std::error::Error>> {
    // Define all entities with their types, names, and slice/swimlane positions
    let mut entities = vec![
        // Slice 0: Create Account
        Entity {
            id: "login_screen".to_string(),
            entity_type: EntityType::View,
            name: "Login\nScreen".to_string(),
            slice: 0,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "new_account_screen".to_string(),
            entity_type: EntityType::View,
            name: "New\nAccount\nScreen".to_string(),
            slice: 0,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "create_user_account_credentials".to_string(),
            entity_type: EntityType::Command,
            name: "Create\nUser Account\nCredentials".to_string(),
            slice: 0,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "user_credentials_projection".to_string(),
            entity_type: EntityType::Projection,
            name: "User\nCredentials\nProjection".to_string(),
            slice: 0,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "user_account_credentials_created".to_string(),
            entity_type: EntityType::Event,
            name: "User Account\nCredentials\nCreated".to_string(),
            slice: 0,
            swimlane: 2,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        // Slice 1: Send Email Verification
        Entity {
            id: "new_account_screen2".to_string(),
            entity_type: EntityType::View,
            name: "New\nAccount\nScreen".to_string(),
            slice: 1,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "verify_email_screen".to_string(),
            entity_type: EntityType::View,
            name: "Verify\nEmail\nAddress\nScreen".to_string(),
            slice: 1,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "email_verifier".to_string(),
            entity_type: EntityType::Automation,
            name: "Email\nVerifier".to_string(),
            slice: 1,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "send_email_verification".to_string(),
            entity_type: EntityType::Command,
            name: "Send Email\nVerification".to_string(),
            slice: 1,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "user_email_verification_token_projection".to_string(),
            entity_type: EntityType::Projection,
            name: "User Email\nVerification\nToken\nProjection".to_string(),
            slice: 1,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "get_account_id_for_email_verification_token".to_string(),
            entity_type: EntityType::Query,
            name: "Get Account\nID for Email\nVerification\nToken".to_string(),
            slice: 1,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "email_verification_message_sent".to_string(),
            entity_type: EntityType::Event,
            name: "Email\nVerification\nMessage Sent".to_string(),
            slice: 1,
            swimlane: 2,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        // Slice 2: Verify Email Address
        Entity {
            id: "verify_email_screen2".to_string(),
            entity_type: EntityType::View,
            name: "Verify Email\nAddress\nScreen".to_string(),
            slice: 2,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "user_profile_screen".to_string(),
            entity_type: EntityType::View,
            name: "User\nProfile\nScreen".to_string(),
            slice: 2,
            swimlane: 0,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "verify_user_email_address".to_string(),
            entity_type: EntityType::Command,
            name: "Verify\nUser Email\nAddress".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "user_email_verification_token_projection2".to_string(),
            entity_type: EntityType::Projection,
            name: "User Email\nVerification\nToken\nProjection".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "user_credentials_projection2".to_string(),
            entity_type: EntityType::Projection,
            name: "User\nCredentials\nProjection".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "get_user_profile".to_string(),
            entity_type: EntityType::Query,
            name: "Get\nUser\nProfile".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
        Entity {
            id: "email_address_verified".to_string(),
            entity_type: EntityType::Event,
            name: "Email\nAddress\nVerified".to_string(),
            slice: 2,
            swimlane: 2,
            x: 0.0,
            y: 0.0,
            width: 80.0,
            height: 50.0,
        },
    ];

    // Define test scenarios
    let test_scenarios = vec![
        TestScenario {
            command: "Create User Account Credentials".to_string(),
            name: "Main Success".to_string(),
            given: vec![],
            when: vec![TestEntry {
                _entity_type: "When".to_string(),
                text: "Create\nUser\nAccount\nCredentials".to_string(),
            }],
            then: vec![TestEntry {
                _entity_type: "Then".to_string(),
                text: "User\nAccount\nCredentials\nCreated".to_string(),
            }],
        },
        TestScenario {
            command: "Create User Account Credentials".to_string(),
            name: "Account Already Exists".to_string(),
            given: vec![TestEntry {
                _entity_type: "Given".to_string(),
                text: "User\nAccount\nCredentials\nCreated".to_string(),
            }],
            when: vec![TestEntry {
                _entity_type: "When".to_string(),
                text: "Create\nUser\nAccount\nCredentials".to_string(),
            }],
            then: vec![TestEntry {
                _entity_type: "Then".to_string(),
                text: "Duplicate\nUser\nAccount\nError".to_string(),
            }],
        },
        TestScenario {
            command: "Send Email Verification".to_string(),
            name: "Main Success".to_string(),
            given: vec![TestEntry {
                _entity_type: "Given".to_string(),
                text: "User\nAccount\nCredentials\nCreated".to_string(),
            }],
            when: vec![TestEntry {
                _entity_type: "When".to_string(),
                text: "Send Email\nVerification".to_string(),
            }],
            then: vec![TestEntry {
                _entity_type: "Then".to_string(),
                text: "Email\nVerification\nMessage\nSent".to_string(),
            }],
        },
        TestScenario {
            command: "Send Email Verification".to_string(),
            name: "No Such User".to_string(),
            given: vec![],
            when: vec![TestEntry {
                _entity_type: "When".to_string(),
                text: "Send Email\nVerification".to_string(),
            }],
            then: vec![TestEntry {
                _entity_type: "Then".to_string(),
                text: "Unknown\nUser\nAccount\nError".to_string(),
            }],
        },
        TestScenario {
            command: "Verify Email Address".to_string(),
            name: "Main Success".to_string(),
            given: vec![
                TestEntry {
                    _entity_type: "Given".to_string(),
                    text: "User\nAccount\nCredentials\nCreated".to_string(),
                },
                TestEntry {
                    _entity_type: "Given".to_string(),
                    text: "Email\nVerification\nMessage\nSent".to_string(),
                },
            ],
            when: vec![TestEntry {
                _entity_type: "When".to_string(),
                text: "Verify\nUser Email\nAddress".to_string(),
            }],
            then: vec![TestEntry {
                _entity_type: "Then".to_string(),
                text: "Email\nAddress\nVerified".to_string(),
            }],
        },
        TestScenario {
            command: "Verify Email Address".to_string(),
            name: "Invalid Verification Token".to_string(),
            given: vec![TestEntry {
                _entity_type: "Given".to_string(),
                text: "User\nAccount\nCredentials\nCreated".to_string(),
            }],
            when: vec![TestEntry {
                _entity_type: "When".to_string(),
                text: "Verify\nUser Email\nAddress".to_string(),
            }],
            then: vec![TestEntry {
                _entity_type: "Then".to_string(),
                text: "Invalid\nVerification\nToken\nError".to_string(),
            }],
        },
    ];

    // Define all connections based on the slices
    let connections = vec![
        // Slice 0: Create Account
        Connection {
            from: "login_screen".to_string(),
            to: "new_account_screen".to_string(),
        },
        Connection {
            from: "new_account_screen".to_string(),
            to: "create_user_account_credentials".to_string(),
        },
        Connection {
            from: "create_user_account_credentials".to_string(),
            to: "user_account_credentials_created".to_string(),
        },
        Connection {
            from: "user_account_credentials_created".to_string(),
            to: "user_credentials_projection".to_string(),
        },
        Connection {
            from: "user_account_credentials_created".to_string(),
            to: "new_account_screen2".to_string(),
        },
        Connection {
            from: "new_account_screen2".to_string(),
            to: "verify_email_screen".to_string(),
        },
        // Slice 1: Send Email Verification
        Connection {
            from: "user_account_credentials_created".to_string(),
            to: "email_verifier".to_string(),
        },
        Connection {
            from: "email_verifier".to_string(),
            to: "send_email_verification".to_string(),
        },
        Connection {
            from: "send_email_verification".to_string(),
            to: "email_verification_message_sent".to_string(),
        },
        Connection {
            from: "email_verification_message_sent".to_string(),
            to: "user_email_verification_token_projection".to_string(),
        },
        // Slice 2: Verify Email Address
        Connection {
            from: "verify_email_screen".to_string(),
            to: "verify_user_email_address".to_string(),
        },
        Connection {
            from: "verify_user_email_address".to_string(),
            to: "email_address_verified".to_string(),
        },
        Connection {
            from: "email_address_verified".to_string(),
            to: "user_credentials_projection2".to_string(),
        },
        Connection {
            from: "email_address_verified".to_string(),
            to: "user_email_verification_token_projection2".to_string(),
        },
        Connection {
            from: "email_address_verified".to_string(),
            to: "verify_email_screen2".to_string(),
        },
        Connection {
            from: "verify_email_screen2".to_string(),
            to: "user_profile_screen".to_string(),
        },
        Connection {
            from: "user_profile_screen".to_string(),
            to: "get_user_profile".to_string(),
        },
    ];

    // Layout entities dynamically and get canvas dimensions
    let (diagram_width, diagram_height) = layout_entities(&mut entities, &connections);

    // Create entity lookup map
    let mut entity_map: HashMap<String, &Entity> = HashMap::new();
    for entity in &entities {
        entity_map.insert(entity.id.clone(), entity);
    }

    // Calculate test scenario space
    let test_scenario_height = 300.0; // Height for test scenario section
    let test_scenario_y_start = diagram_height + 50.0; // Start test scenarios below main diagram
    let canvas_width = diagram_width;
    let canvas_height = test_scenario_y_start + test_scenario_height;

    let padding = 40.0;
    let swimlane_height = 100.0;

    // Build SVG
    let mut svg_content = String::new();
    svg_content.push_str(&format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        canvas_width, canvas_height
    ));

    // Add arrow markers
    svg_content.push_str(
        "<defs>\
        <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\
            <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#586069\" />\
        </marker>\
    </defs>",
    );

    // Add background
    svg_content.push_str(&format!(
        r#"<rect width="{}" height="{}" fill="white" stroke="none"/>"#,
        canvas_width, canvas_height
    ));

    // Draw swimlanes
    let swimlanes = [
        ("UX, Automations", 0),
        ("Commands, Projections, Queries", 1),
        ("User Account Event Stream", 2),
    ];

    for (name, index) in &swimlanes {
        let y = padding + (*index as f64 * swimlane_height);
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#f8f9fa\" stroke=\"#e1e4e8\" stroke-width=\"1\"/>",
            padding,
            y,
            canvas_width - 2.0 * padding,
            swimlane_height - 5.0
        ));

        // Swimlane label
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"12\" fill=\"#586069\" transform=\"rotate(-90, {}, {})\">{}</text>",
            padding / 2.0,
            y + swimlane_height / 2.0,
            padding / 2.0,
            y + swimlane_height / 2.0,
            name
        ));
    }

    // Draw slice boundaries and headers
    let slice_names = [
        "Create Account",
        "Send Email Verification",
        "Verify Email Address",
    ];

    // Calculate slice positions based on actual entity positions
    let slice_positions = calculate_slice_positions(&entities);

    for (i, (slice_name, (start_x, end_x))) in
        slice_names.iter().zip(slice_positions.iter()).enumerate()
    {
        // Draw slice boundary line (except for the first)
        if i > 0 {
            svg_content.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#d1d5da\" stroke-width=\"2\"/>",
                start_x, padding, start_x, canvas_height - padding
            ));
        }

        // Add slice header
        let slice_center = (start_x + end_x) / 2.0;
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" font-weight=\"bold\" fill=\"#24292e\">{}</text>",
            slice_center,
            padding / 2.0 + 5.0,
            slice_name
        ));
    }

    // Draw entities
    for entity in &entities {
        // Draw entity box
        svg_content.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1" rx="4"/>"#,
            entity.x,
            entity.y,
            entity.width,
            entity.height,
            entity.entity_type.color(),
            entity.entity_type.border_color()
        ));

        // Draw entity type label
        svg_content.push_str(&format!(
            r#"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-family="Arial, sans-serif" font-size="10" fill="{}">{}</text>"#,
            entity.x + entity.width / 2.0,
            entity.y + 12.0,
            entity.entity_type.text_color(),
            entity.entity_type.label()
        ));

        // Draw entity name
        let lines: Vec<&str> = entity.name.split('\n').collect();
        let line_height = 14;
        let total_height = (lines.len() as i32 - 1) * line_height;
        let start_y = entity.y as i32 + entity.height as i32 / 2 - total_height / 2;

        for (i, line) in lines.iter().enumerate() {
            svg_content.push_str(&format!(
                r#"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" font-family="Arial, sans-serif" font-size="12" fill="{}">{}</text>"#,
                entity.x + entity.width / 2.0,
                start_y + (i as i32 * line_height),
                entity.entity_type.text_color(),
                line
            ));
        }
    }

    // Draw connections
    for connection in &connections {
        if let (Some(from_entity), Some(to_entity)) = (
            entity_map.get(&connection.from),
            entity_map.get(&connection.to),
        ) {
            let (from_x, from_y, to_x, to_y) = calculate_connection_points(from_entity, to_entity);

            svg_content.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#586069\" stroke-width=\"2\" marker-end=\"url(#arrowhead)\"/>",
                from_x, from_y, to_x, to_y
            ));
        }
    }

    // Draw test scenarios
    render_test_scenarios(&mut svg_content, &test_scenarios, test_scenario_y_start);

    svg_content.push_str("</svg>");

    Ok(svg_content)
}

fn layout_entities(entities: &mut [Entity], _connections: &[Connection]) -> (f64, f64) {
    // Dynamic layout parameters
    let padding = 40.0;
    let swimlane_height = 100.0;
    let entity_h_spacing = 20.0;
    let _entity_v_spacing = 10.0;
    let slice_padding = 60.0;
    let min_slice_width = 200.0;

    // Find max slice and swimlane indices to handle any number
    let max_slice = entities.iter().map(|e| e.slice).max().unwrap_or(0);
    let max_swimlane = entities.iter().map(|e| e.swimlane).max().unwrap_or(0);

    // Group entities by slice and swimlane dynamically
    let mut slice_groups: Vec<Vec<Vec<&mut Entity>>> = vec![];
    for _ in 0..=max_slice {
        let mut swimlanes = vec![];
        for _ in 0..=max_swimlane {
            swimlanes.push(vec![]);
        }
        slice_groups.push(swimlanes);
    }

    for entity in entities.iter_mut() {
        // Set more reasonable entity sizes to match gold master
        entity.width = 80.0;
        entity.height = 50.0;
        slice_groups[entity.slice][entity.swimlane].push(entity);
    }

    // Calculate slice widths based on content
    let mut slice_widths = vec![0.0; max_slice + 1];
    for (slice_idx, slice) in slice_groups.iter().enumerate() {
        let mut max_width: f64 = min_slice_width;

        for swimlane_entities in slice.iter() {
            if !swimlane_entities.is_empty() {
                let width_needed = swimlane_entities.iter().map(|e| e.width).sum::<f64>()
                    + entity_h_spacing * (swimlane_entities.len() as f64 - 1.0)
                    + 40.0; // Extra padding for swimlane
                max_width = max_width.max(width_needed);
            }
        }

        slice_widths[slice_idx] = max_width;
    }

    // Calculate slice positions
    let mut slice_x_positions = vec![padding + 30.0];
    for i in 1..=max_slice {
        let prev_x = slice_x_positions[i - 1];
        let prev_width = slice_widths[i - 1];
        slice_x_positions.push(prev_x + prev_width + slice_padding);
    }

    // Position entities within slices
    for (slice_idx, slice) in slice_groups.iter_mut().enumerate() {
        let slice_x = slice_x_positions[slice_idx];
        let slice_width = slice_widths[slice_idx];

        for (swimlane_idx, swimlane_entities) in slice.iter_mut().enumerate() {
            if swimlane_entities.is_empty() {
                continue;
            }

            // Calculate total width for centering
            let total_width = swimlane_entities.iter().map(|e| e.width).sum::<f64>()
                + entity_h_spacing * (swimlane_entities.len() as f64 - 1.0);

            // Center entities in slice
            let start_x = slice_x + (slice_width - total_width) / 2.0;
            let mut x = start_x;

            // Vertical positioning
            let base_y = padding + (swimlane_idx as f64 * swimlane_height);
            let y = base_y + (swimlane_height - 50.0) / 2.0;

            for entity in swimlane_entities.iter_mut() {
                entity.x = x;
                entity.y = y;
                x += entity.width + entity_h_spacing;
            }
        }
    }

    // Calculate total canvas size
    let total_width = if slice_x_positions.is_empty() {
        200.0
    } else {
        slice_x_positions.last().unwrap() + slice_widths.last().unwrap_or(&0.0) + padding
    };

    let total_height = padding * 2.0 + ((max_swimlane + 1) as f64 * swimlane_height);

    (total_width, total_height)
}

fn calculate_slice_positions(entities: &[Entity]) -> Vec<(f64, f64)> {
    let max_slice = entities.iter().map(|e| e.slice).max().unwrap_or(0);
    let mut slice_bounds = vec![(f64::MAX, 0.0); max_slice + 1];

    for entity in entities {
        let slice = entity.slice;
        slice_bounds[slice].0 = slice_bounds[slice].0.min(entity.x - 20.0);
        slice_bounds[slice].1 = f64::max(slice_bounds[slice].1, entity.x + entity.width + 20.0);
    }

    slice_bounds
}

fn calculate_connection_points(from: &Entity, to: &Entity) -> (f64, f64, f64, f64) {
    let from_center_x = from.x + from.width / 2.0;
    let from_center_y = from.y + from.height / 2.0;
    let to_center_x = to.x + to.width / 2.0;
    let to_center_y = to.y + to.height / 2.0;

    // Determine connection points based on relative positions
    let (from_x, from_y, to_x, to_y) = if to.x > from.x + from.width {
        // To is to the right
        (from.x + from.width, from_center_y, to.x, to_center_y)
    } else if to.y > from.y + from.height {
        // To is below
        (from_center_x, from.y + from.height, to_center_x, to.y)
    } else if to.y < from.y {
        // To is above
        (from_center_x, from.y, to_center_x, to.y + to.height)
    } else {
        // Default: center to center
        (from_center_x, from_center_y, to_center_x, to_center_y)
    };

    (from_x, from_y, to_x, to_y)
}

fn render_test_scenarios(svg_content: &mut String, test_scenarios: &[TestScenario], y_start: f64) {
    // Reorganize test scenarios by command
    let mut scenarios_by_command: HashMap<String, Vec<&TestScenario>> = HashMap::new();
    for scenario in test_scenarios {
        scenarios_by_command
            .entry(scenario.command.clone())
            .or_default()
            .push(scenario);
    }

    let section_width = 300.0;
    let section_spacing = 20.0;
    let row_labels = ["Given", "When", "Then"];
    let row_height = 60.0;
    let _entry_width = 100.0;
    let _entry_height = 45.0;
    let _entry_spacing = 10.0;
    let header_height = 40.0;

    let mut section_x = 50.0;

    // Group and render test scenarios by command
    for (command, scenarios) in scenarios_by_command.iter() {
        // Calculate section dimensions
        let num_scenarios = scenarios.len();
        let section_height = header_height + (3.0 * row_height) + 20.0; // 3 rows + padding

        // Draw section container
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#ffffff\" stroke=\"#d1d5da\" stroke-width=\"1\"/>",
            section_x, y_start, section_width, section_height
        ));

        // Draw section header
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" font-weight=\"bold\" fill=\"#24292e\">{}</text>",
            section_x + section_width / 2.0,
            y_start + 20.0,
            command
        ));

        // Calculate column width for scenarios
        let col_width = (section_width - 60.0) / num_scenarios as f64;

        // Draw row labels and entries
        for (row_idx, row_label) in row_labels.iter().enumerate() {
            let row_y = y_start + header_height + (row_idx as f64 * row_height);

            // Draw row label
            svg_content.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"Arial, sans-serif\" font-size=\"11\" fill=\"#586069\">{}</text>",
                section_x + 10.0,
                row_y + row_height / 2.0,
                row_label
            ));

            // Draw entries for each scenario
            for (scenario_idx, scenario) in scenarios.iter().enumerate() {
                let entries = match row_idx {
                    0 => &scenario.given,
                    1 => &scenario.when,
                    2 => &scenario.then,
                    _ => continue,
                };

                // Draw scenario name header if first row
                if row_idx == 0 {
                    let scenario_x = section_x + 50.0 + (scenario_idx as f64 * col_width);
                    svg_content.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"#586069\">{}</text>",
                        scenario_x + col_width / 2.0,
                        row_y - 5.0,
                        scenario.name
                    ));
                }

                // Draw entries
                let mut entry_y = row_y + 5.0;
                for entry in entries.iter() {
                    let entry_x = section_x + 50.0 + (scenario_idx as f64 * col_width) + 5.0;
                    let entry_w = col_width - 10.0;
                    let entry_h = 35.0;

                    // Determine colors based on content
                    let (bg_color, text_color, border_color) = if entry.text.contains("Error") {
                        ("#fee7e7", "#d73a3a", "#f5c6c6") // Light red for errors
                    } else if entry.text.contains("Verified") || entry.text.contains("Sent") {
                        ("#e6e1ff", "#5b41d9", "#c6b9ff") // Light purple for events
                    } else {
                        ("#e1edff", "#4a6bc7", "#b9d1ff") // Light blue for commands/default
                    };

                    // Draw entry box
                    svg_content.push_str(&format!(
                        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\" rx=\"2\"/>",
                        entry_x, entry_y, entry_w, entry_h, bg_color, border_color
                    ));

                    // Draw entry text (simplified - just first line for now)
                    let text_lines: Vec<&str> = entry.text.split('\n').collect();
                    svg_content.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"9\" fill=\"{}\">{}</text>",
                        entry_x + entry_w / 2.0,
                        entry_y + entry_h / 2.0,
                        text_color,
                        text_lines.first().unwrap_or(&"")
                    ));

                    entry_y += entry_h + 5.0;
                }
            }
        }

        section_x += section_width + section_spacing;
    }
}
