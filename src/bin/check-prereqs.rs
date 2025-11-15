//! Prerequisites checker for Leptos SSG blog project
//!
//! Verifies all required tools and dependencies are installed:
//! - R/Rscript
//! - Python 3
//! - Rust/Cargo
//! - Git
//! - NBDCtools R package
//! - Common R packages used in tutorials
//! - ABCD_DATA_PATH environment variable
//!
//! Usage: cargo run --bin check-prereqs

use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

#[allow(dead_code)]
struct PrereqCheck {
    name: String,
    passed: bool,
    message: String,
}

fn main() -> ExitCode {
    println!("=== Checking Prerequisites ===\n");

    let mut checks = vec![
        // Check R/Rscript
        check_command("Rscript", &["--version"]),
        // Check Python 3
        check_command("python3", &["--version"]),
        // Check Rust/Cargo
        check_command("cargo", &["--version"]),
        // Check Git
        check_command("git", &["--version"]),
    ];

    // Check NBDCtools (requires R)
    if checks.iter().any(|c| c.name == "Rscript" && c.passed) {
        checks.push(check_nbdctools());

        // Check common R packages used in tutorials
        checks.extend(check_r_packages());
    } else {
        checks.push(PrereqCheck {
            name: "NBDCtools".to_string(),
            passed: false,
            message: "⚠  Skipping (R not available)".to_string(),
        });
    }

    // Check ABCD_DATA_PATH
    checks.push(check_abcd_data_path());

    // Print summary
    println!("\n=== Summary ===\n");

    let errors = checks.iter().filter(|c| !c.passed).count();

    if errors == 0 {
        println!("✓ All prerequisites met!\n");
        println!("You can proceed with building and testing.");
        ExitCode::SUCCESS
    } else {
        println!("✗ {errors} error(s) found\n");
        println!("Please resolve the errors above before proceeding.");
        ExitCode::FAILURE
    }
}

fn check_command(cmd: &str, args: &[&str]) -> PrereqCheck {
    print!("Checking {cmd}... ");

    match Command::new(cmd).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let version = if version.is_empty() {
                    String::from_utf8_lossy(&output.stderr)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string()
                } else {
                    version
                };

                println!("✓");
                println!("  {version}");
                PrereqCheck {
                    name: cmd.to_string(),
                    passed: true,
                    message: format!("✓ {cmd}: {version}"),
                }
            } else {
                println!("✗ FAILED");
                println!("  {cmd} found but returned error");
                PrereqCheck {
                    name: cmd.to_string(),
                    passed: false,
                    message: format!("✗ {cmd}: command failed"),
                }
            }
        }
        Err(_) => {
            println!("✗ FAILED");
            let help_msg = match cmd {
                "Rscript" => "  Install R from https://www.r-project.org/ or:\n    macOS: brew install r\n    Ubuntu: sudo apt-get install r-base r-base-dev",
                "python3" => "  Install Python 3 from https://www.python.org/",
                "cargo" => "  Install Rust from https://rustup.rs/",
                "git" => "  Install Git from https://git-scm.com/",
                _ => "  Command not found in PATH",
            };
            println!("{help_msg}");
            PrereqCheck {
                name: cmd.to_string(),
                passed: false,
                message: format!("✗ {cmd}: not found"),
            }
        }
    }
}

fn check_nbdctools() -> PrereqCheck {
    print!("Checking NBDCtools package... ");

    match Command::new("Rscript")
        .args(["-e", "library(NBDCtools)"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("✓");
                println!("  NBDCtools is installed and accessible");
                PrereqCheck {
                    name: "NBDCtools".to_string(),
                    passed: true,
                    message: "✓ NBDCtools: installed".to_string(),
                }
            } else {
                println!("✗ FAILED");
                println!("  NBDCtools not found - contact repo maintainer for installation");
                PrereqCheck {
                    name: "NBDCtools".to_string(),
                    passed: false,
                    message: "✗ NBDCtools: not installed".to_string(),
                }
            }
        }
        Err(e) => {
            println!("✗ FAILED");
            println!("  Error checking NBDCtools: {e}");
            PrereqCheck {
                name: "NBDCtools".to_string(),
                passed: false,
                message: format!("✗ NBDCtools: check failed - {e}"),
            }
        }
    }
}

