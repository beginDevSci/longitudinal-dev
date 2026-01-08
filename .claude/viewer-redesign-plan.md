# Brain Viewer Redesign Plan

> **Branch:** `feature/viewer-structure`
> **Created:** 2025-01-08
> **Status:** Phase 1 Complete

## Overview

This plan addresses horizontal space constraints in the brain viewer by restructuring layouts and improving UX. Changes are organized into phases that can be implemented incrementally, with each phase independently testable.

---

## Architecture Reference

### File Locations

| Component | Path | Purpose |
|-----------|------|---------|
| Viewer layout | `blmm_demo/crates/viewer_app/src/components/brain_viewer.rs` | Main 3-column layout (lines 1828-2200) |
| Control panel | `blmm_demo/crates/viewer_app/src/components/control_panel.rs` | Left panel controls |
| Color legend | `blmm_demo/crates/viewer_app/src/components/color_legend.rs` | Right panel legend |
| Collapsible | `blmm_demo/crates/viewer_app/src/components/collapsible_section.rs` | Reusable collapse component |
| Viewer CSS | `style/input.css` | Lines 4172-4320 |
| Page layout | `src/layout/post_layout.rs` | Tutorial page 3-column grid |
| Left nav | `src/layout/left_nav.rs` | Tutorial navigation sidebar |
| Right TOC | `src/layout/table_of_contents.rs` | Page table of contents |

### Layout Structure (After Phase 1)

```
Page Level (post_layout.rs)
┌─────────────┬────────────────────────────────────┬─────────────┐
│  LeftNav    │           Main Content             │    TOC      │
│  (~200px)   │                                    │  (~250px)   │
│             │  ┌─────────┬───────────────────┐   │             │
│             │  │Controls │      Canvas       │   │             │
│             │  │ (256px) ├───────────────────┤   │             │
│             │  │         │   Legend Strip    │   │             │
│             │  └─────────┴───────────────────┘   │             │
│             │     Viewer Level (brain_viewer.rs) │             │
└─────────────┴────────────────────────────────────┴─────────────┘
```

---

## Phase 1: Layout Restructure (2-Column + Legend Below)

**Goal:** Reclaim ~256px horizontal space by moving legend below canvas
**Risk:** Low
**Impact:** High

### Target Layout

```
Desktop (1024px+):
┌─────────────┬────────────────────────────────────┐
│  Controls   │           Canvas                   │
│  (256px)    │           (flex)                   │
│             ├────────────────────────────────────┤
│             │     Legend Strip (horizontal)      │
└─────────────┴────────────────────────────────────┘
```

### Checklist (Completed 2025-01-08)

#### 1.1 CSS Layout Changes
- [x] **File:** `style/input.css` (lines 4260-4285)
- [x] Create wrapper div for canvas + legend as a unit
- [x] Change `.brain-viewer-layout` desktop grid to 2-column
- [x] Create `.brain-viewer-canvas-area` for canvas + legend wrapper
- [x] Create `.brain-viewer-legend-strip` for horizontal legend below canvas
- [x] Remove `.brain-viewer-right-panel` width constraint on desktop (hidden via CSS)
- [ ] Test: Verify layout at 1024px, 1280px, 1440px widths (visual testing needed)

#### 1.2 Component Restructure
- [x] **File:** `brain_viewer.rs` (lines 1862-2173)
- [x] Wrap canvas and legend in new container div (`.brain-viewer-canvas-area`)
- [x] Move `<ColorLegend>` from right panel to below canvas (in legend strip)
- [x] Move action buttons (Export, Copy link) to legend strip
- [ ] Move "Analysis Tools" section to controls panel (deferred to Phase 2)
- [x] Legacy right panel hidden via CSS (kept for potential mobile fallback)
- [ ] Test: All viewer functionality still works (visual testing needed)

#### 1.3 Responsive Adjustments
- [x] **File:** `style/input.css`
- [x] Mobile (<768px): Legend stacks below canvas naturally
- [x] Tablet (768-1023px): Legend below canvas, controls above
- [ ] Test: All breakpoints render correctly (visual testing needed)
- [ ] Test: Touch interactions still work (visual testing needed)

#### 1.4 Validation
- [x] Build passes: `cargo leptos build --release`
- [ ] No visual regressions at desktop/tablet/mobile (visual testing needed)
- [ ] Canvas renders correctly (visual testing needed)
- [ ] Controls functional (visual testing needed)
- [ ] Legend readable (visual testing needed)
- [ ] Export/share buttons work (visual testing needed)

---

## Phase 2: Legend Compression

**Goal:** Compact legend with expandable details
**Risk:** Low
**Impact:** Medium

### Target Design

