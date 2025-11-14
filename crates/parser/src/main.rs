use clap::Parser as ClapParser;
use longitudinal_parser::{parse_markdown_file, ParseOptions};
use std::path::PathBuf;

/// Markdown to JSON converter for blog posts
#[derive(ClapParser, Debug)]
#[command(name = "md2json")]
#[command(about = "Convert markdown blog posts to JSON format", long_about = None)]
struct Cli {
    /// Input markdown file
    input: PathBuf,

    /// Output JSON file (default: input filename with .json extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show warnings and parsing details
    #[arg(short, long)]
    verbose: bool,

    /// Treat warnings as errors
    #[arg(short, long)]
    strict: bool,

    /// Validate JSON against schema (requires schema files in ../config/schemas/)
    #[arg(long)]
    validate: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let mut path = cli.input.clone();
        path.set_extension("json");
        path
    });

    let options = ParseOptions {
        verbose: cli.verbose,
        strict: cli.strict,
        validate: cli.validate,
    };

    // Parse markdown file
    match parse_markdown_file(&cli.input, &output_path, options) {
        Ok(warnings) => {
            if warnings.is_empty() {
                println!(
                    "✅ Successfully converted {} to {}",
                    cli.input.display(),
                    output_path.display()
                );
            } else {
                println!(
                    "✅ Converted {} to {} with {} warning(s)",
                    cli.input.display(),
                    output_path.display(),
                    warnings.len()
                );

                if cli.verbose {
                    for warning in &warnings {
                        println!("⚠️  {warning}");
                    }
                }
            }

            if cli.strict && !warnings.is_empty() {
                eprintln!("❌ Strict mode: treating warnings as errors");
                std::process::exit(1);
            }

            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error: {e}");
            std::process::exit(1);
        }
    }
}
