/*!
 * Output module
 *
 * Handles formatting and display of validation results
 */

use crate::validators::ValidationResult;
use std::path::Path;

pub fn print_stage1_result(markdown_path: &Path, result: &ValidationResult, _verbose: bool) {
    println!("\n{}", "━".repeat(70));
    println!("{}", result.stage_name);
    println!("{}", "━".repeat(70));

    // Show counts if there are errors or warnings
    if !result.errors.is_empty() {
        println!("✗ Found {} error(s)", result.errors.len());
    }
    if !result.warnings.is_empty() {
        println!("⚠ Found {} warning(s)", result.warnings.len());
    }

    // If no errors or warnings, show success message
    if result.errors.is_empty() && result.warnings.is_empty() {
        println!("✓ All structural checks passed");
    }

    // Print all errors with suggestions
    for error in &result.errors {
        println!();
        if let Some(line) = error.line {
            println!("  Line {}: {}", line, error.message);
        } else {
            println!("  {}", error.message);
        }

        if let Some(suggestion) = &error.suggestion {
            println!("  → {suggestion}");
        }
    }

    // Print warnings
    for warning in &result.warnings {
        println!();
        if let Some(line) = warning.line {
            println!("  ⚠ Line {}: {}", line, warning.message);
        } else {
            println!("  ⚠ {}", warning.message);
        }

        if let Some(suggestion) = &warning.suggestion {
            println!("    → {suggestion}");
        }
    }

    // Status based on errors only (warnings don't fail)
    println!("\nFile: {}", markdown_path.display());
    if result.errors.is_empty() {
        println!("Status: ✅ PASSED");
    } else {
        println!("Status: ❌ FAILED");
    }

    println!("{}", "━".repeat(70));
}

pub fn print_stage2_result(markdown_path: &Path, result: &ValidationResult, _verbose: bool) {
    println!("\n{}", "━".repeat(70));
    println!("{}", result.stage_name);
    println!("{}", "━".repeat(70));

    // Show counts if there are errors or warnings
    if !result.errors.is_empty() {
        println!("✗ Found {} error(s)", result.errors.len());
    }
    if !result.warnings.is_empty() {
        println!("⚠ Found {} warning(s)", result.warnings.len());
    }

    // If no errors or warnings, show success message
    if result.errors.is_empty() && result.warnings.is_empty() {
        println!("✓ All R syntax checks passed");
    }

    // Print all errors with suggestions
    for error in &result.errors {
        println!();
        if let Some(line) = error.line {
            println!("  Line {}: {}", line, error.message);
        } else {
            println!("  {}", error.message);
        }

        if let Some(suggestion) = &error.suggestion {
            println!("  → {suggestion}");
        }
    }

    // Print warnings
    for warning in &result.warnings {
        println!();
        if let Some(line) = warning.line {
            println!("  ⚠ Line {}: {}", line, warning.message);
        } else {
            println!("  ⚠ {}", warning.message);
        }

        if let Some(suggestion) = &warning.suggestion {
            println!("    → {suggestion}");
        }
    }

    // Status based on errors only (warnings don't fail)
    println!("\nFile: {}", markdown_path.display());
    if result.errors.is_empty() {
        println!("Status: ✅ PASSED");
    } else {
        println!("Status: ❌ FAILED");
    }

    println!("{}", "━".repeat(70));
}

pub fn print_stage3_result(markdown_path: &Path, result: &ValidationResult, _verbose: bool) {
    println!("\n{}", "━".repeat(70));
    println!("{}", result.stage_name);
    println!("{}", "━".repeat(70));

    // Show execution metadata
    if let Some(exec_time) = result.metadata.get("execution_time_ms") {
        println!("⏱  Execution time: {exec_time} ms");
    }
    if let Some(blocks) = result.metadata.get("r_blocks_count") {
        println!("📦 R code blocks: {blocks}");
    }

    // Show counts if there are errors or warnings
    if !result.errors.is_empty() {
        println!("✗ Found {} error(s)", result.errors.len());
    }
    if !result.warnings.is_empty() {
        println!("⚠ Found {} warning(s)", result.warnings.len());
    }

    // If no errors or warnings, show success message
    if result.errors.is_empty() && result.warnings.is_empty() {
        println!("✓ R code executed successfully with mock data");
    }

    // Print all errors with suggestions
    for error in &result.errors {
        println!();
        if let Some(line) = error.line {
            println!("  Line {}: {}", line, error.message);
        } else {
            println!("  {}", error.message);
        }

        if let Some(suggestion) = &error.suggestion {
            println!("  → {suggestion}");
        }
    }

    // Print warnings
    for warning in &result.warnings {
        println!();
        if let Some(line) = warning.line {
            println!("  ⚠ Line {}: {}", line, warning.message);
        } else {
            println!("  ⚠ {}", warning.message);
        }

        if let Some(suggestion) = &warning.suggestion {
            println!("    → {suggestion}");
        }
    }

    // Status based on errors only (warnings don't fail)
    println!("\nFile: {}", markdown_path.display());
    if result.errors.is_empty() {
        println!("Status: ✅ PASSED");
    } else {
        println!("Status: ❌ FAILED");
    }

    println!("{}", "━".repeat(70));
}

