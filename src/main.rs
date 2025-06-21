// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Event Modeler - A type-safe Event Modeling diagram generator.
//!
//! This application converts text-based Event Model descriptions (`.eventmodel` files)
//! into visual diagrams (SVG/PDF) suitable for documentation and analysis.
//!
//! ## Current Status
//!
//! **Early Development**: Module structure and type system are complete with comprehensive
//! domain modeling, but core functionality contains `todo!()` placeholders awaiting
//! implementation.
//!
//! ## Module Overview
//!
//! - [`event_model`] - Core Event Modeling concepts (Commands, Events, etc.)
//! - `diagram` - Visual representation and rendering (TODO: reimplement)
//! - `export` - Output formats (PDF, Markdown) (TODO: reimplement)
//! - [`infrastructure`] - Type safety and parsing utilities
//! - [`cli`] - Command-line interface
//!
//! ## Architecture
//!
//! This codebase follows strict type-driven design with zero runtime validation.
//! All validation happens once at system boundaries, and the rest of the code
//! works with types that maintain invariants by construction.

/// The version of Event Modeler, used as the default schema version for YAML files.
/// This must match the version in Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod cli;
// TODO: Re-enable when diagram module is complete
// pub mod diagram;
pub mod event_model;
// TODO: Re-enable when diagram module is complete
// pub mod export;
pub mod infrastructure;

use cli::{Cli, Error};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::from_args()?;
    cli.execute()
}
