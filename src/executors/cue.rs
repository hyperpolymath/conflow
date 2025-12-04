// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! CUE executor
//!
//! Executes CUE commands (vet, export, eval, fmt, def).

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::process::Command;

use super::{resolve_globs, ExecutionResult, Executor};
use crate::errors::ConflowError;
use crate::pipeline::{CueCommand, Input, Output, Stage, Tool};

/// CUE executor
pub struct CueExecutor {
    /// Path to cue binary
    cue_bin: PathBuf,
}

impl CueExecutor {
    /// Create a new CUE executor
    pub fn new() -> Result<Self, ConflowError> {
        let cue_bin =
            which::which("cue").map_err(|_| ConflowError::tool_not_found("cue"))?;

        Ok(Self { cue_bin })
    }

    /// Build the command for a stage
    fn build_command(
        &self,
        stage: &Stage,
        working_dir: &Path,
        resolved_inputs: Option<&[PathBuf]>,
    ) -> Result<(Command, Vec<PathBuf>), ConflowError> {
        let Tool::Cue {
            command,
            schemas,
            flags,
            out_format,
        } = &stage.tool
        else {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Expected CUE tool".to_string(),
            });
        };

        let mut cmd = Command::new(&self.cue_bin);
        cmd.current_dir(working_dir);

        // Add CUE command
        cmd.arg(command.to_string());

        // Resolve input files
        let input_files = if let Some(resolved) = resolved_inputs {
            resolved.to_vec()
        } else {
            let patterns = stage.input.patterns();
            if patterns.is_empty() {
                vec![]
            } else {
                resolve_globs(&patterns, working_dir)?
            }
        };

        // Add schema files first (for vet/export)
        for schema in schemas {
            let schema_path = if schema.is_absolute() {
                schema.clone()
            } else {
                working_dir.join(schema)
            };
            cmd.arg(&schema_path);
        }

        // Add input files
        for input_file in &input_files {
            cmd.arg(input_file);
        }

        // Add output format for export
        if *command == CueCommand::Export {
            if let Some(format) = out_format {
                cmd.arg("--out").arg(format.to_string());
            }
        }

        // Add custom flags
        for flag in flags {
            cmd.arg(flag);
        }

        Ok((cmd, input_files))
    }

    /// Write output to file if specified
    async fn write_output(
        &self,
        stage: &Stage,
        stdout: &str,
        working_dir: &Path,
    ) -> Result<Vec<PathBuf>, ConflowError> {
        let Some(ref output) = stage.output else {
            return Ok(vec![]);
        };

        let output_path = match output {
            Output::File(p) => working_dir.join(p),
            Output::Formatted { path, .. } => working_dir.join(path),
        };

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ConflowError::FileWriteError {
                    path: parent.to_path_buf(),
                    error: e.to_string(),
                }
            })?;
        }

        tokio::fs::write(&output_path, stdout).await.map_err(|e| {
            ConflowError::FileWriteError {
                path: output_path.clone(),
                error: e.to_string(),
            }
        })?;

        Ok(vec![output_path])
    }
}

#[async_trait]
impl Executor for CueExecutor {
    async fn execute(
        &self,
        stage: &Stage,
        working_dir: &Path,
        env: &HashMap<String, String>,
        resolved_inputs: Option<&[PathBuf]>,
    ) -> Result<ExecutionResult, ConflowError> {
        let start = Instant::now();

        let (mut cmd, _input_files) = self.build_command(stage, working_dir, resolved_inputs)?;

        // Add environment variables
        cmd.envs(env);

        // Execute
        let output = cmd.output().await.map_err(|e| ConflowError::ToolExecutionFailed {
            tool: "cue".to_string(),
            error: e.to_string(),
            help: Some("Ensure CUE is installed and accessible".into()),
        })?;

        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            // Write output if needed
            let outputs = self.write_output(stage, &stdout, working_dir).await?;

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

            // Generate helpful error message
            let help = ConflowError::stage_failed_with_help(&stage.name, stderr.clone(), "cue");

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
        Ok(self.cue_bin.exists())
    }

    async fn version(&self) -> Result<String, ConflowError> {
        let output = Command::new(&self.cue_bin)
            .arg("version")
            .output()
            .await
            .map_err(|e| ConflowError::ToolExecutionFailed {
                tool: "cue".to_string(),
                error: e.to_string(),
                help: None,
            })?;

        // Extract version from output (first line typically)
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
        let Tool::Cue { schemas, .. } = &stage.tool else {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Not a CUE stage".to_string(),
            });
        };

        // Note: We don't validate file existence here since that happens at runtime
        // This is for configuration validation only

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::CacheConfig;

    fn make_cue_stage(name: &str, command: CueCommand) -> Stage {
        Stage {
            name: name.into(),
            description: None,
            tool: Tool::Cue {
                command,
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
        }
    }

    #[test]
    fn test_validate_cue_stage() {
        // Skip if CUE not installed
        let Ok(executor) = CueExecutor::new() else {
            return;
        };

        let stage = make_cue_stage("test", CueCommand::Vet);
        assert!(executor.validate_stage(&stage).is_ok());
    }

    #[test]
    fn test_validate_non_cue_stage_fails() {
        let Ok(executor) = CueExecutor::new() else {
            return;
        };

        let stage = Stage {
            name: "test".into(),
            description: None,
            tool: Tool::Shell {
                command: "echo hello".into(),
                shell: "bash".into(),
            },
            input: Input::Single("*.json".into()),
            output: None,
            depends_on: vec![],
            allow_failure: false,
            env: HashMap::new(),
            condition: None,
        };

        assert!(executor.validate_stage(&stage).is_err());
    }
}
