# Brain Viewer UI Polish Checklist

> **Branch:** `feature/viewer-structure`
> **Created:** 2025-01-09
> **Status:** Ready to Begin Implementation
> **Prior Context:** See `viewer-redesign-plan.md` for Phase 1-5 layout restructure (completed)

---

## Overview

This checklist tracks UI polish refinements to the brain viewer, focusing on:
- Analysis Tools panel restructure and typography
- ROI controls layout and table styling
- Canvas-legend visual hierarchy and cohesion
- Brush slider pattern consistency

These changes build on the completed 2-column layout work and address remaining "visual hierarchy, spacing, and content grouping" issues identified in expert review.

---

## Key File Locations

| Component | Path | Purpose |
|-----------|------|---------|
| ROI Panel | `blmm_demo/crates/viewer_app/src/components/roi_panel.rs` | ROI tools, brush slider, buttons |
| Vertex Table | `blmm_demo/crates/viewer_app/src/components/vertex_summary_table.rs` | Top 10 vertices table |
| Control Panel | `blmm_demo/crates/viewer_app/src/components/control_panel.rs` | Hemisphere, threshold, colormap controls |
| Color Legend | `blmm_demo/crates/viewer_app/src/components/color_legend.rs` | Legend strip with expandable details |
| Brain Viewer | `blmm_demo/crates/viewer_app/src/components/brain_viewer.rs` | Main layout container |
| Collapsible | `blmm_demo/crates/viewer_app/src/components/collapsible_section.rs` | Reusable collapse component |
| Viewer CSS | `style/input.css` | Lines ~4172-4970 contain viewer styles |

**Note:** `blmm_demo/` is a symlink to `/Users/shawes/Desktop/blmm_demo`

---

## Build & Test Workflow

```bash
# Build and serve with interactive viewer
make serve

# This runs: WEBGPU_VIEWER=1 cargo xtask build-ssg + python3 http.server
# Site available at: http://localhost:8000

# Navigate to a tutorial page with the viewer to test
```

---

## Process Guidelines

### Checkpoints
- **Commit and tag** after completing Phases 2, 4, and 7
- **Visual validation** after Phases 1-2, 3-4, 5-7
- **Mobile testing** at each phase (especially button wrapping, touch targets, table readability)

### Scope Boundaries
- ✅ Viewer components (`blmm_demo/crates/viewer_app/src/components/*`)
- ✅ Viewer CSS (`style/input.css` viewer sections)
- ❌ Site-level layout, sidebars, page asides
- ❌ Global typography tokens or layout shells

### Class Naming
- Reuse existing viewer classes where possible
- New classes: prefix with `.brain-viewer-*`, `.analysis-tools-*`, `.roi-*`

### Guiding Principle
> If the plan conflicts with real code, favor maintaining UX intent (hierarchy, grouping, readability) and note any deviations.

---

## Implementation Phases

### Phase 0 – Setup & Verification
**Effort:** Low | **Status:** ✅ Complete

- [x] Verify build works: `make serve`
- [x] Confirm viewer loads at localhost:8000
- [x] Take baseline screenshots (Simple Mode, Advanced Mode expanded, ROI table populated) — User provided
- [x] Create working branch or confirm on `feature/viewer-structure`

---

### Phase 1 – Analysis Tools Structure
**Effort:** Medium | **Status:** ✅ Complete
**Files:** `roi_panel.rs`, `brain_viewer.rs`, `style/input.css`

#### 1.1 Identify Current Structure
- [x] Locate in `roi_panel.rs`:
  - Introductory instructions
  - "Vertex Info" section
  - "ROI Tools" section (includes Current ROI summary)
  - "Annotations" section
  - Brush size slider
  - ROI action buttons

#### 1.2 Create Section Containers
Restructure into semantic subsections (static containers, NOT nested collapsibles):

- [x] **Vertex Selection** container with icon header
- [x] **ROI Tools** container with icon header
  - Drawing mode / overlay checkboxes (inline)
  - Brush size control
  - ROI action buttons (grouped: New/Sample/Clear + Update/Compute)
  - **Current ROI summary** (subordinate within ROI Tools)
