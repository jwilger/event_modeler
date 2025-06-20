//! Event model diagram rendering module.
//!
//! This module provides a domain-specific API for creating event model diagrams
//! from parsed YAML event models. It follows a builder pattern with immutable
//! updates and focuses specifically on event modeling concepts rather than
//! generic SVG generation.

pub mod builder;
pub mod entities;
pub mod layout;
pub mod svg;

pub use builder::EventModelDiagram;
