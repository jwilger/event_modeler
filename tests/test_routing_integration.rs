#[test]
fn test_routing_integration() {
    use event_modeler::diagram::{build_diagram_from_domain, render_to_svg};
    use event_modeler::infrastructure::parsing::yaml_converter::convert_yaml_to_domain;
    use event_modeler::infrastructure::parsing::yaml_parser::parse_yaml;

    let yaml_content = r#"workflow: Test Routing System

swimlanes:
  - frontend: "Frontend"
  - backend: "Backend"

views:
  LoginScreen:
    description: User login interface
    swimlane: frontend
    components:
      - LoginForm: Form
  DashboardScreen:
    description: Main dashboard after login
    swimlane: frontend
    components:
      - DataGrid: Grid

commands:
  LoginCommand:
    description: Process user login
    swimlane: frontend
  LoadDataCommand:
    description: Load user data after login
    swimlane: backend

events:
  UserLoggedIn:
    description: User successfully logged in
    swimlane: backend
  DataLoaded:
    description: User data loaded
    swimlane: backend

slices:
  - name: User Login
    connections:
      - "LoginScreen -> LoginCommand"
      - "LoginCommand -> UserLoggedIn"
      - "UserLoggedIn -> LoadDataCommand"
      - "LoadDataCommand -> DataLoaded"
      - "DataLoaded -> DashboardScreen""#;

    // Parse the YAML content
    let yaml_model = parse_yaml(yaml_content).expect("Failed to parse YAML");

    // Convert to domain model
    let domain_model = convert_yaml_to_domain(yaml_model).expect("Failed to convert to domain");

    // Build the diagram
    let diagram = build_diagram_from_domain(&domain_model).expect("Failed to build diagram");

    // Render to SVG
    let svg = render_to_svg(&diagram).expect("Failed to render SVG");

    // Verify the SVG contains orthogonal paths (L-shaped)
    assert!(svg.contains("<path"), "SVG should contain path elements");
    assert!(
        svg.contains(" L "),
        "SVG paths should contain L commands for orthogonal routing"
    );

    // Write to file for manual inspection
    std::fs::write("test_routing_output.svg", &svg).expect("Failed to write SVG file");

    println!("Successfully generated test_routing_output.svg with orthogonal routing!");
}
