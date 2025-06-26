//! Simple parser for Event Model text format.
//!
//! This parser uses the typestate pattern to ensure correct parsing order
//! and converts tokenized input into a structured EventModel.

use crate::infrastructure::types::{Identifier, NonEmptyString};

use super::simple_lexer::{Lexer, TokenKind};

/// Parsed event model structure.
#[derive(Debug)]
pub struct ParsedEventModel {
    pub title: NonEmptyString,
    pub swimlanes: Vec<ParsedSwimlane>,
    pub connectors: Vec<ParsedConnector>,
}

/// Parsed swimlane structure.
#[derive(Debug)]
pub struct ParsedSwimlane {
    pub name: NonEmptyString,
    pub entities: Vec<ParsedEntity>,
}

/// Parsed entity types.
#[derive(Debug, Clone)]
pub enum ParsedEntity {
    Command(NonEmptyString),
    Event(NonEmptyString),
    Projection(NonEmptyString),
    Policy(NonEmptyString),
    ExternalSystem(NonEmptyString),
    Aggregate(NonEmptyString),
}

impl ParsedEntity {
    /// Gets the name of the entity.
    pub fn name(&self) -> &str {
        match self {
            ParsedEntity::Command(name)
            | ParsedEntity::Event(name)
            | ParsedEntity::Projection(name)
            | ParsedEntity::Policy(name)
            | ParsedEntity::ExternalSystem(name)
            | ParsedEntity::Aggregate(name) => name.as_str(),
        }
    }
}

/// Parsed connector between entities.
#[derive(Debug)]
pub struct ParsedConnector {
    pub from: Identifier,
    pub to: Identifier,
}

/// Parser errors.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("Missing title section - models must start with 'Title: <name>'")]
    MissingTitle,

    #[error("Unexpected token: expected {expected}, found {found} at line {line}")]
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    #[error("Duplicate entity name: '{0}' is already defined")]
    DuplicateEntity(String),

    #[error("Unknown entity reference: '{0}' is not defined")]
    UnknownEntity(String),

    #[error("Invalid identifier: '{0}'")]
    InvalidIdentifier(String),
}

/// Simple event model parser.
pub struct EventModelParser;

impl EventModelParser {
    /// Creates a new parser instance.
    pub fn new() -> Self {
        Self
    }

    /// Parses the input text into a ParsedEventModel.
    pub fn parse(&self, input: &str) -> Result<ParsedEventModel, ParseError> {
        let mut lexer = Lexer::new(input);
        let mut parser_state = ParserState::new();

        // Parse title
        parser_state.parse_title(&mut lexer)?;

        // Parse swimlanes and connectors
        loop {
            // Skip any leading newlines
            let mut token = None;
            while let Some(t) = lexer.next_token() {
                if t.kind != TokenKind::Newline {
                    token = Some(t);
                    break;
                }
            }

            let Some(token) = token else {
                break; // End of input
            };

            match token.kind {
                TokenKind::Swimlane => {
                    parser_state.parse_swimlane(&mut lexer)?;
                }
                TokenKind::Text(from) => {
                    // This might be a connector
                    if let Some(arrow) = lexer.next_token() {
                        if arrow.kind == TokenKind::Arrow {
                            parser_state.parse_connector(&mut lexer, from)?;
                        } else {
                            return Err(ParseError::UnexpectedToken {
                                expected: "-> or newline".to_string(),
                                found: format!("{:?}", arrow.kind),
                                line: arrow.line,
                                column: arrow.column,
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "Swimlane or connector".to_string(),
                        found: format!("{:?}", token.kind),
                        line: token.line,
                        column: token.column,
                    });
                }
            }
        }

        Ok(parser_state.build())
    }
}

impl Default for EventModelParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal parser state.
struct ParserState {
    title: Option<NonEmptyString>,
    swimlanes: Vec<ParsedSwimlane>,
    connectors: Vec<ParsedConnector>,
    entity_names: std::collections::HashSet<String>,
}

impl ParserState {
    fn new() -> Self {
        Self {
            title: None,
            swimlanes: Vec::new(),
            connectors: Vec::new(),
            entity_names: std::collections::HashSet::new(),
        }
    }

    fn parse_title(&mut self, lexer: &mut Lexer) -> Result<(), ParseError> {
        let title_token = lexer.next_token().ok_or(ParseError::MissingTitle)?;

        if title_token.kind != TokenKind::Title {
            return Err(ParseError::MissingTitle);
        }

        // Expect colon
        let colon = lexer.next_token().ok_or(ParseError::UnexpectedToken {
            expected: ":".to_string(),
            found: "end of input".to_string(),
            line: title_token.line,
            column: title_token.column,
        })?;

        if colon.kind != TokenKind::Colon {
            return Err(ParseError::UnexpectedToken {
                expected: ":".to_string(),
                found: format!("{:?}", colon.kind),
                line: colon.line,
                column: colon.column,
            });
        }

        // Read the rest of the line as the title
        let title_text = lexer.read_line();
        if title_text.is_empty() {
            return Err(ParseError::InvalidIdentifier("Empty title".to_string()));
        }

        self.title = Some(
            NonEmptyString::parse(title_text.clone())
                .map_err(|_| ParseError::InvalidIdentifier(title_text))?,
        );

        Ok(())
    }

    fn parse_swimlane(&mut self, lexer: &mut Lexer) -> Result<(), ParseError> {
        // We already consumed the Swimlane token

        // Expect colon
        let colon = lexer.next_token().ok_or(ParseError::UnexpectedToken {
            expected: ":".to_string(),
            found: "end of input".to_string(),
            line: 0,
            column: 0,
        })?;

        if colon.kind != TokenKind::Colon {
            return Err(ParseError::UnexpectedToken {
                expected: ":".to_string(),
                found: format!("{:?}", colon.kind),
                line: colon.line,
                column: colon.column,
            });
        }

        // Read the rest of the line as the swimlane name
        let name = lexer.read_line();
        if name.is_empty() {
            return Err(ParseError::InvalidIdentifier(
                "Empty swimlane name".to_string(),
            ));
        }

        let swimlane_name =
            NonEmptyString::parse(name.clone()).map_err(|_| ParseError::InvalidIdentifier(name))?;

        // Parse entities
        let mut entities = Vec::new();

        // Look for entities starting with dash
        loop {
            // Skip whitespace and check for dash
            match lexer.next_token() {
                Some(token) if token.kind == TokenKind::Dash => {
                    // Parse entity
                    let entity = self.parse_entity(lexer)?;
                    entities.push(entity);
                    // read_line already consumed the newline
                }
                Some(token) if token.kind == TokenKind::Newline => {
                    // Empty line ends the swimlane
                    break;
                }
                Some(_) | None => {
                    // Not a dash, so no more entities in this swimlane
                    break;
                }
            }
        }

        let swimlane = ParsedSwimlane {
            name: swimlane_name,
            entities,
        };
        self.swimlanes.push(swimlane);

        Ok(())
    }

    fn parse_entity(&mut self, lexer: &mut Lexer) -> Result<ParsedEntity, ParseError> {
        // We already consumed the dash

        // Get entity type
        let type_token = lexer.next_token().ok_or(ParseError::UnexpectedToken {
            expected: "entity type".to_string(),
            found: "end of input".to_string(),
            line: 0,
            column: 0,
        })?;

        // Expect colon
        let colon = lexer.next_token().ok_or(ParseError::UnexpectedToken {
            expected: ":".to_string(),
            found: "end of input".to_string(),
            line: type_token.line,
            column: type_token.column,
        })?;

        if colon.kind != TokenKind::Colon {
            return Err(ParseError::UnexpectedToken {
                expected: ":".to_string(),
                found: format!("{:?}", colon.kind),
                line: colon.line,
                column: colon.column,
            });
        }

        // Read the rest of the line as the entity name
        let name = lexer.read_line();
        if name.is_empty() {
            return Err(ParseError::InvalidIdentifier(
                "Empty entity name".to_string(),
            ));
        }

        // Check for duplicate
        if !self.entity_names.insert(name.clone()) {
            return Err(ParseError::DuplicateEntity(name));
        }

        let entity_name = NonEmptyString::parse(name.clone())
            .map_err(|_| ParseError::InvalidIdentifier(name.clone()))?;

        // Create entity based on type
        let entity = match type_token.kind {
            TokenKind::Command => ParsedEntity::Command(entity_name),
            TokenKind::Event => ParsedEntity::Event(entity_name),
            TokenKind::Projection => ParsedEntity::Projection(entity_name),
            TokenKind::Policy => ParsedEntity::Policy(entity_name),
            TokenKind::ExternalSystem => ParsedEntity::ExternalSystem(entity_name),
            TokenKind::Aggregate => ParsedEntity::Aggregate(entity_name),
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "entity type".to_string(),
                    found: format!("{:?}", type_token.kind),
                    line: type_token.line,
                    column: type_token.column,
                });
            }
        };

