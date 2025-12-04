// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Tool executors
//!
//! This module provides the executor trait and implementations
//! for various configuration tools (CUE, Nickel, Shell).

mod cue;
mod nickel;
mod shell;

pub use cue::CueExecutor;
pub use nickel::NickelExecutor;
pub use shell::ShellExecutor;

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::errors::ConflowError;
use crate::pipeline::Stage;

/// Result of stage execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,

    /// Standard output
    pub stdout: String,

    /// Standard error
    pub stderr: String,

    /// Exit code
    pub exit_code: i32,

    /// Output files generated
    pub outputs: Vec<PathBuf>,

    /// Execution duration
    pub duration: Duration,

    /// Cache hit or miss
    pub cache_hit: bool,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(stdout: String, duration: Duration, outputs: Vec<PathBuf>) -> Self {
        Self {
            success: true,
            stdout,
            stderr: String::new(),
            exit_code: 0,
            outputs,
            duration,
            cache_hit: false,
        }
    }

    /// Create a failed result
    pub fn failure(stderr: String, exit_code: i32, duration: Duration) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr,
            exit_code,
            outputs: vec![],
            duration,
            cache_hit: false,
        }
    }

    /// Mark this result as a cache hit
    pub fn with_cache_hit(mut self) -> Self {
        self.cache_hit = true;
        self
    }
}

/// Trait for tool executors
#[async_trait]
pub trait Executor: Send + Sync {
    /// Execute a stage
    ///
    /// # Arguments
    /// * `stage` - The stage configuration
    /// * `working_dir` - The working directory for execution
    /// * `env` - Environment variables
    /// * `resolved_inputs` - Input files resolved from previous stages (if any)
    async fn execute(
        &self,
        stage: &Stage,
        working_dir: &Path,
        env: &HashMap<String, String>,
        resolved_inputs: Option<&[PathBuf]>,
    ) -> Result<ExecutionResult, ConflowError>;

    /// Check if tool is available
    async fn check_available(&self) -> Result<bool, ConflowError>;

    /// Get tool version
    async fn version(&self) -> Result<String, ConflowError>;

    /// Validate stage configuration
    fn validate_stage(&self, stage: &Stage) -> Result<(), ConflowError>;
}

/// Resolve glob patterns to file paths
pub fn resolve_globs(patterns: &[&str], base_dir: &Path) -> Result<Vec<PathBuf>, ConflowError> {
    let mut files = Vec::new();

    for pattern in patterns {
        let full_pattern = if Path::new(pattern).is_absolute() {
            pattern.to_string()
        } else {
            base_dir.join(pattern).to_string_lossy().to_string()
        };

        let matches: Vec<_> = glob::glob(&full_pattern)
            .map_err(|e| ConflowError::GlobPattern { message: e.to_string() })?
            .filter_map(Result::ok)
            .collect();

        if matches.is_empty() {
            return Err(ConflowError::NoInputFiles {
                pattern: pattern.to_string(),
            });
        }

        files.extend(matches);
    }

    Ok(files)
}

/// Create a standard executor setup with all built-in executors
pub fn create_default_executors() -> HashMap<String, Box<dyn Executor>> {
    let mut executors: HashMap<String, Box<dyn Executor>> = HashMap::new();

    // Try to create each executor (they may fail if tool not installed)
    if let Ok(cue) = CueExecutor::new() {
        executors.insert("cue".to_string(), Box::new(cue));
    }

    if let Ok(nickel) = NickelExecutor::new() {
        executors.insert("nickel".to_string(), Box::new(nickel));
    }

    // Shell executor always available
    executors.insert("shell".to_string(), Box::new(ShellExecutor::new()));

    executors
}
