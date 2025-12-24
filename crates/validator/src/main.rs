/*!
 * Tutorial Validation Pipeline
 *
 * A 5-stage validation system for markdown tutorials:
 * 1. Structural validation (frontmatter, sections, markers)
 * 2. R syntax validation
 * 3. Dry run with sample data
 * 4. Full execution
 * 5. JSON generation and deployment
 *
 * Features:
 * - Smart hash-based caching
 * - Configurable via TOML
 * - Clear error reporting with suggestions
 * - Both automatic and manual deploy modes
 */

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

// Import parser for frontmatter extraction
use longitudinal_parser::validation::parse_structure;

mod cache;
mod config;
mod executor;
mod output;
mod r_prelude;
mod rscript;
mod stages;
mod validators;

use cache::CacheManager;
use config::Config;

#[derive(Parser, Debug)]
#[command(name = "validate")]
#[command(about = "Validate and test tutorial markdown files", long_about = None)]
struct Args {
    /// Tutorial markdown file(s) to validate
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Force validation even if cached
    #[arg(long, short)]
    force: bool,

    /// Run only specific stages (comma-separated: 1,2,3)
    #[arg(long)]
    stages: Option<String>,

    /// Validate only, skip deployment (Stage 5)
    #[arg(long)]
    no_deploy: bool,

    /// Validate only, don't run execution stages (3-4)
    #[arg(long)]
    validate_only: bool,

    /// Deploy only (assumes validation already passed)
    #[arg(long)]
    deploy_only: bool,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,

    /// Config file path
    #[arg(long, default_value = "config/validation.toml")]
    config: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = Config::load(&args.config)?;

    // Initialize cache manager
    let cache_manager = CacheManager::new(&config.cache.directory);

    // Process each tutorial file
    let mut all_passed = true;

    for file in &args.files {
        println!("\n{}", "=".repeat(80));
        println!("Validating: {}", file.display());
        println!("{}", "=".repeat(80));

        // Check cache unless --force
        if !args.force && config.cache.enabled {
            let cache_key = cache_manager.compute_cache_key(file)?;
            if let Some(cached) = cache_manager.get_cached(&cache_key) {
                println!(
                    "✓ Cache hit (validated {} ago)",
                    humanize_duration(cached.cached_at)
                );
                println!("  Status: {}", cached.result.status.to_uppercase());
                println!("  Execution time: {}ms", cached.result.execution_time_ms);
                println!("\nUse --force to skip cache");
                continue;
            }
        }

        // Stage 1: Structural Validation
        use stages::stage1::Stage1Validator;
        let stage1_validator = Stage1Validator::new(&config.stage1);

        match stage1_validator.validate(file) {
            Ok(stage1_result) => {
                output::print_stage1_result(file, &stage1_result, args.verbose);

                if !stage1_result.passed() {
                    all_passed = false;
                    continue; // Skip to next file
                }
            }
            Err(e) => {
                eprintln!("Stage 1 error: {e}");
                all_passed = false;
                continue;
            }
        }

        // Check if this tutorial should skip R stages based on language
        let skip_r_stages = {
            let content = std::fs::read_to_string(file).unwrap_or_default();
            if let Ok(structure) = parse_structure(&content) {
                let language = structure
                    .frontmatter
                    .fields
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("r");
                let should_skip = config
                    .stages
                    .skip_r_stages_for_languages
                    .iter()
                    .any(|l| l == language);
                if should_skip {
                    println!("\n  ℹ Language '{}' detected - skipping R stages (2, 3, 4)", language);
                }
                should_skip
            } else {
                false
            }
        };

        // Stage 2: R Syntax Validation
        if !skip_r_stages {
            use stages::stage2::Stage2Validator;
            let stage2_validator = Stage2Validator::new(&config.stage2, &config.r);

            match stage2_validator.validate(file) {
                Ok(stage2_result) => {
                    output::print_stage2_result(file, &stage2_result, args.verbose);

                    if !stage2_result.passed() {
                        all_passed = false;
                        continue; // Skip to next file
                    }
                }
                Err(e) => {
                    eprintln!("Stage 2 error: {e}");
                    all_passed = false;
                    continue;
                }
            }
        }

        // Stage 3: Dry Run Execution
        if config.stages.enabled.contains(&3) && !args.validate_only && !skip_r_stages {
            use stages::stage3::Stage3Validator;
            let stage3_validator = Stage3Validator::new(&config.stage3, &config.r);

            match stage3_validator.validate(file) {
                Ok(stage3_result) => {
                    output::print_stage3_result(file, &stage3_result, args.verbose);

                    if !stage3_result.passed() {
                        all_passed = false;
                        continue; // Skip to next file
                    }
                }
                Err(e) => {
                    eprintln!("Stage 3 error: {e}");
                    all_passed = false;
                    continue;
                }
            }
        }

        // Stage 4: Full Execution with Real Data
        let stage4_result = if config.stages.enabled.contains(&4) && !args.validate_only && !skip_r_stages {
            use stages::stage4::Stage4Validator;
            let stage4_validator = Stage4Validator::new(&config.stage4, &config.r);

            match stage4_validator.validate(file) {
                Ok(stage4_result) => {
                    output::print_stage4_result(file, &stage4_result, args.verbose);

                    if !stage4_result.is_passing() {
                        all_passed = false;
                        continue; // Skip to next file
                    }
                    Some(stage4_result)
                }
                Err(e) => {
                    eprintln!("Stage 4 error: {e}");
                    all_passed = false;
                    continue;
                }
            }
        } else {
            None
        };

        // Stage 5: JSON Generation & Deployment
        if config.stages.enabled.contains(&5) && !args.no_deploy && !args.validate_only {
            use stages::stage5::Stage5Validator;
            let stage5_validator = Stage5Validator::new(&config.stage5);

            match stage5_validator.validate(file, stage4_result.as_ref()) {
                Ok(stage5_result) => {
                    output::print_stage5_result(file, &stage5_result, args.verbose);

                    if !stage5_result.is_passing() {
                        all_passed = false;
                        continue; // Skip to next file
                    }
                }
                Err(e) => {
                    eprintln!("Stage 5 error: {e}");
                    all_passed = false;
                    continue;
                }
            }
        }
    }

    // Exit with appropriate code
    if all_passed {
        println!("\n✅ All tutorials passed validation");
        Ok(())
    } else {
        println!("\n❌ Some tutorials failed validation");
        std::process::exit(1);
    }
}

fn humanize_duration(timestamp: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_hours() < 1 {
        format!("{} minutes", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours", duration.num_hours())
    } else {
        format!("{} days", duration.num_days())
    }
}
