//! Tests for the lexer component of the parsing infrastructure.

use event_modeler::infrastructure::parsing::lexer::{Lexer, Token, TokenKind};

#[test]
fn lexer_tokenizes_title_section() {
    let input = "Title: My Event Model";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Title);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
    assert_eq!(
        lexer.next_token().unwrap(),
        Token {
            kind: TokenKind::Text("My Event Model".to_string()),
            line: 1,
            column: 8
        }
    );
    assert!(lexer.next_token().is_none());
}

#[test]
fn lexer_tokenizes_swimlane_definition() {
    let input = "Swimlane: Customer
- Command: PlaceOrder
- Event: OrderPlaced
";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Swimlane);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Text("Customer".to_string())
    );
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Dash);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Command);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Text("PlaceOrder".to_string())
    );
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Dash);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Event);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Text("OrderPlaced".to_string())
    );
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
    assert!(lexer.next_token().is_none());
}

#[test]
fn lexer_tracks_line_and_column_numbers() {
    let input = "Title: Test\nSwimlane: System";
    let mut lexer = Lexer::new(input);
    
    let token = lexer.next_token().unwrap();
    assert_eq!(token.line, 1);
    assert_eq!(token.column, 1);
    
    lexer.next_token(); // Colon
    lexer.next_token(); // Text
    lexer.next_token(); // Newline
    
    let token = lexer.next_token().unwrap();
    assert_eq!(token.line, 2);
    assert_eq!(token.column, 1);
}

#[test]
fn lexer_handles_indented_content() {
    let input = "Swimlane: Test
  - Command: DoSomething
    - Event: SomethingDone
";
    let mut lexer = Lexer::new(input);
    
    // Skip to indented content
    for _ in 0..4 {
        lexer.next_token();
    }
    
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Indent(2));
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Dash);
}

#[test]
fn lexer_handles_connector_syntax() {
    let input = "OrderPlaced -> UpdateInventory";
    let mut lexer = Lexer::new(input);
    
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Text("OrderPlaced".to_string())
    );
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Arrow);
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Text("UpdateInventory".to_string())
    );
}

#[test]
fn lexer_handles_all_entity_types() {
    let entity_types = vec![
        ("Command:", TokenKind::Command),
        ("Event:", TokenKind::Event),
        ("Projection:", TokenKind::Projection),
        ("Policy:", TokenKind::Policy),
        ("External System:", TokenKind::ExternalSystem),
        ("Aggregate:", TokenKind::Aggregate),
    ];
    
    for (input, expected) in entity_types {
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token().unwrap().kind, expected);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
    }
}