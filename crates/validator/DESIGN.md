# Validator Crate Design

## Purpose
The `validator` crate isolates the tutorial validation pipeline from the main web application. This separation:
- Reduces compilation times for the main app (no validator deps in WASM)
- Enables independent versioning and testing of validation logic
- Simplifies CI integration (structure-only mode)
- Clarifies ownership boundaries (validation vs. rendering)

## Architecture Decision: Binary-Only Crate

**Decision:** The validator will be a **binary-only crate** (no public library API).

**Rationale:**
1. **Single consumer:** Only used via CLI in development/CI workflows
2. **Simpler evolution:** No API stability guarantees needed
3. **Clear boundaries:** Main app uses generated JSON artifacts, not validator internals
4. **Future flexibility:** Can expose library API later if multi-binary needs emerge

## Dependencies

### Validator-Specific Dependencies
These dependencies are **only** used by the validator and will move to the new crate:

- `clap` (4.0) - CLI argument parsing (only in main.rs)
- `toml` (0.8) - Config file loading (config.rs)
- `md5` (0.7) - Cache key generation (cache.rs)
- `wait-timeout` (0.2) - R process timeout handling (executor.rs)
- `chrono` - Timestamp handling for cache (shared with test_tutorials binary)

### Shared Dependencies
These remain in workspace root or are duplicated (lightweight):

- `anyhow` (1.0) - Error handling (used throughout codebase)
- `regex` (1.0) - Pattern matching (also in dev-dependencies)
- `md2json` (workspace crate) - Markdown parser (validator's primary API)

**Note on `md2json`:** The validator is the primary consumer of `md2json::validation` API. The main app only uses generated JSON files. This validates the separation—validation logic belongs in its own crate.

## Migration Strategy

### Phase 1.2 Tasks
1. Create `crates/validator/` with its own `Cargo.toml`
2. Move `src/bin/validate/**` → `crates/validator/src/`
3. Move validator-specific dependencies to new crate
4. Update workspace `members` in root `Cargo.toml`
5. Keep `chrono` in root for now (also used by `test_tutorials` binary)

### Path Resolutions
- Cache directory: `.cache/` (relative to workspace root, not crate root)
- Config file: `config/validation.toml` (relative to workspace root)
- Tutorial files: `content/tutorials/*.md` (workspace root)

**Implementation:** Use `std::env::current_dir()` or pass workspace root via env var if needed.

## Future Considerations

### Potential Library API (if needed later)
If we need to expose validation stages programmatically:

```rust
// crates/validator/src/lib.rs
pub mod stages {
    pub use crate::stages::stage1::Stage1Validator;
    pub use crate::stages::stage2::Stage2Validator;
    // etc.
}
```

**Trigger conditions:**
- Multiple binaries need validation logic (e.g., a watch-mode daemon)
- Integration tests require programmatic validation
- Third-party tools want to validate tutorials

Until then, keeping it binary-only reduces maintenance overhead.

## Rollback Plan

If extraction causes unforeseen issues:
```bash
git tag pre-validator-crate  # Created before P1.2
git reset --hard pre-validator-crate
```

All validator functionality continues working from `src/bin/validate/` layout.

---

**Acceptance Criteria for P1.1:**
- ✅ Design decision documented (binary-only vs. library)
- ✅ Dependencies inventoried and categorized
- ✅ Migration strategy outlined
- ✅ Path resolution plan documented
- ✅ Rollback plan defined