```
┌─────────────────────────────────────────────────────────────────┐
│ [Colorbar gradient] Min: -17.86  |  Max: 26.29  | T-Stats ▼    │
└─────────────────────────────────────────────────────────────────┘

Expanded (on click ▼):
┌─────────────────────────────────────────────────────────────────┐
│ [Colorbar gradient] Min: -17.86  |  Max: 26.29  | T-Stats ▲    │
├─────────────────────────────────────────────────────────────────┤
│ Units: statistic value (e.g., t-stat)                          │
│ Range derived from loaded statistics. Click bar to threshold.  │
│ Suggested threshold (heuristic): 2.00                          │
└─────────────────────────────────────────────────────────────────┘
```

### Checklist

#### 2.1 Compact Legend Component
- [ ] **File:** `color_legend.rs`
- [ ] Refactor to horizontal layout (flex row)
- [ ] Always visible: colorbar, min, max, stat name
- [ ] Add expand/collapse toggle (chevron icon)
- [ ] Collapsed by default
- [ ] Expandable: units, range explanation, threshold hint, suggested value

#### 2.2 Legend Strip Styling
- [ ] **File:** `style/input.css`
- [ ] Create `.brain-viewer-legend-strip` styles
- [ ] Horizontal flex layout for compact view
- [ ] Expandable section styling
- [ ] Smooth expand/collapse transition
- [ ] Consistent with dark theme

#### 2.3 Analysis Tools Relocation
- [ ] **File:** `brain_viewer.rs`
- [ ] Move "Analysis Tools" collapsible to bottom of controls panel
- [ ] OR: Include in legend expanded section
- [ ] Ensure vertex info, ROI tools remain accessible

#### 2.4 Validation
- [ ] Legend displays correctly in compact form
- [ ] Expand/collapse works
- [ ] All information still accessible
- [ ] Threshold clicking still works
- [ ] No overflow/clipping issues

---

## Phase 3: Control Panel Improvements

**Goal:** Better UX for common controls
**Risk:** Low
**Impact:** Medium
**Status:** Implementation Complete (2025-01-08) - Visual testing needed

### Checklist

#### 3.1 Hemisphere Segmented Control
- [x] **File:** `control_panel.rs`
- [x] Replace dropdown with segmented button group
- [x] Design: `[ Left | Right ]` toggle buttons
- [x] Active state clearly highlighted (accent color background)
- [x] Keyboard accessible (focus-visible styling, aria-pressed)
- [ ] Test: State changes correctly, viewer updates (visual testing needed)

#### 3.2 Threshold Slider + Input
- [x] **File:** `control_panel.rs`
- [x] Add numeric input box next to slider
- [x] Two-way binding: slider updates input, input updates slider
- [x] Input validation (numeric, within range via HTML5 attributes)
- [x] Display to 2 decimal places
- [ ] Test: Both slider and input work, values sync (visual testing needed)

#### 3.3 Control Panel Styling
- [x] **File:** `style/input.css`
- [x] Consistent spacing (via flex gap)
- [x] Clear visual grouping
- [x] Ensure touch targets ≥44px on touch devices (media query)

#### 3.4 Validation
- [ ] All controls functional (visual testing needed)
- [ ] State persists correctly (visual testing needed)
- [ ] Keyboard navigation works (visual testing needed)
- [ ] Touch interactions work (visual testing needed)
- [ ] No layout shifts on interaction (visual testing needed)

---

## Phase 4: Focus Mode

**Goal:** Full-width viewer overlay for deep exploration
**Risk:** Medium
**Impact:** High

### Target Design

```
Focus Mode Overlay (position: fixed, full viewport):
┌─────────────────────────────────────────────────────────────────┐
│ [Hemisphere] [Contrast ▼] [Threshold ──●──] [Colorbar] [✕ Exit] │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│                                                                 │
│                     MAXIMIZED CANVAS                            │
│                                                                 │
│                                                   [Zoom] [Reset]│
└─────────────────────────────────────────────────────────────────┘
```

### Checklist

#### 4.1 Focus Mode State
- [ ] **File:** `brain_viewer.rs`
- [ ] Add `is_focus_mode: RwSignal<bool>` (separate from fullscreen)
- [ ] Add "Expand" button near canvas controls
- [ ] Button distinct from existing fullscreen icon

#### 4.2 Focus Mode Overlay
- [ ] **File:** `brain_viewer.rs`
- [ ] Create overlay container (position: fixed, inset: 0)
- [ ] Dark scrim background
- [ ] Top strip with key controls only:
  - Hemisphere toggle
  - Contrast dropdown
  - Threshold slider
  - Compact colorbar
  - Exit button
- [ ] Maximized canvas area
- [ ] Floating zoom/reset controls

#### 4.3 Focus Mode CSS
- [ ] **File:** `style/input.css`
- [ ] `.brain-viewer-focus-overlay` - Fixed positioning, z-index, scrim
- [ ] `.brain-viewer-focus-header` - Top control strip
- [ ] `.brain-viewer-focus-canvas` - Maximized canvas
- [ ] `.brain-viewer-focus-exit` - Exit button styling
- [ ] Smooth enter/exit transitions
- [ ] Prevent body scroll when active

