//! Prerequisites checker and installer for Leptos SSG blog project
//!
//! Verifies all required tools and dependencies are installed, and optionally installs them.
//!
//! Usage:
//!   cargo run --bin check-prereqs                 # Check only
//!   cargo run --bin check-prereqs --install       # Check and install missing
//!   cargo run --bin check-prereqs --dry-run       # Preview what would be installed
//!   cargo run --bin check-prereqs --install --dry-run  # Preview install actions

use std::env;
use std::path::Path;
use std::process::{Command, ExitCode};

#[cfg(feature = "ssr")]
use clap::Parser;

#[cfg(feature = "ssr")]
#[derive(Parser)]
#[command(name = "check-prereqs")]
#[command(about = "Check and install prerequisites for longitudinal-dev")]
struct Args {
    /// Install missing prerequisites
    #[arg(long)]
    install: bool,

    /// Show what would be installed without actually installing
    #[arg(long)]
    dry_run: bool,
}

struct PrereqCheck {
    name: String,
    passed: bool,
    message: String,
    install_cmd: Option<String>,
}

fn main() -> ExitCode {
    #[cfg(not(feature = "ssr"))]
    {
        eprintln!("This binary requires the 'ssr' feature");
        return ExitCode::FAILURE;
    }

    #[cfg(feature = "ssr")]
    {
        let args = Args::parse();
        run_checks(args)
    }
}

#[cfg(feature = "ssr")]
fn run_checks(args: Args) -> ExitCode {
    println!("=== Checking Prerequisites ===\n");

    if args.dry_run {
        println!("🔍 DRY RUN MODE - No installations will be performed\n");
    }

    let mut checks = vec![
        // Core tools
        check_rust(),
        check_cargo(),
        check_wasm_target(),
        check_wasm_bindgen(),
        check_node(),
        check_npm(),
        check_git(),
        // Optional tools
        check_command("Rscript", &["--version"], "R for tutorial validation"),
        check_command("python3", &["--version"], "Python 3"),
    ];

    // Check R packages if R is available
    if checks.iter().any(|c| c.name == "Rscript" && c.passed) {
        checks.extend(check_r_packages());
    }

    // Check ABCD_DATA_PATH
    checks.push(check_abcd_data_path());

    // Separate checks into passed and failed
    let failed_checks: Vec<_> = checks.iter().filter(|c| !c.passed).collect();
    let installable: Vec<_> = failed_checks
        .iter()
        .filter(|c| c.install_cmd.is_some())
        .collect();

    // Print summary
    println!("\n=== Summary ===\n");
    let error_count = failed_checks.len();

    if error_count == 0 {
        println!("✓ All prerequisites met!\n");

        // Run npm install if --install flag is set
        if args.install && !args.dry_run {
            println!("📦 Installing npm dependencies...");
            install_npm_deps();
        } else if args.install && args.dry_run {
            println!("Would run: npm install");
        }

        return ExitCode::SUCCESS;
    }

    println!("✗ {error_count} prerequisite(s) missing\n");

    if args.install && !installable.is_empty() {
        println!("🔧 Installing missing prerequisites...\n");

        for check in installable {
            if let Some(cmd) = &check.install_cmd {
                println!("Installing {}...", check.name);

                if args.dry_run {
                    println!("  Would run: {}", cmd);
                } else if install_prerequisite(cmd, &check.name) {
                    println!("  ✓ {} installed successfully", check.name);
                } else {
                    println!("  ✗ Failed to install {}", check.name);
                }
            }
        }

        // Install npm deps after tools are ready
        if !args.dry_run {
            println!("\n📦 Installing npm dependencies...");
            install_npm_deps();
        } else {
            println!("\nWould run: npm install");
        }

        println!("\n✨ Installation complete!");
        println!("Run 'cargo run --bin check-prereqs' to verify.");
        ExitCode::SUCCESS
    } else if args.install {
        println!("No installable prerequisites found.");
        println!("Please install the following manually:");
        for check in &failed_checks {
            println!("  - {}: {}", check.name, check.message);
        }
        ExitCode::FAILURE
    } else {
        println!("Run with --install to automatically install missing prerequisites:");
        println!("  cargo run --bin check-prereqs --install");
        println!("\nOr install manually:");
        for check in &failed_checks {
            println!("  - {}: {}", check.name, check.message);
        }
        ExitCode::FAILURE
    }
}

