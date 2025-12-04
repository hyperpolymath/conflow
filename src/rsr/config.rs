// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! RSR Configuration loading
//!
//! Load org-specific requirements and configuration from .rsr.yaml

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::requirements::{RsrRequirement, RsrRequirementClass};
use crate::ConflowError;

/// RSR Configuration from .rsr.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsrConfig {
    /// RSR version
    #[serde(default = "default_version")]
    pub version: String,

    /// Project metadata
    #[serde(default)]
    pub project: ProjectConfig,

    /// Requirements configuration
    #[serde(default)]
    pub requirements: RequirementsConfig,

    /// Integration settings
    #[serde(default)]
    pub integrations: IntegrationsConfig,

    /// Compliance targets
    #[serde(default)]
    pub compliance: ComplianceConfig,

    /// Custom schemas
    #[serde(default)]
    pub schemas: Vec<SchemaReference>,
}

fn default_version() -> String {
    "1".to_string()
}

/// Project configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: Option<String>,

    /// Project description
    pub description: Option<String>,

    /// Project tier (1-4, higher is more strict)
    pub tier: Option<u8>,

    /// Project tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Requirements configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RequirementsConfig {
    /// Skip specific requirements
    #[serde(default)]
    pub skip: Vec<String>,

    /// Custom requirement definitions
    #[serde(default)]
    pub custom: Vec<RsrRequirement>,

    /// Override requirement classes
    #[serde(default)]
    pub overrides: HashMap<String, RequirementOverride>,

    /// Import requirements from external files
    #[serde(default)]
    pub imports: Vec<PathBuf>,
}

/// Override for a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementOverride {
    /// Override class
    pub class: Option<RsrRequirementClass>,

    /// Skip this requirement
    #[serde(default)]
    pub skip: bool,

    /// Reason for override
    pub reason: Option<String>,
}

/// Integration settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    /// conflow integration
    #[serde(default)]
    pub conflow: ConflowIntegration,

    /// CI integration
    #[serde(default)]
    pub ci: CiIntegration,

    /// Notification settings
    #[serde(default)]
    pub notifications: NotificationSettings,
}

/// conflow integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflowIntegration {
    /// Enable conflow integration
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Custom pipeline file
    pub pipeline: Option<PathBuf>,

    /// Run before compliance check
    #[serde(default)]
    pub run_before_check: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ConflowIntegration {
    fn default() -> Self {
        Self {
            enabled: true,
            pipeline: None,
            run_before_check: false,
        }
    }
}

/// CI integration settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CiIntegration {
    /// CI provider
    pub provider: Option<CiProvider>,

    /// Path to CI config
    pub config: Option<PathBuf>,

    /// Fail CI on non-compliance
    #[serde(default)]
    pub fail_on_noncompliant: bool,

    /// Generate badges
    #[serde(default)]
    pub generate_badges: bool,
}

/// CI Provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CiProvider {
    GitHub,
    GitLab,
    Jenkins,
    CircleCI,
    Travis,
    Azure,
}

/// Notification settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotificationSettings {
    /// Notify on regression
    #[serde(default)]
    pub on_regression: bool,

    /// Notify on improvement
    #[serde(default)]
    pub on_improvement: bool,

    /// Slack webhook
    pub slack_webhook: Option<String>,

    /// Email addresses
    #[serde(default)]
    pub emails: Vec<String>,
}

/// Compliance configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Target compliance level
    pub target_level: Option<TargetLevel>,

    /// Exceptions to requirements
    #[serde(default)]
    pub exceptions: Vec<ComplianceException>,

    /// History tracking
    #[serde(default)]
    pub track_history: bool,

    /// History file path
    pub history_file: Option<PathBuf>,
}

/// Target compliance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetLevel {
    Basic,
    Good,
    Excellent,
}

/// Exception to a requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceException {
    /// Requirement ID
    pub requirement: String,

    /// Reason for exception
    pub reason: String,

    /// Expiration date (ISO 8601)
    pub expires: Option<String>,

    /// Approved by
    pub approved_by: Option<String>,
}

/// Reference to a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaReference {
    /// Schema ID
    pub id: String,

    /// Path to schema file
    pub path: PathBuf,

    /// Schema type
    pub schema_type: Option<String>,
}

