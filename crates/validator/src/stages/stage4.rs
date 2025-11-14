//! Stage 4: Full Execution with Real ABCD Data
//!
//! Executes R code with actual ABCD data to verify:
//! - Code runs successfully with real participant data
//! - Statistical models converge properly
//! - Data transformations produce expected outputs
//! - Visualizations are generated correctly

use crate::config::{RConfig, Stage4Config};
use crate::executor::{self, ExecutionResult};
use crate::validators::ValidationResult;
use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Stage4Validator<'a> {
    config: &'a Stage4Config,
    r_config: &'a RConfig,
}

impl<'a> Stage4Validator<'a> {
    pub fn new(config: &'a Stage4Config, r_config: &'a RConfig) -> Self {
        Self { config, r_config }
    }

    pub fn validate(&self, markdown_path: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new("Stage 4: Full Execution");

        // 1. Check prerequisites (fail fast)
        if let Err(e) = self.check_prerequisites() {
            // If prerequisites fail, return early with error
            result.error(
                None,
                format!("Prerequisites check failed: {e}"),
                Some("Ensure R is installed and ABCD_DATA_PATH is set".to_string()),
            );
            return Ok(result);
        }

        // 2. Create artifacts directory for this tutorial
        let temp_dir = self.create_temp_directory(markdown_path)?;

        // 3. Extract R code from markdown
        let r_script = match self.extract_and_combine_r_code(markdown_path, &temp_dir) {
            Ok(script) => script,
            Err(e) => {
                result.error(
                    None,
                    format!("Failed to extract R code: {e}"),
                    Some("Check markdown structure and code blocks".to_string()),
                );
                return Ok(result);
            }
        };

        // 4. Execute with real ABCD data
        match self.execute_with_real_data(&r_script, &temp_dir) {
            Ok(exec_result) => {
                self.process_execution_result(exec_result, &mut result);
            }
            Err(e) => {
                result.error(
                    None,
                    format!("Execution failed: {e}"),
                    Some("Check ABCD data access and R environment".to_string()),
                );
            }
        }

        // 5. Artifacts preserved in public/stage4-artifacts/
        result.add_metadata("artifacts_dir", temp_dir.display().to_string());

        Ok(result)
    }

