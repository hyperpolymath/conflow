//! RSR Requirement definitions
//!
//! Defines the requirements that RSR uses to evaluate projects,
//! with a focus on configuration-related requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// RSR Requirement class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RsrRequirementClass {
    /// Must be satisfied for compliance
    Mandatory,
    /// Should be satisfied, contributes to score
    Preferential,
    /// Nice to have, minimal impact on score
    Advisory,
}

impl RsrRequirementClass {
    pub fn weight(&self) -> f64 {
        match self {
            Self::Mandatory => 1.0,
            Self::Preferential => 0.5,
            Self::Advisory => 0.2,
        }
    }
}

/// RSR Requirement definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsrRequirement {
    /// Unique requirement ID (e.g., "RSR-CONFIG-002")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Requirement class
    pub class: RsrRequirementClass,

    /// Detailed description
    pub description: String,

    /// Validation checks
    pub validation: ValidationChecks,

    /// Remediation options
    pub remediation: RemediationOptions,

    /// Related requirements
    #[serde(default)]
    pub related: Vec<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Validation checks for a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationChecks {
    /// Files that should exist
    #[serde(default)]
    pub file_exists: Vec<PathBuf>,

    /// Files that should NOT exist
    #[serde(default)]
    pub file_absent: Vec<PathBuf>,

    /// Patterns that should match in specific files
    #[serde(default)]
    pub patterns: Vec<PatternCheck>,

    /// CUE schemas to validate against
    #[serde(default)]
    pub cue_validate: Vec<CueValidation>,

    /// conflow pipeline should be valid
    #[serde(default)]
    pub conflow_valid: bool,

    /// Custom shell check
    #[serde(default)]
    pub shell_check: Option<String>,
}

/// Pattern check within a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCheck {
    /// File to check
    pub file: PathBuf,

    /// Pattern to match (regex)
    pub pattern: String,

    /// Should the pattern match (true) or not match (false)
    #[serde(default = "default_true")]
    pub should_match: bool,
}

fn default_true() -> bool {
    true
}

/// CUE validation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CueValidation {
    /// Files to validate
    pub files: Vec<PathBuf>,

    /// Schema to validate against
    pub schema: PathBuf,
}

/// Remediation options for a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationOptions {
    /// Automatic fixes available
    #[serde(default)]
    pub auto_fix: bool,

    /// Template options
    #[serde(default)]
    pub templates: Vec<RemediationTemplate>,

    /// Manual steps
    #[serde(default)]
    pub manual_steps: Vec<String>,

    /// Documentation link
    #[serde(default)]
    pub docs_url: Option<String>,
}

/// Template for remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationTemplate {
    /// Template name
    pub name: String,

    /// Description
    pub description: String,

    /// conflow template to use
    pub conflow_template: Option<String>,

    /// Files to generate
    #[serde(default)]
    pub generates: Vec<PathBuf>,
}

