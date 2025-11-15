//! Build automation tasks for Leptos SSG blog
//!
//! This is an xtask-style build tool that orchestrates the complete SSG build process.
//! See: https://github.com/matklad/cargo-xtask
//!
//! Usage:
//!   cargo xtask build-ssg [--outdir <path>]
//!   cargo xtask --help

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return ExitCode::FAILURE;
    }

    match args[1].as_str() {
        "build-ssg" => {
            let outdir = parse_outdir(&args);
            build_ssg(&outdir)
        }
        "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        cmd => {
            eprintln!("❌ Unknown command: {cmd}");
            eprintln!();
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!("Leptos SSG Build Tasks");
    println!();
    println!("USAGE:");
    println!("    cargo xtask <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    build-ssg    Build complete static site");
    println!();
    println!("OPTIONS:");
    println!("    --outdir <path>    Output directory (default: dist)");
    println!();
    println!("EXAMPLES:");
    println!("    cargo xtask build-ssg");
    println!("    cargo xtask build-ssg --outdir public");
}

fn parse_outdir(args: &[String]) -> String {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--outdir" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    "dist".to_string()
}

fn build_ssg(outdir: &str) -> ExitCode {
    println!("🏗️  Building complete SSG site...");
    println!("Output directory: {outdir}");

    // Check for optional SITE_BASE_PATH
    if let Ok(base_path) = env::var("SITE_BASE_PATH") {
        if !base_path.is_empty() {
            println!("📍 Using base path: {base_path}");
        }
    }

    // Step 1: Check for wasm32-unknown-unknown target
    if let Err(e) = check_wasm_target() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 2: Build WASM
    if let Err(e) = build_wasm() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 3: Check for wasm-bindgen
    if let Err(e) = check_wasm_bindgen() {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 4: Run wasm-bindgen
    if let Err(e) = run_wasm_bindgen(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 4.5: Optimize WASM
    if let Err(e) = optimize_wasm(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 5: Build Tailwind CSS
    if let Err(e) = build_tailwind(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 6: Copy public assets
    if let Err(e) = copy_public_assets(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 7: Build and run SSG binary
    if let Err(e) = generate_static_html(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 7.5: Compress assets
    if let Err(e) = compress_assets(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 8: Inject FOUC prevention script
    if let Err(e) = inject_fouc_script(outdir) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    // Step 9: Print summary
    print_summary(outdir);

    ExitCode::SUCCESS
}

fn check_wasm_target() -> Result<(), String> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map_err(|e| format!("❌ Failed to run rustup: {e}"))?;

    let installed = String::from_utf8_lossy(&output.stdout);
    if !installed.contains("wasm32-unknown-unknown") {
        return Err("❌ Error: wasm32-unknown-unknown target not installed\n\n\
             Please install it with:\n  rustup target add wasm32-unknown-unknown\n"
            .to_string());
    }

    Ok(())
}

fn build_wasm() -> Result<(), String> {
    println!("📦 Building WASM...");

    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
            "--lib",
            "--no-default-features",
            "--features",
            "hydrate",
        ])
        .status()
        .map_err(|e| format!("❌ Failed to run cargo: {e}"))?;

    if !status.success() {
        return Err("❌ WASM build failed".to_string());
    }

    Ok(())
}

fn check_wasm_bindgen() -> Result<(), String> {
    match Command::new("wasm-bindgen").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err("❌ Error: wasm-bindgen not found in PATH\n\n\
             Please install it with:\n  cargo install wasm-bindgen-cli\n\n\
             Or ensure it's in your PATH\n"
            .to_string()),
    }
}

fn run_wasm_bindgen(outdir: &str) -> Result<(), String> {
    println!("🔗 Running wasm-bindgen...");

    // Create output directory
    let pkg_dir = PathBuf::from(outdir).join("pkg");
    fs::create_dir_all(&pkg_dir).map_err(|e| format!("❌ Failed to create pkg directory: {e}"))?;

    let status = Command::new("wasm-bindgen")
        .arg("target/wasm32-unknown-unknown/release/longitudinal_dev.wasm")
        .args(["--target", "web"])
        .arg("--no-typescript")
        .arg("--out-dir")
        .arg(pkg_dir)
        .args(["--out-name", "blog"])
        .status()
        .map_err(|e| format!("❌ Failed to run wasm-bindgen: {e}"))?;

    if !status.success() {
        return Err("❌ wasm-bindgen failed".to_string());
    }

    Ok(())
}

fn optimize_wasm(outdir: &str) -> Result<(), String> {
    println!("🔧 Optimizing WASM with wasm-opt...");

    let wasm_path = PathBuf::from(outdir)
        .join("pkg")
        .join("blog_bg.wasm");

    if !wasm_path.exists() {
        return Err(format!("❌ WASM file not found: {}", wasm_path.display()));
    }

    // Check if wasm-opt is available
    let wasm_opt_check = Command::new("wasm-opt")
        .arg("--version")
        .output();

    if wasm_opt_check.is_err() {
        eprintln!("⚠️  wasm-opt not found - skipping WASM optimization");
        eprintln!("    Install it with: brew install binaryen");
        eprintln!("    (Build will continue without optimization)");
        return Ok(());
    }

    let status = Command::new("wasm-opt")
        .arg(&wasm_path)
        .args(["-Os", "--enable-bulk-memory", "--strip-debug"])  // Maximum size optimization
        .arg("-o")
        .arg(&wasm_path)
        .status()
        .map_err(|e| format!("❌ Failed to run wasm-opt: {e}"))?;

    if !status.success() {
        return Err("❌ wasm-opt optimization failed".to_string());
    }

    println!("✅ WASM optimization complete");
    Ok(())
}

fn build_tailwind(outdir: &str) -> Result<(), String> {
    println!("🎨 Building Tailwind CSS...");

    let css_output = PathBuf::from(outdir).join("pkg/blog.css");

    let status = Command::new("npx")
        .args(["tailwindcss", "-i", "style/input.css", "-o"])
        .arg(&css_output)
        .arg("--minify")
        .status()
        .map_err(|e| format!("❌ Failed to run npx: {e}"))?;

    if !status.success() {
        return Err("❌ Tailwind CSS build failed\n\n\
             Ensure you have run 'npm install' to set up dependencies:\n  npm install\n"
            .to_string());
    }

    Ok(())
}

fn copy_public_assets(outdir: &str) -> Result<(), String> {
    println!("📁 Copying public assets...");

    let public_dir = Path::new("public");
    if !public_dir.exists() {
        println!("⚠️  No public directory found, skipping asset copy");
        return Ok(());
    }

    // Create output directory
    fs::create_dir_all(outdir).map_err(|e| format!("❌ Failed to create output directory: {e}"))?;

    // Copy recursively
    copy_dir_all(public_dir, Path::new(outdir))?;

    println!("✅ Public assets copied");
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read directory: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
            copy_dir_all(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path).map_err(|e| format!("Failed to copy file: {e}"))?;
        }
    }
    Ok(())
}

fn generate_static_html(outdir: &str) -> Result<(), String> {
    println!("📄 Generating static HTML...");

    // Build SSG binary
    let status = Command::new("cargo")
        .args([
            "build",
            "--bin",
            "longitudinal_dev",
            "--features",
            "ssr",
            "--release",
        ])
        .status()
        .map_err(|e| format!("❌ Failed to run cargo: {e}"))?;

    if !status.success() {
        return Err("❌ SSG binary build failed".to_string());
    }

    // Run SSG binary
    let status = Command::new("./target/release/longitudinal_dev")
        .arg("--outdir")
        .arg(outdir)
        .status()
        .map_err(|e| format!("❌ Failed to run SSG binary: {e}"))?;

    if !status.success() {
        return Err("❌ SSG generation failed".to_string());
    }

    Ok(())
}

fn compress_assets(outdir: &str) -> Result<(), String> {
    println!("🗜️  Compressing assets...");

    let pkg_dir = PathBuf::from(outdir).join("pkg");
    let assets = ["blog_bg.wasm", "blog.js", "blog.css"];

    for asset in &assets {
        let asset_path = pkg_dir.join(asset);

        if !asset_path.exists() {
            eprintln!("⚠️  Asset not found, skipping: {asset}");
            continue;
        }

        // Brotli compression (best ratio, ~77% reduction)
        // IMPORTANT: Use --keep to preserve original file
        let brotli_check = Command::new("brotli").arg("--version").output();

        if brotli_check.is_ok() {
            let brotli_status = Command::new("brotli")
                .args(["--quality=11", "--keep", "--force"])  // --keep preserves original!
                .arg(&asset_path)
                .status();

            match brotli_status {
                Ok(status) if status.success() => {
                    println!("  ✓ Compressed {asset} (brotli)");
                }
                _ => {
                    eprintln!("  ⚠️  Brotli compression failed for {asset}");
                }
            }
        } else {
            eprintln!("  ⚠️  brotli not found - skipping brotli compression for {asset}");
            eprintln!("      Install it with: brew install brotli");
        }

        // Gzip compression (fallback, ~70% reduction)
        // gzip always preserves original when using --keep
        let gzip_status = Command::new("gzip")
            .args(["--best", "--keep", "--force"])  // --keep preserves original!
            .arg(&asset_path)
            .status();

        match gzip_status {
            Ok(status) if status.success() => {
                println!("  ✓ Compressed {asset} (gzip)");
            }
            _ => {
                eprintln!("  ⚠️  Gzip compression failed for {asset}");
            }
        }
    }

    println!("✅ Asset compression complete");
    println!("    Note: Compressed files (.br, .gz) are served automatically by GitHub Pages");
    Ok(())
}

fn inject_fouc_script(outdir: &str) -> Result<(), String> {
    println!("🎨 Injecting theme FOUC prevention script...");

    let theme_script = r#"<script>(function(){const theme=localStorage.getItem("theme")||"dracula";document.documentElement.setAttribute("data-theme",theme)})()</script>"#;

    // Find all HTML files recursively
    inject_script_recursive(Path::new(outdir), theme_script)?;

    Ok(())
}

fn inject_script_recursive(dir: &Path, script: &str) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();

        if path.is_dir() {
            inject_script_recursive(&path, script)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
            inject_script_into_file(&path, script)?;
        }
    }
    Ok(())
}

