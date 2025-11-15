//! Shared R script execution logic for validation pipeline
//!
//! This module provides reusable functions for executing R scripts with
//! timeout enforcement, error parsing, and section attribution.
//! Used by both test_tutorials and the validation pipeline (Stage 3, Stage 4).

use anyhow::{bail, Context, Result};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::time::Instant;
use wait_timeout::ChildExt;

/// Section execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionStatus {
    Passing,
    Failing,
    Unknown, // No progress markers detected
}

impl SectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SectionStatus::Passing => "passing",
            SectionStatus::Failing => "failing",
            SectionStatus::Unknown => "unknown",
        }
    }
}

/// Result of R script execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub data_prep_status: SectionStatus,
    pub analysis_status: SectionStatus,
}

/// Find Rscript executable with fallback logic (respects config.r.executable)
///
/// Tries the configured path first, then falls back to PATH and common locations.
/// This ensures consistent behavior across all validation stages.
///
/// # Arguments
/// * `configured_path` - The Rscript path from config.r.executable
///
/// # Returns
/// The working Rscript path
pub fn find_rscript_with_fallback(configured_path: &str) -> Result<String> {
    // Try configured path first
    if Command::new(configured_path)
        .arg("--version")
        .output()
        .is_ok()
    {
        return Ok(configured_path.to_string());
    }

    // If configured path failed and it's not "Rscript", try PATH fallback
    if configured_path != "Rscript" && Command::new("Rscript").arg("--version").output().is_ok() {
        return Ok("Rscript".to_string());
    }

    // Try common installation locations as last resort
    let common_paths = vec![
        "/Library/Frameworks/R.framework/Resources/bin/Rscript", // macOS R.framework
        "/opt/homebrew/bin/Rscript",                             // Homebrew (Apple Silicon)
        "/usr/local/bin/Rscript",                                // Homebrew (Intel) / Linux
        "/usr/bin/Rscript",                                      // Linux system
    ];

    for path in &common_paths {
        if Path::new(path).exists() && Command::new(path).arg("--version").output().is_ok() {
            return Ok(path.to_string());
        }
    }

    bail!(
        "Rscript not found. Tried:\n\
        1. Configured path: {configured_path}\n\
        2. PATH: Rscript\n\
        3. Common locations: {common_paths:?}\n\n\
        Please install R or update config.r.executable"
    )
}

/// Find Rscript executable in PATH or common locations (legacy, kept for compatibility)
///
/// Returns the path to Rscript if found, None otherwise.
#[allow(dead_code)]
pub fn find_rscript() -> Result<String> {
    find_rscript_with_fallback("Rscript")
}

/// Execute R script with timeout and environment variables
///
/// # Arguments
/// * `configured_rscript_path` - The configured Rscript path (from config.r.executable)
/// * `script_content` - The R code to execute
/// * `timeout_seconds` - Maximum execution time in seconds
/// * `env_vars` - Environment variables to set (name, value pairs)
/// * `working_dir` - Optional working directory for R execution
///
/// # Returns
/// ExecutionResult with success status, output, and timing information
pub fn execute_r_script(
    configured_rscript_path: &str,
    script_content: &str,
    timeout_seconds: u64,
    env_vars: &[(&str, &str)],
    working_dir: Option<&Path>,
) -> Result<ExecutionResult> {
    let start_time = Instant::now();

    // Find Rscript executable with fallback (respects config.r.executable)
    // This ensures Stage 4 uses the same lookup logic as Stage 2/3
    let rscript_path = find_rscript_with_fallback(configured_rscript_path)?;

    // Remove any stale scripts from earlier runs of this process so we never
    // confuse an old temp file with the current one when debugging failures.
    let pid = std::process::id();
    let _ = std::fs::read_dir("/tmp").map(|entries| {
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with(&format!("validate_r_{pid}_")))
                    .unwrap_or(false)
            })
            .for_each(|path| {
                let _ = std::fs::remove_file(path);
            });
    });

    // Create temp file for the script with timestamp for debugging
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_path = format!("/tmp/validate_r_{pid}_{timestamp}.R");
    let mut file = fs::File::create(&temp_path)
        .with_context(|| format!("Failed to create temp R script: {temp_path}"))?;

    file.write_all(script_content.as_bytes())
        .with_context(|| "Failed to write R script content")?;

    // Log script location for debugging
    eprintln!("DEBUG: R script saved to: {temp_path}");

    // Build command with strict flags to prevent contamination
    // --vanilla: Implies --no-save, --no-restore, --no-site-file, --no-init-file, --no-environ
    // --no-environ: Don't read .Renviron files
    // --no-site-file: Don't read Rprofile.site
    // --no-init-file: Don't read .Rprofile
    let mut cmd = Command::new(&rscript_path);
    cmd.args([
        "--vanilla",
        "--no-environ",
        "--no-site-file",
        "--no-init-file",
    ])
    .arg(&temp_path)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    // Unset contaminating environment variables
    cmd.env_remove("R_ENVIRON_USER")
        .env_remove("R_PROFILE_USER")
        .env_remove("R_ENVIRON")
        .env_remove("R_PROFILE");

    // Set a safe default for thread count
    cmd.env("R_DEFAULT_NUM_THREADS", "1");

    // Set user-provided environment variables
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    // Set working directory if provided
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Spawn process
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn Rscript process: {rscript_path}"))?;

    // Wait with timeout
    let timeout_duration = Duration::from_secs(timeout_seconds);
    let status_code = match child.wait_timeout(timeout_duration)? {
        Some(status) => status,
        None => {
            // Timeout occurred - kill the process
            let _ = child.kill();
            let _ = child.wait();

            eprintln!("DEBUG: Script timed out, keeping temp file: {temp_path}");
            eprintln!("DEBUG: Run manually with: Rscript --vanilla --no-environ --no-site-file --no-init-file {temp_path}");

            bail!(
                "R script execution timed out after {timeout_seconds} seconds. \
                Script saved at: {temp_path}\n\
                Run manually with: Rscript --vanilla {temp_path}\n\
                This may indicate an infinite loop, very large dataset, or complex computation."
            );
        }
    };

    // Read output
    let mut stdout = String::new();
    let mut stderr = String::new();

    if let Some(mut stdout_handle) = child.stdout.take() {
        stdout_handle.read_to_string(&mut stdout)?;
    }

    if let Some(mut stderr_handle) = child.stderr.take() {
        stderr_handle.read_to_string(&mut stderr)?;
    }

    // Keep temp file for debugging if there was an error
    eprintln!("DEBUG: Exit status code: {status_code:?}");
    if !status_code.success() {
        eprintln!("DEBUG: Script failed, keeping temp file: {temp_path}");
        eprintln!("DEBUG: Run manually with: Rscript --vanilla --no-environ --no-site-file --no-init-file {temp_path}");
    } else {
        // Clean up temp file on success
        let _ = fs::remove_file(&temp_path);
    }

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    // Parse section status
    let (data_prep_status, analysis_status) = parse_error_sections(&stdout, &stderr);

    Ok(ExecutionResult {
        success: status_code.success(),
        stdout,
        stderr,
        execution_time_ms,
        data_prep_status,
        analysis_status,
    })
}