impl RsrConfig {
    /// Load from file
    pub fn load(path: &Path) -> Result<Self, ConflowError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })?;

        serde_yaml::from_str(&content).map_err(|e| ConflowError::Yaml {
            message: e.to_string(),
        })
    }

    /// Load from project directory (looks for .rsr.yaml)
    pub fn load_from_project(project_root: &Path) -> Result<Self, ConflowError> {
        let config_path = project_root.join(".rsr.yaml");
        Self::load(&config_path)
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), ConflowError> {
        let content = serde_yaml::to_string(self).map_err(|e| ConflowError::Yaml {
            message: e.to_string(),
        })?;

        std::fs::write(path, content).map_err(|e| ConflowError::Io {
            message: e.to_string(),
        })?;

        Ok(())
    }

    /// Check if a requirement should be skipped
    pub fn should_skip(&self, requirement_id: &str) -> bool {
        // Check direct skip list
        if self.requirements.skip.contains(&requirement_id.to_string()) {
            return true;
        }

        // Check overrides
        if let Some(override_cfg) = self.requirements.overrides.get(requirement_id) {
            if override_cfg.skip {
                return true;
            }
        }

        // Check exceptions
        for exception in &self.compliance.exceptions {
            if exception.requirement == requirement_id {
                // Check if exception is still valid
                if let Some(ref expires) = exception.expires {
                    if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expires) {
                        if expiry > chrono::Utc::now() {
                            return true;
                        }
                    }
                } else {
                    // No expiry, always valid
                    return true;
                }
            }
        }

        false
    }

    /// Get class override for a requirement
    pub fn class_override(&self, requirement_id: &str) -> Option<RsrRequirementClass> {
        self.requirements
            .overrides
            .get(requirement_id)
            .and_then(|o| o.class)
    }

    /// Get all custom requirements
    pub fn custom_requirements(&self) -> &[RsrRequirement] {
        &self.requirements.custom
    }

    /// Load imported requirements
    pub fn load_imports(&self, base_path: &Path) -> Result<Vec<RsrRequirement>, ConflowError> {
        let mut requirements = Vec::new();

        for import_path in &self.requirements.imports {
            let full_path = base_path.join(import_path);
            let content = std::fs::read_to_string(&full_path).map_err(|e| ConflowError::Io {
                message: format!("Failed to load import {}: {}", import_path.display(), e),
            })?;

            let imported: Vec<RsrRequirement> =
                serde_yaml::from_str(&content).map_err(|e| ConflowError::Yaml {
                    message: e.to_string(),
                })?;

            requirements.extend(imported);
        }

        Ok(requirements)
    }
}

impl Default for RsrConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            project: ProjectConfig::default(),
            requirements: RequirementsConfig::default(),
            integrations: IntegrationsConfig::default(),
            compliance: ComplianceConfig::default(),
            schemas: Vec::new(),
        }
    }
}

/// Generate a default .rsr.yaml configuration
pub fn generate_default_config(project_name: &str) -> String {
    format!(
        r#"# RSR Configuration
# See: https://rsr.dev/docs/config

version: "1"

project:
  name: "{}"
  # description: "Project description"
  # tier: 2  # 1-4, higher is more strict

requirements:
  # Skip specific requirements
  skip: []

  # Override requirement classes
  overrides: {{}}
    # RSR-CONFIG-001:
    #   class: advisory
    #   reason: "Not applicable for this project"

  # Import custom requirements
  imports: []
    # - .rsr/custom-requirements.yaml

integrations:
  conflow:
    enabled: true
    # pipeline: .conflow.yaml
    # run_before_check: false

  ci:
    # provider: github
    fail_on_noncompliant: false
    generate_badges: true

compliance:
  target_level: good
  track_history: true
  # history_file: .rsr/history.json

  exceptions: []
    # - requirement: RSR-CONFIG-003
    #   reason: "Single environment project"
    #   expires: "2025-12-31T00:00:00Z"
"#,
        project_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_default() {
        let temp = TempDir::new().unwrap();
        let config = RsrConfig::load_from_project(temp.path()).unwrap();

        assert_eq!(config.version, "1");
    }

    #[test]
    fn test_load_config() {
        let temp = TempDir::new().unwrap();

        std::fs::write(
            temp.path().join(".rsr.yaml"),
            r#"
version: "1"
project:
  name: test-project
  tier: 2
requirements:
  skip:
    - RSR-CONFIG-003
"#,
        )
        .unwrap();

        let config = RsrConfig::load_from_project(temp.path()).unwrap();

        assert_eq!(config.project.name, Some("test-project".into()));
        assert_eq!(config.project.tier, Some(2));
        assert!(config.should_skip("RSR-CONFIG-003"));
        assert!(!config.should_skip("RSR-CONFIG-001"));
    }

    #[test]
    fn test_exception_handling() {
        let config = RsrConfig {
            compliance: ComplianceConfig {
                exceptions: vec![
                    ComplianceException {
                        requirement: "RSR-001".into(),
                        reason: "Test".into(),
                        expires: None,
                        approved_by: None,
                    },
                    ComplianceException {
                        requirement: "RSR-002".into(),
                        reason: "Test".into(),
                        expires: Some("2020-01-01T00:00:00Z".into()), // Expired
                        approved_by: None,
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(config.should_skip("RSR-001")); // No expiry
        assert!(!config.should_skip("RSR-002")); // Expired
    }

    #[test]
    fn test_generate_default() {
        let config = generate_default_config("my-project");
        assert!(config.contains("my-project"));
        assert!(config.contains("version:"));
    }
}
