# Tutorials Page Scaling Plan

**Branch:** `feature/tutorials-scaling`
**Status:** Phases 0-3 Complete (Schema, Index Pipeline, URLs, Client-Side Fetch)
**Created:** 2025-01-04
**Updated:** 2025-01-05
**Target:** Scale from 16 → 200-300 tutorials

---

## Executive Summary

Transform `/tutorials/` from "browse everything" to **Finder + Curated Library**:
- Search-first header with ranked results
- Curated discovery blocks (Getting Started, Common Workflows, Recently Updated)
- "Browse by" faceted navigation replacing horizontal tabs
- Always-paginated results (24-36 per page)
- Method family hub pages (`/tutorials/<family>/`)

---

## Approved Decisions

### 1. URL Migration Strategy
**Decision:** Keep `/posts/` redirects **indefinitely** as a permanent compatibility layer.

| Aspect | Decision |
|--------|----------|
| New canonical URL | `/tutorials/<family>/<slug>/` |
| Old URL behavior | 301 redirect to canonical |
| Sunset date | None planned (permanent compat layer) |
| Implementation | Static redirect pages at build time |

**Rationale:** External links accumulate (papers, syllabi, bookmarks). Sunsetting creates link rot. Redirects are cheap.

### 2. Schema Migration Strategy
**Decision:** Optional-first fields with gated enforcement timeline.

| Phase | Enforcement Level |
|-------|-------------------|
| Phase 0 | Optional fields + fallbacks + **warnings** |
| Phase 1 | Hard error only for core required (slug/title/family) |
| Phase 2 | Full enforcement after content migration |

**Field decisions:**
- `difficulty` → Add as optional, populate for all 16
- `timepoints` → Add as optional, use coarse buckets (2, 3_5, 6_plus, irregular)
- `engines` → Change to array, wrap existing `engine` as `["lme4"]`
- `summary` → Add explicit field, derive from description as fallback
- `updated_at` → Map from `date_iso` in index builder (no duplicate maintenance)
- `schema` → Optional version field for future evolution

### 3. Architecture Approach
**Decision:** Two-step refactor to reduce risk.

| Step | Scope |
|------|-------|
| Step A | Extract index generation (JSON artifacts) — keep current UI intact |
| Step B | Switch UI from embedded Vec to fetched index (behind branch/flag) |

**Rationale:** Avoids "big bang" where data pipeline + UI rewrite happen simultaneously.

### 4. Curated Content Ownership
**Decision:** Implementer proposes initial draft; lists live in editable file.

| Collection | Source |
|------------|--------|
| Getting Started | Implementer proposes 6-8 items |
| Common Workflows | Implementer proposes 4 buckets × 3-5 items |
| Recently Updated | Computed automatically from `updated_at` |

**Storage:** `content/tutorial_curations.yaml` (editable without code changes)

---

## Build Pipeline Reference

Understanding the current pipeline is essential for implementation:

```
content/tutorials/*.md          # Source of truth (markdown + frontmatter)
        ↓
   [crates/parser]              # Parses markdown → JSON structure
        ↓
content/posts/*.post.json       # Pre-parsed JSON (HTML content + metadata)
        ↓
   [build.rs / xtask]           # Generates manifest
        ↓
src/generated/manifest.rs       # include_str!() all JSON at compile time
        ↓
   [src/main.rs]                # SSG: generates HTML from posts()
        ↓
dist/                           # Output: static HTML files
```

**Key files:**
| File | Role |
|------|------|
| `content/tutorials/*.md` | Authoring source (16 files) |
| `content/posts/*.post.json` | Parsed output (16 files, auto-generated) |
| `src/generated/manifest.rs` | Compile-time post loading |
| `src/posts.rs` | `posts()` function loads via manifest |
| `src/models/post.rs` | `Post` and `PostMetadata` structs |
| `src/tutorial_catalog.rs` | Current catalog UI (receives `Vec<TutorialData>`) |
| `src/main.rs:88-131` | Tutorial catalog page generation |
| `crates/parser/` | Markdown → JSON conversion |
| `crates/validator/` | Validation pipeline (R code execution) |

---

## File-Level Implementation Outline

### Phase 0: Schema Migration

**Goal:** Add new metadata fields, migrate 16 tutorials, add build warnings.

