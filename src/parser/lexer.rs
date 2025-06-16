use nutype::nutype;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    AtTitle,
    AtDescription,
    Swimlane,
    Wireframe,
    Command,
    Event,
    Projection,
    Query,
    Automation,
    Slice,
    Inputs,
    Outputs,
    Actor,
    Payload,
    Timestamp,
    Data,
    Sources,
    Fields,
    Trigger,
    Action,
    Contains,
    GivenWhenThen,
    Given,
    When,
    Then,
    Link,
    
    // Literals
    StringLiteral(LexedString),
    NumberLiteral(LexedNumber),
    
    // Identifiers
    Identifier(LexedIdentifier),
    
    // Punctuation
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    
    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: LineNumber,
    pub column: ColumnNumber,
    pub byte_offset: ByteOffset,
}

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct LineNumber(u32);

#[nutype(
    validate(greater = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct ColumnNumber(u32);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord),
)]
pub struct ByteOffset(usize);

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, PartialEq),
)]
pub struct LexedString(String);

#[nutype(
    validate(greater_or_equal = 0),
    derive(Debug, Clone, PartialEq),
)]
pub struct LexedNumber(u32);

#[nutype(
    validate(regex = r"^[a-zA-Z_][a-zA-Z0-9_]*$"),
    derive(Debug, Clone, PartialEq),
)]
pub struct LexedIdentifier(String);

pub struct Lexer<'a> {
    input: &'a str,
    current_position: Position,
}

impl<'a> Lexer<'a> {
    pub fn new(_input: &'a str) -> Self {
        todo!()
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        todo!()
    }
    
    pub fn input(&self) -> &'a str {
        self.input
    }
    
    pub fn current_position(&self) -> &Position {
        &self.current_position
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LexError {
    #[error("Unexpected character '{0}' at {1}:{2}")]
    UnexpectedCharacter(char, u32, u32),
    
    #[error("Unterminated string literal at {0}:{1}")]
    UnterminatedString(u32, u32),
    
    #[error("Invalid number literal at {0}:{1}")]
    InvalidNumber(u32, u32),
    
    #[error("Invalid identifier at {0}:{1}")]
    InvalidIdentifier(u32, u32),
}