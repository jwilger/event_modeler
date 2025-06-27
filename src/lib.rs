// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Event Modeler - A type-safe Event Modeling diagram generator.
//!
//! This crate provides tools for generating professional Event Modeling diagrams
//! from YAML-based event model descriptions.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// TODO: Re-enable missing_docs after implementing incremental rendering
// #![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// The version of Event Modeler, used as the default schema version for YAML files.
/// This must match the version in Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Command-line interface.
pub mod cli;

/// Diagram generation and rendering.
pub mod diagram;

/// Event model domain types and operations.
pub mod event_model;

/// Infrastructure and utility types.
pub mod infrastructure;

/// Connector routing using libavoid.
pub mod routing;