/// Print Stage 4 results
pub fn print_stage4_result(markdown_path: &Path, result: &ValidationResult, verbose: bool) {
    println!("{}", "━".repeat(70));
    println!("{}", result.stage_name);
    println!("{}", "━".repeat(70));

    // Print metadata
    if let Some(execution_time) = result.metadata.get("execution_time_ms") {
        println!(
            "⏱  Execution time: {} ms ({:.1}s)",
            execution_time,
            execution_time.parse::<f64>().unwrap_or(0.0) / 1000.0
        );
    }

    // Check if section status is trackable
    let data_prep_status = result.metadata.get("data_prep_status");
    let analysis_status = result.metadata.get("analysis_status");

    let both_unknown = data_prep_status == Some(&"unknown".to_string())
        && analysis_status == Some(&"unknown".to_string());

    if both_unknown {
        // Don't show section status if we can't determine it
        println!("ℹ  Section progress tracking not available (no progress markers in R code)");
    } else {
        // Show section status with appropriate icons
        if let Some(data_prep) = data_prep_status {
            let icon = match data_prep.as_str() {
                "passing" => "✓",
                "failing" => "✗",
                "unknown" => "?",
                _ => "?",
            };
            println!("{icon} Data Preparation: {data_prep}");
        }

        if let Some(analysis) = analysis_status {
            let icon = match analysis.as_str() {
                "passing" => "✓",
                "failing" => "✗",
                "unknown" => "?",
                _ => "?",
            };
            println!("{icon} Statistical Analysis: {analysis}");
        }
    }

    if !result.errors.is_empty() {
        println!("\nErrors:");
        for error in &result.errors {
            if let Some(line) = error.line {
                println!("  Line {}: {}", line, error.message);
            } else {
                println!("  {}", error.message);
            }

            if let Some(suggestion) = &error.suggestion {
                println!("  → {suggestion}");
            }
        }
    }

    if !result.warnings.is_empty() {
        if verbose {
            println!("\nWarnings:");
            for warning in &result.warnings {
                if let Some(line) = warning.line {
                    println!("  ⚠ Line {}: {}", line, warning.message);
                } else {
                    println!("  ⚠ {}", warning.message);
                }

                if let Some(suggestion) = &warning.suggestion {
                    println!("    → {suggestion}");
                }
            }
        } else {
            println!(
                "\n⚠ {} warning(s); re-run with --verbose to see details",
                result.warnings.len()
            );
        }
    }

    println!("\nFile: {}", markdown_path.display());
    if result.is_passing() {
        println!("Status: ✅ PASSED");
    } else {
        println!("Status: ❌ FAILED");
    }

    println!("{}", "━".repeat(70));
}

/// Print Stage 5 results
pub fn print_stage5_result(markdown_path: &Path, result: &ValidationResult, _verbose: bool) {
    println!("{}", "━".repeat(70));
    println!("{}", result.stage_name);
    println!("{}", "━".repeat(70));

    // Only show success messages if stage actually passed
    if result.is_passing() {
        // JSON generation always happens if we got this far without errors
        println!("✓ JSON generated successfully");

        // Only show schema validation message if it actually ran (check metadata flag)
        if result
            .metadata
            .get("schema_validated")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            println!("✓ Schema validation passed");
        }

        // Only show deployment if it actually happened (has deployed_to metadata)
        if let Some(deployed_to) = result.metadata.get("deployed_to") {
            println!("✓ Deployed to: {deployed_to}");
        }

        // Only show status update message if it actually happened (check metadata flag)
        if result
            .metadata
            .get("status_updated")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            println!("✓ Status tracking updated");
        }
    }

    // Print errors
    for error in &result.errors {
        println!();
        if let Some(line) = error.line {
            println!("  ❌ Line {}: {}", line, error.message);
        } else {
            println!("  ❌ {}", error.message);
        }

        if let Some(suggestion) = &error.suggestion {
            println!("  → {suggestion}");
        }
    }

    // Print warnings
    for warning in &result.warnings {
        println!();
        if let Some(line) = warning.line {
            println!("  ⚠ Line {}: {}", line, warning.message);
        } else {
            println!("  ⚠ {}", warning.message);
        }

        if let Some(suggestion) = &warning.suggestion {
            println!("    → {suggestion}");
        }
    }

    // Status
    println!("\nFile: {}", markdown_path.display());
    if result.is_passing() {
        println!("Status: ✅ PASSED");
    } else {
        println!("Status: ❌ FAILED");
    }

    println!("{}", "━".repeat(70));
}