#### 0.1 Update frontmatter schema

**File:** `content/tutorials/*.md` (all 16 files)

Add to frontmatter:
```yaml
difficulty: intro          # NEW: intro | intermediate | advanced
timepoints: 3_5            # NEW: 2 | 3_5 | 6_plus | irregular
engines:                   # CHANGE: array instead of scalar
  - lme4
summary: |                 # NEW: explicit (can derive from description initially)
  Short summary for catalog display...
```

**File:** `crates/parser/src/frontmatter.rs`

Update frontmatter parsing to accept new fields with defaults:
```rust
pub struct TutorialFrontmatter {
    // Existing...
    pub engine: Option<String>,        // Keep for backward compat
    pub engines: Option<Vec<String>>,  // NEW: takes precedence if present
    pub difficulty: Option<String>,    // NEW
    pub timepoints: Option<String>,    // NEW
    pub summary: Option<String>,       // NEW
}
```

#### 0.2 Add schema validation with warnings

**File:** `crates/parser/src/validation.rs` (or new `src/schema_validation.rs`)

```rust
pub fn validate_tutorial_schema(meta: &PostMetadata) -> Vec<Warning> {
    let mut warnings = vec![];

    if meta.difficulty.is_none() {
        warnings.push(Warning::MissingField("difficulty"));
    }
    if meta.timepoints.is_none() {
        warnings.push(Warning::MissingField("timepoints"));
    }
    // ... etc

    warnings
}
```

**File:** `src/main.rs` or `crates/validator/src/main.rs`

Print warnings during build but don't fail:
```rust
for warning in validate_tutorial_schema(&post.metadata) {
    eprintln!("⚠️  {}: {}", post.slug, warning);
}
```

---

### Phase 1: Index Pipeline

**Goal:** Generate JSON artifacts at build time; decouple from HTML serialization.

#### 1.1 Create index generator

**New file:** `src/index_generator.rs`

```rust
use serde::Serialize;

#[derive(Serialize)]
pub struct TutorialIndexEntry {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub method_family: String,
    pub difficulty: String,
    pub outcome_type: String,
    pub engines: Vec<String>,
    pub covariates: String,
    pub timepoints: String,
    pub updated_at: String,
    pub tags: Vec<String>,
    // Precomputed for search
    pub search_text: String,
}

#[derive(Serialize)]
pub struct FamilyEntry {
    pub id: String,
    pub label: String,
    pub count: usize,
    pub order: usize,
}

pub fn generate_tutorial_index(posts: &[Post]) -> Vec<TutorialIndexEntry> { ... }
pub fn generate_family_index(posts: &[Post]) -> Vec<FamilyEntry> { ... }
```

#### 1.2 Add to build output

**File:** `src/main.rs`

After generating HTML, also write JSON:
```rust
// Generate JSON index artifacts
let index = generate_tutorial_index(&posts_with_metadata);
let families = generate_family_index(&posts_with_metadata);

write(
    site_root.join("api/tutorial_index.json"),
    serde_json::to_string(&index)?
)?;
write(
    site_root.join("api/tutorial_families.json"),
    serde_json::to_string(&families)?
)?;
```

#### 1.3 Create curations file

**New file:** `content/tutorial_curations.yaml`

```yaml
getting_started:
  - lmm-random-intercept
  - lgcm-basic
  - gee
  - glmm
  # ... 6-8 beginner-friendly tutorials

workflows:
  time_varying_covariates:
    label: "Time-Varying Covariates"
    tutorials:
      - lmm-time-invariant-covariates
      - lgcm-time-invariant-covariates
      - gee-time-varying-covariate
  # ... more workflow categories

# recently_updated: computed automatically from updated_at
```

**Build step:** Read and output as `api/tutorial_curations.json`

---

### Phase 2: URL Structure & Redirects

**Goal:** New canonical URLs + permanent redirects from `/posts/`.

#### 2.1 Update tutorial page generation

**File:** `src/main.rs`

Change from:
```rust
let post_dir = posts_dir.join(&slug);  // /posts/<slug>/
```

To:
```rust
let family_slug = post.metadata.method_family.to_lowercase();
let tutorial_dir = tutorials_dir
    .join(&family_slug)
    .join(&slug);  // /tutorials/<family>/<slug>/
```

