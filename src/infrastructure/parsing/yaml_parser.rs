// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! YAML parsing types for Event Model format.
//!
//! This module contains Serde-compatible types that map directly to the YAML structure
//! of `.eventmodel` files. These types are used as an intermediate representation
//! before conversion to domain types.

use crate::VERSION;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure of an Event Model YAML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlEventModel {
    /// Optional version field, defaults to current application version
    #[serde(default)]
    pub version: Option<String>,

    /// The name of the workflow being modeled
    pub workflow: String,

    /// Swimlane definitions
    pub swimlanes: Vec<YamlSwimlane>,

    /// Event definitions
    #[serde(default)]
    pub events: HashMap<String, YamlEvent>,

    /// Command definitions
    #[serde(default)]
    pub commands: HashMap<String, YamlCommand>,

    /// View definitions
    #[serde(default)]
    pub views: HashMap<String, YamlView>,

    /// Projection definitions
    #[serde(default)]
    pub projections: HashMap<String, YamlProjection>,

    /// Query definitions
    #[serde(default)]
    pub queries: HashMap<String, YamlQuery>,

    /// Automation definitions
    #[serde(default)]
    pub automations: HashMap<String, YamlAutomation>,

    /// Slice definitions
    #[serde(default)]
    pub slices: IndexMap<String, Vec<String>>,
}

/// Swimlane definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YamlSwimlane {
    /// Simple format: just a name
    Simple(String),
    /// Map format: key is identifier, value is display name
    Map(HashMap<String, String>),
}

/// Event entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlEvent {
    /// Event description
    pub description: String,

    /// Swimlane this event belongs to
    pub swimlane: String,

    /// Event data schema
    #[serde(default)]
    pub data: HashMap<String, YamlField>,
}

/// Command entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlCommand {
    /// Command description
    pub description: String,

    /// Swimlane this command belongs to
    pub swimlane: String,

    /// Command data schema
    #[serde(default)]
    pub data: HashMap<String, YamlField>,

    /// Test scenarios
    #[serde(default)]
    pub tests: HashMap<String, YamlTestScenario>,
}

/// View entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlView {
    /// View description
    pub description: String,

    /// Swimlane this view belongs to
    pub swimlane: String,

    /// UI components
    #[serde(default)]
    pub components: Vec<YamlComponent>,
}

/// Projection entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlProjection {
    /// Projection description
    pub description: String,

    /// Swimlane this projection belongs to
    pub swimlane: String,

    /// Projection fields
    #[serde(default)]
    pub fields: HashMap<String, String>,
}

/// Query entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlQuery {
    /// Swimlane this query belongs to
    pub swimlane: String,

    /// Query inputs
    #[serde(default)]
    pub inputs: HashMap<String, String>,

    /// Query outputs
    pub outputs: YamlQueryOutput,
}

/// Query output structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlQueryOutput {
    /// One-of output variants
    pub one_of: HashMap<String, YamlQueryVariant>,
}

/// Query output variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YamlQueryVariant {
    /// Simple type reference
    Simple(String),
    /// Complex output with fields
    Complex(HashMap<String, String>),
}

/// Automation entity definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlAutomation {
    /// Swimlane this automation belongs to
    pub swimlane: String,
}

/// Field definition in data schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YamlField {
    /// Simple type reference
    Simple(String),
    /// Complex field with properties
    Complex {
        #[serde(rename = "type")]
        field_type: String,
        #[serde(rename = "stream-id")]
        #[serde(default)]
        stream_id: bool,
        #[serde(default)]
        generated: bool,
    },
}

/// Test scenario definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlTestScenario {
    /// Given section - initial state
    #[serde(rename = "Given")]
    #[serde(default)]
    pub given: Vec<YamlTestStep>,

    /// When section - action to test
    #[serde(rename = "When")]
    pub when: Vec<YamlTestStep>,

    /// Then section - expected outcome
    #[serde(rename = "Then")]
    pub then: Vec<YamlTestStep>,
}

/// Test step in a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlTestStep {
    /// Entity name and its data
    #[serde(flatten)]
    pub step: HashMap<String, HashMap<String, String>>,
}

/// UI component definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum YamlComponent {
    /// Simple component with just a name and type
    Simple {
        #[serde(flatten)]
        component: HashMap<String, String>,
    },
    /// Complex component with nested structure
    Complex {
        #[serde(flatten)]
        component: HashMap<String, YamlComplexComponent>,
    },
}

/// Complex UI component with nested properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YamlComplexComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
    #[serde(default)]
    pub actions: Vec<String>,
}

/// Errors that can occur during YAML parsing.
#[derive(Debug, thiserror::Error)]
pub enum YamlParseError {
    /// YAML syntax error without location information.
    #[error("YAML syntax error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Version incompatibility error.
    #[error(
        "Schema version mismatch: file version {file_version} is not compatible with application version {app_version}"
    )]
    VersionMismatch {
        file_version: String,
        app_version: String,
    },

    /// YAML parsing error with location information.
    #[error("YAML error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },
}