- [x] **Annotations** container with icon header
- [x] **Top Vertices** container with icon header + table

#### 1.3 CSS Visual Containment
- [x] Added `.analysis-tools-section` class with:
  - Subtle background tint (`rgba(255, 255, 255, 0.02)`)
  - Border radius matching viewer cards
  - Internal padding (12px)
  - Margin-bottom between sections (12px)
- [x] Added `.analysis-tools-section-header` for consistent headers
- [x] Added `.analysis-tools-section-body` for content areas
- [x] Added `.roi-button-group` and `.roi-btn` for consistent buttons

#### 1.4 Decisions Made
- Current ROI stays **within** ROI Tools (not a separate top-level section) ✓
- Use **static containers** inside existing `CollapsibleSection` (no nested collapsibles) ✓
- Rust structural changes in `roi_panel.rs`, `brain_viewer.rs`, `vertex_info_panel.rs`, `annotation_panel.rs`, `vertex_summary_table.rs` ✓

---

### Phase 2 – Typography & Density
**Effort:** Low-Medium | **Status:** ✅ Complete (merged with Phase 1)
**Files:** `style/input.css`

#### 2.1 Section Headers
- [x] Created `.analysis-tools-section-header` with:
  - 0.8125rem font size, 600 weight
  - `--color-text-secondary` color
  - SVG icon support (16x16)

#### 2.2 Body Text
- [x] Created `.analysis-tools-section-body` with:
  - 0.8125rem font size
  - Line-height 1.5
  - `--color-text-muted` color

#### 2.3 Spacing & Text Compression
- [x] Compressed intro text via `.analysis-tools-section-intro` (0.75rem)
- [x] Consistent vertical rhythm with spacing tokens

#### 2.4 Decisions Made
- Used existing design tokens ✓
- All rem values consistent with existing type scale ✓

---

### 🏁 Checkpoint A: After Phase 2
- [x] Changes implemented (merged with Phase 1)
- [ ] Commit changes
- [ ] Visual validation: Analysis Tools should read as structured sections, not text blob
- [ ] Test at mobile breakpoint

---

### Phase 3 – ROI Buttons & Controls Layout
**Effort:** Medium | **Status:** ✅ Complete (merged with Phase 1)
**Files:** `roi_panel.rs`, `style/input.css`

#### 3.1 Group Actions Logically
- [x] **Create/Load group:** New, Sample, Clear
- [x] **Compute group:** Update, Compute Stats (primary style)

#### 3.2 Layout Pattern
- [x] Horizontal flex layout with graceful wrap
- [x] All buttons share: same height, border-radius, font-size via `.roi-btn`
- [x] Primary style via `.roi-btn-primary` for Compute Stats

#### 3.3 Wrapping Behavior
- [x] Flexbox with `min-width: 5rem` and `flex: 1 1 auto`
- [x] Shortened labels: "New", "Sample", "Clear", "Update", "Compute Stats"

#### 3.4 Touch Targets
- [x] 44px minimum via `@media (pointer: coarse)` rule for `.roi-btn`

#### 3.5 Decisions Made
- Rust changes in `roi_panel.rs` completed ✓
- Graceful wrapping via flexbox ✓

---

### Phase 4 – ROI Table Styling
**Effort:** Low-Medium | **Status:** ✅ Complete (merged with Phase 1)
**Files:** `vertex_summary_table.rs`, `style/input.css`

#### 4.1 Color & Theme Alignment
- [x] Header uses `bg-[var(--color-accent-500)]/10` (subtle accent tint)
- [x] Header text uses `--color-text-secondary` for contrast

#### 4.2 Card Container Styling
- [x] Table now inside `.analysis-tools-section` container
- [x] Section provides consistent border-radius and padding

#### 4.3 Row Spacing & Typography
- [x] Header row: `font-semibold`, `py-1.5`
- [x] Body rows: `py-1.5`, consistent padding
- [x] Value column: `text-right font-mono tabular-nums`

