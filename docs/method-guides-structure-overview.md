# Method Guides: Current Structure Overview

## Purpose of This Document

This document summarizes how our longitudinal data analysis method guides are currently structured. The goal is to facilitate consultation on potential restructuring to improve navigation, accessibility, and user experience—particularly for users at different skill levels.

---

## What Are Method Guides?

Method guides are comprehensive tutorials for statistical methods used in longitudinal research (e.g., Latent Growth Curve Models, Linear Mixed Models). They serve researchers and students learning to apply these methods in R.

**Current examples:**
- `/guides/lgcm/` — Latent Growth Curve Models (~2,260 lines)
- `/guides/lmm/` — Linear Mixed Models (~2,080 lines)

---

## Current Structure

Each guide is a **single long-form markdown page** with hierarchical sections:

```
## Overview
   ├── Why Study [Method]?
   ├── What [Method] Provides
   ├── When [Method] is Appropriate
   ├── What You'll Learn
   └── How This Tutorial is Organized

## Conceptual Foundations
   ├── Two Ways to Understand [Method]
   ├── Side-by-Side Comparison (SEM vs. Multilevel framing)
   ├── Key Components
   ├── Path Diagram / Visual Representation
   └── Interactive Exploration (link to external HTML)

## Model Specification & Fit
   ├── Data Requirements
   ├── Time Coding
   ├── Basic Syntax
   ├── Fit Indices
   └── Troubleshooting

## Worked Example (collapsible <details>)
   ├── Setup
   ├── Simulate Data
   ├── Visualize
   ├── Fit Models
   ├── Compare Models
   └── Interpret Results

## Reference & Resources (collapsible <details>)
   ├── Cheat Sheet / Quick Syntax
   ├── Common Pitfalls Table
   ├── Key References
   └── Related Guides
```

---

## Content Types Currently Mixed Together

The guides blend multiple documentation types on a single page:

| Content Type | Purpose | Example |
|--------------|---------|---------|
| **Conceptual explanation** | Build intuition, explain "why" | Two framings of LGCM (SEM vs. multilevel) |
| **Tutorial/walkthrough** | Step-by-step learning | Worked Example section |
| **Reference material** | Quick lookup | Fit index thresholds, syntax patterns |
| **Troubleshooting/how-to** | Solve specific problems | Common pitfalls, lavaan error fixes |
| **Decision support** | Help users choose | "When to use" tables, method comparisons |

---

## Current UI/Navigation Elements

- **Left sidebar**: Hierarchical outline (H2/H3/H4 headings) with sticky positioning
- **Collapsible sections**: `<details>` elements for Worked Example and Reference sections
- **Callout boxes**: Styled blockquotes for tips, warnings, notes, pitfalls
- **Code blocks**: Syntax-highlighted R code with copy buttons
- **Tables**: Decision matrices, parameter interpretation guides
- **Figures**: Static images (PNG/SVG) for path diagrams, spaghetti plots
- **Interactive elements**: Links to external HTML files (trajectory explorers, model comparisons)

---

## Identified Pain Points

1. **Density**: ~2,200 lines of content on a single page is overwhelming, especially for beginners

2. **Mixed purposes**: Users doing quick syntax lookup must scroll past conceptual content; learners encounter reference material before they're ready

3. **Navigation friction**: Despite the sidebar outline, finding specific information requires scanning or knowing section names

4. **Interactivity is external**: Interactive explorers open in separate browser tabs rather than being embedded in context

5. **One-size-fits-all**: No differentiation between beginner learning path and experienced practitioner lookup

6. **Linear assumption**: Structure assumes sequential reading, but many users jump directly to specific sections

---

## User Types to Consider

| User Type | Goal | Current Experience |
|-----------|------|-------------------|
| **Beginner learner** | Understand method from scratch | Overwhelmed by density; unclear where to start |
| **Intermediate user** | Apply method to their data | Needs to find specific syntax/requirements |
| **Experienced practitioner** | Quick reference lookup | Must scroll past explanations to find syntax |
| **Troubleshooter** | Fix a specific error | Pitfalls section is buried; hard to locate |

---

## Technical Context

- **Framework**: Leptos (Rust-based, SSG output)
- **Content format**: Markdown with YAML frontmatter
- **Rendering**: Custom markdown pipeline with callout transformation, syntax highlighting, heading extraction for outline
- **Current capabilities**: `<details>` collapsibles, tables, code blocks, images, iframes (for embedded interactives)
- **Possible additions**: Tabs, accordions, multi-page routing (all technically feasible)

---

## Options Under Consideration

### A. Multi-Page Structure
Split each method into separate pages: `/guides/lgcm/concepts/`, `/guides/lgcm/tutorial/`, `/guides/lgcm/reference/`

### B. Single Page with Tabs
Keep single URL but use tabs to separate content types (Concepts | Tutorial | Reference | FAQ)

### C. Collapsed Modules with Quick Reference
Single page with most content collapsed by default; surface most-needed content (syntax, fit indices) without requiring expansion

---

## Precedents to Investigate

Please review these documentation examples and frameworks for relevant patterns:

### Frameworks & Principles

| Resource | Why It's Relevant |
|----------|-------------------|
| [Diátaxis Framework](https://diataxis.fr/) | Systematic approach dividing docs into 4 types: Tutorials, How-To Guides, Reference, Explanation. Argues these should NOT be mixed. |
| [Progressive Disclosure - IxDF](https://www.interaction-design.org/literature/topics/progressive-disclosure) | UX pattern for managing complexity—show essentials first, reveal details on demand. |
| [Documentation Structure Best Practices - GitBook](https://docs.gitbook.com/guides/best-practices/documentation-structure-tips) | Practical guidance on organizing technical documentation. |

### Documentation Examples to Study

| Site | Pattern Worth Noting |
|------|---------------------|
| [Tailwind CSS Docs](https://tailwindcss.com/docs) | Excellent sidebar navigation + content chunking. Dense reference material made scannable. |
| [Stripe API Docs](https://stripe.com/docs/api) | Reference with expandable details. Code examples alongside explanations. |
| [React Docs](https://react.dev/learn) | Clear separation of "Learn" (tutorials) vs "Reference" (API docs). Different navigation for each. |
| [dplyr (tidyverse)](https://dplyr.tidyverse.org/) | R package docs with "Articles" (narrative tutorials) split from "Reference" (function docs). |

### What to Look For

When reviewing these precedents, consider:
- How do they separate learning content from reference content?
- What's visible by default vs. requires interaction to reveal?
- How does the sidebar/navigation adapt to content type?
- How do they handle long-form content (chunking, collapsing, pagination)?
- What patterns might translate to statistical method tutorials?

---

## Questions for Consultation

1. Given the content types and user types, what information architecture would best serve both learning and reference use cases?

2. How should we balance discoverability (seeing what's available) vs. focus (not overwhelming users)?

3. Based on the precedents above, which patterns seem most applicable to statistical method tutorials?

4. What's the right level of progressive disclosure—how much should be visible by default vs. hidden?

5. Should the structure differ by user skill level (beginner vs. advanced paths), or is a single flexible structure preferable?

6. How might the Diátaxis framework's four documentation types map to our current content?

---

## Reference Links

- Current LGCM guide: `/guides/lgcm/`
- Current LMM guide: `/guides/lmm/`
- Distill-inspired prototype: `/guides/lgcm-newtest-v1/` (demonstrates narrative approach with inline interactivity)
