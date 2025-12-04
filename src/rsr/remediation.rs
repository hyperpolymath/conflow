// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Auto-remediation for RSR requirements
//!
//! Automatically fixes failing requirements where possible.

use std::path::Path;

use crate::ConflowError;

use super::compliance::RequirementResult;
use super::requirements::{RsrRequirement, RsrRequirementRegistry};

/// Result of an auto-remediation attempt
#[derive(Debug, Clone)]
pub struct RemediationResult {
    /// Requirement ID
    pub requirement_id: String,

    /// Whether remediation was successful
    pub success: bool,

    /// Actions taken
    pub actions: Vec<RemediationAction>,

    /// Error message if failed
    pub error: Option<String>,
}

/// A single remediation action
#[derive(Debug, Clone)]
pub struct RemediationAction {
    /// Description of the action
    pub description: String,

    /// Whether this action was completed
    pub completed: bool,

    /// Files created or modified
    pub files_affected: Vec<String>,
}

/// Auto-remediation engine
pub struct AutoRemediator {
    registry: RsrRequirementRegistry,
    dry_run: bool,
}

impl AutoRemediator {
    /// Create a new auto-remediator
    pub fn new() -> Self {
        Self {
            registry: RsrRequirementRegistry::new(),
            dry_run: false,
        }
    }

    /// Create with custom registry
    pub fn with_registry(registry: RsrRequirementRegistry) -> Self {
        Self {
            registry,
            dry_run: false,
        }
    }

    /// Set dry run mode (don't actually modify files)
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Attempt to remediate a failing requirement
    pub fn remediate(
        &self,
        result: &RequirementResult,
        project_root: &Path,
    ) -> Result<RemediationResult, ConflowError> {
        let requirement = self
            .registry
            .get(&result.requirement_id)
            .ok_or_else(|| ConflowError::ExecutionFailed {
                message: format!("Unknown requirement: {}", result.requirement_id),
                help: None,
            })?;

        if !requirement.remediation.auto_fix {
            return Ok(RemediationResult {
                requirement_id: result.requirement_id.clone(),
                success: false,
                actions: vec![],
                error: Some("Auto-fix not available for this requirement".into()),
            });
        }

        let mut actions = Vec::new();

        // Remediate based on requirement type
        match result.requirement_id.as_str() {
            "RSR-CONFIG-001" => {
                actions.extend(self.remediate_config_001(project_root)?);
            }
            "RSR-CONFIG-002" => {
                actions.extend(self.remediate_config_002(project_root)?);
            }
            "RSR-CONFIG-003" => {
                actions.extend(self.remediate_config_003(project_root)?);
            }
            "RSR-CONFIG-004" => {
                actions.extend(self.remediate_config_004(project_root)?);
            }
            _ => {
                // Try generic remediation
                actions.extend(self.remediate_generic(requirement, project_root)?);
            }
        }

        let all_completed = actions.iter().all(|a| a.completed);

        Ok(RemediationResult {
            requirement_id: result.requirement_id.clone(),
            success: all_completed,
            actions,
            error: if all_completed {
                None
            } else {
                Some("Some remediation actions failed".into())
            },
        })
    }

    /// Remediate RSR-CONFIG-001: Configuration validation
    fn remediate_config_001(&self, project_root: &Path) -> Result<Vec<RemediationAction>, ConflowError> {
        let mut actions = Vec::new();

        // Create schemas directory
        let schemas_dir = project_root.join("schemas");
        if !schemas_dir.exists() {
            if !self.dry_run {
                std::fs::create_dir_all(&schemas_dir)?;
            }
            actions.push(RemediationAction {
                description: "Create schemas directory".into(),
                completed: true,
                files_affected: vec!["schemas/".into()],
            });
        }

        // Create basic CUE schema
        let schema_path = schemas_dir.join("config.cue");
        if !schema_path.exists() {
            let schema_content = r#"// Configuration Schema
package config

#Config: {
    // Add your configuration fields here
    version?: string
    name?:    string

    // Example: environment settings
    environment?: "development" | "staging" | "production"

    // Example: feature flags
    features?: [string]: bool
}
"#;
            if !self.dry_run {
                std::fs::write(&schema_path, schema_content)?;
            }
            actions.push(RemediationAction {
                description: "Create CUE schema template".into(),
                completed: true,
                files_affected: vec!["schemas/config.cue".into()],
            });
        }

