//! Temporary test program for incrementally implementing horizontal slice architecture.
//!
//! This binary is used to test Step 11: Final Polish and layout refinements.

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
    x: usize,
    y: usize,
    width: usize,
    height: usize,
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

    // Step 9: Render horizontal swimlanes + slice boundaries + all entities + connections
    let svg_content = render_swimlanes_and_slices()?;

    // Write to file
    let mut file = fs::File::create(&output_path)?;
    file.write_all(svg_content.as_bytes())?;

    println!("Generated SVG: {}", output_path.display());
    Ok(())
}

fn render_swimlanes_and_slices() -> Result<String, Box<dyn std::error::Error>> {
    // Define all entities with their types, names, and slice/swimlane positions
    let mut entities = vec![
        // Slice 0: Create Account
        Entity {
            id: "login_screen".to_string(),
            entity_type: EntityType::View,
            name: "Login\nScreen".to_string(),
            slice: 0,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "new_account_screen".to_string(),
            entity_type: EntityType::View,
            name: "New\nAccount\nScreen".to_string(),
            slice: 0,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "create_user_account_credentials".to_string(),
            entity_type: EntityType::Command,
            name: "Create\nUser Account\nCredentials".to_string(),
            slice: 0,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "user_credentials_projection".to_string(),
            entity_type: EntityType::Projection,
            name: "User\nCredentials\nProjection".to_string(),
            slice: 0,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "user_account_credentials_created".to_string(),
            entity_type: EntityType::Event,
            name: "User Account\nCredentials\nCreated".to_string(),
            slice: 0,
            swimlane: 2,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        // Slice 1: Send Email Verification
        Entity {
            id: "new_account_screen2".to_string(),
            entity_type: EntityType::View,
            name: "New\nAccount\nScreen".to_string(),
            slice: 1,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "verify_email_screen".to_string(),
            entity_type: EntityType::View,
            name: "Verify\nEmail\nAddress\nScreen".to_string(),
            slice: 1,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "email_verifier".to_string(),
            entity_type: EntityType::Automation,
            name: "Email\nVerifier".to_string(),
            slice: 1,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "send_email_verification".to_string(),
            entity_type: EntityType::Command,
            name: "Send Email\nVerification".to_string(),
            slice: 1,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "user_email_verification_token_projection".to_string(),
            entity_type: EntityType::Projection,
            name: "User Email\nVerification\nToken\nProjection".to_string(),
            slice: 1,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "get_account_id_for_email_verification_token".to_string(),
            entity_type: EntityType::Query,
            name: "Get Account\nID for Email\nVerification\nToken".to_string(),
            slice: 1,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "email_verification_message_sent".to_string(),
            entity_type: EntityType::Event,
            name: "Email\nVerification\nMessage Sent".to_string(),
            slice: 1,
            swimlane: 2,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        // Slice 2: Verify Email Address
        Entity {
            id: "verify_email_screen2".to_string(),
            entity_type: EntityType::View,
            name: "Verify Email\nAddress\nScreen".to_string(),
            slice: 2,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "user_profile_screen".to_string(),
            entity_type: EntityType::View,
            name: "User\nProfile\nScreen".to_string(),
            slice: 2,
            swimlane: 0,
            x: 0,
            y: 0,
            width: 100,
            height: 60,
        },
        Entity {
            id: "verify_user_email_address".to_string(),
            entity_type: EntityType::Command,
            name: "Verify\nUser Email\nAddress".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "user_email_verification_token_projection2".to_string(),
            entity_type: EntityType::Projection,
            name: "User Email\nVerification\nToken\nProjection".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "user_credentials_projection2".to_string(),
            entity_type: EntityType::Projection,
            name: "User\nCredentials\nProjection".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "get_user_profile".to_string(),
            entity_type: EntityType::Query,
            name: "Get\nUser\nProfile".to_string(),
            slice: 2,
            swimlane: 1,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
        },
        Entity {
            id: "email_address_verified".to_string(),
            entity_type: EntityType::Event,
            name: "Email\nAddress\nVerified".to_string(),
            slice: 2,
            swimlane: 2,
            x: 0,
            y: 0,
            width: 120,
            height: 80,
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

    // Layout entities dynamically
    layout_entities(&mut entities, &connections);

    // Create entity lookup map
    let mut entity_map: HashMap<String, &Entity> = HashMap::new();
    for entity in &entities {
        entity_map.insert(entity.id.clone(), entity);
    }

    // Calculate dynamic canvas size based on entity positions
    let mut max_x = 0;
    let mut max_y = 0;
    for entity in &entities {
        max_x = max_x.max(entity.x + entity.width);
        max_y = max_y.max(entity.y + entity.height);
    }
    let canvas_width = max_x + 100; // Add padding

    // Calculate test scenario space
    let test_scenario_height = 250; // Height for test scenario section
    let test_scenario_y_start = max_y + 150; // Start test scenarios below main diagram
    let canvas_height = test_scenario_y_start + test_scenario_height + 50; // Add padding

    let padding = 60;
    let swimlane_height = 140;

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
        let y = padding + (index * swimlane_height);
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#f8f9fa\" stroke=\"#e1e4e8\" stroke-width=\"1\"/>",
            padding,
            y,
            canvas_width - 2 * padding,
            swimlane_height - 5
        ));

        // Swimlane label
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"12\" fill=\"#586069\" transform=\"rotate(-90, {}, {})\">{}</text>",
            padding / 2,
            y + swimlane_height / 2,
            padding / 2,
            y + swimlane_height / 2,
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
        let slice_center = (start_x + end_x) / 2;
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" font-weight=\"bold\" fill=\"#24292e\">{}</text>",
            slice_center,
            padding / 2 + 5,
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
            entity.x + entity.width / 2,
            entity.y + 12,
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
                entity.x + entity.width / 2,
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

fn layout_entities(entities: &mut [Entity], _connections: &[Connection]) {
    let padding = 60;
    let swimlane_height = 140;
    let entity_spacing = 40;
    let _slice_padding = 100;
    let vertical_spacing = 20;

    // Group entities by slice and swimlane
    let mut slice_groups: Vec<Vec<Vec<&mut Entity>>> = vec![
        vec![vec![], vec![], vec![]],
        vec![vec![], vec![], vec![]],
        vec![vec![], vec![], vec![]],
    ];

    for entity in entities.iter_mut() {
        slice_groups[entity.slice][entity.swimlane].push(entity);
    }

    // Layout entities in each slice/swimlane
    let mut slice_x_positions = vec![padding + 50];

    for (slice_idx, slice) in slice_groups.iter_mut().enumerate() {
        let current_x = if slice_idx > 0 {
            slice_x_positions[slice_idx - 1] + 450 // Fixed width per slice for better alignment
        } else {
            slice_x_positions[0]
        };

        if slice_idx < slice_x_positions.len() {
            slice_x_positions[slice_idx] = current_x;
        } else {
            slice_x_positions.push(current_x);
        }

        for (swimlane_idx, swimlane_entities) in slice.iter_mut().enumerate() {
            // Calculate total width needed for this swimlane
            let total_width: usize = swimlane_entities.iter().map(|e| e.width).sum::<usize>()
                + entity_spacing * (swimlane_entities.len().saturating_sub(1));

            // Center entities within the slice
            let start_x = if total_width < 400 {
                current_x + (400 - total_width) / 2 // Center within ~400px slice width
            } else {
                current_x // Too wide to center, just start at slice beginning
            };
            let mut x = start_x;

            // Vertical positioning with better spacing
            let base_y = padding + (swimlane_idx * swimlane_height);
            let y = if swimlane_entities.len() > 1 {
                // Multiple entities - stagger them vertically
                base_y + vertical_spacing
            } else {
                // Single entity - center it
                base_y + (swimlane_height - 80) / 2
            };

            let swimlane_len = swimlane_entities.len();
            for (entity_idx, entity) in swimlane_entities.iter_mut().enumerate() {
                entity.x = x;
                // Stagger entities vertically in busy swimlanes
                entity.y = if swimlane_len > 2 && entity_idx % 2 == 1 {
                    y + vertical_spacing
                } else {
                    y
                };
                x += entity.width + entity_spacing;
            }
        }
    }
}

fn calculate_slice_positions(entities: &[Entity]) -> Vec<(usize, usize)> {
    let mut slice_bounds = vec![(usize::MAX, 0); 3];

    for entity in entities {
        let slice = entity.slice;
        slice_bounds[slice].0 = slice_bounds[slice].0.min(entity.x - 20);
        slice_bounds[slice].1 = slice_bounds[slice].1.max(entity.x + entity.width + 20);
    }

    slice_bounds
}

fn calculate_connection_points(from: &Entity, to: &Entity) -> (usize, usize, usize, usize) {
    let from_center_x = from.x + from.width / 2;
    let from_center_y = from.y + from.height / 2;
    let to_center_x = to.x + to.width / 2;
    let to_center_y = to.y + to.height / 2;

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

fn render_test_scenarios(
    svg_content: &mut String,
    test_scenarios: &[TestScenario],
    y_start: usize,
) {
    let scenario_width = 350;
    let scenario_spacing = 30;
    let row_height = 50;
    let entry_width = 90;
    let entry_height = 40;
    let entry_spacing = 8;
    let _padding = 20;

    let mut x = 50;

    for scenario in test_scenarios {
        // Draw scenario container
        svg_content.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#f8f9fa\" stroke=\"#d1d5da\" stroke-width=\"1\"/>",
            x, y_start, scenario_width, 200
        ));

        // Draw scenario title
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"14\" font-weight=\"bold\" fill=\"#24292e\">{}</text>",
            x + scenario_width / 2,
            y_start + 20,
            scenario.command
        ));

        // Draw scenario name
        svg_content.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"12\" fill=\"#586069\">{}</text>",
            x + scenario_width / 2,
            y_start + 35,
            scenario.name
        ));

        // Draw Given/When/Then labels
        let labels = ["Given", "When", "Then"];
        let entries = [&scenario.given, &scenario.when, &scenario.then];

        for (row_idx, (label, entries)) in labels.iter().zip(entries.iter()).enumerate() {
            let y = y_start + 50 + (row_idx * row_height);

            // Draw row label
            svg_content.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" font-family=\"Arial, sans-serif\" font-size=\"12\" fill=\"#586069\">{}</text>",
                x + 10,
                y + row_height / 2,
                label
            ));

            // Draw entries in this row
            let mut entry_x = x + 60;
            for entry in entries.iter() {
                // Determine colors based on content
                let (bg_color, text_color, border_color) = if entry.text.contains("Created") {
                    ("#5b8def", "white", "#4a6bc7") // Blue for commands/events
                } else if entry.text.contains("Error") {
                    ("#ff6b6b", "white", "#e74c3c") // Red for errors
                } else if entry.text.contains("Verified") || entry.text.contains("Sent") {
                    ("#8b5cf6", "white", "#7c3aed") // Purple for events
                } else {
                    ("#5b8def", "white", "#4a6bc7") // Default blue
                };

                // Draw entry box
                svg_content.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\" rx=\"3\"/>",
                    entry_x,
                    y,
                    entry_width,
                    entry_height,
                    bg_color,
                    border_color
                ));

                // Draw entry text (handle multi-line)
                let lines: Vec<&str> = entry.text.split('\n').collect();
                let line_height = 12;
                let total_height = (lines.len() as i32 - 1) * line_height;
                let text_start_y = y as i32 + entry_height / 2 - total_height / 2;

                for (i, line) in lines.iter().enumerate() {
                    svg_content.push_str(&format!(
                        "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" font-family=\"Arial, sans-serif\" font-size=\"10\" fill=\"{}\">{}</text>",
                        entry_x + entry_width / 2,
                        text_start_y + (i as i32 * line_height),
                        text_color,
                        line
                    ));
                }

                entry_x += entry_width + entry_spacing;
            }
        }

        x += scenario_width + scenario_spacing;
    }
}
