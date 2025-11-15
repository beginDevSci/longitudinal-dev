//! Stage 3: Dry Run Execution with Mock Data
//!
//! Executes R code with mock/sample data to verify:
//! - Code runs without runtime errors
//! - Required R packages are installed
//! - Data transformations work correctly
//! - Expected objects are created

use crate::config::{RConfig, Stage3Config};
use crate::validators::ValidationResult;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Represents a single R code block extracted from markdown
#[derive(Debug)]
pub struct RCodeBlock {
    pub content: String,
    pub line_number: usize,
}

/// Helper struct to track code fence content and location
#[derive(Debug)]
struct CodeFenceContent {
    code: String,
    start_line: usize,
}

pub struct Stage3Validator<'a> {
    config: &'a Stage3Config,
    r_config: &'a RConfig,
}

impl<'a> Stage3Validator<'a> {
    pub fn new(config: &'a Stage3Config, r_config: &'a RConfig) -> Self {
        Self { config, r_config }
    }

    pub fn validate(&self, markdown_path: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new("Stage 3: Dry Run Execution");
        let start_time = Instant::now();

        // Read markdown file
        let content = fs::read_to_string(markdown_path)?;

        // Parse structure using parser crate
        let structure = longitudinal_parser::validation::parse_structure(&content)?;

        // Extract R code blocks (reuse Stage 2 logic)
        let r_blocks = self.extract_r_code_blocks(&content, &structure)?;

        if r_blocks.is_empty() {
            result.error(
                None,
                "No R code blocks found".to_string(),
                Some("Add {.code} sections with ```r code blocks".to_string()),
            );
            return Ok(result);
        }

        // 1. Check required packages first (fast fail)
        let missing_packages = self.check_required_packages()?;
        if !missing_packages.is_empty() {
            result.error(
                None,
                format!(
                    "Missing required R packages: {}",
                    missing_packages.join(", ")
                ),
                Some(format!(
                    "Install with: install.packages(c({}))",
                    missing_packages
                        .iter()
                        .map(|p| format!("\"{p}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
            return Ok(result);
        }

        // 2. Create combined R script with mock data preamble
        let script = self.create_mock_data_script(&r_blocks)?;

        // 3. Execute with timeout
        match self.execute_r_script(&script) {
            Ok(exec_result) => {
                let execution_time_ms = start_time.elapsed().as_millis() as u64;

                // Add execution metadata
                result.add_metadata("execution_time_ms", execution_time_ms.to_string());
                result.add_metadata("r_blocks_count", r_blocks.len().to_string());

                if exec_result.success {
                    // Execution succeeded
                    // Check for warnings in output
                    if exec_result.has_warnings() {
                        result.warning(
                            None,
                            format!(
                                "R code executed with warnings:\n{}",
                                exec_result.get_warnings()
                            ),
                            Some(
                                "Review warnings to ensure they don't indicate issues".to_string(),
                            ),
                        );
                    }
                } else {
                    // Execution failed
                    let (error_msg, suggestion) = self.parse_r_error(&exec_result);
                    result.error(
                        None,
                        format!("R execution failed:\n{error_msg}"),
                        suggestion,
                    );
                }
            }
            Err(e) => {
                result.error(
                    None,
                    format!("Failed to execute R script: {e}"),
                    Some("Ensure R is installed and Rscript is accessible".to_string()),
                );
            }
        }

        Ok(result)
    }

    /// Extract all R code blocks from markdown (reused from Stage 2)
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
                });
            }
        }

        Ok(blocks)
    }

    /// Extract code fence content starting from a given line (reused from Stage 2)
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

