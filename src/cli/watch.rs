// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Watch command - re-run pipeline on file changes

use colored::Colorize;
use miette::Result;
use notify::{RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::cache::FilesystemCache;
use crate::executors::create_default_executors;
use crate::pipeline::{ExecutionOptions, Pipeline, PipelineExecutor};

/// Run the watch command
pub async fn run(pipeline_path: PathBuf, debounce_ms: u64, verbose: bool) -> Result<()> {
    // Check pipeline exists
    if !pipeline_path.exists() {
        return Err(miette::miette!(
            "Pipeline file not found: {}\n\n\
             Run 'conflow init' to create a new project.",
            pipeline_path.display()
        ));
    }

    println!("{}", "Starting watch mode...".bold());
    println!(
        "Watching for changes (debounce: {}ms)",
        debounce_ms
    );
    println!("Press {} to exit.", "Ctrl+C".cyan());
    println!();

    // Create channel for receiving events
    let (tx, rx) = channel();

    // Create debounced watcher
    let mut debouncer = new_debouncer(Duration::from_millis(debounce_ms), tx)
        .map_err(|e| miette::miette!("Failed to create file watcher: {}", e))?;

    // Watch current directory
    debouncer
        .watcher()
        .watch(std::path::Path::new("."), RecursiveMode::Recursive)
        .map_err(|e| miette::miette!("Failed to start watching: {}", e))?;

    // Initial run
    run_pipeline(&pipeline_path, verbose).await;

    // Watch for changes
    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                // Filter out irrelevant events
                let relevant: Vec<_> = events
                    .iter()
                    .filter(|e| {
                        // Skip cache directory
                        !e.path.to_string_lossy().contains(".conflow/cache")
                    })
                    .filter(|e| matches!(e.kind, DebouncedEventKind::Any))
                    .collect();

                if !relevant.is_empty() {
                    println!();
                    println!("{}", "─".repeat(50).dimmed());
                    println!(
                        "{}: {} file(s) changed",
                        "Change detected".yellow(),
                        relevant.len()
                    );

                    if verbose {
                        for event in &relevant {
                            println!("  {}", event.path.display());
                        }
                    }

                    println!();
                    run_pipeline(&pipeline_path, verbose).await;
                }
            }
            Ok(Err(e)) => {
                eprintln!("{}: {:?}", "Watch error".red(), e);
            }
            Err(e) => {
                // Channel closed
                eprintln!("{}: {}", "Channel error".red(), e);
                break;
            }
        }
    }

    Ok(())
}

async fn run_pipeline(pipeline_path: &PathBuf, verbose: bool) {
    let start = std::time::Instant::now();

    // Load pipeline
    let pipeline = match Pipeline::from_file(pipeline_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}: {}", "Failed to load pipeline".red(), e);
            return;
        }
    };

    // Create executor
    let mut executor = PipelineExecutor::new();
    for (name, exec) in create_default_executors() {
        executor.register_executor(&name, exec);
    }

    // Set up cache
    let working_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}: {}", "Failed to get working directory".red(), e);
            return;
        }
    };

    if pipeline.cache.enabled {
        if let Ok(cache) = FilesystemCache::new(
            working_dir.join(&pipeline.cache.directory),
            working_dir.clone(),
        ) {
            executor = executor.with_cache(Box::new(cache));
        }
    }

    let options = ExecutionOptions {
        no_cache: false,
        dry_run: false,
        stages: vec![],
        verbose,
    };

    // Execute
    match executor.execute(&pipeline, &working_dir, &options).await {
        Ok(result) => {
            let elapsed = start.elapsed();
            if result.success {
                println!(
                    "{} ({:.2}s)",
                    "Pipeline completed successfully".green(),
                    elapsed.as_secs_f64()
                );
            } else {
                println!(
                    "{} ({:.2}s)",
                    "Pipeline failed".red(),
                    elapsed.as_secs_f64()
                );
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Pipeline execution error".red(), e);
        }
    }
}