fn check_rust() -> PrereqCheck {
    check_command("rustc", &["--version"], "Rust compiler")
}

fn check_cargo() -> PrereqCheck {
    check_command("cargo", &["--version"], "Cargo package manager")
}

fn check_wasm_target() -> PrereqCheck {
    print!("Checking wasm32-unknown-unknown target... ");

    match Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        Ok(output) => {
            let installed = String::from_utf8_lossy(&output.stdout);
            if installed.contains("wasm32-unknown-unknown") {
                println!("✓");
                PrereqCheck {
                    name: "wasm32-unknown-unknown".to_string(),
                    passed: true,
                    message: "✓ WASM target installed".to_string(),
                    install_cmd: None,
                }
            } else {
                println!("✗ MISSING");
                PrereqCheck {
                    name: "wasm32-unknown-unknown".to_string(),
                    passed: false,
                    message: "✗ WASM target not installed".to_string(),
                    install_cmd: Some("rustup target add wasm32-unknown-unknown".to_string()),
                }
            }
        }
        Err(_) => {
            println!("✗ FAILED");
            PrereqCheck {
                name: "wasm32-unknown-unknown".to_string(),
                passed: false,
                message: "✗ Failed to check WASM target (rustup not found?)".to_string(),
                install_cmd: None,
            }
        }
    }
}

fn check_wasm_bindgen() -> PrereqCheck {
    print!("Checking wasm-bindgen-cli... ");

    const REQUIRED_VERSION: &str = "0.2.104";

    match Command::new("wasm-bindgen").arg("--version").output() {
        Ok(output) => {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if let Some(version) = version_output.split_whitespace().nth(1) {
                if version == REQUIRED_VERSION {
                    println!("✓");
                    println!("  Version {} (matches Cargo.toml)", version);
                    PrereqCheck {
                        name: "wasm-bindgen-cli".to_string(),
                        passed: true,
                        message: format!("✓ wasm-bindgen-cli {}", version),
                        install_cmd: None,
                    }
                } else {
                    println!("⚠ VERSION MISMATCH");
                    println!("  Found: {}, Required: {}", version, REQUIRED_VERSION);
                    PrereqCheck {
                        name: "wasm-bindgen-cli".to_string(),
                        passed: false,
                        message: format!("⚠ Version mismatch: {} (need {})", version, REQUIRED_VERSION),
                        install_cmd: Some(format!("cargo install wasm-bindgen-cli --version {} --force", REQUIRED_VERSION)),
                    }
                }
            } else {
                println!("✗ UNKNOWN VERSION");
                PrereqCheck {
                    name: "wasm-bindgen-cli".to_string(),
                    passed: false,
                    message: "✗ Could not determine version".to_string(),
                    install_cmd: Some(format!("cargo install wasm-bindgen-cli --version {}", REQUIRED_VERSION)),
                }
            }
        }
        Err(_) => {
            println!("✗ NOT FOUND");
            PrereqCheck {
                name: "wasm-bindgen-cli".to_string(),
                passed: false,
                message: "✗ wasm-bindgen-cli not installed".to_string(),
                install_cmd: Some(format!("cargo install wasm-bindgen-cli --version {}", REQUIRED_VERSION)),
            }
        }
    }
}

fn check_node() -> PrereqCheck {
    check_command("node", &["--version"], "Node.js")
}

fn check_npm() -> PrereqCheck {
    check_command("npm", &["--version"], "npm package manager")
}

fn check_git() -> PrereqCheck {
    check_command("git", &["--version"], "Git version control")
}

fn check_command(cmd: &str, args: &[&str], description: &str) -> PrereqCheck {
    print!("Checking {}... ", description);

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
                println!("  {}", version);
                PrereqCheck {
                    name: cmd.to_string(),
                    passed: true,
                    message: format!("✓ {}: {}", cmd, version),
                    install_cmd: None,
                }
            } else {
                println!("✗ FAILED");
                PrereqCheck {
                    name: cmd.to_string(),
                    passed: false,
                    message: format!("✗ {}: command failed", cmd),
                    install_cmd: None,
                }
            }
        }
        Err(_) => {
            println!("✗ NOT FOUND");
            let help_msg = match cmd {
                "Rscript" => "Install R from https://www.r-project.org/",
                "python3" => "Install Python from https://www.python.org/",
                "node" => "Install Node.js from https://nodejs.org/",
                "npm" => "Install Node.js from https://nodejs.org/",
                "git" => "Install Git from https://git-scm.com/",
                _ => "Command not found in PATH",
            };
            println!("  {}", help_msg);
            PrereqCheck {
                name: cmd.to_string(),
                passed: false,
                message: format!("✗ {}: {}", cmd, help_msg),
                install_cmd: None,
            }
        }
    }
}

