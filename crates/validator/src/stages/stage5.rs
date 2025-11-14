//! Stage 5: JSON Generation & Deployment
//!
//! Converts validated markdown to JSON post format and deploys to content/posts/
//! - Calls md2json parser to generate JSON
//! - Injects execution metadata from Stage 4
//! - Validates JSON against schema
//! - Copies JSON to content/posts/ directory
//! - Updates deployment status tracking

use crate::config::Stage5Config;
use crate::validators::ValidationResult;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use url::Url;

pub struct Stage5Validator<'a> {
    config: &'a Stage5Config,
}

impl<'a> Stage5Validator<'a> {
    pub fn new(config: &'a Stage5Config) -> Self {
        Self { config }
    }

    pub fn validate(
        &self,
        markdown_path: &Path,
        stage4_result: Option<&ValidationResult>,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::new("Stage 5: JSON Generation & Deployment");

        // 1. Generate JSON from markdown using md2json parser
        let json_value = match self.generate_json(markdown_path) {
            Ok(json) => json,
            Err(e) => {
                result.error(
                    None,
                    format!("Failed to generate JSON: {e}"),
                    Some("Check markdown structure and ensure parser is built".to_string()),
                );
                return Ok(result);
            }
        };

        // 2. Inject execution metadata from Stage 4 if available
        let enriched_json = if self.config.inject_metadata && stage4_result.is_some() {
            self.add_metadata(json_value, stage4_result.unwrap())?
        } else {
            json_value
        };

        // 3. Validate JSON against schema if enabled
        if self.config.validate_schema {
            if let Err(e) = self.validate_json_schema(&enriched_json) {
                result.error(
                    None,
                    format!("Schema validation failed: {e}"),
                    Some(
                        "Review markdown structure and ensure all required fields are present"
                            .to_string(),
                    ),
                );
                return Ok(result);
            }
            // Add metadata flag to indicate schema validation ran successfully
            result.add_metadata("schema_validated", "true".to_string());
        }

        // 4. Deploy JSON if auto_deploy is enabled
        let deployed_path = if self.config.auto_deploy {
            match self.deploy_json(markdown_path, &enriched_json) {
                Ok(path) => {
                    result.add_metadata("deployed_to", path.display().to_string());
                    Some(path)
                }
                Err(e) => {
                    result.error(
                        None,
                        format!("Deployment failed: {e}"),
                        Some("Check output directory permissions".to_string()),
                    );
                    return Ok(result);
                }
            }
        } else {
            None
        };

        // 5. Update status tracking if enabled
        if self.config.track_status && deployed_path.is_some() {
            if let Err(e) = self.update_status(markdown_path, &enriched_json) {
                result.warning(
                    None,
                    format!("Failed to update status tracking: {e}"),
                    Some("Deployment succeeded but status file not updated".to_string()),
                );
            } else {
                // Add metadata flag to indicate status was successfully updated
                result.add_metadata("status_updated", "true".to_string());
            }
        }

        Ok(result)
    }

