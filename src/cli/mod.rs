// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! CLI command definitions and handlers
//!
//! Defines the command-line interface for conflow.

pub mod analyze;
pub mod cache;
pub mod graph;
pub mod init;
pub mod rsr;
pub mod run;
pub mod validate;
pub mod watch;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Configuration flow orchestrator
///
/// Intelligently manage CUE, Nickel, and configuration workflows.
#[derive(Parser, Debug)]
#[clap(
    name = "conflow",
    version,
    about = "Configuration flow orchestrator for CUE, Nickel, and config validation",
    long_about = None,
    after_help = "Examples:\n\
        conflow init                    Initialize a new project\n\
        conflow analyze config.yaml     Analyze a config file\n\
        conflow run                     Execute the pipeline\n\
        conflow watch                   Watch for changes and re-run\n\n\
        See 'conflow <command> --help' for more information on a specific command."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[clap(short, long, global = true)]
    pub verbose: bool,

    /// Change to directory before executing
    #[clap(short = 'C', long, global = true, value_name = "DIR")]
    pub directory: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new conflow project
    Init {
        /// Project name (defaults to current directory name)
        name: Option<String>,

        /// Use a template (cue-validation, nickel-generation, full-pipeline, kubernetes)
        #[clap(short, long)]
        template: Option<String>,
    },

    /// Analyze configuration files and recommend tools
    Analyze {
        /// Files to analyze
        files: Vec<PathBuf>,

        /// Output format
        #[clap(short, long, default_value = "text", value_parser = ["text", "json"])]
        format: OutputFormat,
    },

    /// Run the pipeline
    Run {
        /// Pipeline file
        #[clap(short, long, default_value = ".conflow.yaml")]
        pipeline: PathBuf,

        /// Run only specific stages
        #[clap(short, long)]
        stage: Vec<String>,

        /// Skip cache (force re-execution)
        #[clap(long)]
        no_cache: bool,

        /// Dry run (show what would be done)
        #[clap(long)]
        dry_run: bool,
    },

    /// Watch mode - re-run pipeline on file changes
    Watch {
        /// Pipeline file
        #[clap(short, long, default_value = ".conflow.yaml")]
        pipeline: PathBuf,

        /// Debounce delay in milliseconds
        #[clap(long, default_value = "500")]
        debounce: u64,
    },

    /// Validate pipeline configuration
    Validate {
        /// Pipeline file to validate
        #[clap(default_value = ".conflow.yaml")]
        pipeline: PathBuf,
    },

    /// Cache management
    Cache {
        #[clap(subcommand)]
        action: CacheAction,
    },

    /// Show pipeline as a graph
    Graph {
        /// Pipeline file
        #[clap(default_value = ".conflow.yaml")]
        pipeline: PathBuf,

        /// Output format
        #[clap(short, long, default_value = "text", value_parser = ["text", "dot", "mermaid"])]
        format: GraphFormat,
    },

    /// RSR (Rhodium Standard Repository) integration
    Rsr {
        #[clap(subcommand)]
        action: RsrAction,
    },
}

/// RSR integration actions
#[derive(Subcommand, Debug, Clone)]
pub enum RsrAction {
    /// Check RSR compliance
    Check {
        /// Specific requirements to check (default: all)
        #[clap(short, long)]
        requirement: Vec<String>,

        /// Output format
        #[clap(short, long, default_value = "text", value_parser = ["text", "json"])]
        format: OutputFormat,
    },

    /// Show RSR requirements
    Requirements {
        /// Filter by tag
        #[clap(short, long)]
        tag: Option<String>,

        /// Show only specific requirement
        #[clap(short, long)]
        id: Option<String>,
    },

    /// List available RSR schemas
    Schemas {
        /// Filter by tag
        #[clap(short, long)]
        tag: Option<String>,
    },

    /// Export an RSR schema
    Schema {
        /// Schema ID to export
        id: String,

        /// Output file (default: stdout)
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
}

/// Cache management actions
#[derive(Subcommand, Debug, Clone)]
pub enum CacheAction {
    /// Show cache statistics
    Stats,

    /// Clear the cache
    Clear {
        /// Skip confirmation
        #[clap(short, long)]
        yes: bool,
    },

    /// List cached entries
    List,
}

/// Output format for analyze command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Graph output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFormat {
    Text,
    Dot,
    Mermaid,
}

impl std::str::FromStr for GraphFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "dot" => Ok(Self::Dot),
            "mermaid" => Ok(Self::Mermaid),
            _ => Err(format!("Unknown graph format: {}", s)),
        }
    }
}
