//! Tests for the parser component of the parsing infrastructure.

use event_modeler::infrastructure::parsing::{EventModelParser, ParseError};
use event_modeler::event_model::entities::{Command, Event, Projection, Policy};

#[test]
fn parser_requires_title_section_first() {
    let input = "Swimlane: Test";
    let result = EventModelParser::new().parse(input);
    
    match result {
        Err(ParseError::MissingTitle) => (),
        _ => panic!("Expected MissingTitle error"),
    }
}

#[test]
fn parser_parses_minimal_valid_model() {
    let input = "Title: Simple Model
Swimlane: System
- Event: SystemStarted
";
    
    let result = EventModelParser::new().parse(input);
    assert!(result.is_ok());
    
    let model = result.unwrap();
    assert_eq!(model.title.as_str(), "Simple Model");
    assert_eq!(model.swimlanes.len(), 1);
    assert_eq!(model.swimlanes[0].name.as_str(), "System");
    assert_eq!(model.swimlanes[0].entities.len(), 1);
}

#[test]
fn parser_parses_complete_model() {
    let input = "Title: Order Processing System
Swimlane: Customer
- Command: PlaceOrder
- Event: OrderPlaced

Swimlane: Order Service
- Aggregate: Order
- Policy: OrderValidation
- Event: OrderValidated
- Projection: OrderSummary

Swimlane: External
- External System: Payment Gateway
";
    
    let result = EventModelParser::new().parse(input);
    assert!(result.is_ok());
    
    let model = result.unwrap();
    assert_eq!(model.title.as_str(), "Order Processing System");
    assert_eq!(model.swimlanes.len(), 3);
    
    // Verify Customer swimlane
    let customer = &model.swimlanes[0];
    assert_eq!(customer.name.as_str(), "Customer");
    assert_eq!(customer.entities.len(), 2);
    
    // Verify Order Service swimlane
    let service = &model.swimlanes[1];
    assert_eq!(service.name.as_str(), "Order Service");
    assert_eq!(service.entities.len(), 4);
    
    // Verify External swimlane
    let external = &model.swimlanes[2];
    assert_eq!(external.name.as_str(), "External");
    assert_eq!(external.entities.len(), 1);
}

#[test]
fn parser_handles_connectors() {
    let input = "Title: With Connectors
Swimlane: System
- Command: StartProcess
- Event: ProcessStarted

ProcessStarted -> StartProcess
";
    
    let result = EventModelParser::new().parse(input);
    assert!(result.is_ok());
    
    let model = result.unwrap();
    assert_eq!(model.connectors.len(), 1);
    
    let connector = &model.connectors[0];
    assert_eq!(connector.from.as_str(), "ProcessStarted");
    assert_eq!(connector.to.as_str(), "StartProcess");
}

#[test]
fn parser_enforces_unique_entity_names() {
    let input = "Title: Duplicate Names
Swimlane: System
- Event: DuplicateName
- Command: DuplicateName
";
    
    let result = EventModelParser::new().parse(input);
    match result {
        Err(ParseError::DuplicateEntity(name)) => {
            assert_eq!(name, "DuplicateName");
        }
        _ => panic!("Expected DuplicateEntity error"),
    }
}

#[test]
fn parser_validates_connector_references() {
    let input = "Title: Invalid Connector
Swimlane: System
- Event: ValidEvent

ValidEvent -> NonExistentEntity
";
    
    let result = EventModelParser::new().parse(input);
    match result {
        Err(ParseError::UnknownEntity(name)) => {
            assert_eq!(name, "NonExistentEntity");
        }
        _ => panic!("Expected UnknownEntity error"),
    }
}

#[test]
fn parser_provides_line_numbers_in_errors() {
    let input = "Title: Error Test
Swimlane: System
- Invalid: Entity
";
    
    let result = EventModelParser::new().parse(input);
    match result {
        Err(ParseError::UnexpectedToken { line, column, .. }) => {
            assert_eq!(line, 3);
            assert!(column > 0);
        }
        _ => panic!("Expected UnexpectedToken error with position"),
    }
}

#[test]
fn parser_handles_empty_swimlanes() {
    let input = "Title: Empty Swimlane
Swimlane: EmptyLane

Swimlane: HasContent
- Event: SomeEvent
";
    
    let result = EventModelParser::new().parse(input);
    assert!(result.is_ok());
    
    let model = result.unwrap();
    assert_eq!(model.swimlanes.len(), 2);
    assert_eq!(model.swimlanes[0].entities.len(), 0);
    assert_eq!(model.swimlanes[1].entities.len(), 1);
}

#[test]
fn parser_preserves_entity_order() {
    let input = "Title: Order Test
Swimlane: System
- Command: First
- Event: Second
- Policy: Third
- Projection: Fourth
";
    
    let result = EventModelParser::new().parse(input);
    assert!(result.is_ok());
    
    let model = result.unwrap();
    let entities = &model.swimlanes[0].entities;
    
    assert_eq!(entities.len(), 4);
    // Verify order is preserved
    match &entities[0] {
        event_modeler::event_model::entities::Entity::Command(c) => {
            assert_eq!(c.name.as_str(), "First");
        }
        _ => panic!("Expected Command"),
    }
    match &entities[1] {
        event_modeler::event_model::entities::Entity::Event(e) => {
            assert_eq!(e.name.as_str(), "Second");
        }
        _ => panic!("Expected Event"),
    }
}