#### 4.4 Placement
- [x] Table in "Top Vertices" section within Analysis Tools
- [x] Section header provides clear title
- [x] Consistent margins via section container

#### 4.5 Decisions Made
- Table stays in place within Analysis Tools flow ✓
- Removed redundant card wrapper (section provides styling) ✓

---

### 🏁 Checkpoint A+B: After Phases 1-4
- [ ] Commit changes
- [ ] Tag: `viewer-polish-phase4` (optional)
- [ ] Visual validation: ROI table should feel native to viewer
- [ ] Test at mobile breakpoint (table readability)

---

### Phase 5 – Canvas & Vertical Flow
**Effort:** Low | **Status:** ✅ Complete
**Files:** `style/input.css`

#### 5.1 Canvas Top Breathing Room
- [x] Evaluate adding 12-16px top padding inside `.brain-viewer-canvas-wrapper`
  - Already has `padding: var(--spacing-3)` (12px) at desktop - meets target
- [x] Verify it doesn't break alignment with tutorial content
- [x] If problematic, use smallest value that improves breathing room
  - No change needed; existing padding sufficient

#### 5.2 Canvas-Legend Gap
- [x] Measure current gap (currently `var(--spacing-3)` = 12px)
- [x] Target: `var(--spacing-2)` (~8px) for tighter visual connection
- [x] Goal: legend feels attached to canvas without merging
  - Updated `.brain-viewer-canvas-area` gap at mobile and desktop breakpoints

#### 5.3 Alignment Within Canvas Column
- [x] Ensure canvas and legend left edges align
  - Both are flex children of same column, naturally aligned
- [x] Consistent shadows/borders between elements
  - Canvas: elevated (box-shadow), Legend: standard border (intentional hierarchy)
- [x] **Note:** This is canvas-column alignment only, not cross-column with left panel

#### 5.4 Decisions Made
- Focus on **canvas + legend** alignment within right column ✓
- Target specific spacing token, not arbitrary delta ✓
- Canvas already had sufficient breathing room (12px) ✓

---

### Phase 6 – Legend Details Refinement
**Effort:** Low | **Status:** ✅ Complete
**Files:** `color_legend.rs`, `style/input.css`

#### 6.1 Spacing
- [x] Increase gap in `.legend-details-inner` from `--spacing-1` to `--spacing-2`
- [x] Maintain consistent vertical rhythm

#### 6.2 Micro-Headings
- [x] Add small headings for:
  - Units
  - Source
  - Tip
  - Suggested threshold
- [x] Style: slightly bolder, same label color as elsewhere
  - Labels: 600 weight, 0.75rem, uppercase, letter-spacing 0.03em
  - Uses existing `--color-text-secondary` for consistency

#### 6.3 Tone & Contrast
- [x] Mute body text relative to headings
  - Values use `--color-text-muted` at 0.8125rem
- [x] Verify WCAG AA contrast preserved
  - Using existing color tokens which are accessibility-compliant

---

### Phase 7 – Brush Slider Consistency
**Effort:** Low | **Status:** ✅ Complete
**Files:** `roi_panel.rs`, `style/input.css`

#### 7.1 Adopt Threshold Pattern
- [x] Implement slider + numeric pill pattern (same as threshold control)
- [x] Reuse existing classes: `.threshold-control`, `.threshold-slider`, `.threshold-input`

#### 7.2 Visual Alignment
- [x] Flex row or stacked pair mirroring threshold control
- [x] Tightly grouped and clearly related

#### 7.3 Two-Way Sync
- [x] Slider changes update numeric value (via `on_brush_slider_input`)
- [x] Numeric input changes update slider and brush size (via `on_brush_input_change`)

#### 7.4 Decisions Made
- Scope to `roi_panel.rs` specifically ✓
- **Reuse existing CSS pattern**, not custom one-off ✓

---

### 🏁 Checkpoint C: After Phase 7
- [ ] Commit changes
- [ ] Tag: `viewer-polish-phase7` (optional)
- [ ] Visual validation: Canvas dominant, legend connected, brush slider consistent
- [ ] Test at mobile breakpoint

