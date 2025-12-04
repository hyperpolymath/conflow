// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025 conflow contributors

//! Filesystem-based cache implementation
//!
//! Stores cache entries as JSON files in a cache directory.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{Cache, CacheStats, CachedEntry, CachedResult, ContentHasher};
use crate::errors::ConflowError;
use crate::executors::ExecutionResult;
use crate::pipeline::Stage;

/// Filesystem-based cache
pub struct FilesystemCache {
    /// Cache directory
    cache_dir: PathBuf,
    /// Base directory for resolving relative paths
    base_dir: PathBuf,
}

impl FilesystemCache {
    /// Create a new filesystem cache
    pub fn new(cache_dir: PathBuf, base_dir: PathBuf) -> Result<Self, ConflowError> {
        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).map_err(|e| ConflowError::CacheError {
                message: format!("Failed to create cache directory: {}", e),
            })?;
        }

        Ok(Self { cache_dir, base_dir })
    }

    /// Create cache with default directory
    pub fn default_cache(base_dir: PathBuf) -> Result<Self, ConflowError> {
        let cache_dir = base_dir.join(".conflow").join("cache");
        Self::new(cache_dir, base_dir)
    }

    /// Compute cache key for a stage
    fn cache_key(&self, stage: &Stage) -> Result<String, ConflowError> {
        let mut hasher = ContentHasher::new();
        hasher.hash_stage(stage, &self.base_dir)
    }

    /// Get path for a cache entry
    fn cache_path(&self, key: &str) -> PathBuf {
        // Use first 2 chars as directory for better filesystem performance
        let (prefix, rest) = key.split_at(2.min(key.len()));
        self.cache_dir.join(prefix).join(format!("{}.json", rest))
    }

    /// List all cache entries
    async fn list_entries(&self) -> Result<Vec<CachedEntry>, ConflowError> {
        let mut entries = Vec::new();

        if !self.cache_dir.exists() {
            return Ok(entries);
        }

        // Walk cache directory
        for prefix_dir in std::fs::read_dir(&self.cache_dir)
            .map_err(|e| ConflowError::CacheError {
                message: format!("Failed to read cache directory: {}", e),
            })?
        {
            let prefix_dir = prefix_dir
                .map_err(|e| ConflowError::CacheError {
                    message: format!("Failed to read cache entry: {}", e),
                })?
                .path();

            if !prefix_dir.is_dir() {
                continue;
            }

            for entry_file in std::fs::read_dir(&prefix_dir)
                .map_err(|e| ConflowError::CacheError {
                    message: format!("Failed to read cache subdirectory: {}", e),
                })?
            {
                let entry_file = entry_file
                    .map_err(|e| ConflowError::CacheError {
                        message: format!("Failed to read cache file: {}", e),
                    })?
                    .path();

                if entry_file.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }

                // Read and parse entry
                if let Ok(content) = std::fs::read_to_string(&entry_file) {
                    if let Ok(entry) = serde_json::from_str::<CachedEntry>(&content) {
                        entries.push(entry);
                    }
                }
            }
        }

        Ok(entries)
    }
}

