// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! # conflow - Configuration Flow Orchestrator
//!
//! `conflow` is a high-assurance orchestration tool designed to manage the 
//! lifecycle of complex configuration ecosystems. It intelligently chooses 
//! between CUE (for validation-heavy data) and Nickel (for logic-heavy 
//! configuration) based on the problem domain.
//!
//! CORE ARCHITECTURE:
//! 1. **Analyzer**: Heuristic engine that recommends tools based on config complexity.
//! 2. **Pipeline**: Dependency-aware execution engine for chaining config tools.
//! 3. **Cache**: Content-addressable storage to skip redundant evaluations.
//! 4. **RSR**: Implementation of the Rhodium Standard Repository compliance checks.

pub mod analyzer;
pub mod cache;
pub mod cli;
pub mod errors;
pub mod executors;
pub mod pipeline;
pub mod rsr;
pub mod utils;

// CONVENIENCE: Re-export the primary error and result types.
pub use errors::{ConflowError, ConflowResult};
pub use pipeline::{Pipeline, Stage};

// RSR COMPLIANCE: Interfaces for verified repository state.
pub use rsr::{ComplianceChecker, ComplianceLevel, ComplianceReport, RsrHooks};

/// The semantic version of the conflow crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