fn inject_script_into_file(path: &Path, script: &str) -> Result<(), String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    // Replace <head> with <head><script>
    let modified = content.replace("<head>", &format!("<head>{script}"));

    fs::write(path, modified).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

    Ok(())
}

fn print_summary(outdir: &str) {
    println!("\n✅ SSG build complete! Output in {outdir}/");
    println!("\n📊 Asset sizes:");

    let pkg_path = Path::new(outdir).join("pkg");

    // Print WASM files with compression comparison
    println!("\n  WASM assets:");
    let wasm_assets = [
        ("blog_bg.wasm", "Optimized binary"),
        ("blog_bg.wasm.br", "Brotli compressed (served to modern browsers)"),
        ("blog_bg.wasm.gz", "Gzip compressed (fallback)"),
    ];

    for (filename, description) in &wasm_assets {
        let path = pkg_path.join(filename);
        if let Ok(metadata) = fs::metadata(&path) {
            let size = format_size(metadata.len());
            let name = path.file_name().unwrap().to_string_lossy();
            println!("    {:<20} {:>10}  — {}", name, size, description);
        }
    }

    // Print JS and CSS
    println!("\n  Other assets:");
    let other_assets = [
        ("blog.js", "JavaScript glue code"),
        ("blog.js.br", "Brotli compressed"),
        ("blog.js.gz", "Gzip compressed"),
        ("blog.css", "Stylesheet"),
        ("blog.css.br", "Brotli compressed"),
        ("blog.css.gz", "Gzip compressed"),
    ];

    for (filename, description) in &other_assets {
        let path = pkg_path.join(filename);
        if let Ok(metadata) = fs::metadata(&path) {
            let size = format_size(metadata.len());
            let name = path.file_name().unwrap().to_string_lossy();
            println!("    {:<20} {:>10}  — {}", name, size, description);
        }
    }

    // Print HTML files (root level)
    println!("\n  HTML pages:");
    if let Ok(entries) = fs::read_dir(Path::new(outdir)) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("html") {
                if let Ok(metadata) = fs::metadata(&path) {
                    let size = format_size(metadata.len());
                    let name = path.file_name().unwrap().to_string_lossy();
                    println!("    {:<20} {:>10}", name, size);
                }
            }
        }
    }

    // Print summary of tutorial pages
    let posts_dir = Path::new(outdir).join("posts");
    if posts_dir.exists() {
        let mut post_count = 0;
        let mut total_size = 0u64;

        if let Ok(entries) = fs::read_dir(&posts_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let index_html = path.join("index.html");
                    if let Ok(metadata) = fs::metadata(&index_html) {
                        post_count += 1;
                        total_size += metadata.len();
                    }
                }
            }
        }

        if post_count > 0 {
            println!("\n  Tutorial pages:");
            println!("    {} tutorials totaling {}", post_count, format_size(total_size));
            let avg_size = total_size / post_count as u64;
            println!("    Average size per tutorial: {}", format_size(avg_size));
        }
    }

    println!("\n💡 Tip: Run 'cd dist && python3 -m http.server 8000' to test locally");
    println!("   (Note: Python's server won't serve .br/.gz; use GitHub Pages for compression)");
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.1}M", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}K", bytes as f64 / KB as f64)
    } else {
        format!("{bytes}B")
    }
}
