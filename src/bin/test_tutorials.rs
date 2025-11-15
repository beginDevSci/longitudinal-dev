//! Tutorial Code Testing Binary
//!
//! Extracts R code from all posts, executes it sequentially, and generates
//! generated/validation/test-results.json with pass/fail status and error messages.
//!
//! Usage: cargo run --bin test_tutorials

use longitudinal_dev::models::post::Post;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

/// Test result for a single section of a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionResult {
    pub status: String, // "passing" | "failing" | "skipped"
    pub error_message: Option<String>,
}

/// Complete test result for a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub status: String, // "passing" | "failing" | "unknown"
    pub tested_at: String,
    pub data_prep_status: String,
    pub analysis_status: String,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

/// Extract R code blocks from Data Preparation section
fn extract_data_prep_code(post: &Post) -> Vec<String> {
    let mut code_blocks = Vec::new();

    for block in &post.data_prep.content_blocks {
        if let longitudinal_dev::models::data_preparation::ContentBlock::Code(code_data) = block {
            if code_data.language.to_lowercase() == "r" {
                code_blocks.push(code_data.content.to_string());
            }
        }
    }

    code_blocks
}

/// Extract R code blocks from Statistical Analysis section
fn extract_analysis_code(post: &Post) -> Vec<String> {
    let mut code_blocks = Vec::new();

    for block in &post.statistical_analysis.content_blocks {
        if let longitudinal_dev::models::statistical_analysis::ContentBlock::Code(code_data) = block
        {
            if code_data.language.to_lowercase() == "r" {
                code_blocks.push(code_data.content.to_string());
            }
        }
    }

    code_blocks
}

/// Create complete R script with section markers for a post
fn create_test_script(post: &Post) -> String {
    let mut script = String::new();

    // Add header
    let title = &post.title;
    let now = chrono::Utc::now();
    script.push_str(&format!("# Test script for: {title}\n"));
    script.push_str(&format!("# Generated: {now}\n\n"));

    // Extract data prep code
    let data_prep_blocks = extract_data_prep_code(post);
    if !data_prep_blocks.is_empty() {
        script.push_str("# ===== DATA PREPARATION SECTION =====\n");
        script.push_str("cat('Starting Data Preparation...\\n')\n\n");
        for (i, block) in data_prep_blocks.iter().enumerate() {
            script.push_str(&format!("# Data Prep Block {}\n", i + 1));
            script.push_str(block);
            script.push_str("\n\n");
        }
        script.push_str("cat('Data Preparation completed.\\n')\n\n");
    }

    // Extract analysis code
    let analysis_blocks = extract_analysis_code(post);
    if !analysis_blocks.is_empty() {
        script.push_str("# ===== STATISTICAL ANALYSIS SECTION =====\n");
        script.push_str("cat('Starting Statistical Analysis...\\n')\n\n");
        for (i, block) in analysis_blocks.iter().enumerate() {
            script.push_str(&format!("# Analysis Block {}\n", i + 1));
            script.push_str(block);
            script.push_str("\n\n");
        }
        script.push_str("cat('Statistical Analysis completed.\\n')\n\n");
    }

    script.push_str("cat('All sections completed successfully.\\n')\n");
    script
}

/// Find Rscript executable in common locations
fn find_rscript() -> Option<String> {
    // Try PATH first
    if Command::new("Rscript").arg("--version").output().is_ok() {
        return Some("Rscript".to_string());
    }

    // Check common installation locations
    let common_paths = vec![
        "/Library/Frameworks/R.framework/Resources/bin/Rscript", // macOS R.framework
        "/opt/homebrew/bin/Rscript",                             // Homebrew (Apple Silicon)
        "/usr/local/bin/Rscript",                                // Homebrew (Intel) / Linux
        "/usr/bin/Rscript",                                      // Linux system
    ];

    for path in common_paths {
        if std::path::Path::new(path).exists()
            && Command::new(path).arg("--version").output().is_ok()
        {
            return Some(path.to_string());
        }
    }

    None
}

/// Execute R script and capture results
fn execute_r_script(slug: &str, script_content: &str) -> TestResult {
    let start_time = Instant::now();

    // Find Rscript executable
    let rscript_path = match find_rscript() {
        Some(path) => path,
        None => {
            return TestResult {
                status: "failing".to_string(),
                tested_at: chrono::Utc::now().to_rfc3339(),
                data_prep_status: "unknown".to_string(),
                analysis_status: "unknown".to_string(),
                error_message: Some("Rscript not found in PATH or common installation locations. Please ensure R is installed and accessible.".to_string()),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            };
        }
    };

    // Create temp file for the script
    let temp_path = format!("/tmp/test_{slug}.R");
    match fs::File::create(&temp_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(script_content.as_bytes()) {
                return TestResult {
                    status: "failing".to_string(),
                    tested_at: chrono::Utc::now().to_rfc3339(),
                    data_prep_status: "unknown".to_string(),
                    analysis_status: "unknown".to_string(),
                    error_message: Some(format!("Failed to write test script: {e}")),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        }
        Err(e) => {
            return TestResult {
                status: "failing".to_string(),
                tested_at: chrono::Utc::now().to_rfc3339(),
                data_prep_status: "unknown".to_string(),
                analysis_status: "unknown".to_string(),
                error_message: Some(format!("Failed to create test script file: {e}")),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            };
        }
    }

    // Execute with Rscript
    println!("  📝 Executing R script: {temp_path}");
    let output = Command::new(&rscript_path)
        .arg(&temp_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let execution_time_ms = start_time.elapsed().as_millis() as u64;

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);

            if result.status.success() {
                println!("  ✅ Passed ({execution_time_ms} ms)");
                TestResult {
                    status: "passing".to_string(),
                    tested_at: chrono::Utc::now().to_rfc3339(),
                    data_prep_status: "passing".to_string(),
                    analysis_status: "passing".to_string(),
                    error_message: None,
                    execution_time_ms,
                }
            } else {
                // Determine which section failed based on output
                let (data_prep_status, analysis_status, error_msg) =
                    parse_error_output(&stdout, &stderr);

                println!("  ❌ Failed ({execution_time_ms} ms)");
                println!("     Data Prep: {data_prep_status}, Analysis: {analysis_status}");

                TestResult {
                    status: "failing".to_string(),
                    tested_at: chrono::Utc::now().to_rfc3339(),
                    data_prep_status,
                    analysis_status,
                    error_message: Some(error_msg),
                    execution_time_ms,
                }
            }
        }
        Err(e) => {
            println!("  ❌ Execution error");
            TestResult {
                status: "failing".to_string(),
                tested_at: chrono::Utc::now().to_rfc3339(),
                data_prep_status: "unknown".to_string(),
                analysis_status: "unknown".to_string(),
                error_message: Some(format!("Failed to execute Rscript: {e}")),
                execution_time_ms,
            }
        }
    }
}

