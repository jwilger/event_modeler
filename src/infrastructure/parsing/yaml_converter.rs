// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! Conversion from YAML parsing types to domain types.
//!
//! This module handles the transformation from the intermediate YAML parsing
//! representation to the strongly-typed domain model.

use crate::event_model::yaml_types as domain;
use crate::infrastructure::parsing::yaml_parser as parsing;

/// Converts a parsed YAML model into the domain representation.
///
/// This function performs all necessary validation and transformation:
/// - Validates all entity references (swimlanes, etc.)
/// - Converts stringly-typed data to strongly-typed domain objects
/// - Ensures all invariants are met
pub fn convert_yaml_to_domain(
    _yaml: parsing::YamlEventModel,
) -> Result<domain::YamlEventModel, ConversionError> {
    // TODO: Implement full conversion from parsing types to domain types
    // This requires:
    // 1. Converting all string types to nutype wrappers via NonEmptyString
    // 2. Validating entity references
    // 3. Converting test scenarios
    // 4. Converting UI component hierarchies
    // 5. Converting slice connections

    todo!("YAML to domain conversion not yet implemented")
}

/// Errors that can occur during conversion.
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    /// A required field was empty.
    #[error("Field '{0}' cannot be empty")]
    EmptyField(String),

    /// An unknown swimlane was referenced.
    #[error("Unknown swimlane reference: {0}")]
    UnknownSwimlane(String),

    /// A slice connection was invalid.
    #[error("Invalid connection syntax: {0}")]
    InvalidConnection(String),

    /// A collection that must be non-empty was empty.
    #[error("Collection '{0}' must not be empty")]
    EmptyCollection(String),
}
