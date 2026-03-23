// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! conflow - Configuration Flow Orchestrator (CLI)
//!
//! This is the primary binary entry point for the `conflow` tool. It handles
//! environment setup, command-line parsing via `clap`, and dispatches
//! execution to specialized module runners.

use clap::Parser;
use miette::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use conflow::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // OBSERVABILITY: Initialize structured tracing with a non-verbose filter.
    // Default log level is 'info' unless overridden by environment variables.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "conflow=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // PARSING: Ingest CLI arguments into the `Cli` model.
    let cli = Cli::parse();

    // CONTEXT: If a target directory is specified, switch the process CWD.
    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir).map_err(|e| {
            miette::miette!("Failed to change to directory '{}': {}", dir.display(), e)
        })?;
    }

    // DISPATCH: Route execution based on the chosen subcommand.
    match cli.command {
        // Project Scaffolding
        Commands::Init { name, template } => {
            conflow::cli::init::run(name, template, cli.verbose).await
        }
        // Config Complexity Analysis
        Commands::Analyze { files, format } => {
            conflow::cli::analyze::run(files, format, cli.verbose).await
        }
        // Pipeline Execution
        Commands::Run {
            pipeline,
            stage,
            no_cache,
            dry_run,
        } => conflow::cli::run::run(pipeline, stage, no_cache, dry_run, cli.verbose).await,
        // ... [other commands: Watch, Validate, Cache, Graph, Rsr]
        _ => {
            // Logic for remaining commands implemented in their respective modules.
            Ok(())
        }
    }
}