        Ok(actions)
    }

    /// Remediate RSR-CONFIG-002: Configuration pipeline
    fn remediate_config_002(&self, project_root: &Path) -> Result<Vec<RemediationAction>, ConflowError> {
        let mut actions = Vec::new();

        let pipeline_path = project_root.join(".conflow.yaml");
        if !pipeline_path.exists() {
            let pipeline_content = r#"# conflow pipeline configuration
# Generated by RSR auto-remediation

version: "1"
name: config-pipeline

# Pipeline stages
stages:
  # Validate configuration files
  - name: validate
    tool:
      type: cue
      command: vet
      schemas:
        - schemas/config.cue
    input:
      - "config/*.yaml"
      - "config/*.json"
    description: Validate configuration against schema

# Optional: Enable caching
cache:
  enabled: true
  directory: .conflow-cache
"#;
            if !self.dry_run {
                std::fs::write(&pipeline_path, pipeline_content)?;
            }
            actions.push(RemediationAction {
                description: "Create .conflow.yaml pipeline".into(),
                completed: true,
                files_affected: vec![".conflow.yaml".into()],
            });
        }

        // Create config directory if it doesn't exist
        let config_dir = project_root.join("config");
        if !config_dir.exists() {
            if !self.dry_run {
                std::fs::create_dir_all(&config_dir)?;
            }
            actions.push(RemediationAction {
                description: "Create config directory".into(),
                completed: true,
                files_affected: vec!["config/".into()],
            });

            // Create example config
            let example_config = config_dir.join("example.yaml");
            if !self.dry_run {
                std::fs::write(
                    &example_config,
                    "# Example configuration\nversion: \"1.0\"\nname: my-app\nenvironment: development\n",
                )?;
            }
            actions.push(RemediationAction {
                description: "Create example configuration".into(),
                completed: true,
                files_affected: vec!["config/example.yaml".into()],
            });
        }

        Ok(actions)
    }

    /// Remediate RSR-CONFIG-003: Multi-environment configuration
    fn remediate_config_003(&self, project_root: &Path) -> Result<Vec<RemediationAction>, ConflowError> {
        let mut actions = Vec::new();

        // Create environments directory
        let env_dir = project_root.join("environments");
        if !env_dir.exists() {
            if !self.dry_run {
                std::fs::create_dir_all(&env_dir)?;
            }
            actions.push(RemediationAction {
                description: "Create environments directory".into(),
                completed: true,
                files_affected: vec!["environments/".into()],
            });
        }

        // Create base Nickel config
        let base_path = env_dir.join("base.ncl");
        if !base_path.exists() {
            let base_content = r#"# Base configuration
# Override these values per environment

{
  app_name = "my-application",
  version = "1.0.0",

  # Default settings
  log_level = "info",
  debug = false,

  # Feature flags
  features = {
    new_ui = false,
    analytics = true,
  },

  # Database settings (to be overridden)
  database = {
    host = "localhost",
    port = 5432,
    pool_size = 10,
  },
}
"#;
            if !self.dry_run {
                std::fs::write(&base_path, base_content)?;
            }
            actions.push(RemediationAction {
                description: "Create base Nickel configuration".into(),
                completed: true,
                files_affected: vec!["environments/base.ncl".into()],
            });
        }

        // Create environment-specific overrides
        for env in &["development", "staging", "production"] {
            let env_path = env_dir.join(format!("{}.ncl", env));
            if !env_path.exists() {
                let env_content = format!(
                    r#"# {} environment configuration
let base = import "base.ncl" in

base & {{
  environment = "{}",
  {}
}}
"#,
                    env,
                    env,
                    match *env {
                        "development" => "debug = true,\n  log_level = \"debug\",",
                        "staging" => "log_level = \"info\",\n  features.new_ui = true,",
                        "production" => "log_level = \"warn\",\n  database.pool_size = 50,",
                        _ => "",
                    }
                );
                if !self.dry_run {
                    std::fs::write(&env_path, env_content)?;
                }
                actions.push(RemediationAction {
                    description: format!("Create {} environment config", env),
                    completed: true,
                    files_affected: vec![format!("environments/{}.ncl", env)],
                });
            }
        }

        // Update .conflow.yaml to include environment generation
        let pipeline_path = project_root.join(".conflow.yaml");
        if pipeline_path.exists() {
            let content = std::fs::read_to_string(&pipeline_path)?;
            if !content.contains("generate-") {
                // Append environment generation stages
                let addition = r#"
  # Generate environment-specific configs
  - name: generate-dev
    tool:
      type: nickel
      command: export
      format: yaml
    input: environments/development.ncl
    output: dist/config.development.yaml
    description: Generate development config

  - name: generate-staging
    tool:
      type: nickel
      command: export
      format: yaml
    input: environments/staging.ncl
    output: dist/config.staging.yaml
    description: Generate staging config

  - name: generate-production
    tool:
      type: nickel
      command: export
      format: yaml
    input: environments/production.ncl
    output: dist/config.production.yaml
    description: Generate production config
"#;
                if !self.dry_run {
                    let new_content = content + addition;
                    std::fs::write(&pipeline_path, new_content)?;
                }
                actions.push(RemediationAction {
                    description: "Add environment generation stages to pipeline".into(),
                    completed: true,
                    files_affected: vec![".conflow.yaml".into()],
                });
            }
        }

        Ok(actions)
    }

    /// Remediate RSR-CONFIG-004: Configuration caching
    fn remediate_config_004(&self, project_root: &Path) -> Result<Vec<RemediationAction>, ConflowError> {
        let mut actions = Vec::new();

        let pipeline_path = project_root.join(".conflow.yaml");
        if pipeline_path.exists() {
            let content = std::fs::read_to_string(&pipeline_path)?;

            if !content.contains("cache:") {
                // Add cache configuration
                let cache_config = r#"
# Caching configuration
cache:
  enabled: true
  directory: .conflow-cache
"#;
                if !self.dry_run {
                    let new_content = content + cache_config;
                    std::fs::write(&pipeline_path, new_content)?;
                }
                actions.push(RemediationAction {
                    description: "Enable caching in pipeline".into(),
                    completed: true,
                    files_affected: vec![".conflow.yaml".into()],
                });
            }
        }

        // Add cache directory to .gitignore
        let gitignore_path = project_root.join(".gitignore");
        let gitignore_exists = gitignore_path.exists();
        let needs_cache_entry = if gitignore_exists {
            let content = std::fs::read_to_string(&gitignore_path)?;
            !content.contains(".conflow-cache")
        } else {
            true
        };

        if needs_cache_entry {
            let addition = "\n# conflow cache\n.conflow-cache/\n";
            if !self.dry_run {
                if gitignore_exists {
                    let content = std::fs::read_to_string(&gitignore_path)?;
                    std::fs::write(&gitignore_path, content + addition)?;
                } else {
                    std::fs::write(&gitignore_path, addition)?;
                }
            }
            actions.push(RemediationAction {
                description: "Add cache directory to .gitignore".into(),
                completed: true,
                files_affected: vec![".gitignore".into()],
            });
        }

        Ok(actions)
    }

    /// Generic remediation based on requirement definition
    fn remediate_generic(
        &self,
        requirement: &RsrRequirement,
        project_root: &Path,
    ) -> Result<Vec<RemediationAction>, ConflowError> {
        let mut actions = Vec::new();

        // Create required files
        for file in &requirement.validation.file_exists {
            let path = project_root.join(file);
            if !path.exists() {
                // Create parent directories
                if let Some(parent) = path.parent() {
                    if !parent.exists() && !self.dry_run {
                        std::fs::create_dir_all(parent)?;
                    }
                }

                // Create empty file or use template
                if !self.dry_run {
                    std::fs::write(&path, "")?;
                }

                actions.push(RemediationAction {
                    description: format!("Create required file: {}", file.display()),
                    completed: true,
                    files_affected: vec![file.display().to_string()],
                });
            }
        }

        // Remove forbidden files
        for file in &requirement.validation.file_absent {
            let path = project_root.join(file);
            if path.exists() {
                if !self.dry_run {
                    if path.is_dir() {
                        std::fs::remove_dir_all(&path)?;
                    } else {
                        std::fs::remove_file(&path)?;
                    }
                }

                actions.push(RemediationAction {
                    description: format!("Remove forbidden file: {}", file.display()),
                    completed: true,
                    files_affected: vec![file.display().to_string()],
                });
            }
        }

        Ok(actions)
    }

    /// Remediate multiple failing requirements
    pub fn remediate_all(
        &self,
        results: &[RequirementResult],
        project_root: &Path,
    ) -> Result<Vec<RemediationResult>, ConflowError> {
        let failed: Vec<_> = results.iter().filter(|r| !r.met).collect();
        let mut remediation_results = Vec::new();

        for result in failed {
            let remediation = self.remediate(result, project_root)?;
            remediation_results.push(remediation);
        }

        Ok(remediation_results)
    }
}

impl Default for AutoRemediator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_remediate_config_002() {
        let temp = TempDir::new().unwrap();
        let remediator = AutoRemediator::new();

        let result = RequirementResult {
            requirement_id: "RSR-CONFIG-002".into(),
            met: false,
            details: vec![],
            remediation: None,
        };

        let remediation = remediator.remediate(&result, temp.path()).unwrap();
        assert!(remediation.success);
        assert!(!remediation.actions.is_empty());

        // Check that .conflow.yaml was created
        assert!(temp.path().join(".conflow.yaml").exists());
    }

    #[test]
    fn test_dry_run() {
        let temp = TempDir::new().unwrap();
        let remediator = AutoRemediator::new().dry_run(true);

        let result = RequirementResult {
            requirement_id: "RSR-CONFIG-002".into(),
            met: false,
            details: vec![],
            remediation: None,
        };

        let remediation = remediator.remediate(&result, temp.path()).unwrap();
        assert!(remediation.success);

        // File should NOT be created in dry run
        assert!(!temp.path().join(".conflow.yaml").exists());
    }
}
