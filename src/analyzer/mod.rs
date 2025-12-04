// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Configuration analyzer
//!
//! Analyzes configuration files and recommends appropriate tools.

mod complexity;
mod config_detector;
mod patterns;
mod recommender;

pub use complexity::Complexity;
pub use config_detector::ConfigFormat;
pub use recommender::{Alternative, RecommendedTool, ToolRecommendation};

use std::path::Path;

use crate::errors::ConflowError;

/// Result of analyzing a configuration file
#[derive(Debug)]
pub struct Analysis {
    /// Detected configuration format
    pub format: ConfigFormat,
    /// Complexity analysis
    pub complexity: Complexity,
    /// Tool recommendation
    pub recommendation: ToolRecommendation,
}

/// Configuration analyzer
pub struct ConfigAnalyzer;

impl ConfigAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze a configuration file
    pub async fn analyze(&self, path: &Path) -> Result<Analysis, ConflowError> {
        // Read file content
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            ConflowError::FileReadError {
                path: path.to_path_buf(),
                error: e.to_string(),
            }
        })?;

        // Detect format
        let format = config_detector::detect_format(&content, path)?;

        // Analyze complexity
        let complexity = complexity::analyze_complexity(&content, format);

        // Generate recommendation
        let recommendation = recommender::recommend_tool(&complexity);

        Ok(Analysis {
            format,
            complexity,
            recommendation,
        })
    }
}

impl Default for ConfigAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
