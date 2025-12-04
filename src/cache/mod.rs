// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Caching layer for pipeline results
//!
//! Provides file-based caching to avoid redundant stage executions.

mod filesystem;
mod hash;

pub use filesystem::FilesystemCache;
pub use hash::ContentHasher;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::errors::ConflowError;
use crate::executors::ExecutionResult;
use crate::pipeline::Stage;

/// Trait for cache implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get cached result for a stage
    async fn get(&self, stage: &Stage) -> Result<Option<ExecutionResult>, ConflowError>;

    /// Store result for a stage
    async fn store(&self, stage: &Stage, result: &ExecutionResult) -> Result<(), ConflowError>;

    /// Invalidate cache for a stage
    async fn invalidate(&self, stage: &Stage) -> Result<(), ConflowError>;

    /// Clear all cached results
    async fn clear(&self) -> Result<(), ConflowError>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats, ConflowError>;
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Oldest entry timestamp
    pub oldest_entry: Option<SystemTime>,
    /// Newest entry timestamp
    pub newest_entry: Option<SystemTime>,
}

impl CacheStats {
    /// Format size for display
    pub fn formatted_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size_bytes >= GB {
            format!("{:.2} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.2} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.2} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", self.size_bytes)
        }
    }
}

/// Cached result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedEntry {
    /// When the entry was cached
    pub timestamp: SystemTime,
    /// Stage name
    pub stage_name: String,
    /// Cache key (content hash)
    pub cache_key: String,
    /// The execution result
    pub result: CachedResult,
}

/// Serializable execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub outputs: Vec<std::path::PathBuf>,
    pub duration_ms: u64,
}

impl From<&ExecutionResult> for CachedResult {
    fn from(result: &ExecutionResult) -> Self {
        Self {
            success: result.success,
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
            exit_code: result.exit_code,
            outputs: result.outputs.clone(),
            duration_ms: result.duration.as_millis() as u64,
        }
    }
}

impl From<CachedResult> for ExecutionResult {
    fn from(cached: CachedResult) -> Self {
        Self {
            success: cached.success,
            stdout: cached.stdout,
            stderr: cached.stderr,
            exit_code: cached.exit_code,
            outputs: cached.outputs,
            duration: std::time::Duration::from_millis(cached.duration_ms),
            cache_hit: true,
        }
    }
}
