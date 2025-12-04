// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Content hashing for cache keys
//!
//! Uses BLAKE3 for fast, secure content hashing.

use blake3::Hasher;
use std::path::Path;

use crate::errors::ConflowError;
use crate::pipeline::{Input, Stage};

/// Content hasher for generating cache keys
pub struct ContentHasher {
    hasher: Hasher,
}

impl ContentHasher {
    /// Create a new content hasher
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
        }
    }

    /// Hash a stage configuration and its inputs to create a cache key
    pub fn hash_stage(&mut self, stage: &Stage, base_dir: &Path) -> Result<String, ConflowError> {
        // Hash stage name
        self.hasher.update(stage.name.as_bytes());

        // Hash tool configuration
        let tool_json = serde_json::to_string(&stage.tool)
            .map_err(|e| ConflowError::CacheError {
                message: format!("Failed to serialize tool config: {}", e),
            })?;
        self.hasher.update(tool_json.as_bytes());

        // Hash input specification
        let input_json = serde_json::to_string(&stage.input)
            .map_err(|e| ConflowError::CacheError {
                message: format!("Failed to serialize input config: {}", e),
            })?;
        self.hasher.update(input_json.as_bytes());

        // Hash output specification
        if let Some(ref output) = stage.output {
            let output_json = serde_json::to_string(output)
                .map_err(|e| ConflowError::CacheError {
                    message: format!("Failed to serialize output config: {}", e),
                })?;
            self.hasher.update(output_json.as_bytes());
        }

        // Hash environment variables
        for (k, v) in &stage.env {
            self.hasher.update(k.as_bytes());
            self.hasher.update(v.as_bytes());
        }

        // Hash input file contents
        let input_files = self.collect_input_files(stage, base_dir)?;
        for file in input_files {
            self.hash_file(&file)?;
        }

        Ok(self.hasher.finalize().to_hex().to_string())
    }

    /// Hash a single file's contents
    pub fn hash_file(&mut self, path: &Path) -> Result<(), ConflowError> {
        if !path.exists() {
            return Ok(()); // Don't fail on missing files - they'll be caught later
        }

        let content = std::fs::read(path).map_err(|e| ConflowError::FileReadError {
            path: path.to_path_buf(),
            error: e.to_string(),
        })?;

        self.hasher.update(&content);
        Ok(())
    }

    /// Collect input files for a stage
    fn collect_input_files(
        &self,
        stage: &Stage,
        base_dir: &Path,
    ) -> Result<Vec<std::path::PathBuf>, ConflowError> {
        let patterns = stage.input.patterns();

        if patterns.is_empty() {
            return Ok(vec![]);
        }

        let mut files = Vec::new();

        for pattern in patterns {
            let full_pattern = if Path::new(pattern).is_absolute() {
                pattern.to_string()
            } else {
                base_dir.join(pattern).to_string_lossy().to_string()
            };

            let matches = glob::glob(&full_pattern)
                .map_err(|e| ConflowError::GlobPattern { message: e.to_string() })?;

            for entry in matches {
                if let Ok(path) = entry {
                    files.push(path);
                }
            }
        }

        // Sort for consistent ordering
        files.sort();

        Ok(files)
    }

    /// Hash arbitrary bytes
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize and get the hash
    pub fn finalize(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }
}

impl Default for ContentHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a quick hash of a string
pub fn hash_string(s: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(s.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Compute hash of a file
pub fn hash_file(path: &Path) -> Result<String, ConflowError> {
    let content = std::fs::read(path).map_err(|e| ConflowError::FileReadError {
        path: path.to_path_buf(),
        error: e.to_string(),
    })?;

    let mut hasher = Hasher::new();
    hasher.update(&content);
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("hello");
        let hash2 = hash_string("hello");
        let hash3 = hash_string("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hasher_consistent() {
        let mut hasher1 = ContentHasher::new();
        hasher1.update(b"test data");
        let hash1 = hasher1.finalize();

        let mut hasher2 = ContentHasher::new();
        hasher2.update(b"test data");
        let hash2 = hasher2.finalize();

        assert_eq!(hash1, hash2);
    }
}
