// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Pipeline validation
//!
//! Validates pipeline configuration before execution.

use std::collections::HashSet;
use std::path::Path;

use crate::errors::ConflowError;
use crate::pipeline::{DagBuilder, Input, Pipeline, Stage, Tool};

/// Pipeline validator
pub struct PipelineValidator;

impl PipelineValidator {
    /// Validate a pipeline configuration
    pub fn validate(pipeline: &Pipeline) -> Result<ValidationResult, ConflowError> {
        let mut result = ValidationResult::new();

        // Check for empty stages
        if pipeline.stages.is_empty() {
            result.add_error("Pipeline has no stages defined");
        }

        // Check for duplicate stage names
        let mut seen_names = HashSet::new();
        for stage in &pipeline.stages {
            if !seen_names.insert(&stage.name) {
                result.add_error(&format!("Duplicate stage name: '{}'", stage.name));
            }
        }

        // Validate DAG structure (checks for cycles and unknown dependencies)
        match DagBuilder::build(pipeline) {
            Ok(_) => {}
            Err(ConflowError::CircularDependency { stages }) => {
                result.add_error(&format!("Circular dependency: {}", stages.join(" → ")));
            }
            Err(ConflowError::UnknownDependency { stage, dependency }) => {
                result.add_error(&format!(
                    "Stage '{}' depends on unknown stage '{}'",
                    stage, dependency
                ));
            }
            Err(e) => {
                result.add_error(&format!("DAG validation error: {}", e));
            }
        }

        // Validate each stage
        for stage in &pipeline.stages {
            Self::validate_stage(stage, pipeline, &mut result);
        }

        Ok(result)
    }

    /// Validate a single stage
    fn validate_stage(stage: &Stage, pipeline: &Pipeline, result: &mut ValidationResult) {
        // Validate tool configuration
        match &stage.tool {
            Tool::Cue {
                command,
                schemas,
                flags,
                ..
            } => {
                Self::validate_cue_stage(stage, schemas, result);
            }
            Tool::Nickel {
                command,
                file,
                flags,
                ..
            } => {
                Self::validate_nickel_stage(stage, file, result);
            }
            Tool::Shell { command, shell } => {
                if command.is_empty() {
                    result.add_error(&format!("Stage '{}': Shell command is empty", stage.name));
                }
            }
        }

        // Validate input references
        if let Input::FromStage { from_stage } = &stage.input {
            // Check that referenced stage exists
            if pipeline.get_stage(from_stage).is_none() {
                result.add_error(&format!(
                    "Stage '{}': Input references unknown stage '{}'",
                    stage.name, from_stage
                ));
            }

            // Warn if not in depends_on
            if !stage.depends_on.contains(from_stage) {
                result.add_warning(&format!(
                    "Stage '{}': References stage '{}' output but doesn't declare dependency. \
                     This will be added implicitly.",
                    stage.name, from_stage
                ));
            }
        }

        // Check input patterns aren't empty
        match &stage.input {
            Input::Single(s) if s.is_empty() => {
                result.add_error(&format!("Stage '{}': Input pattern is empty", stage.name));
            }
            Input::Multiple(v) if v.is_empty() => {
                result.add_error(&format!(
                    "Stage '{}': Input list is empty",
                    stage.name
                ));
            }
            _ => {}
        }
    }

    /// Validate CUE-specific stage configuration
    fn validate_cue_stage(
        stage: &Stage,
        schemas: &[std::path::PathBuf],
        result: &mut ValidationResult,
    ) {
        // Warn if vet command has no schemas
        if let Tool::Cue {
            command: crate::pipeline::CueCommand::Vet,
            ..
        } = &stage.tool
        {
            if schemas.is_empty() {
                result.add_warning(&format!(
                    "Stage '{}': CUE vet command without schemas - validation will be minimal",
                    stage.name
                ));
            }
        }
    }

    /// Validate Nickel-specific stage configuration
    fn validate_nickel_stage(
        stage: &Stage,
        file: &Option<std::path::PathBuf>,
        result: &mut ValidationResult,
    ) {
        // Export requires a file
        if let Tool::Nickel {
            command: crate::pipeline::NickelCommand::Export,
            ..
        } = &stage.tool
        {
            if file.is_none() {
                // File might come from input, which is OK
                if !matches!(&stage.input, Input::Single(_) | Input::Multiple(_)) {
                    result.add_warning(&format!(
                        "Stage '{}': Nickel export without explicit file - ensure input provides it",
                        stage.name
                    ));
                }
            }
        }
    }

