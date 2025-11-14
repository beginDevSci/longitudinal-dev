//! Stage 2: R Syntax Validation
//!
//! Validates R code in markdown tutorials without executing code.
//! Checks:
//! - R syntax using Rscript parse()
//! - Library calls in first code block
//! - Optional: String literals, function allowlist

use crate::config::{RConfig, Stage2Config};
use crate::validators::ValidationResult;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Represents a single R code block extracted from markdown
#[derive(Debug)]
pub struct RCodeBlock {
    pub content: String,
    pub line_number: usize,
    pub block_index: usize,
}

pub struct Stage2Validator<'a> {
    config: &'a Stage2Config,
    r_config: &'a RConfig,
}

impl<'a> Stage2Validator<'a> {
    pub fn new(config: &'a Stage2Config, r_config: &'a RConfig) -> Self {
        Self { config, r_config }
    }

    pub fn validate(&self, markdown_path: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new("Stage 2: R Syntax Validation");

        // Read markdown file
        let content = fs::read_to_string(markdown_path)?;

        // Parse structure using parser crate
        let structure = longitudinal_parser::validation::parse_structure(&content)?;

        // Extract R code blocks
        let r_blocks = self.extract_r_code_blocks(&content, &structure)?;

        if r_blocks.is_empty() {
            result.error(
                None,
                "No R code blocks found".to_string(),
                Some("Add {.code} sections with ```r code blocks".to_string()),
            );
            return Ok(result);
        }

        // Validate R syntax for each block
        self.validate_r_syntax(&r_blocks, &mut result)?;

        // Check library calls in first block if configured
        if self.config.require_libraries_first {
            self.validate_library_calls(&r_blocks[0], &mut result);
        }

        Ok(result)
    }

    /// Extract all R code blocks from markdown
    fn extract_r_code_blocks(
        &self,
        content: &str,
        structure: &longitudinal_parser::validation::ParsedStructure,
    ) -> Result<Vec<RCodeBlock>> {
        let mut blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Find all subsections with {.code} marker
        let code_subsections: Vec<_> = structure
            .subsections
            .iter()
            .filter(|s| s.marker.as_deref() == Some("code"))
            .collect();

        for subsection in code_subsections {
            // Find the code fence after this heading
            if let Some(code_content) =
                self.extract_code_fence_content(&lines, subsection.line_number)
            {
                blocks.push(RCodeBlock {
                    content: code_content.code,
                    line_number: code_content.start_line,
                    block_index: blocks.len(),
                });
            }
        }

        Ok(blocks)
    }

    /// Extract code fence content starting from a given line
    fn extract_code_fence_content(
        &self,
        lines: &[&str],
        start_line: usize,
    ) -> Option<CodeFenceContent> {
        // Convert to 0-based index
        let start_idx = start_line.saturating_sub(1);

        // Find the first ```r after this line
        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            if line.trim().starts_with("```r") {
                // Found start of code fence
                let mut code_lines = Vec::new();
                let fence_start_line = i + 1; // Convert back to 1-based

                // Collect lines until closing ```
                for (_j, code_line) in lines.iter().enumerate().skip(i + 1) {
                    if code_line.trim().starts_with("```") {
                        // Found end of code fence
                        return Some(CodeFenceContent {
                            code: code_lines.join("\n"),
                            start_line: fence_start_line + 1, // Line after ```r
                        });
                    }
                    code_lines.push(*code_line);
                }
            }
        }

        None
    }

    /// Validate R syntax using Rscript
    fn validate_r_syntax(
        &self,
        blocks: &[RCodeBlock],
        result: &mut ValidationResult,
    ) -> Result<()> {
        for block in blocks {
            match self.check_r_syntax(&block.content) {
                Ok(_) => {
                    // Syntax valid - no error
                }
                Err(e) => {
                    result.error(
                        Some(block.line_number),
                        format!(
                            "R syntax error in code block {}: {}",
                            block.block_index + 1,
                            e
                        ),
                        Some("Fix R syntax errors".to_string()),
                    );
                }
            }
        }

        Ok(())
    }

    /// Check R syntax by writing to temp file and using Rscript parse()
    fn check_r_syntax(&self, r_code: &str) -> Result<()> {
        // Create temp file with timestamp to avoid collisions
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis();
        let temp_file = format!("/tmp/validate_r_{timestamp}.R");

        // Write R code to temp file
        fs::write(&temp_file, r_code).context("Failed to write temp R file")?;

        // Determine Rscript path:
        // 1. Try config.r.executable first
        // 2. Fall back to "Rscript" in PATH
        // 3. Emit clear error if neither works

        let rscript_path = &self.r_config.executable;

        // Try configured path first
        let output_result = Command::new(rscript_path)
            .args(["--vanilla", "-e", &format!("parse('{temp_file}')")])
            .output();

        let output = match output_result {
            Ok(out) => out,
            Err(e) if rscript_path != "Rscript" => {
                // Config path failed, try PATH fallback
                Command::new("Rscript")
                    .args([
                        "--vanilla",
                        "-e",
                        &format!("parse('{temp_file}')"),
                    ])
                    .output()
                    .with_context(|| format!(
                        "Failed to execute Rscript. Tried:\n  1. {rscript_path} (config.r.executable): {e}\n  2. Rscript (PATH): not found\n\nPlease install R or update config.r.executable"
                    ))?
            }
            Err(e) => {
                // PATH fallback also failed
                let _ = fs::remove_file(&temp_file);
                return Err(anyhow::anyhow!(
                    "Failed to execute Rscript from PATH: {e}\n\nPlease install R or set config.r.executable to the full path"
                ));
            }
        };

        // Clean up temp file
        let _ = fs::remove_file(&temp_file);

        // Check if syntax check passed
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Extract just the error message, removing warnings
            let error_msg = stderr
                .lines()
                .filter(|line| line.contains("Error") || line.contains("unexpected"))
                .collect::<Vec<_>>()
                .join("\n");

            let msg = if error_msg.is_empty() {
                stderr.trim().to_string()
            } else {
                error_msg
            };

            return Err(anyhow::anyhow!("{msg}"));
        }

        Ok(())
    }

    /// Validate that first code block contains library() calls
    fn validate_library_calls(&self, first_block: &RCodeBlock, result: &mut ValidationResult) {
        let has_library =
            first_block.content.contains("library(") || first_block.content.contains("require(");

        if !has_library {
            result.error(
                Some(first_block.line_number),
                "First code block must load required packages".to_string(),
                Some(
                    "Add library() or require() calls (e.g., library(tidyverse), library(lavaan))"
                        .to_string(),
                ),
            );
        }
    }
}

/// Helper struct to track code fence content and location
#[derive(Debug)]
struct CodeFenceContent {
    code: String,
    start_line: usize,
}
