//! Simple JSON syntax validator using serde_json
//!
//! Usage: cargo run --bin validate-json -- <file.json>
//! Exit code: 0 if valid, 1 if invalid

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.json>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo run --bin validate-json -- content/posts/hello-world.post.json");
        process::exit(1);
    }

    let filepath = &args[1];

    // Read file
    let contents = match fs::read_to_string(filepath) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("❌ Error reading {filepath}: {e}");
            process::exit(1);
        }
    };

    // Validate JSON syntax
    match serde_json::from_str::<serde_json::Value>(&contents) {
        Ok(_) => {
            // Valid JSON - silent success
            process::exit(0);
        }
        Err(e) => {
            eprintln!("❌ Invalid JSON in {filepath}");
            eprintln!("   Error: {e}");
            process::exit(1);
        }
    }
}
