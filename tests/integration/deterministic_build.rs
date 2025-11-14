// Integration test: Verify SSG builds are byte-for-byte deterministic
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Hash all HTML files in a directory tree
fn hash_tree(dir: &Path) -> Vec<(String, blake3::Hash)> {
    let mut entries = vec![];
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "html").unwrap_or(false) {
            let bytes = fs::read(path).expect("read HTML file");
            let rel = path
                .strip_prefix(dir)
                .unwrap()
                .to_string_lossy()
                .to_string();
            entries.push((rel, blake3::hash(&bytes)));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

#[test]
fn ssg_outputs_are_deterministic() {
    // Locate the SSG binary (cargo test builds it with --bin flag)
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_longitudinal_dev"));

    if !bin.exists() {
        panic!(
            "SSG binary not found at {bin:?}. Run: cargo build --bin longitudinal_dev --features ssr"
        );
    }

    // Create two clean temp directories
    let t1 = tempfile::tempdir().expect("create temp dir 1");
    let t2 = tempfile::tempdir().expect("create temp dir 2");

    eprintln!("🔨 Running SSG build #1 in {:?}", t1.path());
    let s1 = Command::new(&bin)
        .args(["--outdir", t1.path().to_str().unwrap()])
        .status()
        .expect("run SSG build 1");
    assert!(s1.success(), "SSG build 1 failed");

    eprintln!("🔨 Running SSG build #2 in {:?}", t2.path());
    let s2 = Command::new(&bin)
        .args(["--outdir", t2.path().to_str().unwrap()])
        .status()
        .expect("run SSG build 2");
    assert!(s2.success(), "SSG build 2 failed");

    // Hash trees and compare
    eprintln!("🔍 Comparing output files...");
    let h1 = hash_tree(t1.path());
    let h2 = hash_tree(t2.path());

    assert_eq!(
        h1.len(),
        h2.len(),
        "Different number of output files: {} vs {}",
        h1.len(),
        h2.len()
    );

    for ((p1, h1v), (p2, h2v)) in h1.iter().zip(h2.iter()) {
        assert_eq!(p1, p2, "Different file sets: {p1} vs {p2}");
        assert_eq!(
            h1v, h2v,
            "Non-deterministic content in {p1}\n  Build 1: {h1v:?}\n  Build 2: {h2v:?}"
        );
    }

    let len = h1.len();
    eprintln!("✅ All {len} HTML files are byte-for-byte identical");
}
