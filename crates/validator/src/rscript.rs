/*!
 * Rscript execution helpers
 *
 * Shared utilities for finding and executing Rscript with fallback logic
 */

use anyhow::{Context, Result};
use std::process::Command;

/// Try to execute Rscript with fallback to PATH
///
/// First tries the configured executable path, then falls back to "Rscript" in PATH.
/// Returns clear error messages indicating what was tried and why it failed.
///
/// # Arguments
/// * `configured_path` - The Rscript path from config.r.executable
/// * `command_builder` - A closure that takes the Rscript path and builds a Command
///
/// # Returns
/// The configured Rscript path that worked, for use in subsequent calls
pub fn find_rscript_with_fallback<F>(
    configured_path: &str,
    mut command_builder: F,
) -> Result<String>
where
    F: FnMut(&str) -> Command,
{
    // Try configured path first
    let mut cmd = command_builder(configured_path);
    match cmd.output() {
        Ok(_) => {
            // Configured path works
            Ok(configured_path.to_string())
        }
        Err(config_err) if configured_path != "Rscript" => {
            // Config path failed, try PATH fallback
            let mut fallback_cmd = command_builder("Rscript");
            match fallback_cmd.output() {
                Ok(_) => {
                    // PATH fallback works
                    Ok("Rscript".to_string())
                }
                Err(_) => {
                    // Both failed
                    Err(anyhow::anyhow!(
                        "Failed to execute Rscript. Tried:\n  1. {configured_path} (config.r.executable): {config_err}\n  2. Rscript (PATH): not found\n\nPlease install R or update config.r.executable"
                    ))
                }
            }
        }
        Err(e) => {
            // PATH fallback also failed (or was the only option)
            Err(anyhow::anyhow!(
                "Failed to execute Rscript from PATH: {e}\n\nPlease install R or set config.r.executable to the full path"
            ))
        }
    }
}

/// Execute an Rscript command with fallback logic
///
/// This is a convenience wrapper around find_rscript_with_fallback that executes
/// the command and returns the output.
pub fn execute_rscript_with_fallback<F>(
    configured_path: &str,
    command_builder: F,
) -> Result<std::process::Output>
where
    F: FnMut(&str) -> Command + Clone,
{
    // First, find a working Rscript path
    let working_path = find_rscript_with_fallback(configured_path, command_builder.clone())?;

    // Now execute with the working path
    let mut cmd = command_builder.clone()(&working_path);
    cmd.output()
        .with_context(|| format!("Failed to execute Rscript at {working_path}"))
}
