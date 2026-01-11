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
| Viewer layout | `crates/viewer_app/src/components/brain_viewer.rs` | Main 3-column layout (lines 1828-2200) |
| Control panel | `crates/viewer_app/src/components/control_panel.rs` | Left panel controls |
| Color legend | `crates/viewer_app/src/components/color_legend.rs` | Right panel legend |
| Collapsible | `crates/viewer_app/src/components/collapsible_section.rs` | Reusable collapse component |
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
**Status:** Implementation Complete (2025-01-08) - Visual testing needed

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
- [x] **File:** `brain_viewer.rs`
- [x] Add `is_focus_mode: RwSignal<bool>` (separate from fullscreen)
- [x] Add "Expand" button near canvas controls (maximize-2 icon)
- [x] Button distinct from existing fullscreen icon (expand vs browser fullscreen)

#### 4.2 Focus Mode Overlay
- [x] **File:** `brain_viewer.rs`
- [x] Create overlay container (position: fixed, inset: 0)
- [x] Top strip with key controls only:
  - [x] Hemisphere toggle (segmented L/R)
  - [x] Threshold slider + input
  - [x] Compact colorbar (ColorLegend component)
  - [x] Exit button (X icon)
- [x] Maximized canvas area (via CSS transforms)
- [x] Floating zoom/reset controls (existing, repositioned via CSS)
- Note: Contrast dropdown not included (can use sidebar before entering focus)

#### 4.3 Focus Mode CSS
- [x] **File:** `style/input.css` (lines 4490-4656)
- [x] `.brain-viewer-focus-active` - Fixed positioning, z-index on container
- [x] `.brain-viewer-focus-header` - Top control strip with backdrop blur
- [x] `.brain-viewer-layout-focus` - Maximized canvas layout
- [x] `.brain-viewer-focus-exit` - Exit button styling
- [x] Mobile-responsive header (wraps, hides legend)
- [x] Prevent body scroll when active (JS toggle)

#### 4.4 Focus Mode Behavior
- [x] ESC key exits focus mode (priority over selection clear)
- [x] Click outside canvas does NOT exit (intentional)
- [x] Hemisphere + threshold controls work in focus mode
- [ ] Export/share work in focus mode (visual testing needed)
- [x] State syncs between embedded and focus mode (same signals)

#### 4.5 Validation (Visual testing needed)
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
**Status:** Implementation Complete (2025-01-08) - Visual testing needed

### Checklist

#### 5.1 Collapse Affordances
- [x] **File:** `left_nav.rs`, `table_of_contents.rs`
- [x] Left sidebar has collapse toggle (chevron button)
- [x] Right TOC has collapse toggle (chevron button)
- [x] Store collapse state in localStorage (`left-nav-collapsed`, `right-toc-collapsed`)

#### 5.2 Left Sidebar Collapse
- [x] **File:** `left_nav.rs` + inline Tailwind
- [x] Collapsed state: w-14 (56px) icon strip
- [x] Chevron rotates when collapsed
- [x] Smooth transition animation (duration-300)
- [x] localStorage persistence via Effect

#### 5.3 Right TOC Collapse
- [x] **File:** `table_of_contents.rs` + `style/input.css`
- [x] Collapsed state: 48px with toggle visible
- [x] Uses CSS transform (translateX) for smooth collapse
- [x] localStorage persistence via Effect

#### 5.4 Validation (Visual testing needed)
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

## Development Workflow

### Running the Development Server

**IMPORTANT:** This project uses Static Site Generation (SSG), not Server-Side Rendering (SSR). The correct way to run locally is:

```bash
# Build and serve with interactive brain viewer (RECOMMENDED)
make serve

# This runs: WEBGPU_VIEWER=1 cargo xtask build-ssg + python3 http.server
# Site available at: http://localhost:8000
```

### Why NOT to use `cargo leptos watch`

While `cargo leptos watch/serve` is configured in `Cargo.toml`, this project generates static HTML pages and exits. The Leptos server doesn't stay running because it's in SSG mode, not SSR mode. Use `make serve` instead.

### Feature Flags

The `webgpu-viewer` feature must be enabled for the interactive brain viewer:

| Build Command | webgpu-viewer | Result |
|---------------|---------------|--------|
| `make serve` | ✅ Included | Interactive viewer works |
| `make ssg` | ✅ Included | Interactive viewer works |
| `make ssg-minimal` | ❌ Excluded | "requires webgpu-viewer" message |

### Architecture Notes

1. **SSG Build** (`src/bin/xtask.rs`): Builds WASM + generates static HTML
2. **crates/viewer_app/**: Contains viewer components (integrated into repo)
3. **style/input.css**: All viewer CSS (tracked, changes visible after `make serve`)

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| "requires webgpu-viewer feature" | WASM built without feature | Run `make serve` (not `make ssg-minimal`) |
| Server exits immediately | SSG mode, not SSR | Use `make serve`, not `cargo leptos watch` |
| CSS changes not visible | Old build in dist/ | Run `make serve` to rebuild |
| Port 3000 conflicts | Wrong server | Kill with `pkill -f cargo-leptos`, use `make serve` |

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
code crates/viewer_app/src/components/brain_viewer.rs

# Control panel
code crates/viewer_app/src/components/control_panel.rs

# Color legend
code crates/viewer_app/src/components/color_legend.rs

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
