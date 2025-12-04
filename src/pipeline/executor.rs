// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Pipeline executor
//!
//! Orchestrates the execution of pipeline stages in dependency order.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use colored::Colorize;
use tokio::sync::RwLock;

use crate::cache::Cache;
use crate::errors::ConflowError;
use crate::executors::{ExecutionResult, Executor};
use crate::pipeline::{DagBuilder, Pipeline, Stage};

/// Pipeline execution options
#[derive(Debug, Clone, Default)]
pub struct ExecutionOptions {
    /// Skip cache lookups
    pub no_cache: bool,
    /// Only show what would be done
    pub dry_run: bool,
    /// Only run specific stages
    pub stages: Vec<String>,
    /// Verbose output
    pub verbose: bool,
}

/// Result of executing a pipeline
#[derive(Debug)]
pub struct PipelineResult {
    /// Results for each stage
    pub results: HashMap<String, ExecutionResult>,
    /// Total execution time
    pub duration: Duration,
    /// Whether all stages succeeded
    pub success: bool,
}

/// Pipeline executor
pub struct PipelineExecutor {
    /// Registered executors by tool name
    executors: HashMap<String, Box<dyn Executor>>,
    /// Cache layer
    cache: Option<Arc<RwLock<Box<dyn Cache>>>>,
}

