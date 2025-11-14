# Longitudinal Writer Package

The Writer package powers the tutorial editor domain models that feed both the SSG and the
validator pipeline. It currently provides type-safe models, export helpers, and validation
utilities; UI integration remains future work.

## What Exists

- Domain models (`src/domain.rs`) mirroring the six required tutorial sections.
- Markdown exporter (`src/export.rs`) that emits parser-compliant content (H1 sections,
  correct frontmatter field names, v2 block markers).
- Validation helpers (`src/validation.rs`) that ensure tutorials are export-ready.
- Reactive state management utilities (`src/state.rs`) for eventual Leptos UI bindings.
- Parser integration tests confirming exported markdown can be consumed by `md2json`.

## Usage (Library Mode)

```rust
use longitudinal_writer::*;

let mut tutorial = Tutorial::new();
tutorial.title = "My Tutorial".into();
tutorial.metadata.family = Some("LGCM".into());

if let Some(section) = tutorial.sections.iter_mut().find(|s| s.section_type == SectionType::Overview) {
    section.blocks.push(Block::Paragraph { content: "Intro text".into() });
}

let issues = TutorialValidator::validate(&tutorial);
if TutorialValidator::can_export(&tutorial) {
    let markdown = MarkdownExporter::export(&tutorial)?;
    std::fs::write("tutorial.md", markdown)?;
}
```

## Roadmap Snapshot

- UI surfaces (Leptos components) live outside this crate and are still pending.
- Block editors for v2 flexible sections will build on the existing domain model.
- Autosave, draft management, and live validation will sit on top of the provided state
  helpers once the UI layer is resumed.

**Last Updated:** 2025-10-31 — package maintained as a reusable library while UI work is
paused.