#### 2.2 Generate redirect pages

**File:** `src/main.rs`

Add redirect generation:
```rust
// Generate /posts/<slug>/ redirect pages
for post in &all_posts {
    let family = post.metadata.method_family.to_lowercase();
    let redirect_html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta http-equiv="refresh" content="0; url=/tutorials/{}/{}/">
    <link rel="canonical" href="/tutorials/{}/{}/">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="/tutorials/{}/{}/">new location</a>...</p>
</body>
</html>"#, family, post.slug, family, post.slug, family, post.slug);

    let redirect_dir = posts_dir.join(&post.slug);
    create_dir_all(&redirect_dir)?;
    write(redirect_dir.join("index.html"), redirect_html)?;
}
```

#### 2.3 Update internal links

**File:** `src/tutorial_catalog.rs`

Update `TutorialCard`:
```rust
// Old: let href = base_path::join(&format!("posts/{}/", tutorial.slug));
let href = base_path::join(&format!(
    "tutorials/{}/{}/",
    tutorial.method_family.to_lowercase(),
    tutorial.slug
));
```

---

### Phase 3: Client-Side Index Loading

**Goal:** Remove embedded data, fetch index on demand.

#### 3.1 Create fetch-based catalog

**File:** `src/tutorial_catalog.rs`

Replace `TutorialCatalog` that takes `Vec<TutorialData>` with:
```rust
#[island]
pub fn TutorialCatalog() -> impl IntoView {
    // Signals for data + loading state
    let tutorials = RwSignal::new(Vec::<TutorialIndexEntry>::new());
    let loading = RwSignal::new(true);

    // Fetch on mount (or on first interaction for deferred loading)
    Effect::new(move |_| {
        spawn_local(async move {
            let resp = gloo_net::http::Request::get("/api/tutorial_index.json")
                .send()
                .await
                .unwrap();
            let data: Vec<TutorialIndexEntry> = resp.json().await.unwrap();
            tutorials.set(data);
            loading.set(false);
        });
    });

    // ... rest of component
}
```

#### 3.2 Update main.rs to not embed data

**File:** `src/main.rs`

Change:
```rust
// Old: <TutorialCatalog tutorials=tutorial_data.clone() />
<TutorialCatalog />  // No props - fetches its own data
```

---

### Phase 4-11: UI Components (Summary)

| Phase | Key Files | Changes |
|-------|-----------|---------|
| 4 (Filters) | `src/tutorial_catalog.rs` | Collapsible filter groups, empty states, clear all |
| 5 (Search) | `src/tutorial_catalog.rs` | Ranked search, highlighting, prominent placement |
| 6 (Pagination) | `src/tutorial_catalog.rs` | Page controls, URL state, view modes |
| 7 (Curated) | `src/tutorial_catalog.rs`, `src/curated_blocks.rs` | Getting Started, Workflows blocks |
| 8 (Hubs) | `src/main.rs`, `src/family_hub.rs` | Generate `/tutorials/<family>/` pages |
| 9 (Perf) | `src/tutorial_catalog.rs` | Deferred loading, minimal hydration |
| 10 (A11y) | `src/tutorial_catalog.rs` | ARIA, keyboard nav, skip links |
| 11 (Tests) | `src/bin/validate-json.rs` | Schema checks, slug uniqueness |

---

## Implementation Checklist

### Phase 0: Lock the Contract (Schema)

- [ ] **0.1 Update frontmatter in all 16 tutorials**
  - [ ] Add `difficulty` field
  - [ ] Add `timepoints` field
  - [ ] Convert `engine` → `engines` array
  - [ ] Add explicit `summary` field

- [ ] **0.2 Update parser to handle new fields**
  - [ ] `crates/parser/src/frontmatter.rs`
  - [ ] Add fallback logic (engine → engines, description → summary)

- [ ] **0.3 Add build-time validation warnings**
  - [ ] Warn on missing new fields
  - [ ] Validate enum values
  - [ ] Check slug uniqueness

---

### Phase 1: Index Pipeline

- [ ] **1.1 Create `src/index_generator.rs`**
  - [ ] `TutorialIndexEntry` struct
  - [ ] `FamilyEntry` struct
  - [ ] `generate_tutorial_index()` function
  - [ ] `generate_family_index()` function
  - [ ] Precompute `search_text` field

