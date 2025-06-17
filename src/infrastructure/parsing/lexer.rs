//! Lexical analyzer for Event Model DSL.
//!
//! The lexer converts raw text input into a stream of tokens that can be
//! consumed by the parser. It handles:
//!
//! - Keywords (swimlane, event, command, etc.)
//! - Literals (strings, numbers)
//! - Identifiers
//! - Punctuation (braces, brackets, colons)
//! - Whitespace and comments

use crate::infrastructure::types::{Identifier, NonEmptyString, NonNegativeInt};

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The type and value of the token.
    pub kind: TokenKind,
    /// The source location of the token.
    pub span: Span,
}

/// All possible token types in the Event Model DSL.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    /// `@title` keyword for model title.
    AtTitle,
    /// `@description` keyword for model description.
    AtDescription,
    /// `swimlane` keyword for defining swimlanes.
    Swimlane,
    /// `wireframe` keyword for UI mockups.
    Wireframe,
    /// `command` keyword for user actions.
    Command,
    /// `event` keyword for state changes.
    Event,
    /// `projection` keyword for read models.
    Projection,
    /// `query` keyword for data retrieval.
    Query,
    /// `automation` keyword for system triggers.
    Automation,
    /// `slice` keyword for vertical feature divisions.
    Slice,
    /// `inputs` keyword for input fields.
    Inputs,
    /// `outputs` keyword for output fields.
    Outputs,
    /// `actor` keyword for command actors.
    Actor,
    /// `payload` keyword for command data.
    Payload,
    /// `timestamp` keyword for event ordering.
    Timestamp,
    /// `data` keyword for event data.
    Data,
    /// `sources` keyword for projection sources.
    Sources,
    /// `fields` keyword for projection fields.
    Fields,
    /// `trigger` keyword for automation triggers.
    Trigger,
    /// `action` keyword for automation actions.
    Action,
    /// `contains` keyword for slice contents.
    Contains,
    /// `given_when_then` keyword for acceptance criteria.
    GivenWhenThen,
    /// `given` keyword for preconditions.
    Given,
    /// `when` keyword for actions.
    When,
    /// `then` keyword for expectations.
    Then,
    /// `link` keyword for documentation links.
    Link,

    // Literals
    /// String literal value.
    StringLiteral(NonEmptyString),
    /// Numeric literal value.
    NumberLiteral(NonNegativeInt),

    // Identifiers
    /// User-defined identifier.
    Identifier(Identifier),

    // Punctuation
    /// Opening brace `{`.
    LeftBrace,
    /// Closing brace `}`.
    RightBrace,
    /// Opening bracket `[`.
    LeftBracket,
    /// Closing bracket `]`.
    RightBracket,
    /// Colon `:`.
    Colon,
    /// Comma `,`.
    Comma,

    // End of file
    /// End of input marker.
    Eof,
}

/// Source location information for a token.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// Starting position of the token.
    pub start: Position,
    /// Ending position of the token.
    pub end: Position,
}

/// A position in the source text.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// Line number (1-indexed).
    pub line: LineNumber,
    /// Column number (1-indexed).
    pub column: ColumnNumber,
    /// Byte offset from the start of input.
    pub byte_offset: ByteOffset,
}

/// Line number in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineNumber(u32);

impl LineNumber {
    /// Creates a line number (1-indexed).
    ///
    /// # Panics
    /// Panics if line is 0.
    pub fn new(line: u32) -> Self {
        assert!(line > 0, "Line numbers must be 1-indexed");
        Self(line)
    }

    /// Returns the line number.
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Column number in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColumnNumber(u32);

impl ColumnNumber {
    /// Creates a column number (1-indexed).
    ///
    /// # Panics
    /// Panics if column is 0.
    pub fn new(column: u32) -> Self {
        assert!(column > 0, "Column numbers must be 1-indexed");
        Self(column)
    }

    /// Returns the column number.
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Byte offset in source text.
pub type ByteOffset = usize;

/// Lexical analyzer for Event Model DSL.
///
/// The lexer maintains internal state to track the current position
/// in the input and produces tokens on demand.
pub struct Lexer<'a> {
    input: &'a str,
    current_position: Position,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input string.
    pub fn new(_input: &'a str) -> Self {
        todo!()
    }

    /// Returns the next token from the input.
    ///
    /// # Errors
    ///
    /// Returns `LexError` if:
    /// - An unexpected character is encountered
    /// - A string literal is unterminated
    /// - A number literal is invalid
    /// - An identifier is invalid
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        todo!()
    }

    /// Returns the input string being lexed.
    pub fn input(&self) -> &'a str {
        self.input
    }

    /// Returns the current position in the input.
    pub fn current_position(&self) -> &Position {
        &self.current_position
    }
}

/// Errors that can occur during lexical analysis.
#[derive(Debug, thiserror::Error)]
pub enum LexError {
    /// An unexpected character was encountered.
    #[error("Unexpected character '{0}' at {1}:{2}")]
    UnexpectedCharacter(char, u32, u32),

    /// A string literal was not properly terminated.
    #[error("Unterminated string literal at {0}:{1}")]
    UnterminatedString(u32, u32),

    /// A number literal could not be parsed.
    #[error("Invalid number literal at {0}:{1}")]
    InvalidNumber(u32, u32),

    /// An identifier contains invalid characters.
    #[error("Invalid identifier at {0}:{1}")]
    InvalidIdentifier(u32, u32),
}
