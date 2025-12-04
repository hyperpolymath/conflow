// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Pipeline definition structures
//!
//! Defines the schema for .conflow.yaml files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Pipeline definition from .conflow.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Pipeline version (for future compatibility)
    #[serde(default = "default_version")]
    pub version: String,

    /// Pipeline name
    pub name: String,

    /// Pipeline description
    #[serde(default)]
    pub description: Option<String>,

    /// Stages in execution order
    pub stages: Vec<Stage>,

    /// Global environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
}

fn default_version() -> String {
    "1".to_string()
}

impl Pipeline {
    /// Load pipeline from a YAML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, crate::ConflowError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::ConflowError::FileReadError {
                path: path.to_path_buf(),
                error: e.to_string(),
            }
        })?;

        Self::from_yaml(&content)
    }

    /// Parse pipeline from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, crate::ConflowError> {
        serde_yaml::from_str(yaml).map_err(Into::into)
    }

    /// Serialize pipeline to YAML
    pub fn to_yaml(&self) -> Result<String, crate::ConflowError> {
        serde_yaml::to_string(self).map_err(Into::into)
    }

    /// Get a stage by name
    pub fn get_stage(&self, name: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.name == name)
    }

    /// Get all stage names
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name.as_str()).collect()
    }
}

/// A single pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    /// Stage name (must be unique within pipeline)
    pub name: String,

    /// Stage description
    #[serde(default)]
    pub description: Option<String>,

    /// Tool to execute
    pub tool: Tool,

    /// Input specification
    pub input: Input,

    /// Output specification
    #[serde(default)]
    pub output: Option<Output>,

    /// Stage dependencies (other stage names)
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Continue pipeline even if this stage fails
    #[serde(default)]
    pub allow_failure: bool,

    /// Environment variables for this stage
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Condition for running this stage
    #[serde(default)]
    pub condition: Option<StageCondition>,
}

impl Stage {
    /// Get the tool name for this stage
    pub fn tool_name(&self) -> &str {
        match &self.tool {
            Tool::Cue { .. } => "cue",
            Tool::Nickel { .. } => "nickel",
            Tool::Shell { .. } => "shell",
        }
    }
}

/// Tool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Tool {
    /// CUE tool
    Cue {
        /// CUE command (vet, eval, export, etc.)
        command: CueCommand,

        /// Schema files
        #[serde(default)]
        schemas: Vec<PathBuf>,

        /// Additional CUE flags
        #[serde(default)]
        flags: Vec<String>,

        /// Output format for export
        #[serde(default)]
        out_format: Option<OutputFormat>,
    },

    /// Nickel tool
    Nickel {
        /// Nickel command (export, typecheck, etc.)
        command: NickelCommand,

        /// Entry point file
        #[serde(default)]
        file: Option<PathBuf>,

        /// Additional Nickel flags
        #[serde(default)]
        flags: Vec<String>,

        /// Output format for export
        #[serde(default)]
        format: Option<OutputFormat>,
    },

    /// Shell command
    Shell {
        /// Shell command to run
        command: String,

        /// Shell to use (bash, sh, etc.)
        #[serde(default = "default_shell")]
        shell: String,
    },
}

fn default_shell() -> String {
    "bash".to_string()
}

/// CUE commands
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CueCommand {
    /// Validate configuration against schema
    Vet,
    /// Export to a format (JSON, YAML, etc.)
    Export,
    /// Evaluate CUE expressions
    Eval,
    /// Format CUE files
    Fmt,
    /// Print definitions
    Def,
}

impl std::fmt::Display for CueCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vet => write!(f, "vet"),
            Self::Export => write!(f, "export"),
            Self::Eval => write!(f, "eval"),
            Self::Fmt => write!(f, "fmt"),
            Self::Def => write!(f, "def"),
        }
    }
}

/// Nickel commands
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NickelCommand {
    /// Export to JSON/YAML/TOML
    Export,
    /// Type check only
    Typecheck,
    /// Query specific path
    Query,
    /// Format Nickel files
    Format,
}

impl std::fmt::Display for NickelCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Export => write!(f, "export"),
            Self::Typecheck => write!(f, "typecheck"),
            Self::Query => write!(f, "query"),
            Self::Format => write!(f, "format"),
        }
    }
}

/// Input specification for a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input {
    /// Single file or glob pattern
    Single(String),

    /// Multiple files or glob patterns
    Multiple(Vec<String>),

    /// From previous stage output
    FromStage {
        /// Name of the stage to get output from
        from_stage: String,
    },
}

impl Input {
    /// Check if this input references another stage
    pub fn references_stage(&self) -> Option<&str> {
        match self {
            Self::FromStage { from_stage } => Some(from_stage),
            _ => None,
        }
    }

