# Markdown Pipeline Architecture

This document describes the markdown processing pipelines used in the longitudinal-dev site, identifies the transformation points for Phase 2 enhancements, and provides guidance for implementing AST-based transformations.

## Executive Summary

**Key Finding:** The site uses **two completely different content pipelines**:

| Route | Source Format | Processing | Rendering |
|-------|---------------|------------|-----------|
| `/tutorials` | JSON (`.post.json`) | Typed models via serde | Leptos components |
| `/guides` | Markdown (`.md`) | pulldown-cmark | `inner_html` injection |

**Phase 2 work focuses exclusively on the `/guides` pipeline**, specifically `src/guides.rs::render_markdown_to_html()`.

---

## Pipeline 1: Tutorials (`/tutorials`)

### Overview

Tutorials (ABCD Examples) do NOT use markdown-to-HTML conversion at render time. They use a structured JSON → typed model → Leptos components pipeline.

### Flow Diagram

```
content/posts/*.post.json
        │
        ▼
  build.rs (generates manifest.rs)
        │
        ▼
  src/generated/manifest.rs
        │
        ▼
  load_posts_raw() → Vec<JsonPost>
        │
        ▼
  JsonPost::into_post() → Post (typed model)
        │
        ▼
  PostLayout component → Leptos view macros
        │
        ▼
  .to_html() → Static HTML
```

### Key Files

| File | Purpose |
|------|---------|
| `content/posts/*.post.json` | Source content (JSON) |
| `build.rs` | Generates manifest at compile time |
| `src/generated/manifest.rs` | Auto-generated loader |
| `src/models/json_dto.rs` | DTO for JSON deserialization |
| `src/models/conversion.rs` | DTO → Post conversion |
| `src/models/post.rs` | Typed Post model |
| `src/layout/post_layout.rs` | Leptos rendering component |

### Markdown Files in `content/tutorials/`

The `.md` files in `content/tutorials/` are **not** used for rendering. They exist for:
1. **Editor prefill**: The suggestion/edit modal loads these for user editing
2. **Baseline hashing**: SHA-256 hash for detecting content changes
3. **Human-readable source**: Easier authoring before JSON conversion

This separation means tutorials have rich, type-safe rendering with custom Leptos components for each section type.

### Implications for Phase 2

**No changes needed to the tutorial pipeline.** Tutorials already have:
- Structured sections (no AST parsing needed)
- Syntax highlighting via `syntect` (in `src/syntax_highlight.rs`)
- Custom rendering per content block type

---

## Pipeline 2: Guides (`/guides`)

### Overview

Guides use a traditional markdown → HTML pipeline via `pulldown-cmark`. This is where Phase 2 transformations will be inserted.

### Flow Diagram

```
content/guides/*.md
        │
        ▼
  fs::read_to_string()
        │
        ▼
  parse_frontmatter() → (GuideFrontmatter, markdown_body)
        │
        ▼
  render_markdown_to_html(markdown_body) ◀── INSERTION POINT
        │
        ▼
  Guide { ..., html_content }
        │
        ▼
  GuideLayout component (inner_html=html_content)
        │
        ▼
  .to_html() → Static HTML
```

### Key Files

| File | Purpose |
|------|---------|
| `content/guides/*.md` | Source content (Markdown + YAML frontmatter) |
| `src/guides.rs` | Loading, parsing, and rendering |
| `src/models/guide.rs` | Guide model and frontmatter struct |
| `src/layout/guide_layout.rs` | Leptos rendering component |

### Current `render_markdown_to_html()` Implementation

Location: `src/guides.rs:36-48`

```rust
fn render_markdown_to_html(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(content, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}
```

**Current pulldown-cmark options:**
- `ENABLE_TABLES` ✓
- `ENABLE_STRIKETHROUGH` ✓
- `ENABLE_HEADING_ATTRIBUTES` ✓

**Missing options that may be needed:**
- `ENABLE_FOOTNOTES` - For academic content
- `ENABLE_TASKLISTS` - For checklists
- `ENABLE_SMART_PUNCTUATION` - For typography

---

## Phase 2: AST Transformation Layer

### Recommended Insertion Point

The transformation layer should be inserted **between parsing and HTML rendering** in `src/guides.rs`:

```rust
fn render_markdown_to_html(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(content, options);

    // ══════════════════════════════════════════════════════════
    // PHASE 2 INSERTION POINT: AST Transformations
    // ══════════════════════════════════════════════════════════
    let events: Vec<Event> = parser.collect();
    let transformed = transform_markdown_events(events);
    // ══════════════════════════════════════════════════════════

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed.into_iter());

    html_output
}
```

### Why AST-Based (Not Regex)

1. **Safety**: Won't accidentally transform code inside ` ``` ` blocks
2. **Context-awareness**: Can detect "inside blockquote" vs "top-level"
3. **Composability**: Multiple transforms can be chained cleanly
4. **Robustness**: pulldown-cmark already handles edge cases

### Transformation Functions to Implement

#### 1. Callout Detection

Detect GitHub-style callouts in blockquotes:

