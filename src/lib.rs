// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! # conflow - Configuration Flow Orchestrator
//!
//! `conflow` intelligently orchestrates CUE, Nickel, and configuration validation workflows.
//!
//! ## Features
//!
//! - **Intelligent analysis** - Recommends CUE vs Nickel based on complexity
//! - **Pipeline orchestration** - Chain tools with dependency management
//! - **Smart caching** - Only re-run what changed
//! - **Educational** - Learn why certain tools fit certain problems
//! - **RSR Integration** - Full integration with Rhodium Standard Repository
//!
//! ## Quick Start
//!
//! ```bash
//! # Initialize a new project
//! conflow init my-project
//!
//! # Analyze existing configs
//! conflow analyze config.yaml
//!
//! # Run pipeline
//! conflow run
//!
//! # Check RSR compliance
//! conflow rsr check
//! ```

pub mod analyzer;
pub mod cache;
pub mod cli;
pub mod errors;
pub mod executors;
pub mod pipeline;
pub mod rsr;
pub mod utils;

// Re-export commonly used types
pub use errors::{ConflowError, ConflowResult};
pub use pipeline::{Pipeline, Stage};

// Re-export RSR types
pub use rsr::{ComplianceChecker, ComplianceLevel, ComplianceReport, RsrHooks};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