- [ ] **1.2 Update `src/main.rs` to generate JSON**
  - [ ] Write `dist/api/tutorial_index.json`
  - [ ] Write `dist/api/tutorial_families.json`
  - [ ] Write `dist/api/tutorial_curations.json`

- [ ] **1.3 Create `content/tutorial_curations.yaml`**
  - [ ] Define getting_started list
  - [ ] Define workflow categories
  - [ ] Add to build pipeline

---

### Phase 2: URL Structure

- [ ] **2.1 Change tutorial output path**
  - [ ] `/tutorials/<family>/<slug>/` structure
  - [ ] Update `src/main.rs` generation loop

- [ ] **2.2 Generate redirect pages**
  - [ ] Create `/posts/<slug>/index.html` redirects
  - [ ] Include canonical tag
  - [ ] Include meta refresh

- [ ] **2.3 Update all internal links**
  - [ ] `TutorialCard` href
  - [ ] Any navigation components
  - [ ] Curated block links

---

### Phase 3-11: (See original checklist above)

---

## Curated Content Proposal

Based on existing 16 tutorials:

### Getting Started (6 tutorials, progression-friendly)

| Order | Slug | Rationale |
|-------|------|-----------|
| 1 | `lmm-random-intercept` | Foundation concept, simplest mixed model |
| 2 | `lmm-random-slopes` | Natural next step from random intercept |
| 3 | `lgcm-basic` | Introduces growth modeling paradigm |
| 4 | `gee` | Alternative to mixed models, different assumptions |
| 5 | `glmm` | Extends LMM to non-continuous outcomes |
| 6 | `residualized-change-score` | Simple change-score approach |

### Common Workflows

**Time-Varying Effects:**
- `lmm-time-invariant-covariates`
- `lgcm-time-invariant-covariates`
- `gee-time-varying-covariate`

**Complex Designs:**
- `lgcm-multiple-groups`
- `lgcm-nesting`
- `glmm-interactions`

**Growth & Change:**
- `lgcm-basic`
- `mlgcm`
- `difference-score_paired-ttest`
- `difference-score_simple-regression`

**Non-Continuous Outcomes:**
- `glmm`
- `glmm-interactions`

---

## Configuration Decisions

| Setting | Value | Notes |
|---------|-------|-------|
| Pagination | 24 per page | Configurable |
| Default view | Detailed Cards | |
| Search ranking | title > tags > summary | |
| Facet strategy | Show counts; disable 0-count | |
| URL state | Query params | Shareable |
| Redirects | Permanent (no sunset) | |
| Schema enforcement | Phased (warnings → errors) | |

---

## Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 0 | Schema migration (frontmatter fields) | ✅ Complete |
| Phase 1 | Index pipeline (JSON artifacts) | ✅ Complete |
| Phase 2 | URL restructure + redirects | ✅ Complete |
| Phase 3 | Client-side fetch + catalog UI | ✅ Complete |
| Phase 4+ | Additional UI enhancements | Future |

## Completed Features

- [x] Tutorial frontmatter extended (difficulty, timepoints, engines[], summary)
- [x] JSON index generation (`/api/tutorial_index.json`, `tutorial_families.json`, `tutorial_curations.json`)
- [x] URL restructure: `/tutorials/<family>/<slug>/`
- [x] 301 redirects from `/posts/<slug>/`
- [x] Canonical tags on tutorial pages
- [x] Client-side fetch mode (Landing/Browse modes)
- [x] Curated landing page (Featured, Workflows, Browse by Method, Recently Updated)
- [x] Pagination, ranked search, filter chips
- [x] ViewToggle (Cards/Table views)
- [x] Sidebar filters in Landing mode

## Next Steps (Future Phases)

- URL state persistence (`?family=lgcm&page=2`)
- localStorage caching for fetched index
- Method family hub pages (`/tutorials/<family>/`)

---

## Sign-off

- [x] URL strategy confirmed (permanent redirects)
- [x] Schema migration approach confirmed (phased)
- [x] Architecture approach confirmed (two-step)
- [x] Curated content ownership confirmed (implementer proposes, editable file)
- [x] Phases 0-3 implementation complete

---

*Document version: 3.0*
*Last updated: 2025-01-05*
