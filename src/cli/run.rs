// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Run command - execute the pipeline

use colored::Colorize;
use miette::Result;
use std::path::PathBuf;

use crate::cache::FilesystemCache;
use crate::executors::create_default_executors;
use crate::pipeline::{ExecutionOptions, Pipeline, PipelineExecutor, PipelineValidator};

/// Run the pipeline
pub async fn run(
    pipeline_path: PathBuf,
    stages: Vec<String>,
    no_cache: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Check pipeline exists
    if !pipeline_path.exists() {
        return Err(miette::miette!(
            "Pipeline file not found: {}\n\n\
             Run 'conflow init' to create a new project.",
            pipeline_path.display()
        ));
    }

    // Load pipeline
    let pipeline = Pipeline::from_file(&pipeline_path).map_err(|e| {
        miette::miette!("Failed to load pipeline: {}", e)
    })?;

    // Validate pipeline
    let validation = PipelineValidator::validate(&pipeline)?;

    if !validation.is_valid() {
        eprintln!("{}", "Pipeline validation failed:".red().bold());
        for error in &validation.errors {
            eprintln!("  {} {}", "✗".red(), error);
        }
        return Err(miette::miette!("Pipeline configuration is invalid"));
    }

    if validation.has_warnings() && verbose {
        eprintln!("{}", "Pipeline warnings:".yellow().bold());
        for warning in &validation.warnings {
            eprintln!("  {} {}", "⚠".yellow(), warning);
        }
        eprintln!();
    }

    // Create executor
    let mut executor = PipelineExecutor::new();

    // Register executors
    for (name, exec) in create_default_executors() {
        executor.register_executor(&name, exec);
    }

    // Check required tools are available
    let missing_tools = executor.check_tools(&pipeline).await?;
    if !missing_tools.is_empty() {
        eprintln!("{}", "Missing required tools:".red().bold());
        for tool in &missing_tools {
            eprintln!("  {} {}", "✗".red(), tool);
            match tool.as_str() {
                "cue" => eprintln!("    Install: {}", "https://cuelang.org/docs/install/".cyan()),
                "nickel" => eprintln!("    Install: {}", "https://nickel-lang.org/getting-started/".cyan()),
                _ => {}
            }
        }
        return Err(miette::miette!("Required tools are not installed"));
    }

    // Set up cache
    let working_dir = std::env::current_dir().map_err(|e| {
        miette::miette!("Failed to get current directory: {}", e)
    })?;

    if !no_cache && pipeline.cache.enabled {
        let cache = FilesystemCache::new(
            working_dir.join(&pipeline.cache.directory),
            working_dir.clone(),
        )?;
        executor = executor.with_cache(Box::new(cache));
    }

    // Create execution options
    let options = ExecutionOptions {
        no_cache,
        dry_run,
        stages,
        verbose,
    };

    // Execute
    let result = executor.execute(&pipeline, &working_dir, &options).await?;

    if !result.success {
        // Find which stage failed
        for (name, stage_result) in &result.results {
            if !stage_result.success {
                eprintln!();
                eprintln!("{}", format!("Stage '{}' failed:", name).red().bold());
                if !stage_result.stderr.is_empty() {
                    eprintln!("{}", stage_result.stderr.dimmed());
                }
                break;
            }
        }
        return Err(miette::miette!("Pipeline execution failed"));
    }

    // Print outputs
    let outputs: Vec<_> = result
        .results
        .values()
        .flat_map(|r| r.outputs.iter())
        .collect();

    if !outputs.is_empty() {
        println!();
        println!("{}:", "Outputs".bold());
        for output in outputs {
            println!("  - {}", output.display());
        }
    }

    Ok(())
}
