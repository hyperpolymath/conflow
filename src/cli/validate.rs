// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Validate command - check pipeline configuration

use colored::Colorize;
use miette::Result;
use std::path::PathBuf;

use crate::pipeline::{Pipeline, PipelineValidator};

/// Run the validate command
pub async fn run(pipeline_path: PathBuf, verbose: bool) -> Result<()> {
    println!("{}", "Validating pipeline...".bold());
    println!();

    // Check pipeline exists
    if !pipeline_path.exists() {
        return Err(miette::miette!(
            "Pipeline file not found: {}\n\n\
             Run 'conflow init' to create a new project.",
            pipeline_path.display()
        ));
    }

    // Load pipeline
    let pipeline = match Pipeline::from_file(&pipeline_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  {} Failed to parse pipeline", "✗".red());
            eprintln!();
            return Err(miette::miette!("Parse error: {}", e));
        }
    };

    println!("  {} Pipeline file is valid YAML", "✓".green());

    // Validate pipeline structure
    let validation = PipelineValidator::validate(&pipeline)?;

    // Check for file existence
    let cwd = std::env::current_dir().map_err(|e| miette::miette!("Failed to get current directory: {}", e))?;
    let missing_files = PipelineValidator::validate_files(&pipeline, &cwd)?;

    // Report results
    let mut has_issues = false;

    if !validation.errors.is_empty() {
        has_issues = true;
        println!();
        println!("{}:", "Errors".red().bold());
        for error in &validation.errors {
            println!("  {} {}", "✗".red(), error);
        }
    }

    if !missing_files.is_empty() {
        has_issues = true;
        println!();
        println!("{}:", "Missing files".yellow().bold());
        for missing in &missing_files {
            println!("  {} {}", "⚠".yellow(), missing);
        }
    }

    if !validation.warnings.is_empty() {
        println!();
        println!("{}:", "Warnings".yellow().bold());
        for warning in &validation.warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
    }

    if verbose {
        println!();
        println!("{}:", "Pipeline summary".bold());
        println!("  Name: {}", pipeline.name);
        println!("  Stages: {}", pipeline.stages.len());
        for stage in &pipeline.stages {
            let deps = if stage.depends_on.is_empty() {
                String::new()
            } else {
                format!(" [depends: {}]", stage.depends_on.join(", "))
            };
            println!("    - {} ({}){}", stage.name, stage.tool_name(), deps.dimmed());
        }
    }

    println!();

    if has_issues {
        if validation.is_valid() && missing_files.is_empty() {
            println!(
                "{}",
                "Pipeline is valid but has warnings.".yellow().bold()
            );
            Ok(())
        } else {
            Err(miette::miette!("Pipeline validation failed"))
        }
    } else {
        println!("{}", "Pipeline is valid!".green().bold());
        Ok(())
    }
}
