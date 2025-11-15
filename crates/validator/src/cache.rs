/*!
 * Cache module
 *
 * Implements smart hash-based caching with:
 * - Tutorial slug in hash (prevent collisions)
 * - Validator version in hash (auto-invalidate)
 * - Markdown content, data version, package versions
 */

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedResult {
    pub cache_key: String,
    pub cached_at: DateTime<Utc>,
    pub markdown_path: String,
    pub result: ValidationResult,
    pub metadata: ValidationMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub status: String, // "passed" or "failed"
    pub execution_time_ms: u64,
    pub stages: StageResults,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageResults {
    pub stage1: String,
    pub stage2: String,
    pub stage3: String,
    pub stage4: String,
    pub stage5: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationMetadata {
    pub abcd_version: String,
    pub r_version: String,
    pub packages: std::collections::HashMap<String, String>,
    pub validator_version: String,
}

pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
        }
    }

    /// Compute cache key including slug, validator version, content, data version, and package versions
    pub fn compute_cache_key(&self, markdown_path: &Path) -> Result<String> {
        let mut hasher = md5::Context::new();

        // Include tutorial slug to prevent collisions
        if let Some(stem) = markdown_path.file_stem() {
            hasher.consume(stem.to_string_lossy().as_bytes());
        }

        // Include validator version (from Cargo.toml)
        let validator_version = env!("CARGO_PKG_VERSION");
        hasher.consume(validator_version.as_bytes());

        // Hash markdown content
        let content = fs::read_to_string(markdown_path)?;
        hasher.consume(content.as_bytes());

        // TODO: Hash ABCD data version
        // let data_version = self.get_abcd_data_version()?;
        // hasher.consume(data_version.as_bytes());

        // TODO: Hash R package versions
        // let pkg_versions = self.get_r_package_versions(&["tidyverse", "lavaan", "NBDCtools"])?;
        // hasher.consume(pkg_versions.as_bytes());

        Ok(format!("{:x}", hasher.compute()))
    }

    /// Check if cached result exists and is valid
    pub fn get_cached(&self, cache_key: &str) -> Option<CachedResult> {
        let cache_path = self.cache_dir.join(format!("{cache_key}.json"));

        if !cache_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&cache_path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save validation result to cache
    #[allow(dead_code)]
    pub fn save_cached(&self, cached_result: &CachedResult) -> Result<()> {
        // Ensure cache directory exists
        fs::create_dir_all(&self.cache_dir)?;

        let cache_path = self
            .cache_dir
            .join(format!("{}.json", cached_result.cache_key));
        let json = serde_json::to_string_pretty(cached_result)?;
        fs::write(cache_path, json)?;

        Ok(())
    }
}