fn check_abcd_data_path() -> PrereqCheck {
    print!("Checking ABCD_DATA_PATH... ");

    match env::var("ABCD_DATA_PATH") {
        Ok(path) => {
            println!("✓");
            println!("  Set to: {path}");

            // Check if path exists
            if Path::new(&path).exists() {
                println!("  Directory exists: ✓");

                // Check if directory has files
                if let Ok(entries) = std::fs::read_dir(&path) {
                    let file_count = entries.count();
                    if file_count > 0 {
                        println!("  Contains {file_count} files");
                    } else {
                        println!("  ⚠  Directory is empty");
                    }
                }

                PrereqCheck {
                    name: "ABCD_DATA_PATH".to_string(),
                    passed: true,
                    message: format!("✓ ABCD_DATA_PATH: {path}"),
                }
            } else {
                println!("  ✗ Directory does not exist");
                println!("  Update ABCD_DATA_PATH to point to your ABCD phenotype directory");
                PrereqCheck {
                    name: "ABCD_DATA_PATH".to_string(),
                    passed: false,
                    message: format!("✗ ABCD_DATA_PATH: directory not found - {path}"),
                }
            }
        }
        Err(_) => {
            println!("⚠  NOT SET");
            println!("  ABCD_DATA_PATH environment variable not set");
            println!("  Set it with: export ABCD_DATA_PATH=/path/to/abcd/phenotype");
            println!("  Code will fall back to default: /Users/shawes/abcd/6_0/phenotype");
            PrereqCheck {
                name: "ABCD_DATA_PATH".to_string(),
                passed: true, // Not a hard failure - has fallback
                message: "⚠ ABCD_DATA_PATH: not set (using fallback)".to_string(),
            }
        }
    }
}

fn check_r_packages() -> Vec<PrereqCheck> {
    println!("\nChecking R packages used in tutorials...");

    // Core packages that are critical for most tutorials
    let core_packages = vec!["tidyverse", "arrow", "gtsummary", "rstatix"];

    // Statistical analysis packages
    let analysis_packages = vec![
        "lme4",     // Linear mixed models
        "lmerTest", // Tests for lme4
        "lavaan",   // SEM and latent growth models
        "geepack",  // Generalized estimating equations
        "lcmm",     // Growth mixture models
        "glmmTMB",  // GLMMs
    ];

    // Visualization and output packages
    let utility_packages = vec!["ggeffects", "performance", "sjPlot", "kableExtra", "gt"];

    let mut checks = Vec::new();
    let mut all_packages = Vec::new();
    all_packages.extend(core_packages.clone());
    all_packages.extend(analysis_packages.clone());
    all_packages.extend(utility_packages.clone());

    println!("  Scanning for installed packages...\n");

    for package in &all_packages {
        let check = check_r_package(package);

        // Only show warnings for failed core packages
        if !check.passed && core_packages.contains(package) {
            println!("  ⚠  {package} is missing (used by multiple tutorials)");
        } else if !check.passed && analysis_packages.contains(package) {
            println!("  ℹ  {package} is optional (used by specific tutorial types)");
        }

        checks.push(check);
    }

    let installed = checks.iter().filter(|c| c.passed).count();
    let missing = checks.len() - installed;

    println!(
        "\n  Summary: {}/{} packages installed",
        installed,
        checks.len()
    );

    if missing > 0 {
        println!("  Missing {missing} package(s) - some tutorials may fail Stage 3");
        println!("  Install missing packages with: install.packages(c(...))\n");
    } else {
        println!("  All packages available! ✓\n");
    }

    checks
}

fn check_r_package(package: &str) -> PrereqCheck {
    match Command::new("Rscript")
        .args(["-e", &format!("library({package})")])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                PrereqCheck {
                    name: format!("R:{package}"),
                    passed: true,
                    message: format!("✓ R package: {package}"),
                }
            } else {
                PrereqCheck {
                    name: format!("R:{package}"),
                    passed: false,
                    message: format!("✗ R package: {package} not installed"),
                }
            }
        }
        Err(e) => PrereqCheck {
            name: format!("R:{package}"),
            passed: false,
            message: format!("✗ R package {package}: check failed - {e}"),
        },
    }
}