    /// Generate JSON from markdown using md2json parser
    fn generate_json(&self, markdown_path: &Path) -> Result<Value> {
        // Find the md2json parser binary
        let parser_path = self.find_parser_binary()?;

        // Create temp file for JSON output (platform-independent)
        let temp_dir = std::env::temp_dir();
        let temp_json_path = temp_dir.join(format!("stage5_{}.json", std::process::id()));

        // Execute parser with output file
        let output = Command::new(&parser_path)
            .arg(markdown_path)
            .arg("--output")
            .arg(&temp_json_path)
            .output()
            .with_context(|| format!("Failed to execute parser: {}", parser_path.display()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Parser failed: {stderr}");
        }

        // Read generated JSON file
        let json_content =
            fs::read_to_string(&temp_json_path).context("Failed to read parser output file")?;

        let json: Value = serde_json::from_str(&json_content).context("Failed to parse JSON")?;

        // Clean up temp file
        let _ = fs::remove_file(&temp_json_path);

        Ok(json)
    }

    /// Find the md2json parser binary (builds if missing)
    fn find_parser_binary(&self) -> Result<PathBuf> {
        // Platform-specific binary name
        let binary_name = if cfg!(windows) {
            "md2json.exe"
        } else {
            "md2json"
        };

        // Try workspace target directory first (most common for workspace builds)
        let workspace_release_path = PathBuf::from("target/release").join(binary_name);
        if workspace_release_path.exists() {
            return Ok(workspace_release_path);
        }

        let workspace_debug_path = PathBuf::from("target/debug").join(binary_name);
        if workspace_debug_path.exists() {
            return Ok(workspace_debug_path);
        }

        // Try crate-specific target directory (for standalone builds)
        let release_path = PathBuf::from("crates/parser/target/release").join(binary_name);
        if release_path.exists() {
            return Ok(release_path);
        }

        let debug_path = PathBuf::from("crates/parser/target/debug").join(binary_name);
        if debug_path.exists() {
            return Ok(debug_path);
        }

        // Parser not found - attempt to build it automatically
        eprintln!("md2json parser not found, building...");

        let parser_dir = PathBuf::from("crates/parser");
        if !parser_dir.exists() {
            bail!(
                "Parser directory not found at './crates/parser'. \
                Cannot build parser automatically."
            );
        }

        // Build parser in release mode
        let build_output = Command::new("cargo")
            .current_dir(&parser_dir)
            .args(["build", "--release"])
            .output()
            .context("Failed to run 'cargo build' for parser")?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            bail!("Parser build failed:\n{stderr}");
        }

        eprintln!("✓ Parser built successfully");

        // Check again after building - try workspace target first (where cargo puts it)
        if workspace_release_path.exists() {
            return Ok(workspace_release_path);
        }

        if release_path.exists() {
            return Ok(release_path);
        }

        bail!(
            "Parser built but binary not found at expected locations:\n  - {}\n  - {}",
            workspace_release_path.display(),
            release_path.display()
        )
    }

    /// Inject execution metadata from Stage 4 into JSON
    fn add_metadata(&self, mut json: Value, stage4_result: &ValidationResult) -> Result<Value> {
        // Ensure metadata object exists
        if json.get("metadata").is_none() {
            json["metadata"] = json!({});
        }

        // Add validation metadata
        json["metadata"]["validated_at"] = json!(chrono::Utc::now().to_rfc3339());
        json["metadata"]["validator_version"] = json!(env!("CARGO_PKG_VERSION"));

        // Add Stage 4 execution metadata if available
        if let Some(execution_time) = stage4_result.metadata.get("execution_time_ms") {
            json["metadata"]["execution_time_ms"] =
                json!(execution_time.parse::<u64>().unwrap_or(0));
        }

        if let Some(data_prep_status) = stage4_result.metadata.get("data_prep_status") {
            json["metadata"]["data_prep_status"] = json!(data_prep_status);
        }

        if let Some(analysis_status) = stage4_result.metadata.get("analysis_status") {
            json["metadata"]["analysis_status"] = json!(analysis_status);
        }

        // Add warning count
        json["metadata"]["warnings"] = json!(stage4_result.warnings.len());

        Ok(json)
    }

    /// Validate JSON against post schema
    fn validate_json_schema(&self, json: &Value) -> Result<()> {
        use jsonschema::JSONSchema;

        // Build schema directory path
        let schema_dir = PathBuf::from(&self.config.schema_dir);

        if !schema_dir.exists() {
            bail!("Schema directory not found: {}", schema_dir.display());
        }

        // Discover and load all schema files
        let mut schema_documents: Vec<(String, Value)> = Vec::new();

        for entry in fs::read_dir(&schema_dir)
            .with_context(|| format!("Failed to read schema directory: {}", schema_dir.display()))?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            // Only process .json files
            if !path.extension().map(|e| e == "json").unwrap_or(false) {
                continue;
            }

            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", path.display()))?;

            // Load schema content
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read schema: {}", path.display()))?;

            let mut schema_json: Value = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse schema JSON: {}", path.display()))?;

            // Ensure schema has an absolute $id using file:// URL
            // Get canonical path for absolute file URL
            let canonical_path = path
                .canonicalize()
                .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

            // Convert to file:// URL using url crate (handles Windows UNC paths correctly)
            let file_url = Url::from_file_path(&canonical_path)
                .map_err(|_| {
                    anyhow::anyhow!(
                        "Failed to convert path to file URL: {}",
                        canonical_path.display()
                    )
                })?
                .to_string();

            // Inject $id if not present or update if present
            schema_json["$id"] = json!(file_url);

            schema_documents.push((filename.to_string(), schema_json));
        }

