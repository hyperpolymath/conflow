// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Error types with educational messages
//!
//! conflow provides helpful, educational error messages that guide users
//! toward solutions rather than leaving them confused.

mod educational;
mod recovery;

pub use educational::EducationalMessage;
pub use recovery::RecoverySuggestion;

use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for conflow operations
pub type ConflowResult<T> = Result<T, ConflowError>;

/// Main error type for conflow
#[derive(Error, Debug, Diagnostic)]
pub enum ConflowError {
    // ─────────────────────────────────────────────────────────────────────────
    // Tool Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Tool '{tool}' not found")]
    #[diagnostic(
        code(conflow::tool_not_found),
        help("{suggestion}")
    )]
    ToolNotFound {
        tool: String,
        suggestion: String,
    },

    #[error("Tool '{tool}' execution failed: {error}")]
    #[diagnostic(code(conflow::tool_execution_failed))]
    ToolExecutionFailed {
        tool: String,
        error: String,
        #[help]
        help: Option<String>,
    },

    #[error("Executor not found for tool: {tool}")]
    #[diagnostic(
        code(conflow::executor_not_found),
        help("Available executors: cue, nickel, shell")
    )]
    ExecutorNotFound { tool: String },

    // ─────────────────────────────────────────────────────────────────────────
    // Pipeline Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Pipeline file not found: {path}")]
    #[diagnostic(
        code(conflow::pipeline_not_found),
        help("Create a pipeline with 'conflow init' or create .conflow.yaml manually")
    )]
    PipelineNotFound { path: PathBuf },

    #[error("Invalid pipeline configuration: {reason}")]
    #[diagnostic(code(conflow::invalid_pipeline))]
    InvalidPipeline {
        reason: String,
        #[help]
        help: Option<String>,
    },

    #[error("Stage '{stage}' is invalid: {reason}")]
    #[diagnostic(code(conflow::invalid_stage))]
    InvalidStage { stage: String, reason: String },

    #[error("Circular dependency detected")]
    #[diagnostic(
        code(conflow::circular_dependency),
        help("Review your stage dependencies to remove the cycle")
    )]
    CircularDependency { stages: Vec<String> },

    #[error("Stage '{stage}' depends on unknown stage '{dependency}'")]
    #[diagnostic(
        code(conflow::unknown_dependency),
        help("Check that '{dependency}' is defined in your pipeline")
    )]
    UnknownDependency { stage: String, dependency: String },

    #[error("Stage '{stage}' not found in pipeline")]
    #[diagnostic(code(conflow::stage_not_found))]
    StageNotFound { stage: String },

    // ─────────────────────────────────────────────────────────────────────────
    // Execution Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Stage '{stage}' failed")]
    #[diagnostic(code(conflow::stage_failed))]
    StageFailed {
        stage: String,
        stderr: String,
        #[help]
        help: Option<String>,
    },

    #[error("Execution failed: {message}")]
    #[diagnostic(code(conflow::execution_failed))]
    ExecutionFailed {
        message: String,
        #[help]
        help: Option<String>,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // File Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("File not found: {path}")]
    #[diagnostic(code(conflow::file_not_found))]
    FileNotFound {
        path: PathBuf,
        #[help]
        help: Option<String>,
    },

    #[error("Failed to read file '{path}': {error}")]
    #[diagnostic(code(conflow::file_read_error))]
    FileReadError { path: PathBuf, error: String },

    #[error("Failed to write file '{path}': {error}")]
    #[diagnostic(code(conflow::file_write_error))]
    FileWriteError { path: PathBuf, error: String },

    #[error("No input files matched pattern: {pattern}")]
    #[diagnostic(
        code(conflow::no_input_files),
        help("Check that files matching '{pattern}' exist in your project")
    )]
    NoInputFiles { pattern: String },

    // ─────────────────────────────────────────────────────────────────────────
    // Cache Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Cache error: {message}")]
    #[diagnostic(code(conflow::cache_error))]
    CacheError { message: String },

    // ─────────────────────────────────────────────────────────────────────────
    // Validation Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("CUE validation failed")]
    #[diagnostic(code(conflow::cue_validation_failed))]
    CueValidationFailed {
        file: PathBuf,
        line: Option<usize>,
        column: Option<usize>,
        message: String,
        #[help]
        help: Option<String>,
    },

    #[error("Nickel type error")]
    #[diagnostic(code(conflow::nickel_type_error))]
    NickelTypeError {
        file: PathBuf,
        message: String,
        #[help]
        help: Option<String>,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Analysis Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Could not detect configuration format for: {path}")]
    #[diagnostic(
        code(conflow::format_detection_failed),
        help("Supported formats: JSON, YAML, TOML, CUE, Nickel")
    )]
    FormatDetectionFailed { path: PathBuf },

    // ─────────────────────────────────────────────────────────────────────────
    // IO/System Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("IO error: {message}")]
    #[diagnostic(code(conflow::io_error))]
    Io { message: String },

    #[error("YAML parsing error: {message}")]
    #[diagnostic(code(conflow::yaml_error))]
    Yaml { message: String },

    #[error("JSON parsing error: {message}")]
    #[diagnostic(code(conflow::json_error))]
    Json { message: String },

    #[error("TOML parsing error: {message}")]
    #[diagnostic(code(conflow::toml_error))]
    Toml { message: String },

    #[error("Glob pattern error: {message}")]
    #[diagnostic(code(conflow::glob_error))]
    GlobPattern { message: String },
}

