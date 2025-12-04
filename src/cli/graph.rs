// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Graph command - visualize pipeline as a graph

use miette::Result;
use std::path::PathBuf;

use super::GraphFormat;
use crate::pipeline::{DagBuilder, Pipeline};

/// Run the graph command
pub async fn run(pipeline_path: PathBuf, format: GraphFormat, _verbose: bool) -> Result<()> {
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

    // Build DAG
    let dag = DagBuilder::build(&pipeline)?;

    // Output in requested format
    let output = match format {
        GraphFormat::Text => dag.to_text(&pipeline)?,
        GraphFormat::Dot => dag.to_dot(),
        GraphFormat::Mermaid => dag.to_mermaid(),
    };

    println!("{}", output);

    Ok(())
}
