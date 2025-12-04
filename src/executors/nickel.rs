// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Nickel executor
//!
//! Executes Nickel commands (export, typecheck, query, format).

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::process::Command;

use super::{resolve_globs, ExecutionResult, Executor};
use crate::errors::ConflowError;
use crate::pipeline::{Input, NickelCommand, Output, OutputFormat, Stage, Tool};

/// Nickel executor
pub struct NickelExecutor {
    /// Path to nickel binary
    nickel_bin: PathBuf,
}

impl NickelExecutor {
    /// Create a new Nickel executor
    pub fn new() -> Result<Self, ConflowError> {
        let nickel_bin =
            which::which("nickel").map_err(|_| ConflowError::tool_not_found("nickel"))?;

        Ok(Self { nickel_bin })
    }

    /// Build the command for a stage
    fn build_command(
        &self,
        stage: &Stage,
        working_dir: &Path,
        resolved_inputs: Option<&[PathBuf]>,
    ) -> Result<(Command, Vec<PathBuf>), ConflowError> {
        let Tool::Nickel {
            command,
            file,
            flags,
            format,
        } = &stage.tool
        else {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Expected Nickel tool".to_string(),
            });
        };

        let mut cmd = Command::new(&self.nickel_bin);
        cmd.current_dir(working_dir);

        // Add Nickel command
        cmd.arg(command.to_string());

        // Determine the input file(s)
        let input_files = if let Some(resolved) = resolved_inputs {
            resolved.to_vec()
        } else if let Some(f) = file {
            vec![working_dir.join(f)]
        } else {
            let patterns = stage.input.patterns();
            if patterns.is_empty() {
                vec![]
            } else {
                resolve_globs(&patterns, working_dir)?
            }
        };

        // Add input file (Nickel typically works with single files)
        if let Some(input_file) = input_files.first() {
            cmd.arg(input_file);
        }

        // Add output format for export
        if *command == NickelCommand::Export {
            if let Some(fmt) = format {
                let format_flag = match fmt {
                    OutputFormat::Json => "json",
                    OutputFormat::Yaml => "yaml",
                    OutputFormat::Toml => "toml",
                    _ => "json", // Default to JSON
                };
                cmd.arg("--format").arg(format_flag);
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
impl Executor for NickelExecutor {
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
            tool: "nickel".to_string(),
            error: e.to_string(),
            help: Some("Ensure Nickel is installed and accessible".into()),
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
        Ok(self.nickel_bin.exists())
    }

    async fn version(&self) -> Result<String, ConflowError> {
        let output = Command::new(&self.nickel_bin)
            .arg("--version")
            .output()
            .await
            .map_err(|e| ConflowError::ToolExecutionFailed {
                tool: "nickel".to_string(),
                error: e.to_string(),
                help: None,
            })?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        let version = version_str.trim().to_string();

        Ok(version)
    }

    fn validate_stage(&self, stage: &Stage) -> Result<(), ConflowError> {
        let Tool::Nickel { .. } = &stage.tool else {
            return Err(ConflowError::InvalidStage {
                stage: stage.name.clone(),
                reason: "Not a Nickel stage".to_string(),
            });
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nickel_stage(name: &str, command: NickelCommand) -> Stage {
        Stage {
            name: name.into(),
            description: None,
            tool: Tool::Nickel {
                command,
                file: Some(PathBuf::from("config.ncl")),
                flags: vec![],
                format: Some(OutputFormat::Json),
            },
            input: Input::Single("config.ncl".into()),
            output: Some(Output::File(PathBuf::from("output.json"))),
            depends_on: vec![],
            allow_failure: false,
            env: HashMap::new(),
            condition: None,
        }
    }

    #[test]
    fn test_validate_nickel_stage() {
        let Ok(executor) = NickelExecutor::new() else {
            return; // Skip if Nickel not installed
        };

        let stage = make_nickel_stage("test", NickelCommand::Export);
        assert!(executor.validate_stage(&stage).is_ok());
    }
}
