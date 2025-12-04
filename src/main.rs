// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! conflow - Configuration Flow Orchestrator
//!
//! Intelligently orchestrate CUE, Nickel, and configuration validation workflows.

use clap::Parser;
use miette::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use conflow::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "conflow=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    let cli = Cli::parse();

    // Change to specified directory if provided
    if let Some(ref dir) = cli.directory {
        std::env::set_current_dir(dir).map_err(|e| {
            miette::miette!("Failed to change to directory '{}': {}", dir.display(), e)
        })?;
    }

    // Dispatch to command handlers
    match cli.command {
        Commands::Init { name, template } => {
            conflow::cli::init::run(name, template, cli.verbose).await
        }
        Commands::Analyze { files, format } => {
            conflow::cli::analyze::run(files, format, cli.verbose).await
        }
        Commands::Run {
            pipeline,
            stage,
            no_cache,
            dry_run,
        } => conflow::cli::run::run(pipeline, stage, no_cache, dry_run, cli.verbose).await,
        Commands::Watch { pipeline, debounce } => {
            conflow::cli::watch::run(pipeline, debounce, cli.verbose).await
        }
        Commands::Validate { pipeline } => {
            conflow::cli::validate::run(pipeline, cli.verbose).await
        }
        Commands::Cache { action } => conflow::cli::cache::run(action, cli.verbose).await,
        Commands::Graph { pipeline, format } => {
            conflow::cli::graph::run(pipeline, format, cli.verbose).await
        }
        Commands::Rsr { action } => conflow::cli::rsr::run(action, cli.verbose).await,
    }
}