#[async_trait]
impl Cache for FilesystemCache {
    async fn get(&self, stage: &Stage) -> Result<Option<ExecutionResult>, ConflowError> {
        let key = self.cache_key(stage)?;
        let path = self.cache_path(&key);

        if !path.exists() {
            return Ok(None);
        }

        // Read cached entry
        let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
            ConflowError::CacheError {
                message: format!("Failed to read cache entry: {}", e),
            }
        })?;

        let entry: CachedEntry = serde_json::from_str(&content).map_err(|e| {
            ConflowError::CacheError {
                message: format!("Failed to parse cache entry: {}", e),
            }
        })?;

        // Verify outputs still exist
        for output in &entry.result.outputs {
            if !output.exists() {
                // Cache invalid - outputs missing
                // Delete the stale entry
                let _ = tokio::fs::remove_file(&path).await;
                return Ok(None);
            }
        }

        // Convert to ExecutionResult
        let mut result: ExecutionResult = entry.result.into();
        result.cache_hit = true;

        Ok(Some(result))
    }

    async fn store(&self, stage: &Stage, result: &ExecutionResult) -> Result<(), ConflowError> {
        let key = self.cache_key(stage)?;
        let path = self.cache_path(&key);

        // Create parent directory
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                ConflowError::CacheError {
                    message: format!("Failed to create cache directory: {}", e),
                }
            })?;
        }

        let entry = CachedEntry {
            timestamp: SystemTime::now(),
            stage_name: stage.name.clone(),
            cache_key: key,
            result: CachedResult::from(result),
        };

        let json = serde_json::to_string_pretty(&entry).map_err(|e| ConflowError::CacheError {
            message: format!("Failed to serialize cache entry: {}", e),
        })?;

        tokio::fs::write(&path, json).await.map_err(|e| ConflowError::CacheError {
            message: format!("Failed to write cache entry: {}", e),
        })?;

        Ok(())
    }

    async fn invalidate(&self, stage: &Stage) -> Result<(), ConflowError> {
        let key = self.cache_key(stage)?;
        let path = self.cache_path(&key);

        if path.exists() {
            tokio::fs::remove_file(&path).await.map_err(|e| {
                ConflowError::CacheError {
                    message: format!("Failed to remove cache entry: {}", e),
                }
            })?;
        }

        Ok(())
    }

    async fn clear(&self) -> Result<(), ConflowError> {
        if self.cache_dir.exists() {
            tokio::fs::remove_dir_all(&self.cache_dir).await.map_err(|e| {
                ConflowError::CacheError {
                    message: format!("Failed to clear cache: {}", e),
                }
            })?;

            tokio::fs::create_dir_all(&self.cache_dir).await.map_err(|e| {
                ConflowError::CacheError {
                    message: format!("Failed to recreate cache directory: {}", e),
                }
            })?;
        }

        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats, ConflowError> {
        let entries = self.list_entries().await?;

        let mut stats = CacheStats {
            entries: entries.len(),
            size_bytes: 0,
            oldest_entry: None,
            newest_entry: None,
        };

        for entry in &entries {
            // Update timestamps
            match stats.oldest_entry {
                None => stats.oldest_entry = Some(entry.timestamp),
                Some(oldest) if entry.timestamp < oldest => {
                    stats.oldest_entry = Some(entry.timestamp)
                }
                _ => {}
            }

            match stats.newest_entry {
                None => stats.newest_entry = Some(entry.timestamp),
                Some(newest) if entry.timestamp > newest => {
                    stats.newest_entry = Some(entry.timestamp)
                }
                _ => {}
            }
        }

        // Calculate total size
        if self.cache_dir.exists() {
            stats.size_bytes = Self::dir_size(&self.cache_dir)?;
        }

        Ok(stats)
    }
}

impl FilesystemCache {
    /// Calculate directory size recursively
    fn dir_size(path: &Path) -> Result<u64, ConflowError> {
        let mut size = 0;

        if path.is_file() {
            return Ok(path.metadata().map(|m| m.len()).unwrap_or(0));
        }

        for entry in std::fs::read_dir(path).map_err(|e| ConflowError::CacheError {
            message: format!("Failed to read directory: {}", e),
        })? {
            let entry = entry.map_err(|e| ConflowError::CacheError {
                message: format!("Failed to read entry: {}", e),
            })?;

            let path = entry.path();
            if path.is_dir() {
                size += Self::dir_size(&path)?;
            } else {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{CueCommand, Input, Tool};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn make_test_stage(name: &str) -> Stage {
        Stage {
            name: name.into(),
            description: None,
            tool: Tool::Cue {
                command: CueCommand::Vet,
                schemas: vec![],
                flags: vec![],
                out_format: None,
            },
            input: Input::Single("*.json".into()),
            output: None,
            depends_on: vec![],
            allow_failure: false,
            env: HashMap::new(),
            condition: None,
        }
    }

    #[tokio::test]
    async fn test_cache_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            FilesystemCache::new(temp_dir.path().to_path_buf(), temp_dir.path().to_path_buf())
                .unwrap();

        let stage = make_test_stage("test");
        let result = ExecutionResult::success(
            "output".into(),
            std::time::Duration::from_millis(100),
            vec![],
        );

        // Store
        cache.store(&stage, &result).await.unwrap();

        // Retrieve
        let cached = cache.get(&stage).await.unwrap();
        assert!(cached.is_some());

        let cached = cached.unwrap();
        assert!(cached.cache_hit);
        assert_eq!(cached.stdout, "output");
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            FilesystemCache::new(temp_dir.path().to_path_buf(), temp_dir.path().to_path_buf())
                .unwrap();

        let stage = make_test_stage("test");
        let result = ExecutionResult::success(
            "output".into(),
            std::time::Duration::from_millis(100),
            vec![],
        );

        // Store
        cache.store(&stage, &result).await.unwrap();

        // Verify stored
        assert!(cache.get(&stage).await.unwrap().is_some());

        // Invalidate
        cache.invalidate(&stage).await.unwrap();

        // Verify gone
        assert!(cache.get(&stage).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache =
            FilesystemCache::new(temp_dir.path().to_path_buf(), temp_dir.path().to_path_buf())
                .unwrap();

        let stage = make_test_stage("test");
        let result = ExecutionResult::success(
            "output".into(),
            std::time::Duration::from_millis(100),
            vec![],
        );

        cache.store(&stage, &result).await.unwrap();

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.entries, 1);

        cache.clear().await.unwrap();

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.entries, 0);
    }
}