    /// Check if required R packages are installed
    fn check_required_packages(&self) -> Result<Vec<String>> {
        if self.r_config.required_packages.is_empty() {
            return Ok(Vec::new());
        }

        let check_script = format!(
            r#"
            required <- c({})
            missing <- required[!sapply(required, requireNamespace, quietly = TRUE)]
            cat(paste(missing, collapse = "\n"))
            "#,
            self.r_config
                .required_packages
                .iter()
                .map(|p| format!(r#""{p}""#))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // Use Rscript helper with fallback
        let output = crate::rscript::execute_rscript_with_fallback(
            &self.r_config.executable,
            |rscript_path| {
                let mut cmd = Command::new(rscript_path);
                cmd.args(["--vanilla", "-e", &check_script]);
                cmd
            },
        )?;

        let missing_pkgs: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with('[')) // Filter R output artifacts
            .map(|s| s.trim().to_string())
            .collect();

        Ok(missing_pkgs)
    }

    /// Create R script with mock data preamble and user code
    fn create_mock_data_script(&self, r_blocks: &[RCodeBlock]) -> Result<String> {
        let mut script = String::new();

        // Add header
        script.push_str("# Stage 3: Dry Run Execution\n");
        script.push_str(&format!("# Generated: {}\n\n", chrono::Utc::now()));

        // Ensure parallel::detectCores returns a sane value before any packages (like lavaan) initialize
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

        // Add CPU sanitization prelude (shared with Stage 4)
        script.push_str(&crate::r_prelude::cpu_sanitization_prelude());

        // Add session info logging
        script.push_str(&crate::r_prelude::session_info_logging());

        // Load mock data generator
        let mock_data_path = std::env::current_dir()?.join("crates/validator/src/mock_data.R");

        if !mock_data_path.exists() {
            return Err(anyhow::anyhow!(
                "Mock data generator not found at: {}",
                mock_data_path.display()
            ));
        }

        script.push_str(&format!(
            "# Load mock data generator\nsource('{}')\n\n",
            mock_data_path.display()
        ));

        // Add user code blocks with section markers and lavaan diagnostics
        script.push_str("# ===== USER CODE =====\n\n");

        for (i, block) in r_blocks.iter().enumerate() {
            script.push_str(&format!(
                "# Code Block {} (Line {})\n",
                i + 1,
                block.line_number
            ));
            script.push_str(&format!(
                "cat(sprintf('Executing block {}...\\n'))\n",
                i + 1
            ));

            // Check if this block contains lavaan::sem() or lavaan::growth() calls
            let has_lavaan_fit = block.content.contains("sem(")
                || block.content.contains("growth(")
                || block.content.contains("cfa(")
                || block.content.contains("lavaan(");

            if has_lavaan_fit {
                // Wrap lavaan fitting calls with diagnostics
                script.push_str("# DEBUG: Checking data before lavaan fit\n");
                script.push_str("if (exists('df_wide')) {\n");
                script.push_str("  numeric_cols <- sapply(df_wide, is.numeric)\n");
                script.push_str("  if (sum(numeric_cols) > 0) {\n");
                script.push_str("    cov_matrix <- tryCatch(cov(df_wide[, numeric_cols], use='complete.obs'), error=function(e) NULL)\n");
                script.push_str("    if (!is.null(cov_matrix)) {\n");
                script.push_str("      eig <- eigen(cov_matrix, only.values=TRUE)$values\n");
                script.push_str("      cat(sprintf('DEBUG: Covariance eigenvalues: min=%.2e, max=%.2e\\n', min(eig), max(eig)))\n");
                script.push_str("      if (min(eig) < 1e-8) cat('WARNING: Near-singular covariance matrix detected\\n')\n");
                script.push_str("    }\n");
                script.push_str("  }\n");
                script.push_str("}\n\n");
            }

            script.push_str(&block.content);

            // Add lavaan options inspection after fitting
            if has_lavaan_fit {
                script.push_str("\n# DEBUG: Capture lavaan options if fit object exists\n");
                script.push_str("if (exists('fit') && inherits(fit, 'lavaan')) {\n");
                script.push_str("  opts <- try(lavInspect(fit, 'options'), silent=TRUE)\n");
                script.push_str("  if (!inherits(opts, 'try-error')) {\n");
                script.push_str("    cat('DEBUG: lavaan options:\\n')\n");
                script.push_str("    problem_opts <- c('se', 'test', 'estimator', 'missing', 'check.gradient', 'optim.method')\n");
                script.push_str("    for (opt in problem_opts) {\n");
                script.push_str("      if (opt %in% names(opts)) {\n");
                script.push_str("        cat(sprintf('  %s = %s\\n', opt, opts[[opt]]))\n");
                script.push_str("      }\n");
                script.push_str("    }\n");
                script.push_str("  }\n");
                script.push_str("}\n");
            }

            script.push_str("\n\n");
        }

        script.push_str("cat('All code blocks executed successfully.\\n')\n");

        Ok(script)
    }

    /// Execute R script and capture results with real timeout enforcement
    ///
    /// Now uses the shared executor module for consistency across all stages
    fn execute_r_script(&self, script_content: &str) -> Result<ExecutionResult> {
        // Create mock data directory if it doesn't exist
        let mock_data_path = "/tmp/mock_abcd_data";
        fs::create_dir_all(mock_data_path).context("Failed to create mock ABCD data directory")?;

        // Create temp directory for R artifacts (plots, etc.)
        let run_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let temp_dir = std::path::PathBuf::from(format!("/tmp/stage3_run_{run_id}"));
        fs::create_dir_all(&temp_dir)
            .with_context(|| format!("Failed to create temp directory: {}", temp_dir.display()))?;

        // Set environment variables for mock data
        let sample_rows_str = self.config.sample_rows.to_string();
        let env_vars = vec![
            ("ABCD_DATA_PATH", mock_data_path),
            ("MOCK_N_PARTICIPANTS", sample_rows_str.as_str()),
        ];

        // Use shared executor (respects config.r.executable)
        let exec_result = crate::executor::execute_r_script(
            &self.r_config.executable,
            script_content,
            self.config.timeout_seconds,
            &env_vars,
            Some(&temp_dir), // Use temp directory for artifacts
        )?;

        // Clean up temp directory (we don't need Stage 3 artifacts)
        let _ = fs::remove_dir_all(&temp_dir);

        // Convert shared executor result to Stage 3's ExecutionResult
        Ok(ExecutionResult {
            success: exec_result.success,
            stdout: exec_result.stdout,
            stderr: exec_result.stderr,
        })
    }

    /// Parse R error output to extract meaningful error messages
    fn parse_r_error(&self, exec_result: &ExecutionResult) -> (String, Option<String>) {
        let combined_output = format!("{}\n{}", exec_result.stdout, exec_result.stderr);

        // Extract error lines (avoid false positives from statistical output)
        let error_lines: Vec<&str> = combined_output
            .lines()
            .filter(|line| {
                let line_lower = line.to_lowercase();
                // Match actual errors, but exclude statistical output
                (line_lower.starts_with("error")
                    || line_lower.contains("error in ")
                    || line_lower.contains("error:")
                    || line_lower.contains("execution halted")
                    || line_lower.contains("could not find function"))
                // Exclude common false positives from statistical packages
                && !line_lower.contains("standard error")
                && !line_lower.contains("root mean squared error")
                && !line_lower.contains("squared error")
                && !line_lower.contains("type i error")
                && !line_lower.contains("type ii error")
            })
            .take(10) // Limit to first 10 error lines
            .collect();

        let error_msg = if error_lines.is_empty() {
            exec_result.stderr.clone()
        } else {
            error_lines.join("\n")
        };

        // Generate helpful suggestion based on error type
        let suggestion = if error_msg.contains("could not find function") {
            Some("Check that all required packages are loaded with library() calls".to_string())
        } else if error_msg.contains("object") && error_msg.contains("not found") {
            Some("Ensure all variables are defined before use".to_string())
        } else if error_msg.contains("package") && error_msg.contains("not available") {
            Some("Install missing R packages".to_string())
        } else {
            Some("Review R error message above and fix code issues".to_string())
        };

        (error_msg, suggestion)
    }
}

/// Result of R script execution
#[derive(Debug)]
struct ExecutionResult {
    success: bool,
    stdout: String,
    stderr: String,
}

impl ExecutionResult {
    fn has_warnings(&self) -> bool {
        self.stderr.contains("Warning") || self.stdout.contains("Warning")
    }

    fn get_warnings(&self) -> String {
        let combined = format!("{}\n{}", self.stdout, self.stderr);
        combined
            .lines()
            .filter(|line| line.contains("Warning"))
            .take(5)
            .collect::<Vec<_>>()
            .join("\n")
    }
}