    /// Get input patterns (excludes stage references)
    pub fn patterns(&self) -> Vec<&str> {
        match self {
            Self::Single(s) => vec![s.as_str()],
            Self::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
            Self::FromStage { .. } => vec![],
        }
    }
}

/// Output specification for a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Output {
    /// Simple file path
    File(PathBuf),

    /// Formatted output specification
    Formatted {
        /// Output path
        path: PathBuf,
        /// Output format
        format: OutputFormat,
    },
}

impl Output {
    /// Get the output path
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::File(p) => p,
            Self::Formatted { path, .. } => path,
        }
    }

    /// Get the output format if specified
    pub fn format(&self) -> Option<OutputFormat> {
        match self {
            Self::File(_) => None,
            Self::Formatted { format, .. } => Some(*format),
        }
    }
}

/// Output format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Yaml,
    Toml,
    Cue,
    Text,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Yaml => write!(f, "yaml"),
            Self::Toml => write!(f, "toml"),
            Self::Cue => write!(f, "cue"),
            Self::Text => write!(f, "text"),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Cache directory
    #[serde(default = "default_cache_dir")]
    pub directory: PathBuf,

    /// Cache invalidation strategy
    #[serde(default)]
    pub invalidation: CacheInvalidation,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: default_cache_dir(),
            invalidation: CacheInvalidation::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from(".conflow/cache")
}

/// Cache invalidation strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheInvalidation {
    /// Invalidate on content hash change (default)
    #[default]
    ContentHash,
    /// Invalidate on modification time
    Mtime,
    /// Never invalidate (manual only)
    Manual,
}

/// Condition for running a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StageCondition {
    /// Run only if file exists
    FileExists(PathBuf),
    /// Run only if environment variable is set
    EnvSet(String),
    /// Run only if environment variable equals value
    EnvEquals { var: String, value: String },
    /// Always run (default)
    Always,
    /// Never run (skip)
    Never,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_pipeline() {
        let yaml = r#"
version: "1"
name: "test-pipeline"
stages:
  - name: "validate"
    tool:
      type: cue
      command: vet
      schemas:
        - schema.cue
    input: "*.json"
"#;

        let pipeline = Pipeline::from_yaml(yaml).unwrap();
        assert_eq!(pipeline.name, "test-pipeline");
        assert_eq!(pipeline.stages.len(), 1);
        assert_eq!(pipeline.stages[0].name, "validate");
    }

    #[test]
    fn test_parse_nickel_stage() {
        let yaml = r#"
version: "1"
name: "generate"
stages:
  - name: "gen"
    tool:
      type: nickel
      command: export
      file: config.ncl
    input: "config.ncl"
    output: "out.json"
"#;

        let pipeline = Pipeline::from_yaml(yaml).unwrap();
        match &pipeline.stages[0].tool {
            Tool::Nickel { command, file, .. } => {
                assert_eq!(*command, NickelCommand::Export);
                assert_eq!(file.as_ref().unwrap(), &PathBuf::from("config.ncl"));
            }
            _ => panic!("Expected Nickel tool"),
        }
    }

    #[test]
    fn test_parse_from_stage_input() {
        let yaml = r#"
version: "1"
name: "chain"
stages:
  - name: "first"
    tool:
      type: nickel
      command: export
    input: "input.ncl"
    output: "intermediate.json"
  - name: "second"
    tool:
      type: cue
      command: vet
    input:
      from_stage: first
    depends_on:
      - first
"#;

        let pipeline = Pipeline::from_yaml(yaml).unwrap();
        assert_eq!(pipeline.stages.len(), 2);

        match &pipeline.stages[1].input {
            Input::FromStage { from_stage } => {
                assert_eq!(from_stage, "first");
            }
            _ => panic!("Expected FromStage input"),
        }
    }

    #[test]
    fn test_round_trip_yaml() {
        let pipeline = Pipeline {
            version: "1".into(),
            name: "test".into(),
            description: Some("A test pipeline".into()),
            stages: vec![Stage {
                name: "validate".into(),
                description: None,
                tool: Tool::Cue {
                    command: CueCommand::Vet,
                    schemas: vec![PathBuf::from("schema.cue")],
                    flags: vec![],
                    out_format: None,
                },
                input: Input::Single("*.json".into()),
                output: None,
                depends_on: vec![],
                allow_failure: false,
                env: HashMap::new(),
                condition: None,
            }],
            env: HashMap::new(),
            cache: CacheConfig::default(),
        };

        let yaml = pipeline.to_yaml().unwrap();
        let parsed = Pipeline::from_yaml(&yaml).unwrap();

        assert_eq!(parsed.name, pipeline.name);
        assert_eq!(parsed.stages.len(), pipeline.stages.len());
    }
}
