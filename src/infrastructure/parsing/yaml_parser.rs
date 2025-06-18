// Copyright (c) 2025 John Wilger
// SPDX-License-Identifier: MIT

//! YAML parsing types for Event Model format.
//!
//! This module contains Serde-compatible types that map directly to the YAML structure
//! of `.eventmodel` files. These types are used as an intermediate representation
//! before conversion to domain types.

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
    pub slices: HashMap<String, Vec<String>>,
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
}