```markdown
> [!tip]
> Content here

> [!warning]
> Content here

> **Note:** Content here
```

**Implementation approach:**
- Look for `Event::Start(Tag::BlockQuote)` followed by `Event::Text` matching patterns
- Replace with custom HTML structure with appropriate classes

#### 2. Module Wrapping

Detect H2 headings for collapsible modules:

```markdown
## Worked Example
...content...

## Reference & Resources
...content...
```

**Implementation approach:**
- Track H2 headings matching `Worked Example` or `Reference & Resources`
- Wrap content until next H2 (or EOF) in `<details>` / collapsible div

#### 3. Math Rendering

Detect LaTeX delimiters and render via KaTeX:

```markdown
Inline: $y_{it}$
Display: $$y = mx + b$$
```

**Implementation approach:**
- Scan text events for `$...$` and `$$...$$` patterns
- Replace with KaTeX-rendered HTML (server-side)
- Consider adding `katex` crate dependency

#### 4. Table Wrapping

Wrap tables for responsive scrolling:

**Implementation approach:**
- Detect `Event::Start(Tag::Table(_))`
- Wrap in `<div class="table-wrapper overflow-x-auto">...</div>`

#### 5. Code Block Enhancement

Add language classes and prepare for copy buttons:

**Implementation approach:**
- Detect `Event::Start(Tag::CodeBlock(kind))`
- Add wrapper div with language indicator
- Optionally integrate with existing `syntax_highlight.rs`

---

## Proposed Module Structure

Create a new module for AST transformations:

```
src/
├── guides.rs              # Existing: loading and main render fn
├── markdown/              # NEW: transformation module
│   ├── mod.rs             # Re-exports
│   ├── transform.rs       # Main transform pipeline
│   ├── callouts.rs        # Callout detection & rendering
│   ├── modules.rs         # Collapsible module wrapping
│   ├── math.rs            # KaTeX integration
│   └── tables.rs          # Table wrapping
```

### Shared Transform API

```rust
// src/markdown/transform.rs

use pulldown_cmark::Event;

/// Transform a stream of pulldown-cmark events.
pub fn transform_markdown_events(events: Vec<Event>) -> Vec<Event> {
    let events = transform_callouts(events);
    let events = transform_modules(events);
    let events = transform_math(events);
    let events = wrap_tables(events);
    events
}
```

---

## Constraints and Considerations

### 1. SSG Context

All transformations happen at **build time** during SSG generation. There's no runtime parsing. This means:
- Performance is less critical (one-time cost)
- Can use heavier dependencies if needed
- Output must be valid static HTML

### 2. Styling Coordination

Transformations produce HTML with specific class names. These must be coordinated with:
- `tailwind.config.js` for utility classes
- Custom CSS for callout/module styling
- Dark mode variants

### 3. Client-Side Behavior

Some features need JS hydration:
- Collapsible modules (toggle state)
- Copy code buttons (clipboard API)
- Scrollspy for TOC

These should be implemented as Leptos islands or minimal vanilla JS.

### 4. Backward Compatibility

The `/guides` pipeline is new, so there's no legacy content to migrate. However, the callout syntax should support multiple formats:
- `> [!tip]` (GitHub-style)
- `> **Tip:**` (bold-prefix style)

---

## Test Content

The `content/guides/lgcm.md` file now includes test cases for all Phase 2 features:

| Feature | Test Content |
|---------|--------------|
| Callouts (tip) | `> [!tip]` in Overview section |
| Callouts (warning) | `> [!warning]` in Conceptual Foundations |
| Callouts (important) | `> [!important]` in Model Specification |
| Callouts (caution) | `> [!caution]` in Reference section |
| Callouts (bold) | `> **Note:**` in Overview section |
| Inline math | `$y_{it}$`, `$\eta_{0i}$` |
| Display math | `$$y_{it} = \eta_{0i} + ...$$` |
| Tables | Multiple tables throughout |
| Code blocks | R code with language fence |
| H2 sections | `## Worked Example`, `## Reference & Resources` |

---

## Phase 2 Implementation (Completed)

The markdown transformation pipeline has been implemented in `src/markdown/`. Here's what was built:

### Module Structure

```
src/markdown/
├── mod.rs          # Re-exports transform_markdown_events
├── transform.rs    # Main pipeline orchestration
├── callouts.rs     # Blockquote → callout transformation
├── tables.rs       # Table wrapper injection
├── modules.rs      # H2 section → collapsible <details>
└── math.rs         # LaTeX → KaTeX HTML rendering
```

### Transformation Order

```rust
pub fn transform_markdown_events(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let events = callouts::transform_callouts(events);
    let events = tables::wrap_tables(events);
    let events = modules::wrap_modules(events);
    let events = math::render_math(events);
    events
}
```

### Callouts (`callouts.rs`)

**Supported syntax:**
- GitHub-style: `> [!tip]`, `> [!warning]`, `> [!important]`, `> [!caution]`, `> [!note]`, `> [!info]`, `> [!didactic]`
- Bold-prefix: `> **Warning:** content`

