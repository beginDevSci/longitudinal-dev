/*!
 * Configuration module
 *
 * Loads and parses validation.toml config file
 */

use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub cache: CacheConfig,
    pub stages: StagesConfig,
    pub stage1: Stage1Config,
    pub stage2: Stage2Config,
    pub stage3: Stage3Config,
    pub stage4: Stage4Config,
    pub stage5: Stage5Config,
    pub r: RConfig,
    #[allow(dead_code)]
    pub output: OutputConfig,
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub directory: String,
    #[allow(dead_code)]
    pub max_age_hours: u64,
}

#[derive(Debug, Deserialize)]
pub struct StagesConfig {
    pub enabled: Vec<u8>,
    /// Languages that skip R validation stages (2, 3, 4)
    #[serde(default)]
    pub skip_r_stages_for_languages: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Stage1Config {
    pub required_frontmatter: Vec<String>,
    pub required_sections: Vec<String>,
    pub valid_markers: Vec<String>,
    pub frontmatter_types: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Stage2Config {
    #[allow(dead_code)]
    pub check_string_literals: bool,
    #[allow(dead_code)]
    pub check_custom_functions: bool,
    #[allow(dead_code)]
    pub allowed_functions_file: String,
    pub require_libraries_first: bool,
}

#[derive(Debug, Deserialize)]
pub struct Stage3Config {
    pub sample_rows: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct Stage4Config {
    pub timeout_seconds: u64,
    pub capture_warnings: bool,
    pub treat_warnings_as: String,
}

#[derive(Debug, Deserialize)]
pub struct Stage5Config {
    pub auto_deploy: bool,
    pub output_dir: String,
    pub schema_dir: String,
    pub track_status: bool,
    pub status_file: String,
    pub overwrite_existing: bool,
    pub inject_metadata: bool,
    pub validate_schema: bool,
}

#[derive(Debug, Deserialize)]
pub struct RConfig {
    pub executable: String,
    pub required_packages: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    #[allow(dead_code)]
    pub terminal: bool,
    #[allow(dead_code)]
    pub log_file: bool,
    #[allow(dead_code)]
    pub log_directory: String,
    #[allow(dead_code)]
    pub verbose: bool,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;

        // Allow R_EXECUTABLE env var to override config
        if let Ok(r_executable) = std::env::var("R_EXECUTABLE") {
            config.r.executable = r_executable;
        }

        Ok(config)
    }
}