    /// Check that required files exist (runtime validation)
    pub fn validate_files(pipeline: &Pipeline, base_path: &Path) -> Result<Vec<String>, ConflowError> {
        let mut missing = Vec::new();

        for stage in &pipeline.stages {
            // Check schema files
            if let Tool::Cue { schemas, .. } = &stage.tool {
                for schema in schemas {
                    let full_path = base_path.join(schema);
                    if !full_path.exists() {
                        missing.push(format!(
                            "Stage '{}': Schema file not found: {}",
                            stage.name,
                            schema.display()
                        ));
                    }
                }
            }

            // Check Nickel files
            if let Tool::Nickel { file: Some(f), .. } = &stage.tool {
                let full_path = base_path.join(f);
                if !full_path.exists() {
                    missing.push(format!(
                        "Stage '{}': Nickel file not found: {}",
                        stage.name,
                        f.display()
                    ));
                }
            }
        }

        Ok(missing)
    }
}

/// Result of pipeline validation
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, message: &str) {
        self.errors.push(message.to_string());
    }

    pub fn add_warning(&mut self, message: &str) {
        self.warnings.push(message.to_string());
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{CacheConfig, CueCommand};
    use std::collections::HashMap;

    #[test]
    fn test_validate_empty_pipeline() {
        let pipeline = Pipeline {
            version: "1".into(),
            name: "empty".into(),
            description: None,
            stages: vec![],
            env: HashMap::new(),
            cache: CacheConfig::default(),
        };

        let result = PipelineValidator::validate(&pipeline).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors[0].contains("no stages"));
    }

    #[test]
    fn test_validate_duplicate_names() {
        let pipeline = Pipeline {
            version: "1".into(),
            name: "test".into(),
            description: None,
            stages: vec![
                Stage {
                    name: "dup".into(),
                    description: None,
                    tool: Tool::Cue {
                        command: CueCommand::Vet,
                        schemas: vec![],
                        flags: vec![],
                        out_format: None,
                    },
                    input: Input::Single("*.json".into()),
                    output: None,
                    depends_on: vec![],
                    allow_failure: false,
                    env: HashMap::new(),
                    condition: None,
                },
                Stage {
                    name: "dup".into(),
                    description: None,
                    tool: Tool::Cue {
                        command: CueCommand::Vet,
                        schemas: vec![],
                        flags: vec![],
                        out_format: None,
                    },
                    input: Input::Single("*.yaml".into()),
                    output: None,
                    depends_on: vec![],
                    allow_failure: false,
                    env: HashMap::new(),
                    condition: None,
                },
            ],
            env: HashMap::new(),
            cache: CacheConfig::default(),
        };

        let result = PipelineValidator::validate(&pipeline).unwrap();
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("Duplicate")));
    }

    #[test]
    fn test_validate_from_stage_without_dependency() {
        let pipeline = Pipeline {
            version: "1".into(),
            name: "test".into(),
            description: None,
            stages: vec![
                Stage {
                    name: "first".into(),
                    description: None,
                    tool: Tool::Cue {
                        command: CueCommand::Export,
                        schemas: vec![],
                        flags: vec![],
                        out_format: None,
                    },
                    input: Input::Single("*.json".into()),
                    output: Some(crate::pipeline::Output::File("out.json".into())),
                    depends_on: vec![],
                    allow_failure: false,
                    env: HashMap::new(),
                    condition: None,
                },
                Stage {
                    name: "second".into(),
                    description: None,
                    tool: Tool::Cue {
                        command: CueCommand::Vet,
                        schemas: vec![],
                        flags: vec![],
                        out_format: None,
                    },
                    input: Input::FromStage {
                        from_stage: "first".into(),
                    },
                    output: None,
                    depends_on: vec![], // Missing dependency declaration
                    allow_failure: false,
                    env: HashMap::new(),
                    condition: None,
                },
            ],
            env: HashMap::new(),
            cache: CacheConfig::default(),
        };

        let result = PipelineValidator::validate(&pipeline).unwrap();
        // Should be valid (implicit dependency) but have warning
        assert!(result.is_valid());
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.contains("implicitly")));
    }
}