/// Parses a YAML event model from a string.
///
/// This function:
/// 1. Parses the YAML into intermediate types
/// 2. Validates the schema version (if present)
/// 3. Returns the parsed model or an error
pub fn parse_yaml(input: &str) -> Result<YamlEventModel, YamlParseError> {
    // Parse the YAML
    let mut model: YamlEventModel = serde_yaml::from_str(input).map_err(|e| {
        // Extract location information if available
        if let Some(location) = e.location() {
            YamlParseError::ParseError {
                line: location.line(),
                column: location.column(),
                message: e.to_string(),
            }
        } else {
            YamlParseError::YamlError(e)
        }
    })?;

    // If no version specified, use current version
    if model.version.is_none() {
        model.version = Some(VERSION.to_string());
    }

    // For now, we accept any version since we're pre-1.0
    // Post-1.0, we'll add version compatibility checks here

    Ok(model)
}

/// Checks if a file version is compatible with the current application version.
///
/// Currently always returns true as we're pre-1.0 and have no compatibility guarantees.
/// Post-1.0, this will implement semantic versioning checks.
#[allow(dead_code)] // Will be used post-1.0 for version compatibility checks
fn is_version_compatible(file_version: &str, app_version: &str) -> bool {
    // Pre-1.0: Accept any version
    let _ = (file_version, app_version); // Silence unused warnings
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_event_model_deserializes_minimal_file() {
        let yaml = r#"
workflow: Test Workflow
swimlanes:
  - test: "Test Lane"
"#;
        let model: YamlEventModel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(model.workflow, "Test Workflow");
        assert_eq!(model.swimlanes.len(), 1);
    }

    #[test]
    fn yaml_event_model_deserializes_with_version() {
        let yaml = r#"
version: "0.3.0"
workflow: Test Workflow
swimlanes:
  - test: "Test Lane"
"#;
        let model: YamlEventModel = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(model.version, Some("0.3.0".to_string()));
    }

    #[test]
    fn yaml_field_deserializes_simple_type() {
        let yaml = "String";
        let field: YamlField = serde_yaml::from_str(yaml).unwrap();
        match field {
            YamlField::Simple(s) => assert_eq!(s, "String"),
            _ => panic!("Expected simple field"),
        }
    }

    #[test]
    fn yaml_field_deserializes_complex_type() {
        let yaml = r#"
type: UserAccountId
stream-id: true
generated: true
"#;
        let field: YamlField = serde_yaml::from_str(yaml).unwrap();
        match field {
            YamlField::Complex {
                field_type,
                stream_id,
                generated,
            } => {
                assert_eq!(field_type, "UserAccountId");
                assert!(stream_id);
                assert!(generated);
            }
            _ => panic!("Expected complex field"),
        }
    }

    #[test]
    fn parse_yaml_adds_default_version() {
        let yaml = r#"
workflow: Test Workflow
swimlanes:
  - test: "Test Lane"
"#;
        let model = parse_yaml(yaml).unwrap();
        assert_eq!(model.version, Some(VERSION.to_string()));
    }

    #[test]
    fn parse_yaml_preserves_explicit_version() {
        let yaml = r#"
version: "0.2.0"
workflow: Test Workflow
swimlanes:
  - test: "Test Lane"
"#;
        let model = parse_yaml(yaml).unwrap();
        assert_eq!(model.version, Some("0.2.0".to_string()));
    }

    #[test]
    fn parse_yaml_handles_syntax_errors() {
        let yaml = r#"
workflow: Test Workflow
swimlanes
  - test: "Test Lane"  # Missing colon
"#;
        let result = parse_yaml(yaml);
        assert!(matches!(result, Err(YamlParseError::ParseError { .. })));
    }

    #[test]
    fn parse_yaml_extracts_error_location() {
        // Use actual invalid YAML syntax - unclosed quote
        let yaml = r#"workflow: Test Workflow
swimlanes:
  - test: "Test Lane
  - backend: "Backend"
"#;
        let result = parse_yaml(yaml);
        match result {
            Err(YamlParseError::ParseError {
                line,
                column,
                message,
            }) => {
                // serde_yaml reports 1-indexed line numbers
                println!("Error at line {}, column {}: {}", line, column, message);
                assert!(line > 0); // Line should be greater than 0
                assert!(column > 0); // Column should be greater than 0
                assert!(!message.is_empty());
            }
            Err(e) => panic!("Expected ParseError but got: {:?}", e),
            Ok(_) => panic!("Expected an error but parsing succeeded"),
        }
    }

    #[test]
    fn parse_yaml_location_with_duplicate_key() {
        // Duplicate keys should cause an error
        let yaml = r#"workflow: Test Workflow
swimlanes:
  - test: "Test Lane"
workflow: Another Workflow  # Duplicate key
"#;
        let result = parse_yaml(yaml);
        match result {
            Err(YamlParseError::ParseError {
                line,
                column,
                message,
            }) => {
                println!("Error at line {}, column {}: {}", line, column, message);
                // Note: serde_yaml reports duplicate key errors at the start of the document
                assert!(line > 0);
                assert!(column > 0);
                assert!(message.contains("duplicate"));
            }
            Err(e) => panic!("Expected ParseError but got: {:?}", e),
            Ok(_) => panic!("Expected an error but parsing succeeded"),
        }
    }

    #[test]
    fn is_version_compatible_accepts_any_version_pre_1_0() {
        // Pre-1.0, we accept any version
        assert!(is_version_compatible("0.1.0", "0.3.0"));
        assert!(is_version_compatible("0.3.0", "0.1.0"));
        assert!(is_version_compatible("1.0.0", "0.3.0"));
        assert!(is_version_compatible("0.3.0", "1.0.0"));
    }
}
