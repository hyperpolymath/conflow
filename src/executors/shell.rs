// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Shell executor
//!
//! Executes arbitrary shell commands.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::process::Command;

use super::{ExecutionResult, Executor};
use crate::errors::ConflowError;
use crate::pipeline::{Stage, Tool};

/// Shell executor
pub struct ShellExecutor;

impl ShellExecutor {
    /// Create a new shell executor
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Executor for ShellExecutor {
    async fn execute(
        &self,
        stage: &Stage,
        working_dir: &Path,
        env: &HashMap<String, String>,
        _resolved_inputs: Option<&[PathBuf]>,
    ) -> Result<ExecutionResult, ConflowError> {
        let Tool::Shell { command, shell } = &stage.tool else {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Expected Shell tool".to_string(),
            });
        };

        let start = Instant::now();

        let mut cmd = Command::new(shell);
        cmd.arg("-c").arg(command);
        cmd.current_dir(working_dir);
        cmd.envs(env);

        let output = cmd.output().await.map_err(|e| ConflowError::ToolExecutionFailed {
            tool: "shell".to_string(),
            error: e.to_string(),
            help: Some(format!("Shell '{}' may not be available", shell)),
        })?;

        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Collect output files if specified
        let outputs = if let Some(ref out) = stage.output {
            vec![out.path().clone()]
        } else {
            vec![]
        };

        if output.status.success() {
            Ok(ExecutionResult {
                success: true,
                stdout,
                stderr,
                exit_code: 0,
                outputs,
                duration,
                cache_hit: false,
            })
        } else {
            let exit_code = output.status.code().unwrap_or(-1);

            Ok(ExecutionResult {
                success: false,
                stdout,
                stderr,
                exit_code,
                outputs: vec![],
                duration,
                cache_hit: false,
            })
        }
    }

    async fn check_available(&self) -> Result<bool, ConflowError> {
        // Shell is always available (we assume basic shell exists)
        Ok(true)
    }

    async fn version(&self) -> Result<String, ConflowError> {
        // Return bash version as the default
        let output = Command::new("bash")
            .arg("--version")
            .output()
            .await
            .map_err(|e| ConflowError::ToolExecutionFailed {
                tool: "bash".to_string(),
                error: e.to_string(),
                help: None,
            })?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let version = version_str
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_string();

        Ok(version)
    }

    fn validate_stage(&self, stage: &Stage) -> Result<(), ConflowError> {
        let Tool::Shell { command, .. } = &stage.tool else {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Not a Shell stage".to_string(),
            });
        };

        if command.is_empty() {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Shell command is empty".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Input;

    fn make_shell_stage(name: &str, command: &str) -> Stage {
        Stage {
            name: name.into(),
            description: None,
            tool: Tool::Shell {
                command: command.into(),
                shell: "bash".into(),
            },
            input: Input::Single("*".into()),
            output: None,
            depends_on: vec![],
            allow_failure: false,
            env: HashMap::new(),
            condition: None,
        }
    }

    #[test]
    fn test_validate_shell_stage() {
        let executor = ShellExecutor::new();
        let stage = make_shell_stage("test", "echo hello");
        assert!(executor.validate_stage(&stage).is_ok());
    }

    #[test]
    fn test_validate_empty_command_fails() {
        let executor = ShellExecutor::new();
        let stage = make_shell_stage("test", "");
        assert!(executor.validate_stage(&stage).is_err());
    }

    #[tokio::test]
    async fn test_execute_simple_command() {
        let executor = ShellExecutor::new();
        let stage = make_shell_stage("test", "echo hello");

        let result = executor
            .execute(&stage, Path::new("."), &HashMap::new(), None)
            .await
            .unwrap();

        assert!(result.success);
        assert!(result.stdout.contains("hello"));
    }
}