impl From<std::io::Error> for ConflowError {
    fn from(e: std::io::Error) -> Self {
        Self::Io { message: e.to_string() }
    }
}

impl From<serde_yaml::Error> for ConflowError {
    fn from(e: serde_yaml::Error) -> Self {
        Self::Yaml { message: e.to_string() }
    }
}

impl From<serde_json::Error> for ConflowError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json { message: e.to_string() }
    }
}

impl From<toml::de::Error> for ConflowError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml { message: e.to_string() }
    }
}

impl From<glob::PatternError> for ConflowError {
    fn from(e: glob::PatternError) -> Self {
        Self::GlobPattern { message: e.to_string() }
    }
}

impl ConflowError {
    /// Create a tool not found error with installation suggestion
    pub fn tool_not_found(tool: &str) -> Self {
        let suggestion = match tool {
            "cue" => "Install CUE: https://cuelang.org/docs/install/".to_string(),
            "nickel" => "Install Nickel: https://nickel-lang.org/getting-started/".to_string(),
            _ => format!("Install {} and ensure it's in your PATH", tool),
        };

        Self::ToolNotFound {
            tool: tool.to_string(),
            suggestion,
        }
    }

    /// Create a file not found error with context
    pub fn file_not_found_in_stage(path: PathBuf, stage: &str) -> Self {
        Self::FileNotFound {
            path: path.clone(),
            help: Some(format!(
                "Required by stage '{}'. Check that the file exists.",
                stage
            )),
        }
    }

    /// Create a stage failed error with helpful context
    pub fn stage_failed_with_help(stage: &str, stderr: String, tool: &str) -> Self {
        let help = Self::generate_help_for_tool_error(tool, &stderr);
        Self::StageFailed {
            stage: stage.to_string(),
            stderr,
            help,
        }
    }

    /// Generate helpful suggestions based on tool output
    fn generate_help_for_tool_error(tool: &str, stderr: &str) -> Option<String> {
        match tool {
            "cue" => Self::parse_cue_error(stderr),
            "nickel" => Self::parse_nickel_error(stderr),
            _ => None,
        }
    }

    fn parse_cue_error(stderr: &str) -> Option<String> {
        // Common CUE error patterns and helpful suggestions
        if stderr.contains("undefined field") {
            Some("A field is used but not defined in the schema. Check your CUE definitions.".into())
        } else if stderr.contains("conflicting values") {
            Some("Two values cannot be unified. This often means a constraint was violated.".into())
        } else if stderr.contains("cannot use") {
            Some("Type mismatch detected. Verify your data matches the expected types.".into())
        } else {
            None
        }
    }

    fn parse_nickel_error(stderr: &str) -> Option<String> {
        // Common Nickel error patterns
        if stderr.contains("type error") {
            Some("Type mismatch in Nickel code. Check function arguments and return types.".into())
        } else if stderr.contains("unbound identifier") {
            Some("A variable or function is used but not defined. Check for typos.".into())
        } else if stderr.contains("contract violation") {
            Some("A value doesn't satisfy its contract. Review your type annotations.".into())
        } else {
            None
        }
    }
}