        if schema_documents.is_empty() {
            bail!(
                "No schema files found in directory: {}",
                schema_dir.display()
            );
        }

        // Find the root schema (post.schema.json)
        let root_schema = schema_documents
            .iter()
            .find(|(name, _)| name == "post.schema.json")
            .map(|(_, schema)| schema.clone())
            .ok_or_else(|| anyhow::anyhow!("post.schema.json not found in schema directory"))?;

        // Build JSONSchema with all documents pre-loaded
        let mut options = JSONSchema::options();

        for (name, schema) in &schema_documents {
            let id = schema
                .get("$id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Schema {name} missing $id field"))?;

            options.with_document(id.to_string(), schema.clone());
        }

        // Compile root schema with all references pre-loaded
        let compiled = options
            .compile(&root_schema)
            .map_err(|e| anyhow::anyhow!("Schema compilation failed: {e}"))?;

        // Validate JSON against schema
        if let Err(errors) = compiled.validate(json) {
            let error_messages: Vec<String> = errors.map(|e| format!("- {e}")).collect();

            bail!("JSON validation failed:\n{}", error_messages.join("\n"));
        }

        Ok(())
    }

    /// Deploy JSON to output directory
    fn deploy_json(&self, markdown_path: &Path, json: &Value) -> Result<PathBuf> {
        // Extract slug - check multiple possible locations
        let slug = if let Some(slug_str) = json["slug"].as_str() {
            slug_str.to_string()
        } else if let Some(slug_str) = json["frontmatter"]["slug"].as_str() {
            slug_str.to_string()
        } else if let Some(slug_str) = json["metadata"]["slug"].as_str() {
            slug_str.to_string()
        } else {
            // Fallback: derive slug from markdown filename
            markdown_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Could not derive slug from filename"))?
        };

        // Construct output path
        let output_dir = PathBuf::from(&self.config.output_dir);
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).with_context(|| {
                format!(
                    "Failed to create output directory: {}",
                    output_dir.display()
                )
            })?;
        }

        let output_path = output_dir.join(format!("{slug}.post.json"));

        // Check if file exists and overwrite is disabled
        if output_path.exists() && !self.config.overwrite_existing {
            bail!(
                "Output file already exists: {}. Set overwrite_existing = true to replace",
                output_path.display()
            );
        }

        // Write JSON file (pretty-printed)
        let json_str = serde_json::to_string_pretty(json).context("Failed to serialize JSON")?;

        fs::write(&output_path, json_str)
            .with_context(|| format!("Failed to write JSON: {}", output_path.display()))?;

        Ok(output_path)
    }

    /// Update deployment status tracking
    fn update_status(&self, markdown_path: &Path, json: &Value) -> Result<()> {
        let status_path = PathBuf::from(&self.config.status_file);

        // Extract slug - check multiple possible locations with fallback
        let slug = if let Some(slug_str) = json["slug"].as_str() {
            slug_str.to_string()
        } else if let Some(slug_str) = json["frontmatter"]["slug"].as_str() {
            slug_str.to_string()
        } else if let Some(slug_str) = json["metadata"]["slug"].as_str() {
            slug_str.to_string()
        } else {
            // Fallback: derive slug from markdown filename
            markdown_path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Could not derive slug from filename"))?
        };

        // Create status entry
        let status_entry = json!({
            "file": markdown_path.display().to_string(),
            "slug": slug,
            "validated_at": chrono::Utc::now().to_rfc3339(),
            "stage": "deployed",
        });

        // Load existing status history
        let mut history: Vec<Value> = if status_path.exists() {
            let content = fs::read_to_string(&status_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        // Append new entry
        history.push(status_entry);

        // Ensure status directory exists
        if let Some(parent) = status_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write updated history
        let history_str = serde_json::to_string_pretty(&history)?;
        fs::write(&status_path, history_str)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_parser_binary() {
        let config = Stage5Config {
            auto_deploy: false,
            output_dir: "content/posts".to_string(),
            schema_dir: "config/schemas".to_string(),
            track_status: false,
            status_file: "generated/validation/status.json".to_string(),
            overwrite_existing: true,
            inject_metadata: true,
            validate_schema: true,
        };

        let validator = Stage5Validator::new(&config);

        // Should find or build parser binary automatically
        let result = validator.find_parser_binary();
        assert!(
            result.is_ok(),
            "Parser binary should be found or built automatically. Error: {:?}",
            result.err()
        );

        // Verify the path exists
        if let Ok(path) = result {
            assert!(
                path.exists(),
                "Parser binary path should exist: {}",
                path.display()
            );
        }
    }

    #[test]
    fn test_metadata_injection() {
        let config = Stage5Config {
            auto_deploy: false,
            output_dir: "content/posts".to_string(),
            schema_dir: "config/schemas".to_string(),
            track_status: false,
            status_file: "generated/validation/status.json".to_string(),
            overwrite_existing: true,
            inject_metadata: true,
            validate_schema: true,
        };

        let validator = Stage5Validator::new(&config);

        let json = json!({
            "frontmatter": {
                "slug": "test-post"
            }
        });

        let mut stage4_result = ValidationResult::new("Stage 4");
        stage4_result.add_metadata("execution_time_ms", "1500".to_string());
        stage4_result.add_metadata("data_prep_status", "passing".to_string());

        let enriched = validator.add_metadata(json, &stage4_result).unwrap();

        assert!(enriched["metadata"]["validated_at"].is_string());
        assert_eq!(enriched["metadata"]["execution_time_ms"], 1500);
        assert_eq!(enriched["metadata"]["data_prep_status"], "passing");
    }

    #[test]
    fn test_schema_validation_valid_json() {
        let config = Stage5Config {
            auto_deploy: false,
            output_dir: "content/posts".to_string(),
            schema_dir: "config/schemas".to_string(),
            track_status: false,
            status_file: "generated/validation/status.json".to_string(),
            overwrite_existing: true,
            inject_metadata: false,
            validate_schema: true,
        };

        let validator = Stage5Validator::new(&config);

        // Create a minimal valid post JSON matching actual schema requirements
        let valid_json = json!({
            "title": "Test Post",
            "overview": {
                "summary": "Test summary"
            },
            "data_access": {
                "items": []
            },
            "data_preparation": {
                "content_blocks": [
                    {
                        "type": "code",
                        "data": {
                            "title": "Test Code",
                            "content": "test content",
                            "language": "r"
                        }
                    }
                ]
            },
            "statistical_analysis": {
                "content_blocks": [
                    {
                        "type": "code",
                        "data": {
                            "title": "Test Analysis",
                            "content": "test content",
                            "language": "r"
                        }
                    }
                ]
            },
            "discussion": {
                "paragraphs": ["Test discussion paragraph"]
            },
            "additional_resources": {
                "cards": []
            }
        });

        // Should pass validation
        let result = validator.validate_json_schema(&valid_json);
        assert!(
            result.is_ok(),
            "Valid JSON should pass schema validation. Error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_schema_validation_invalid_json() {
        let config = Stage5Config {
            auto_deploy: false,
            output_dir: "content/posts".to_string(),
            schema_dir: "config/schemas".to_string(),
            track_status: false,
            status_file: "generated/validation/status.json".to_string(),
            overwrite_existing: true,
            inject_metadata: false,
            validate_schema: true,
        };

        let validator = Stage5Validator::new(&config);

        // Create invalid JSON (missing required 'title' field)
        let invalid_json = json!({
            // Missing "title" field - required by schema
            "overview": {
                "summary": "Test summary"
            },
            "data_access": {
                "items": []
            },
            "data_preparation": {
                "content_blocks": [
                    {
                        "type": "code",
                        "data": {
                            "title": "Test",
                            "content": "test",
                            "language": "r"
                        }
                    }
                ]
            },
            "statistical_analysis": {
                "content_blocks": [
                    {
                        "type": "code",
                        "data": {
                            "title": "Test",
                            "content": "test",
                            "language": "r"
                        }
                    }
                ]
            },
            "discussion": {
                "paragraphs": ["Test"]
            },
            "additional_resources": {
                "cards": []
            }
        });

        // Should fail validation
        let result = validator.validate_json_schema(&invalid_json);
        assert!(
            result.is_err(),
            "Invalid JSON (missing 'title') should fail schema validation"
        );

        // Check error message mentions the problem
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("title") || err_msg.contains("required"),
            "Error message should mention the missing required field. Got: {err_msg}"
        );
    }
}