/// Built-in RSR requirements related to configuration
pub fn builtin_config_requirements() -> Vec<RsrRequirement> {
    vec![
        RsrRequirement {
            id: "RSR-CONFIG-001".into(),
            name: "Configuration validation".into(),
            class: RsrRequirementClass::Mandatory,
            description: "Configuration files must be validated against a schema".into(),
            validation: ValidationChecks {
                file_exists: vec![],
                file_absent: vec![],
                patterns: vec![],
                cue_validate: vec![],
                conflow_valid: false,
                shell_check: None,
            },
            remediation: RemediationOptions {
                auto_fix: true,
                templates: vec![RemediationTemplate {
                    name: "cue-validation".into(),
                    description: "Add CUE schema validation".into(),
                    conflow_template: Some("cue-validation".into()),
                    generates: vec![
                        PathBuf::from(".conflow.yaml"),
                        PathBuf::from("schemas/config.cue"),
                    ],
                }],
                manual_steps: vec![
                    "Create a CUE schema for your configuration".into(),
                    "Add validation to your build process".into(),
                ],
                docs_url: Some("https://rsr.dev/requirements/config-001".into()),
            },
            related: vec!["RSR-CONFIG-002".into()],
            tags: vec!["config".into(), "validation".into()],
        },
        RsrRequirement {
            id: "RSR-CONFIG-002".into(),
            name: "Configuration pipeline orchestration".into(),
            class: RsrRequirementClass::Preferential,
            description:
                "Use conflow for configuration pipeline orchestration instead of ad-hoc scripts"
                    .into(),
            validation: ValidationChecks {
                file_exists: vec![PathBuf::from(".conflow.yaml")],
                file_absent: vec![],
                patterns: vec![],
                cue_validate: vec![],
                conflow_valid: true,
                shell_check: None,
            },
            remediation: RemediationOptions {
                auto_fix: true,
                templates: vec![
                    RemediationTemplate {
                        name: "cue-validation".into(),
                        description: "Simple schema validation".into(),
                        conflow_template: Some("cue-validation".into()),
                        generates: vec![PathBuf::from(".conflow.yaml")],
                    },
                    RemediationTemplate {
                        name: "nickel-generation".into(),
                        description: "Programmatic config generation".into(),
                        conflow_template: Some("nickel-generation".into()),
                        generates: vec![PathBuf::from(".conflow.yaml")],
                    },
                    RemediationTemplate {
                        name: "full-pipeline".into(),
                        description: "Generate + validate + export".into(),
                        conflow_template: Some("full-pipeline".into()),
                        generates: vec![PathBuf::from(".conflow.yaml")],
                    },
                ],
                manual_steps: vec![
                    "Run 'conflow init' to create a pipeline".into(),
                    "Define stages for your config workflow".into(),
                    "Replace ad-hoc scripts with conflow run".into(),
                ],
                docs_url: Some("https://rsr.dev/requirements/config-002".into()),
            },
            related: vec!["RSR-CONFIG-001".into(), "RSR-CONFIG-003".into()],
            tags: vec!["config".into(), "orchestration".into(), "conflow".into()],
        },
        RsrRequirement {
            id: "RSR-CONFIG-003".into(),
            name: "Multi-environment configuration".into(),
            class: RsrRequirementClass::Preferential,
            description:
                "Environment-specific configurations should be generated, not duplicated".into(),
            validation: ValidationChecks {
                file_exists: vec![],
                file_absent: vec![],
                patterns: vec![PatternCheck {
                    file: PathBuf::from(".conflow.yaml"),
                    pattern: r"generate-.*env|environment".into(),
                    should_match: true,
                }],
                cue_validate: vec![],
                conflow_valid: true,
                shell_check: None,
            },
            remediation: RemediationOptions {
                auto_fix: true,
                templates: vec![RemediationTemplate {
                    name: "multi-env".into(),
                    description: "Multi-environment config generation".into(),
                    conflow_template: Some("multi-env".into()),
                    generates: vec![
                        PathBuf::from(".conflow.yaml"),
                        PathBuf::from("environments/"),
                    ],
                }],
                manual_steps: vec![
                    "Create a base configuration in Nickel".into(),
                    "Create environment-specific overrides".into(),
                    "Use conflow to generate all environments".into(),
                ],
                docs_url: Some("https://rsr.dev/requirements/config-003".into()),
            },
            related: vec!["RSR-CONFIG-002".into()],
            tags: vec!["config".into(), "environments".into(), "dry".into()],
        },
        RsrRequirement {
            id: "RSR-CONFIG-004".into(),
            name: "Configuration caching".into(),
            class: RsrRequirementClass::Advisory,
            description: "Configuration generation should use caching to avoid redundant work"
                .into(),
            validation: ValidationChecks {
                file_exists: vec![],
                file_absent: vec![],
                patterns: vec![PatternCheck {
                    file: PathBuf::from(".conflow.yaml"),
                    pattern: r"cache:\s*\n\s*enabled:\s*true".into(),
                    should_match: true,
                }],
                cue_validate: vec![],
                conflow_valid: false,
                shell_check: None,
            },
            remediation: RemediationOptions {
                auto_fix: true,
                templates: vec![],
                manual_steps: vec![
                    "Add cache configuration to .conflow.yaml".into(),
                    "Ensure cache directory is in .gitignore".into(),
                ],
                docs_url: Some("https://rsr.dev/requirements/config-004".into()),
            },
            related: vec!["RSR-CONFIG-002".into()],
            tags: vec!["config".into(), "performance".into(), "caching".into()],
        },
    ]
}

/// Registry of all RSR requirements
#[derive(Debug, Default)]
pub struct RsrRequirementRegistry {
    requirements: HashMap<String, RsrRequirement>,
}

impl RsrRequirementRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();

        // Load built-in requirements
        for req in builtin_config_requirements() {
            registry.requirements.insert(req.id.clone(), req);
        }

        registry
    }

    /// Get a requirement by ID
    pub fn get(&self, id: &str) -> Option<&RsrRequirement> {
        self.requirements.get(id)
    }

    /// Get all requirements
    pub fn all(&self) -> impl Iterator<Item = &RsrRequirement> {
        self.requirements.values()
    }

    /// Get requirements by tag
    pub fn by_tag(&self, tag: &str) -> Vec<&RsrRequirement> {
        self.requirements
            .values()
            .filter(|r| r.tags.contains(&tag.to_string()))
            .collect()
    }

    /// Get requirements by class
    pub fn by_class(&self, class: RsrRequirementClass) -> Vec<&RsrRequirement> {
        self.requirements
            .values()
            .filter(|r| r.class == class)
            .collect()
    }

    /// Register a custom requirement
    pub fn register(&mut self, requirement: RsrRequirement) {
        self.requirements.insert(requirement.id.clone(), requirement);
    }

    /// Load requirements from a YAML file
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), crate::ConflowError> {
        let content = std::fs::read_to_string(path).map_err(|e| crate::ConflowError::Io {
            message: e.to_string(),
        })?;

        let reqs: Vec<RsrRequirement> =
            serde_yaml::from_str(&content).map_err(|e| crate::ConflowError::Yaml {
                message: e.to_string(),
            })?;

        for req in reqs {
            self.requirements.insert(req.id.clone(), req);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_requirements() {
        let reqs = builtin_config_requirements();
        assert!(reqs.len() >= 4);

        // Check RSR-CONFIG-002 exists
        assert!(reqs.iter().any(|r| r.id == "RSR-CONFIG-002"));
    }

    #[test]
    fn test_registry() {
        let registry = RsrRequirementRegistry::new();

        let req = registry.get("RSR-CONFIG-002").unwrap();
        assert_eq!(req.class, RsrRequirementClass::Preferential);
        assert!(req.validation.conflow_valid);
    }

    #[test]
    fn test_by_tag() {
        let registry = RsrRequirementRegistry::new();
        let config_reqs = registry.by_tag("config");
        assert!(config_reqs.len() >= 4);
    }
}
