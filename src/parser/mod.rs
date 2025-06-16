pub mod lexer;
pub mod ast;

use ast::EventModel;
use lexer::{Lexer, LexError};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }
    
    pub fn parse(&mut self) -> Result<EventModel, ParseError> {
        todo!()
    }
    
    pub fn lexer(&self) -> &Lexer<'a> {
        &self.lexer
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Lexical error: {0}")]
    LexError(#[from] LexError),
    
    #[error("Syntax error at {0}:{1}: expected {2}")]
    SyntaxError(u32, u32, String),
    
    #[error("Semantic error: {0}")]
    SemanticError(String),
}