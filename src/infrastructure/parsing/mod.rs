//! Text parsing infrastructure for Event Model files.
//!
//! This module handles the technical details of reading `.eventmodel` files
//! and converting them into structured data. This is infrastructure code
//! that supports the domain but is not part of the domain itself.
//!
//! The parsing process converts text like:
//!
//! ```text
//! title: "Order Processing System"
//!
//! swimlane Customer:
//!   wireframe "Place Order" -> command "Submit Order"
//! ```
//!
//! Into domain objects that represent the Event Model.

pub mod ast;
pub mod lexer;

use ast::EventModel;
use lexer::{LexError, Lexer};

/// Parser for Event Model DSL files.
///
/// The parser consumes tokens from a lexer and builds an AST
/// representing the event model structure.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    /// Parses the input and returns an `EventModel` AST.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if:
    /// - The lexer encounters invalid tokens
    /// - The syntax is invalid
    /// - Required elements are missing
    pub fn parse(&mut self) -> Result<EventModel, ParseError> {
        todo!()
    }

    /// Returns a reference to the internal lexer.
    pub fn lexer(&self) -> &Lexer<'a> {
        &self.lexer
    }
}

/// Errors that can occur during parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// An error occurred during lexical analysis.
    #[error("Lexical error: {0}")]
    LexError(#[from] LexError),

    /// A syntax error occurred at a specific position.
    #[error("Syntax error at {0}:{1}: expected {2}")]
    SyntaxError(u32, u32, String),

    /// A semantic error was detected in the model.
    #[error("Semantic error: {0}")]
    SemanticError(String),
}
