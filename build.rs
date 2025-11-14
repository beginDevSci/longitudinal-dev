use serde_json::Value;
use std::error::Error;
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=content/posts");
    println!("cargo:rerun-if-changed=config/schemas");

    let posts_dir = Path::new("content/posts");
    let schemas_dir = Path::new("config/schemas");

    // Load all schema files into a single combined schema to resolve $refs
    // Note: combined_schema is reserved for future schema composition
    let _combined_schema = serde_json::json!({
        "$id": "https://example.org/schema/post.json",
        "type": "object"
    });

    // Load all schema files and register them with absolute file:// URLs
    let mut schema_documents = Vec::new();

    for entry in fs::read_dir(schemas_dir).expect("Failed to read schemas directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Only process .json files
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read schema: {}", path.display()));

        let mut schema_json: Value = serde_json::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse schema JSON: {}", path.display()));

        // Generate absolute file:// URL using url crate (handles Windows UNC paths correctly)
        let canonical_path = path
            .canonicalize()
            .unwrap_or_else(|_| panic!("Failed to canonicalize path: {}", path.display()));

        let file_url = Url::from_file_path(&canonical_path)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to convert path to file URL: {}",
                    canonical_path.display()
                )
            })
            .to_string();

        // Inject $id
        schema_json["$id"] = serde_json::json!(file_url);

        schema_documents.push(schema_json);
    }

    // Find root schema
    let root_schema = schema_documents
        .iter()
        .find(|s| {
            s.get("$id")
                .and_then(|v| v.as_str())
                .map(|id| id.contains("post.schema.json"))
                .unwrap_or(false)
        })
        .expect("post.schema.json not found")
        .clone();

    // Build JSONSchema with all documents pre-loaded
    let mut options = jsonschema::JSONSchema::options();

    for schema in &schema_documents {
        let id = schema
            .get("$id")
            .and_then(|v| v.as_str())
            .expect("Schema missing $id field");
        options.with_document(id.to_string(), schema.clone());
    }

    // Compile root schema
    let compiled = options
        .compile(&root_schema)
        .expect("Failed to compile post.schema.json");

    // Discover posts
    let mut posts: Vec<(String, PathBuf)> = vec![];
    for entry in fs::read_dir(posts_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            if stem.ends_with(".post") {
                let slug = stem.trim_end_matches(".post").to_string();
                posts.push((slug, path));
            }
        }
    }
    posts.sort_by(|a, b| a.0.cmp(&b.0));

    // Validate and collect include paths
    let mut gen = String::new();
    gen.push_str("// AUTO-GENERATED. DO NOT EDIT.\n");
    gen.push_str("use crate::models::json_dto::JsonPost;\n\n");
    gen.push_str("/// All discovered post slugs (from content/posts/*.post.json)\n");
    gen.push_str("pub static ALL_SLUGS: &[&str] = &[\n");
    for (slug, _) in &posts {
        gen.push_str(&format!("  \"{slug}\",\n"));
    }
    gen.push_str("];\n\n");
    gen.push_str("/// Load and deserialize all posts as raw DTOs using `serde`.\n");
    gen.push_str("/// REQUIREMENT: `JsonPost` must implement `Deserialize`.\n");
    gen.push_str("pub fn load_posts_raw() -> Vec<JsonPost> {\n");
    gen.push_str("  Vec::from([\n");

    let mut had_errors = false;
    for (slug, path) in &posts {
        let raw = fs::read_to_string(path)?;
        let json: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                let path_display = path.display();
                let line = e.line();
                let column = e.column();
                eprintln!("{path_display}:{line}:{column}: error: Invalid JSON");
                eprintln!("  {e}");
                had_errors = true;
                continue;
            }
        };

        // Check schema version (optional field, warn if not "2")
        if let Some(version) = json.get("schema_version").and_then(|v| v.as_str()) {
            if version != "2" {
                eprintln!("warning: {slug} uses schema_version '{version}', expected '2'");
            }
        }

        // Validate against schema
        if let Err(errors) = compiled.validate(&json) {
            had_errors = true;
            let path_display = path.display();
            eprintln!("{path_display}: Schema validation errors in '{slug}':");
            for err in errors {
                eprintln!("  {err} at {}", err.instance_path);
            }
        }

        // Emit include_str! and deserialize at runtime
        // Path is relative to src/generated/, so need ../../ to reach project root
        let rel = path
            .strip_prefix(".")
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        gen.push_str(&format!(
                "    {{\n      let json = include_str!(\"../../{rel}\");\n      serde_json::from_str(json).expect(\"deserialize JsonPost: {slug}\")\n    }},\n"
            ));
    }

    gen.push_str("  ])\n}\n");

    if had_errors {
        panic!("One or more posts failed schema validation. See errors above.");
    }

    fs::create_dir_all("src/generated").ok();
    fs::write("src/generated/manifest.rs", gen)?;

    Ok(())
}
