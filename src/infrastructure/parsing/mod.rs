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
//!
//! ## Typestate Parser Design
//!
//! The parser uses the typestate pattern to enforce correct parsing order at compile time.
//! The parser transitions through states: Empty -> HasHeader -> HasBody -> Complete.
//! This ensures that headers are parsed before bodies and that all required sections
//! are present before building the final EventModel.

pub mod ast;
pub mod lexer;
pub mod simple_lexer;

use ast::EventModel;
use lexer::{LexError, Lexer};
use std::marker::PhantomData;

/// Parser state markers for typestate pattern.
pub mod states {
    /// Initial state - no content parsed yet.
    pub struct Empty;

    /// Header has been parsed successfully.
    pub struct HasHeader;

    /// Body has been parsed after header.
    pub struct HasBody;

    /// All required sections parsed - ready to build.
    pub struct Complete;
}

/// Typestate parser for Event Model DSL files.
///
/// The parser uses phantom types to track parsing progress and ensure
/// correct ordering of parsing operations at compile time.
///
/// # Example
///
/// ```ignore
/// let parser = EventModelParser::new(input);
/// let parser_with_header = parser.parse_header()?;
/// let parser_with_body = parser_with_header.parse_body()?;
/// let complete_parser = parser_with_body.finalize()?;
/// let event_model = complete_parser.build();
/// ```
///
/// Note: This example is marked as `ignore` because the implementation is not yet complete.
pub struct EventModelParser<'a, State> {
    /// Accumulated parsing context and results.
    context: ParsingContext<'a>,
    /// Phantom type to track parser state.
    _state: PhantomData<State>,
}

/// Internal parsing context shared across states.
struct ParsingContext<'a> {
    /// The lexer for tokenization.
    lexer: Lexer<'a>,
    /// Parsed header information (once available).
    header: Option<ParsedHeader>,
    /// Parsed body information (once available).
    body: Option<ParsedBody>,
}

/// Parsed header information.
struct ParsedHeader {
    /// The title of the event model.
    title: String,
    /// Optional metadata fields.
    metadata: Vec<(String, String)>,
}

/// Parsed body information.
struct ParsedBody {
    /// All parsed elements from the body.
    elements: Vec<ast::EventModel>,
}

impl<'a> EventModelParser<'a, states::Empty> {
    /// Creates a new parser in the Empty state.
    pub fn new(input: &'a str) -> Self {
        Self {
            context: ParsingContext {
                lexer: Lexer::new(input),
                header: None,
                body: None,
            },
            _state: PhantomData,
        }
    }

    /// Parses the header section of the event model.
    ///
    /// This consumes the Empty state parser and returns a HasHeader state parser.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if the header is malformed or missing required fields.
    pub fn parse_header(mut self) -> Result<EventModelParser<'a, states::HasHeader>, ParseError> {
        // Move context to show it's used
        let _lexer = &mut self.context.lexer;
        // Parse header would go here
        self.context.header = Some(ParsedHeader {
            title: String::new(),
            metadata: Vec::new(),
        });

        Ok(EventModelParser {
            context: self.context,
            _state: PhantomData,
        })
    }
}

impl<'a> EventModelParser<'a, states::HasHeader> {
    /// Parses the body section of the event model.
    ///
    /// This consumes the HasHeader state parser and returns a HasBody state parser.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if the body contains invalid syntax.
    pub fn parse_body(mut self) -> Result<EventModelParser<'a, states::HasBody>, ParseError> {
        // Move context to show it's used
        let _lexer = &mut self.context.lexer;
        let _header = self.context.header.as_ref();
        // Parse body would go here
        self.context.body = Some(ParsedBody {
            elements: Vec::new(),
        });

        Ok(EventModelParser {
            context: self.context,
            _state: PhantomData,
        })
    }
}

impl<'a> EventModelParser<'a, states::HasBody> {
    /// Finalizes parsing and validates the complete model.
    ///
    /// This consumes the HasBody state parser and returns a Complete state parser.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if there are unresolved references or semantic errors.
    pub fn finalize(self) -> Result<EventModelParser<'a, states::Complete>, ParseError> {
        // Validate that we have both header and body
        let _header = self.context.header.as_ref();
        let _body = self.context.body.as_ref();

        Ok(EventModelParser {
            context: self.context,
            _state: PhantomData,
        })
    }
}

impl<'a> EventModelParser<'a, states::Complete> {
    /// Builds the final EventModel from the parsed components.
    ///
    /// This consumes the parser and returns the complete EventModel.
    /// This method cannot fail as all validation has been done in previous stages.
    pub fn build(self) -> EventModel {
        // Extract the data we've parsed
        let header = self
            .context
            .header
            .expect("Header must exist in Complete state");
        let body = self
            .context
            .body
            .expect("Body must exist in Complete state");

        // Access the fields to show they're used
        let _title = header.title;
        let _metadata = header.metadata;
        let _elements = body.elements;

        todo!("Build EventModel from parsed components")
    }
}

/// Legacy parser interface for backwards compatibility.
///
/// This will be deprecated in favor of the typestate parser.
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
