// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! Pipeline definitions and types
//!
//! This module defines the core data structures for conflow pipelines,
//! including stages, tools, inputs, outputs, and configuration.

mod dag;
mod definition;
mod executor;
mod validation;

pub use dag::DagBuilder;
pub use definition::*;
pub use executor::{ExecutionOptions, PipelineExecutor, PipelineResult};
pub use validation::PipelineValidator;
