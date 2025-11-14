# longitudinal_parser (md2json CLI)

The `longitudinal_parser` crate ships the `md2json` CLI, which converts structured tutorial markdown into the JSON schema consumed by the SSG
and validator pipeline. The binary is invoked automatically during Stage 5, but it can also
be run manually while debugging parser behaviour.

## Status

- ✅ Supports flexible block-based schema for Data Preparation and Statistical Analysis.
- ✅ CLI with verbose, strict, and schema-validation flags.
- ✅ Section-specific parsers matching the contracts in `config/schemas/`.
- ✅ Comprehensive unit tests covering all section parsers.

## Quick Start

```bash
cd crates/parser
cargo build --release

# Convert an existing tutorial (strict mode + schema validation)
./target/release/md2json \
  ../../content/tutorials/lgcm-basic.md \
  --output ../../content/posts/lgcm-basic.post.json \
  --strict --validate
```

## CLI Reference

```
md2json [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input markdown file

Options:
  -o, --output <OUTPUT>  Output JSON file (defaults to replacing .md with .json)
  -v, --verbose          Print detailed parsing information
  -s, --strict           Treat warnings as errors
      --validate         Validate generated JSON against config/schemas/
  -h, --help             Show help text
```

## Supported Features

- Frontmatter extraction (title, slug, schema version, metadata)
- Section detection and ordering enforcement
- Overview panels (summary, stats, features)
- Data Access requirements table + method cards
- Flexible blocks for Data Preparation & Statistical Analysis (`code`, `output`, `note`)
- Discussion paragraphs, insights, limitations, and resource cards

Parser marker behaviour is documented in `docs/authoring/AUTHORING_GUIDE.md`.

## Development

```bash
# Run the parser test suite
cargo test

# Lint formatting of generated JSON in tests
cargo test -- --nocapture
```

When adding new syntax, update both the parser module under `src/parsers/` and the
corresponding schema under `config/schemas/`.

**Last Updated:** 2025-10-31