---

### Phase 8 – Accessibility & Consistency Pass
**Effort:** Low | **Status:** Not Started

#### 8.1 Keyboard & Focus
- [ ] Verify keyboard navigation through all controls
- [ ] Focus rings visible and unobstructed
- [ ] Test: hemisphere, threshold, brush sliders, ROI controls, legend disclosure

#### 8.2 Contrast
- [ ] Check section headers in Analysis Tools
- [ ] Check ROI table header
- [ ] Check legend micro-headings
- [ ] Check muted body text
- [ ] Adjust if needed for WCAG AA

#### 8.3 Consistency
- [ ] All primary controls share coherent accent color
- [ ] Cards share consistent radius, shadow, padding
- [ ] Headings follow predictable type scale

---

### Phase 9 – Final QA & Screenshots
**Effort:** Low | **Status:** Not Started

#### 9.1 Capture Screenshots
- [ ] Simple Mode, Analysis Tools collapsed
- [ ] Advanced Mode, Analysis Tools expanded
- [ ] ROI table populated with data
- [ ] Legend details expanded

#### 9.2 Visual Comparison
- [ ] Canvas feels more central and visually dominant
- [ ] Analysis Tools reads as structured sections
- [ ] ROI table feels native to viewer
- [ ] Legend feels tightly connected to canvas

#### 9.3 Final Adjustments
- [ ] If any area still feels heavy/disjointed, adjust spacing/typography
- [ ] Stay within established structures (no new components)

---

## Session Handoff Notes

### If Resuming This Work

1. Check current branch: `git branch --show-current` (should be `feature/viewer-structure`)
2. Check git status for any uncommitted changes
3. Review this checklist to find current phase
4. Run `make serve` to verify build
5. Continue from next unchecked item

### Key Decisions Summary

| Topic | Decision |
|-------|----------|
| Current ROI placement | Within ROI Tools, not separate section |
| Nested collapsibles | No - use static containers |
| Typography values | Use existing design tokens, not hardcoded |
| Button wrapping | Graceful fallback, not rigid prevention |
| Touch targets | Preserve 44px minimum |
| Table location | Stays in place within Analysis Tools |
| Canvas-legend gap | Target `--spacing-2` (~8px) |
| Brush slider | Reuse `.threshold-control` pattern |
| Alignment scope | Canvas column only, not cross-column |

### Files Most Likely to Change

1. `blmm_demo/crates/viewer_app/src/components/roi_panel.rs` - Phases 1, 3, 7
2. `style/input.css` - Phases 1-7
3. `blmm_demo/crates/viewer_app/src/components/vertex_summary_table.rs` - Phase 4
4. `blmm_demo/crates/viewer_app/src/components/color_legend.rs` - Phase 6

### Expert Review Context

This work addresses feedback from an expert UI/UX review that identified:
- "Wall of text" in Analysis Tools panel
- ROI table styling mismatch
- Canvas visual competition with expanded sidebar
- Legend details spacing issues
- Inconsistent brush slider pattern
- Vertical flow fragmentation

The expert estimated the viewer was "~70% complete" with remaining work being "visual hierarchy, spacing, and content grouping" - not structural.

---

## Completion Status

- [x] Phase 0 – Setup & Verification
- [x] Phase 1 – Analysis Tools Structure
- [x] Phase 2 – Typography & Density
- [x] Phase 3 – ROI Buttons & Controls Layout
- [x] Phase 4 – ROI Table Styling
- [x] **Checkpoint A+B** (user approved)
- [x] Phase 5 – Canvas & Vertical Flow
- [x] Phase 6 – Legend Details Refinement
- [x] Phase 7 – Brush Slider Consistency
- [x] **Checkpoint C** (committed)
- [ ] Phase 8 – Accessibility & Consistency Pass
- [ ] Phase 9 – Final QA & Screenshots

**Current Phase:** Phase 8 – Accessibility & Consistency Pass
**Last Updated:** 2025-01-09