**Type mapping:**
| Marker | CSS Class |
|--------|-----------|
| `[!tip]` | `callout-tip` |
| `[!warning]`, `[!important]` | `callout-warning` |
| `[!note]` | `callout-note` |
| `[!pitfall]`, `[!caution]` | `callout-pitfall` |
| `[!info]` | `callout-info` |
| `[!didactic]` | `callout-didactic` |

**Output structure:**
```html
<div class="callout callout-tip" role="note" aria-label="Tip">
  <div class="callout-title"><span>Tip</span></div>
  <div class="callout-body">
    <!-- callout content -->
  </div>
</div>
```

**Edge case handled:** pulldown-cmark splits `[!tip]` into multiple text events (`"["`, `"!tip"`, `"]"`). The parser concatenates early text to detect the pattern.

### Tables (`tables.rs`)

**Output structure:**
```html
<div class="table-wrapper">
  <div class="table-container">
    <table>...</table>
  </div>
</div>
```

### Modules (`modules.rs`)

**Detected H2 headings (case-insensitive):**
- "Worked Example"
- "Reference & Resources" / "Reference and Resources"
- "References & Resources" / "References and Resources"

**Output structure:**
```html
<details class="tutorial-module">
  <summary>Worked Example</summary>
  <div class="module-content">
    <!-- section content until next H2 -->
  </div>
</details>
```

### Math (`math.rs`)

**Supported delimiters:**
- Inline: `$...$` and `\(...\)`
- Display: `$$...$$` and `\[...\]`

**Dependencies:**
- `katex = "0.4"` (uses embedded QuickJS, no Node.js required)
- Only enabled for `ssr` feature

**Output structure:**
```html
<span class="math-inline"><span class="katex">...</span></span>
<div class="math-display"><span class="katex">...</span></div>
```

**Safety:**
- Math detection skips code blocks and inline code
- Inline math does not span across newlines

**Known limitations:**
- Error handling shows LaTeX source in `<code>` on KaTeX parse failure

### Authoring Requirements for Math

**Inline Math: Use `\(...\)` Delimiters**

For inline math with subscripts, use LaTeX parenthesis delimiters `\(...\)` instead of `$...$`:

```markdown
<!-- ❌ WRONG - $...$ with underscores gets corrupted -->
$y_{it}$ for person $i$

<!-- ✅ CORRECT - use \(...\) for inline math with subscripts -->
\(y_{it}\) for person \(i\)
```

**Why?** The `$...$` syntax conflicts with markdown's underscore emphasis parsing. The `\(...\)` delimiters are pre-processed before markdown runs, preventing this issue.

**Common patterns:**
- `\(y_{ij}\)` (subscripts)
- `\(\beta_0\)` (Greek with subscript)
- `\(\eta_{0i}\)` (nested subscripts)
- `\(\lambda_t\)` (Greek with subscript)
- `\(\epsilon_{it}\)` (error terms)

**Display math** (`$$...$$` on separate lines) works correctly without special handling:

```markdown
$$
y_{it} = \eta_{0i} + \eta_{1i} \cdot \lambda_t + \epsilon_{it}
$$
```

**Simple inline math** without underscores can still use `$...$`:
- `$x$`, `$i$`, `$t$` - all work fine with `$...$`

### Integration Point

The pipeline is wired into `src/guides.rs::render_markdown_to_html()`:

```rust
fn render_markdown_to_html(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(content, options);
    let events: Vec<Event> = parser.collect();
    let transformed = transform_markdown_events(events);

    let mut html_output = String::new();
    html::push_html(&mut html_output, transformed.into_iter());
    html_output
}
```

---

## Next Steps (Post-Phase 2)

1. ~~Create `src/markdown/` module~~ ✅
2. ~~Implement callout detection~~ ✅
3. ~~Add table wrapping~~ ✅
4. ~~Implement module wrapping~~ ✅
5. ~~Add math rendering~~ ✅
6. **Coordinate styling** with Tailwind/CSS - Add CSS for callout colors, module toggles, math fonts
7. **Add client-side behavior** - Collapsible toggle JS, auto-expand on anchor navigation, copy buttons

---

## Appendix: pulldown-cmark Event Types

Common events you'll work with:

```rust
enum Event<'a> {
    Start(Tag<'a>),           // Opening tag
    End(TagEnd),              // Closing tag
    Text(CowStr<'a>),         // Text content
    Code(CowStr<'a>),         // Inline code
    Html(CowStr<'a>),         // Raw HTML
    SoftBreak,                // Soft line break
    HardBreak,                // Hard line break
    Rule,                     // Horizontal rule
    // ... others
}

enum Tag<'a> {
    Paragraph,
    Heading { level, .. },
    BlockQuote,
    CodeBlock(CodeBlockKind<'a>),
    List(Option<u64>),
    Item,
    Table(Vec<Alignment>),
    TableHead,
    TableRow,
    TableCell,
    Emphasis,
    Strong,
    Link { .. },
    Image { .. },
    // ... others
}
```

See [pulldown-cmark docs](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/) for complete reference.