#### 4.4 Focus Mode Behavior
- [ ] ESC key exits focus mode
- [ ] Click outside canvas does NOT exit (intentional)
- [ ] All controls work in focus mode
- [ ] Export/share work in focus mode
- [ ] State syncs between embedded and focus mode

#### 4.5 Validation
- [ ] Focus mode activates cleanly
- [ ] All controls functional in focus mode
- [ ] ESC and button exit work
- [ ] No state loss on exit
- [ ] Canvas renders correctly at large size
- [ ] Keyboard navigation works
- [ ] Touch interactions work

---

## Phase 5: Page Sidebar Collapsibility (Optional)

**Goal:** Allow collapsing page-level sidebars for more viewer space
**Risk:** Medium (affects site-wide layout)
**Impact:** Medium

### Checklist

#### 5.1 Collapse Affordances
- [ ] **File:** `post_layout.rs`
- [ ] Add collapse toggle (chevron) to left sidebar edge
- [ ] Add collapse toggle to right TOC edge
- [ ] Store collapse state in localStorage

#### 5.2 Left Sidebar Collapse
- [ ] **File:** `left_nav.rs` + `style/input.css`
- [ ] Collapsed state: ~48px icon strip
- [ ] Show method family icons only
- [ ] Hover/click expands temporarily OR click chevron to expand
- [ ] Smooth transition animation

#### 5.3 Right TOC Collapse
- [ ] **File:** `table_of_contents.rs` + `style/input.css`
- [ ] Collapsed state: ~48px with TOC icon
- [ ] Click to expand
- [ ] Smooth transition

#### 5.4 Validation
- [ ] Sidebars collapse/expand smoothly
- [ ] State persists on refresh
- [ ] Navigation still accessible
- [ ] No content jumping on toggle
- [ ] Works at all breakpoints

---

## Deferred Items (Not in Scope)

| Item | Reason |
|------|--------|
| Histogram sparkline on threshold | High complexity, uncertain user value |
| Searchable contrast dropdown | Overkill for typical 3-6 options |
| Auto-collapse sidebar hints | Potentially annoying UX |
| "Both" hemisphere option | Not currently supported by renderer |
| Colormap preview swatches | Nice-to-have, not essential |

---

## Testing Protocol (Each Phase)

### Breakpoint Testing
- [ ] Mobile: 375px (iPhone SE)
- [ ] Mobile: 390px (iPhone 14)
- [ ] Tablet: 768px
- [ ] Tablet: 1024px
- [ ] Desktop: 1280px
- [ ] Desktop: 1440px
- [ ] Desktop: 1920px

### Functional Testing
- [ ] Viewer loads without errors
- [ ] Canvas renders brain surface
- [ ] All dropdowns work
- [ ] Threshold slider works
- [ ] Color legend updates
- [ ] Export screenshot works
- [ ] Copy view link works
- [ ] Fullscreen works
- [ ] Keyboard navigation works

### Interaction Testing
- [ ] Mouse: drag to rotate
- [ ] Mouse: scroll to zoom
- [ ] Touch: drag to rotate
- [ ] Touch: pinch to zoom
- [ ] Keyboard: arrow keys rotate
- [ ] Keyboard: +/- zoom
- [ ] Keyboard: R reset

### Accessibility Testing
- [ ] Focus indicators visible
- [ ] Screen reader announces controls
- [ ] Color contrast sufficient
- [ ] Touch targets ≥44px

---

## Rollback Points

| Phase | Rollback Command |
|-------|------------------|
| Pre-Phase 1 | `git stash` or `git checkout .` |
| After Phase 1 | `git revert HEAD` or tag `v-phase1` |
| After Phase 2 | `git revert HEAD` or tag `v-phase2` |
| After Phase 3 | `git revert HEAD` or tag `v-phase3` |
| After Phase 4 | `git revert HEAD` or tag `v-phase4` |

**Recommendation:** Create a git tag after each successfully validated phase.

```bash
git tag -a viewer-redesign-phase1 -m "Phase 1: 2-column layout complete"
```

---

## Session Handoff Notes

When resuming work on this plan:

1. Check current branch: `git branch --show-current` (should be `feature/viewer-structure`)
2. Check which phases are complete by reviewing git log or tags
3. Read this document for context
4. Continue from the next unchecked item

### Key Files Quick Reference

```bash
# Viewer layout
code blmm_demo/crates/viewer_app/src/components/brain_viewer.rs

# Control panel
code blmm_demo/crates/viewer_app/src/components/control_panel.rs

# Color legend
code blmm_demo/crates/viewer_app/src/components/color_legend.rs

# Viewer CSS
code style/input.css +4172

# Page layout
code src/layout/post_layout.rs
```

---

## Approval Status

- [ ] Plan reviewed by user
- [ ] Phase 1 approved to begin
- [ ] Phase 2 approved to begin
- [ ] Phase 3 approved to begin
- [ ] Phase 4 approved to begin
- [ ] Phase 5 approved to begin (optional)