        Ok(entity)
    }

    fn parse_connector(&mut self, lexer: &mut Lexer, from: String) -> Result<(), ParseError> {
        // We already have the from and consumed the arrow

        // Get target
        let to_token = lexer.next_token().ok_or(ParseError::UnexpectedToken {
            expected: "connector target".to_string(),
            found: "end of input".to_string(),
            line: 0,
            column: 0,
        })?;

        let to = match to_token.kind {
            TokenKind::Text(text) => text,
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "connector target".to_string(),
                    found: format!("{:?}", to_token.kind),
                    line: to_token.line,
                    column: to_token.column,
                });
            }
        };

        // Verify both entities exist
        if !self.entity_names.contains(&from) {
            return Err(ParseError::UnknownEntity(from));
        }
        if !self.entity_names.contains(&to) {
            return Err(ParseError::UnknownEntity(to));
        }

        let from_id = Identifier::parse(from.clone())
            .map_err(|_| ParseError::InvalidIdentifier(from.clone()))?;
        let to_id =
            Identifier::parse(to.clone()).map_err(|_| ParseError::InvalidIdentifier(to.clone()))?;

        let connector = ParsedConnector {
            from: from_id,
            to: to_id,
        };
        self.connectors.push(connector);

        Ok(())
    }

    fn build(self) -> ParsedEventModel {
        let title = self.title.expect("Title must be set");

        ParsedEventModel {
            title,
            swimlanes: self.swimlanes,
            connectors: self.connectors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_requires_title_section_first() {
        let input = "Swimlane: Test";
        let parser = EventModelParser::new();
        let result = parser.parse(input);

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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
        if let Err(e) = &result {
            panic!("Parse error: {e:?}");
        }
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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
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
Invalid: Entity
";

        let parser = EventModelParser::new();
        let result = parser.parse(input);
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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
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

        let parser = EventModelParser::new();
        let result = parser.parse(input);
        assert!(result.is_ok());

        let model = result.unwrap();
        let entities = &model.swimlanes[0].entities;

        assert_eq!(entities.len(), 4);
        // Verify order is preserved
        match &entities[0] {
            ParsedEntity::Command(name) => {
                assert_eq!(name.as_str(), "First");
            }
            _ => panic!("Expected Command"),
        }
        match &entities[1] {
            ParsedEntity::Event(name) => {
                assert_eq!(name.as_str(), "Second");
            }
            _ => panic!("Expected Event"),
        }
    }
}