/// Parse R output to determine which section failed
///
/// Returns (data_prep_status, analysis_status)
pub fn parse_error_sections(stdout: &str, stderr: &str) -> (SectionStatus, SectionStatus) {
    let combined_output = format!("{stdout}\n{stderr}");

    // Check if we got past data preparation
    let data_prep_completed = combined_output.contains("Data Preparation completed");
    let analysis_started = combined_output.contains("Starting Statistical Analysis");
    let has_error = stderr.contains("Error");

    // If no markers present, we can't determine section status
    let markers_present = data_prep_completed || analysis_started;

    if !markers_present {
        // No progress markers detected - section status is unknown
        return (SectionStatus::Unknown, SectionStatus::Unknown);
    }

    // Markers are present, determine section status
    let data_prep_status = if data_prep_completed {
        SectionStatus::Passing
    } else if has_error {
        SectionStatus::Failing
    } else {
        SectionStatus::Unknown
    };

    let analysis_status = if analysis_started {
        if has_error {
            SectionStatus::Failing
        } else {
            SectionStatus::Passing
        }
    } else if data_prep_completed {
        // Data prep completed but analysis didn't start - could mean analysis failed
        SectionStatus::Failing
    } else {
        SectionStatus::Unknown
    };

    (data_prep_status, analysis_status)
}

/// Extract meaningful error message from R output
///
/// Returns the first error message found, with surrounding context
pub fn extract_error_message(stderr: &str) -> String {
    let lines: Vec<&str> = stderr.lines().collect();

    // Find first line containing "Error"
    for (i, line) in lines.iter().enumerate() {
        if line.contains("Error") {
            // Return error line plus 2 lines of context
            let start = i;
            let end = (i + 3).min(lines.len());
            return lines[start..end].join("\n");
        }
    }

    // If no Error found, return last 5 lines of stderr
    if lines.len() > 5 {
        lines[lines.len() - 5..].join("\n")
    } else {
        stderr.to_string()
    }
}

/// Extract warnings from R output
///
/// Returns a vector of warning messages
pub fn extract_warnings(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter(|line| line.contains("Warning") && !line.contains("Conflicts"))
        .map(|line| line.trim().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_rscript() {
        // Should find R installation on development machine
        let result = find_rscript();
        assert!(result.is_ok(), "R should be installed for tests");
    }

    #[test]
    fn test_parse_error_sections() {
        let stdout = "Starting Data Preparation...\nData Preparation completed.\n";
        let stderr = "Error in function: object not found";

        let (data_prep, analysis) = parse_error_sections(stdout, stderr);
        assert_eq!(
            data_prep,
            SectionStatus::Passing,
            "Data prep should have passed"
        );
        assert_eq!(
            analysis,
            SectionStatus::Failing,
            "Analysis should have failed (not started)"
        );
    }

    #[test]
    fn test_parse_error_sections_no_markers() {
        let stdout = "Some R output\nMore output\n";
        let stderr = "";

        let (data_prep, analysis) = parse_error_sections(stdout, stderr);
        assert_eq!(
            data_prep,
            SectionStatus::Unknown,
            "Should be unknown without markers"
        );
        assert_eq!(
            analysis,
            SectionStatus::Unknown,
            "Should be unknown without markers"
        );
    }

    #[test]
    fn test_extract_error_message() {
        let stderr = "Some output\nError in lm: object not found\nAdditional context\n";
        let msg = extract_error_message(stderr);
        assert!(msg.contains("Error in lm"), "Should extract error line");
    }

    #[test]
    fn test_extract_warnings() {
        let stderr = "Warning: deprecated function\nInfo: something\nWarning: check params\n";
        let warnings = extract_warnings(stderr);
        assert_eq!(warnings.len(), 2, "Should find 2 warnings");
    }
}
