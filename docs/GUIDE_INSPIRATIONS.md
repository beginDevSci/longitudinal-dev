# Method Guides: Design Inspirations

This document captures the conceptual lineage and stylistic rationale behind the Guide system. It helps future contributors understand *why* the design is the way it is, ensuring consistency as new methods are added.

---

## Design Philosophy

The Method Guides system balances three goals:

1. **Pedagogical clarity** - Concepts before code, intuition before formulas
2. **Practical utility** - Runnable examples, copy-paste syntax, real diagnostics
3. **Structural consistency** - Predictable 6-section format across all methods

---

## Primary Inspirations

### 1. RStudio's Tidymodels & Vignettes

The structural clarity and pedagogical tone draw heavily from:

- R package vignettes (esp. `lme4`, `lavaan`, `brms`)
- Tidyverse-style documentation
- Stepwise explanation of models with runnable examples

**Influence:**
- Clear, approachable, example-driven structure
- Emphasis on showing code in context
- Consistent formatting across related packages

### 2. Applied Statistics Textbooks

Foundational influences from:

- **Singer & Willett (2003)** - *Applied Longitudinal Data Analysis*
- **Bollen & Curran (2006)** - *Latent Curve Models*
- **Raudenbush & Bryk (2002)** - *Hierarchical Linear Models*

**Influence:**
- Standardized sections (overview → theory → fit → interpretation)
- Emphasis on intuition + principled explanation
- Careful attention to assumptions and diagnostics
- Mathematical notation conventions

### 3. Statistical Computing Tutorials

Borrowed ideas from:

- **scikit-learn documentation** - Collapsible examples, simple workflows
- **Stan and brms documentation** - Concept first, code second
- **PyTorch & TensorFlow guides** - Strong consistency, modularity

**Influence:**
- Reusable 6-section structure
- Collapsible examples for long-form content
- Quick-reference cheat sheets
- Common pitfalls sections

### 4. Modern Long-Form Educational UX

Sources include:

- **Notion** - Collapsible blocks, clean typography
- **Distill.pub** - Academic rigor with web-native presentation
- **ObservableHQ** - Interactive exploration, progressive disclosure

**Influence:**
- Clean content spacing
- Scannability through consistent heading hierarchy
- Interactive/expandable sections for optional depth
- Callouts for emphasis without disrupting flow

### 5. ABCD Tutorial Lineage

The guide system preserves much of the tone and pedagogical clarity established in the earlier Nuxt-based ABCD tutorials:

- Structured walkthrough format
- Emphasis on conceptual grounding
- Friendly, motivating language
- Clear code blocks with narrative context

**Influence:**
- Ensures continuity with existing ABCD-focused materials
- Familiar structure for returning users
- Validated teaching approach from prior work

---

## The 6-Section Structure

Each guide follows a fixed structure:

| Section | Purpose |
|---------|---------|
| **Overview** | What problem does this solve? When to use it? |
| **Conceptual Foundations** | Theory, equations, intuition |
| **Model Specification & Fit** | Syntax, assumptions, diagnostics |
| **Interpretation** | Reading output, effect sizes, visualization |
| **Worked Example** | Complete runnable code (collapsible) |
| **Reference & Resources** | Cheat sheet, pitfalls, citations (collapsible) |

This structure was chosen because:

1. **Mirrors learning progression** - Context → Theory → Practice → Application
2. **Supports skimming** - Experts can jump to Worked Example directly
3. **Enables comparison** - Same sections across methods aids cross-method learning
4. **Scales to complexity** - Works for simple and advanced methods alike

---

## Visual & Interaction Design Choices

### Collapsible Sections

"Worked Example" and "Reference & Resources" are collapsed by default:

- Reduces cognitive load for readers focused on concepts
- Full code remains accessible but doesn't dominate the page
- Print/export can expand all sections automatically

### Callout Types

| Type | Color | Use Case |
|------|-------|----------|
| `[!tip]` | Green | Best practices, recommendations |
| `[!note]` | Blue | Additional context, clarifications |
| `[!warning]` | Amber | Cautions, things to watch for |
| `[!important]` | Amber | Key points, must-know information |
| `[!caution]` | Red | Common pitfalls, errors to avoid |

Callouts are used sparingly to maintain impact.

### Tables

- Wrapped for horizontal scrolling on mobile
- Used for structured comparisons (fit indices, pitfalls, requirements)
- Kept compact—no decorative tables

### Code Blocks

- R syntax highlighting
- Copy button for quick extraction
- Minimal comments—narrative provides context

### Math Rendering

- Server-side KaTeX for fast, consistent rendering
- Display equations centered and separated
- Inline math for parameter references in prose

---

## Technical Implementation Notes

### Markdown Pipeline

Guides use a custom AST transformation layer:

```
content/guides/*.md
       ↓
  preprocess_inline_math()  ← Handles \(...\) before markdown
       ↓
  pulldown-cmark parser
       ↓
  transform_markdown_events()
    - callouts.rs  ← [!tip] → styled divs
    - tables.rs    ← responsive wrappers
    - modules.rs   ← H2 → collapsible details
    - math.rs      ← $$ → KaTeX HTML
       ↓
  html::push_html()
       ↓
  Static HTML in dist/
```

### Authoring Requirements

Math syntax must account for markdown's underscore handling:

```markdown
<!-- Inline math with subscripts -->
\(y_{it}\) for person \(i\)     ✅ Works
$y_{it}$                        ❌ Underscores break

<!-- Display math -->
$$
y_{it} = \eta_{0i} + \epsilon_{it}
$$                              ✅ Works (on separate lines)
```

---

## What This System Achieves

- **Structural uniformity** across all method guides
- **Pipeline compatibility** with markdown → Leptos SSG
- **Future-proof foundation** for adding new methods (GEE, GLMM, DTSA, LCGA, etc.)
- **Contributor clarity** on design rationale and conventions
- **Continuity** with established ABCD tutorial materials

---

## Future Considerations

Potential enhancements documented in `GUIDES_V1_1_TODO.md`:

- TOC sidebar for long guides
- Heading anchor links for sharing
- "Copy all code" button for Worked Examples
- Print stylesheet improvements
- Cross-links between related guides
- Difficulty badges on index cards

---

*Last updated: December 2024*