impl PipelineExecutor {
    /// Create a new pipeline executor
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
            cache: None,
        }
    }

    /// Register an executor for a tool
    pub fn register_executor(&mut self, name: &str, executor: Box<dyn Executor>) {
        self.executors.insert(name.to_string(), executor);
    }

    /// Set the cache layer
    pub fn with_cache(mut self, cache: Box<dyn Cache>) -> Self {
        self.cache = Some(Arc::new(RwLock::new(cache)));
        self
    }

    /// Execute a pipeline
    pub async fn execute(
        &self,
        pipeline: &Pipeline,
        working_dir: &Path,
        options: &ExecutionOptions,
    ) -> Result<PipelineResult, ConflowError> {
        let start = Instant::now();

        // Build and validate DAG
        let dag = DagBuilder::build(pipeline)?;
        let execution_order = dag.topological_order()?;

        // Filter stages if specific ones requested
        let stages_to_run: Vec<usize> = if options.stages.is_empty() {
            execution_order
        } else {
            execution_order
                .into_iter()
                .filter(|&idx| options.stages.contains(&pipeline.stages[idx].name))
                .collect()
        };

        // Print execution plan
        self.print_execution_plan(pipeline, &stages_to_run, &dag)?;

        if options.dry_run {
            return Ok(PipelineResult {
                results: HashMap::new(),
                duration: start.elapsed(),
                success: true,
            });
        }

        // Execute stages in order
        let mut results = HashMap::new();
        let mut all_success = true;

        // Merge global and stage environments
        let global_env = &pipeline.env;

        for idx in stages_to_run {
            let stage = &pipeline.stages[idx];

            // Merge environments (stage overrides global)
            let mut env = global_env.clone();
            env.extend(stage.env.clone());

            // Try cache first
            if !options.no_cache {
                if let Some(ref cache) = self.cache {
                    let cache_read = cache.read().await;
                    if let Ok(Some(cached)) = cache_read.get(stage).await {
                        println!("  {} {} {}", "✓".green(), stage.name.bold(), "(cached)".dimmed());
                        results.insert(stage.name.clone(), cached);
                        continue;
                    }
                }
            }

            // Execute stage
            print!("  {} {}...", "→".blue(), stage.name);

            let result = self
                .execute_stage(stage, working_dir, &env, &results)
                .await?;

            // Print result
            if result.success {
                println!(
                    "\r  {} {} ({:.2}s)",
                    "✓".green(),
                    stage.name.bold(),
                    result.duration.as_secs_f64()
                );

                // Cache successful result
                if !options.no_cache {
                    if let Some(ref cache) = self.cache {
                        let mut cache_write = cache.write().await;
                        let _ = cache_write.store(stage, &result).await;
                    }
                }
            } else {
                println!("\r  {} {} failed", "✗".red(), stage.name.bold());

                if options.verbose {
                    if !result.stderr.is_empty() {
                        eprintln!("{}", result.stderr.dimmed());
                    }
                }

                if !stage.allow_failure {
                    all_success = false;
                    results.insert(stage.name.clone(), result);
                    break;
                }
            }

            results.insert(stage.name.clone(), result);
        }

        let duration = start.elapsed();

        // Print summary
        println!();
        if all_success {
            println!(
                "{}",
                format!("Pipeline completed successfully in {:.2}s", duration.as_secs_f64()).green()
            );
        } else {
            println!(
                "{}",
                format!("Pipeline failed after {:.2}s", duration.as_secs_f64()).red()
            );
        }

        Ok(PipelineResult {
            results,
            duration,
            success: all_success,
        })
    }

    /// Execute a single stage
    async fn execute_stage(
        &self,
        stage: &Stage,
        working_dir: &Path,
        env: &HashMap<String, String>,
        previous_results: &HashMap<String, ExecutionResult>,
    ) -> Result<ExecutionResult, ConflowError> {
        let tool_name = stage.tool_name();

        let executor = self.executors.get(tool_name).ok_or_else(|| {
            ConflowError::ExecutorNotFound {
                tool: tool_name.to_string(),
            }
        })?;

        // Resolve stage input if it references another stage
        let resolved_input = self.resolve_stage_input(stage, previous_results)?;

        executor
            .execute(stage, working_dir, env, resolved_input.as_deref())
            .await
    }

    /// Resolve input from a previous stage
    fn resolve_stage_input(
        &self,
        stage: &Stage,
        previous_results: &HashMap<String, ExecutionResult>,
    ) -> Result<Option<Vec<std::path::PathBuf>>, ConflowError> {
        if let Some(from_stage) = stage.input.references_stage() {
            let prev = previous_results.get(from_stage).ok_or_else(|| {
                ConflowError::ExecutionFailed {
                    message: format!(
                        "Stage '{}' depends on '{}' which hasn't been executed",
                        stage.name, from_stage
                    ),
                    help: None,
                }
            })?;

            Ok(Some(prev.outputs.clone()))
        } else {
            Ok(None)
        }
    }

    /// Print the execution plan
    fn print_execution_plan(
        &self,
        pipeline: &Pipeline,
        stages: &[usize],
        dag: &DagBuilder,
    ) -> Result<(), ConflowError> {
        println!();
        println!("{}: {}", "Pipeline".bold(), pipeline.name);
        println!("{}", "═".repeat(50));
        println!(
            "Execution plan ({} stage{}):",
            stages.len(),
            if stages.len() == 1 { "" } else { "s" }
        );
        println!();

        for (i, &idx) in stages.iter().enumerate() {
            let stage = &pipeline.stages[idx];
            let deps = dag.dependencies(&stage.name).unwrap_or_default();

            print!("  {}. {} ({})", i + 1, stage.name.bold(), stage.tool_name());

            if !deps.is_empty() {
                print!(" {}", format!("[depends: {}]", deps.join(", ")).dimmed());
            }

            println!();
        }

        println!();

        Ok(())
    }

    /// Check if all required tools are available
    pub async fn check_tools(&self, pipeline: &Pipeline) -> Result<Vec<String>, ConflowError> {
        let mut missing = Vec::new();

        let tool_names: std::collections::HashSet<_> =
            pipeline.stages.iter().map(|s| s.tool_name()).collect();

        for tool in tool_names {
            if let Some(executor) = self.executors.get(tool) {
                match executor.check_available().await {
                    Ok(true) => {}
                    Ok(false) => missing.push(tool.to_string()),
                    Err(_) => missing.push(tool.to_string()),
                }
            } else {
                missing.push(tool.to_string());
            }
        }

        Ok(missing)
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}