fn check_abcd_data_path() -> PrereqCheck {
    print!("Checking ABCD_DATA_PATH... ");

    match env::var("ABCD_DATA_PATH") {
        Ok(path) => {
            println!("✓");
            println!("  Set to: {}", path);

            if Path::new(&path).exists() {
                println!("  Directory exists: ✓");
                PrereqCheck {
                    name: "ABCD_DATA_PATH".to_string(),
                    passed: true,
                    message: format!("✓ ABCD_DATA_PATH: {}", path),
                    install_cmd: None,
                }
            } else {
                println!("  ✗ Directory does not exist");
                PrereqCheck {
                    name: "ABCD_DATA_PATH".to_string(),
                    passed: false,
                    message: format!("✗ ABCD_DATA_PATH: directory not found - {}", path),
                    install_cmd: None,
                }
            }
        }
        Err(_) => {
            println!("⚠ NOT SET");
            println!("  Will use fallback path");
            PrereqCheck {
                name: "ABCD_DATA_PATH".to_string(),
                passed: true, // Not a hard failure
                message: "⚠ ABCD_DATA_PATH: not set (using fallback)".to_string(),
                install_cmd: None,
            }
        }
    }
}

fn check_r_packages() -> Vec<PrereqCheck> {
    println!("\nChecking R packages...");

    let core_packages = ["tidyverse", "arrow", "gtsummary", "rstatix"];
    let analysis_packages = ["lme4", "lmerTest", "lavaan", "geepack", "lcmm", "glmmTMB"];
    let utility_packages = ["ggeffects", "performance", "sjPlot", "kableExtra", "gt"];

    let mut all_packages: Vec<&str> = Vec::new();
    all_packages.extend(core_packages.iter());
    all_packages.extend(analysis_packages.iter());
    all_packages.extend(utility_packages.iter());

    let mut checks = Vec::new();

    for package in &all_packages {
        let check = check_r_package(package);
        if !check.passed && core_packages.contains(package) {
            println!("  ⚠ {} is missing (used by multiple tutorials)", package);
        }
        checks.push(check);
    }

    let installed = checks.iter().filter(|c| c.passed).count();
    println!("\n  R packages: {}/{} installed", installed, checks.len());

    checks
}

fn check_r_package(package: &str) -> PrereqCheck {
    match Command::new("Rscript")
        .args(["-e", &format!("library({})", package)])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                PrereqCheck {
                    name: format!("R:{}", package),
                    passed: true,
                    message: format!("✓ R package: {}", package),
                    install_cmd: None,
                }
            } else {
                PrereqCheck {
                    name: format!("R:{}", package),
                    passed: false,
                    message: format!("✗ R package {} not installed", package),
                    install_cmd: None, // R packages need manual install
                }
            }
        }
        Err(e) => PrereqCheck {
            name: format!("R:{}", package),
            passed: false,
            message: format!("✗ R package {}: check failed - {}", package, e),
            install_cmd: None,
        },
    }
}

fn install_prerequisite(cmd: &str, name: &str) -> bool {
    println!("  Running: {}", cmd);

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }

    let status = Command::new(parts[0])
        .args(&parts[1..])
        .status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                true
            } else {
                eprintln!("  ✗ Installation failed for {}", name);
                false
            }
        }
        Err(e) => {
            eprintln!("  ✗ Failed to run installation command: {}", e);
            false
        }
    }
}

fn install_npm_deps() {
    let status = Command::new("npm")
        .arg("install")
        .status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!("  ✓ npm dependencies installed");
            } else {
                println!("  ✗ npm install failed");
            }
        }
        Err(e) => {
            println!("  ✗ Failed to run npm install: {}", e);
        }
    }
}
