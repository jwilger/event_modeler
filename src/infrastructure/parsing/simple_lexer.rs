//! Simple lexer for the Event Model format used in tests.
//!
//! This lexer handles the simple format:
//! ```
//! Title: Model Name
//! Swimlane: Lane Name
//! - Entity: EntityName
//! Connection -> Target
//! ```

/// Token types for the simple Event Model format.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Title,
    Swimlane,
    Command,
    Event,
    Projection,
    Policy,
    ExternalSystem,
    Aggregate,

    // Punctuation
    Colon,
    Dash,
    Arrow,
    Newline,

    // Content
    Text(String),
    Indent(usize),
}

/// A token with its position in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

/// Simple lexer for Event Model files.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Creates a new lexer for the given input.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Returns the next token from the input.
    pub fn next_token(&mut self) -> Option<Token> {
        if self.position >= self.input.len() {
            return None;
        }

        let token_start_line = self.line;
        let token_start_column = self.column;

        // Check for newline
        if self.current_char() == Some('\n') {
            self.advance();
            return Some(Token {
                kind: TokenKind::Newline,
                line: token_start_line,
                column: token_start_column,
            });
        }

        // Check for indentation at start of line
        if self.column == 1
            && (self.current_char() == Some(' ') || self.current_char() == Some('\t'))
        {
            let indent_level = self.count_indentation();
            if indent_level > 0 {
                return Some(Token {
                    kind: TokenKind::Indent(indent_level),
                    line: token_start_line,
                    column: token_start_column,
                });
            }
        }

        self.skip_whitespace_except_newline();

        // Check for punctuation
        match self.current_char() {
            Some(':') => {
                self.advance();
                return Some(Token {
                    kind: TokenKind::Colon,
                    line: token_start_line,
                    column: token_start_column,
                });
            }
            Some('-') => {
                self.advance();
                // Check for arrow ->
                if self.current_char() == Some('>') {
                    self.advance();
                    return Some(Token {
                        kind: TokenKind::Arrow,
                        line: token_start_line,
                        column: token_start_column,
                    });
                } else {
                    return Some(Token {
                        kind: TokenKind::Dash,
                        line: token_start_line,
                        column: token_start_column,
                    });
                }
            }
            _ => {}
        }

        // Read a word
        let word = self.read_word();

        // Check if it's a keyword
        let kind = match word.as_str() {
            "Title" => TokenKind::Title,
            "Swimlane" => TokenKind::Swimlane,
            "Command" => TokenKind::Command,
            "Event" => TokenKind::Event,
            "Projection" => TokenKind::Projection,
            "Policy" => TokenKind::Policy,
            "External" => {
                // Check for "External System"
                self.skip_whitespace_except_newline();
                if self.peek_word() == "System" {
                    self.read_word(); // consume "System"
                    TokenKind::ExternalSystem
                } else {
                    TokenKind::Text(word)
                }
            }
            "Aggregate" => TokenKind::Aggregate,
            _ => TokenKind::Text(word),
        };

        Some(Token {
            kind,
            line: token_start_line,
            column: token_start_column,
        })
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.input[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn skip_whitespace_except_newline(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn count_indentation(&mut self) -> usize {
        let mut count = 0;
        while let Some(ch) = self.current_char() {
            if ch == ' ' {
                count += 1;
                self.advance();
            } else if ch == '\t' {
                count += 4; // treat tab as 4 spaces
                self.advance();
            } else {
                break;
            }
        }
        count
    }

    fn read_word(&mut self) -> String {
        let mut word = String::new();

        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        word
    }

    fn peek_word(&self) -> String {
        let mut temp_position = self.position;
        let mut word = String::new();

        while temp_position < self.input.len() {
            let ch = self.input[temp_position];
            if ch.is_alphanumeric() || ch == '_' {
                word.push(ch);
                temp_position += 1;
            } else {
                break;
            }
        }

        word
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_tokenizes_title_section() {
        let input = "Title: My Event Model";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Title);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        // After Title:, we get individual tokens
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Text("My".to_string())
        );
        // "Event" is recognized as a keyword even in title context
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Event);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Text("Model".to_string())
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
}