/// Parse R error output to determine which section failed
fn parse_error_output(stdout: &str, stderr: &str) -> (String, String, String) {
    let combined_output = format!("{stdout}\n{stderr}");

    // Check if we got past data preparation
    let data_prep_completed = combined_output.contains("Data Preparation completed");
    let analysis_started = combined_output.contains("Starting Statistical Analysis");

    let (data_prep_status, analysis_status) = if data_prep_completed {
        if analysis_started {
            ("passing".to_string(), "failing".to_string())
        } else {
            ("passing".to_string(), "unknown".to_string())
        }
    } else {
        ("failing".to_string(), "skipped".to_string())
    };

    // Extract error message from stderr
    let error_msg = if !stderr.is_empty() {
        stderr
            .lines()
            .filter(|line| line.contains("Error") || line.contains("error"))
            .take(5)
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "Script execution failed without specific error message".to_string()
    };

    (data_prep_status, analysis_status, error_msg)
}

/// Test all posts and generate results JSON
fn main() {
    println!("🧪 Tutorial Code Testing Runner");
    println!("================================\n");

    // Check for Rscript
    match find_rscript() {
        Some(path) => println!("✓ Found Rscript: {path}\n"),
        None => {
            eprintln!("❌ Rscript not found in PATH or common locations.");
            eprintln!("   Please install R and ensure Rscript is accessible.");
            eprintln!("   Common locations checked:");
            eprintln!("     - /Library/Frameworks/R.framework/Resources/bin/Rscript (macOS)");
            eprintln!("     - /opt/homebrew/bin/Rscript (Homebrew)");
            eprintln!("     - /usr/local/bin/Rscript");
            eprintln!("     - /usr/bin/Rscript");
            std::process::exit(1);
        }
    }

    // Load all posts
    println!("📚 Loading posts...");
    let posts = longitudinal_dev::posts::posts();
    println!("   Found {} posts\n", posts.len());

    // Test each post
    let mut results: HashMap<String, TestResult> = HashMap::new();
    let mut passing_count = 0;
    let mut failing_count = 0;

    for (i, post) in posts.iter().enumerate() {
        println!(
            "[{}/{}] Testing: {} ({})",
            i + 1,
            posts.len(),
            post.title,
            post.slug
        );

        // Create test script
        let script = create_test_script(post);

        // Skip if no R code found
        if script.contains("# Data Prep Block") || script.contains("# Analysis Block") {
            // Execute and capture results
            let result = execute_r_script(post.slug.as_ref(), &script);

            if result.status == "passing" {
                passing_count += 1;
            } else if result.status == "failing" {
                failing_count += 1;
            }

            results.insert(post.slug.to_string(), result);
        } else {
            println!("  ⚠️  No R code blocks found, skipping");
            results.insert(
                post.slug.to_string(),
                TestResult {
                    status: "unknown".to_string(),
                    tested_at: chrono::Utc::now().to_rfc3339(),
                    data_prep_status: "unknown".to_string(),
                    analysis_status: "unknown".to_string(),
                    error_message: Some("No R code blocks found in post".to_string()),
                    execution_time_ms: 0,
                },
            );
        }
        println!();
    }

    // Write results to JSON
    let results_path = Path::new("generated/validation/test-results.json");
    println!("💾 Writing results to: {}", results_path.display());

    if let Some(parent) = results_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!(
                "❌ Failed to create results directory {}: {e}",
                parent.display()
            );
            std::process::exit(1);
        }
    }

    match serde_json::to_string_pretty(&results) {
        Ok(json) => {
            if let Err(e) = fs::write(results_path, json) {
                eprintln!("❌ Failed to write results file: {e}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to serialize results: {e}");
            std::process::exit(1);
        }
    }

    // Print summary
    println!("\n📊 Test Summary");
    println!("================================");
    println!("✅ Passing: {passing_count}");
    println!("❌ Failing: {failing_count}");
    println!(
        "⚠️  Unknown: {}",
        posts.len() - passing_count - failing_count
    );
    println!("\n✅ Results saved to: {}", results_path.display());
}