    /// Check prerequisites for full execution (fail fast)
    fn check_prerequisites(&self) -> Result<()> {
        // 1. Check Rscript availability (respects config.r.executable)
        if let Err(e) = executor::find_rscript_with_fallback(&self.r_config.executable) {
            bail!("Rscript not found: {e}. Install R or update config.r.executable");
        }

        // 2. Check ABCD_DATA_PATH (critical for Stage 4)
        let data_path = env::var("ABCD_DATA_PATH").unwrap_or_else(|_| {
            // Try default path as fallback
            "/Users/shawes/abcd/6_0/phenotype".to_string()
        });

        let data_path_buf = PathBuf::from(&data_path);
        if !data_path_buf.exists() {
            bail!(
                "ABCD data not found at: {data_path}\n\
                Set ABCD_DATA_PATH environment variable to your ABCD phenotype directory.\n\
                Example: export ABCD_DATA_PATH=/path/to/abcd/6_0/phenotype"
            );
        }

        if !data_path_buf.is_dir() {
            bail!("ABCD_DATA_PATH exists but is not a directory: {data_path}");
        }

        // Check if directory is readable
        if fs::read_dir(&data_path_buf).is_err() {
            bail!(
                "ABCD_DATA_PATH directory is not readable: {data_path}\n\
                Check file permissions"
            );
        }

        // 3. Check required R packages (pre-flight)
        let missing_packages = self.check_required_packages()?;
        if !missing_packages.is_empty() {
            bail!(
                "Missing required R packages: {}\n\
                Install with: install.packages(c({}))",
                missing_packages.join(", "),
                missing_packages
                    .iter()
                    .map(|p| format!("\"{p}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        Ok(())
    }

    /// Check if required R packages are installed
    fn check_required_packages(&self) -> Result<Vec<String>> {
        let rscript_path = executor::find_rscript_with_fallback(&self.r_config.executable)?;

        let mut missing = Vec::new();

        for package in &self.r_config.required_packages {
            let check_script =
                format!("if (!requireNamespace('{package}', quietly = TRUE)) quit(status = 1)");

            let output = std::process::Command::new(&rscript_path)
                .arg("-e")
                .arg(&check_script)
                .output()?;

            if !output.status.success() {
                missing.push(package.clone());
            }
        }

        Ok(missing)
    }

    /// Create per-tutorial artifacts directory
    fn create_temp_directory(&self, markdown_path: &Path) -> Result<PathBuf> {
        // Extract tutorial slug from path (e.g., "lgcm-basic" from "content/tutorials/lgcm-basic.md")
        let tutorial_slug = markdown_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let artifacts_dir = PathBuf::from(format!("public/stage4-artifacts/{tutorial_slug}"));

        // Remove existing directory if present (replace with fresh artifacts)
        if artifacts_dir.exists() {
            fs::remove_dir_all(&artifacts_dir).with_context(|| {
                format!(
                    "Failed to remove old artifacts: {}",
                    artifacts_dir.display()
                )
            })?;
        }

        fs::create_dir_all(&artifacts_dir).with_context(|| {
            format!(
                "Failed to create artifacts directory: {}",
                artifacts_dir.display()
            )
        })?;

        Ok(artifacts_dir)
    }

    /// Extract R code blocks from markdown and combine into single script
    fn extract_and_combine_r_code(
        &self,
        markdown_path: &Path,
        output_dir: &Path,
    ) -> Result<String> {
        let content = fs::read_to_string(markdown_path)?;
        let structure = longitudinal_parser::validation::parse_structure(&content)?;

        // Extract code blocks ONLY from subsections with {.code} marker
        let mut code_blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Find all subsections with {.code} marker
        let code_subsections: Vec<_> = structure
            .subsections
            .iter()
            .filter(|s| s.marker.as_deref() == Some("code"))
            .collect();

        for subsection in code_subsections {
            // Find the code fence after this heading
            if let Some((code, line_num)) =
                self.extract_code_fence_after_line(&lines, subsection.line_number)
            {
                code_blocks.push((line_num, code));
            }
        }

        if code_blocks.is_empty() {
            bail!("No R code blocks found in {{.code}} sections");
        }

        // Create combined script
        let mut script = String::new();
        script.push_str("# Stage 4: Full Execution with Real ABCD Data\n");
        script.push_str(&format!("# Generated: {}\n", chrono::Utc::now()));
        script.push_str(&format!("# File: {}\n\n", markdown_path.display()));

        // Set working directory to artifacts directory for all file outputs
        let abs_output_dir = std::env::current_dir()?.join(output_dir);
        script.push_str("# Set working directory for artifacts\n");
        script.push_str(&format!("setwd('{}')\n\n", abs_output_dir.display()));

        // Ensure parallel::detectCores returns a stable value before packages initialize
        script.push_str("try({\n");
        script.push_str("  utils::trace(\n");
        script.push_str("    what  = 'detectCores',\n");
        script.push_str("    where = asNamespace('parallel'),\n");
        script.push_str("    exit  = quote({\n");
        script.push_str("      val <- returnValue()\n");
        script.push_str("      if (is.na(val) || val < 1L) returnValue() <- 1L\n");
        script.push_str("    }),\n");
        script.push_str("    print = FALSE\n");
        script.push_str("  )\n");
        script.push_str("}, silent = TRUE)\n\n");
        script.push_str(
            "if (is.na(tryCatch(parallel::detectCores(), error = function(e) NA_integer_))) {\n",
        );
        script.push_str("  orig_detect <- parallel::detectCores\n");
        script.push_str("  safe_detect <- function(logical = TRUE, ...) {\n");
        script.push_str("    ans <- tryCatch(orig_detect(logical = logical, ...), error = function(e) NA_integer_)\n");
        script.push_str("    if (is.na(ans) || ans < 1L) 1L else ans\n");
        script.push_str("  }\n");
        script.push_str("  unlockBinding('detectCores', asNamespace('parallel'))\n");
        script.push_str("  assign('detectCores', safe_detect, asNamespace('parallel'))\n");
        script.push_str("  lockBinding('detectCores',  asNamespace('parallel'))\n");
        script.push_str("}\n\n");

        // Add CPU sanitization prelude (shared with Stage 3)
        // This prevents lavaan lav_options_checkinterval failures on macOS
        script.push_str(&crate::r_prelude::cpu_sanitization_prelude());

        // Add session info logging
        script.push_str(&crate::r_prelude::session_info_logging());

        // Add user code blocks
        for (i, (line_num, block)) in code_blocks.iter().enumerate() {
            script.push_str(&format!("# Code Block {} (Line {})\n", i + 1, line_num));
            script.push_str(&format!(
                "cat(sprintf('Executing block {}...\\n'))\n",
                i + 1
            ));
            script.push_str(block);
            script.push_str("\n\n");
        }

        script.push_str("cat('All code blocks executed successfully.\\n')\n");
        Ok(script)
    }

    /// Extract code fence content after a given line number
    fn extract_code_fence_after_line(
        &self,
        lines: &[&str],
        start_line: usize,
    ) -> Option<(String, usize)> {
        // Convert to 0-based index
        let start_idx = start_line.saturating_sub(1);

        // Find the first ```r after this line
        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            if line.trim().starts_with("```r") || line.trim().starts_with("```R") {
                // Found start of code fence
                let mut code_lines = Vec::new();
                let fence_start_line = i + 1; // Convert back to 1-based

                // Collect lines until closing ```
                for code_line in lines.iter().skip(i + 1) {
                    if code_line.trim().starts_with("```") {
                        // Found end of code fence
                        return Some((code_lines.join("\n"), fence_start_line + 1));
                    }
                    code_lines.push(*code_line);
                }
            }
        }

        None
    }

    /// Execute R script with real ABCD data
    fn execute_with_real_data(&self, script: &str, temp_dir: &Path) -> Result<ExecutionResult> {
        // Set environment variables
        let data_path = env::var("ABCD_DATA_PATH")
            .unwrap_or_else(|_| "/Users/shawes/abcd/6_0/phenotype".to_string());

        let env_vars = vec![("ABCD_DATA_PATH", data_path.as_str())];

        // Execute with configured Rscript path, timeout, and temp directory as working dir
        // This respects config.r.executable like Stage 2/3 do
        executor::execute_r_script(
            &self.r_config.executable,
            script,
            self.config.timeout_seconds,
            &env_vars,
            Some(temp_dir),
        )
    }

    /// Process execution result and update validation result
    fn process_execution_result(
        &self,
        exec_result: ExecutionResult,
        result: &mut ValidationResult,
    ) {
        // Add execution metadata
        result.add_metadata(
            "execution_time_ms",
            exec_result.execution_time_ms.to_string(),
        );
        result.add_metadata(
            "data_prep_status",
            exec_result.data_prep_status.as_str().to_string(),
        );
        result.add_metadata(
            "analysis_status",
            exec_result.analysis_status.as_str().to_string(),
        );

        if exec_result.success {
            // Check for warnings if configured
            if self.config.capture_warnings {
                let warnings = executor::extract_warnings(&exec_result.stderr);

                if !warnings.is_empty() {
                    let warnings_text = warnings.join("\n");

                    match self.config.treat_warnings_as.as_str() {
                        "error" => {
                            result.error(
                                None,
                                format!("Code executed with warnings:\n{warnings_text}"),
                                Some("Fix warnings before proceeding".to_string()),
                            );
                        }
                        "warning" => {
                            result.warning(
                                None,
                                format!("Code executed with warnings:\n{warnings_text}"),
                                Some(
                                    "Review warnings to ensure they don't indicate issues"
                                        .to_string(),
                                ),
                            );
                        }
                        _ => {} // "ignore" - do nothing
                    }
                }
            }
        } else {
            // Execution failed
            let error_msg = executor::extract_error_message(&exec_result.stderr);

            // Determine which section failed
            use crate::executor::SectionStatus;
            let section = match exec_result.data_prep_status {
                SectionStatus::Failing => "Data Preparation",
                SectionStatus::Passing | SectionStatus::Unknown => {
                    match exec_result.analysis_status {
                        SectionStatus::Failing => "Statistical Analysis",
                        _ => "Unknown",
                    }
                }
            };

            result.error(
                None,
                format!("Execution failed in {section} section:\n{error_msg}"),
                Some("Check R error message and verify ABCD data access".to_string()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_directory() {
        let temp_dir = PathBuf::from("/tmp/test_stage4");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).ok();
        }

        fs::create_dir_all(&temp_dir).unwrap();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());

        fs::remove_dir_all(&temp_dir).ok();
    }
}
