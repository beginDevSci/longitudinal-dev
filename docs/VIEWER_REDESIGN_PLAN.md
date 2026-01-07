# Brain Viewer UI/UX Redesign Plan

> **Working Checklist** - Track progress with checkboxes as phases/steps complete.

---

## Phase 0: Foundation & Design Decisions ✅ COMPLETE

Establish design direction before any implementation

- [x] **Step 0.1: Define Design Principles**
  - [x] Decide: Light theme (integrated with site) - using site design tokens
  - [x] Decide: Primary persona - Tutorial reader (guided exploration), with advanced mode for researchers
  - [x] Document design tokens to use (colors, spacing, typography, border radii)

- [x] **Step 0.2: Audit Existing Site Design System**
  - [x] Review style/input.css for existing Tailwind tokens and patterns
  - [x] Identify reusable components (buttons, dropdowns, cards, collapsibles)
  - [x] Document viewer-specific styles → Created `VIEWER_DESIGN_TOKENS.md`

- [x] **Step 0.3: Create Design Mockups**
  - [x] Reviewed current three-column layout via screenshots
  - [x] Defined "Simple" vs. "Advanced" mode approach
  - [x] Got stakeholder feedback - proceed with token alignment first

**Tag:** `viewer-redesign-phase0-complete`

---

## Phase 1: Quick Wins (Visual Polish) 🔄 IN PROGRESS

Low-risk changes that improve appearance without restructuring

- [x] **Step 1.1: Token Alignment** (added step - replace hardcoded colors with CSS variables)
  - [x] `control_panel.rs` - Left sidebar, form controls
  - [x] `collapsible_section.rs` - Section headers
  - [x] `color_legend.rs` - Legend card
  - [x] `camera_presets.rs` - Preset buttons
  - [x] `brain_viewer.rs` - Main container, canvas, right panel
  - [x] `roi_panel.rs` - ROI tools
  - [x] `vertex_info_panel.rs` - Vertex info display
  - [x] `annotation_panel.rs` - Annotations
  - [x] `vertex_summary_table.rs` - Top 10 table
  - [x] `brain_viewer_facade/lib.rs` - Fallback/placeholder styling

- [ ] **Step 1.2: Canvas Container Styling**
  - [ ] Replace dashed border with subtle shadow or solid border matching site aesthetic
  - [ ] Add proper rounded corners consistent with site cards
  - [ ] Ensure canvas has appropriate padding/margins

- [ ] **Step 1.3: Typography & Spacing Cleanup**
  - [ ] Audit font sizes for hierarchy (headers vs. labels vs. help text)
  - [ ] Ensure consistent spacing between sections
  - [ ] Improve contrast ratios for accessibility

- [ ] **Step 1.4: Dropdown/Select Styling**
  - [ ] Style selected values to look "selected" not "disabled" (darker text, not gray)
  - [ ] Match dropdown styling to site's existing form elements
  - [ ] Add subtle hover/focus states

- [ ] **Step 1.5: Button Consistency**
  - [ ] Audit all buttons (Export, Copy, ROI tools, Camera presets)
  - [ ] Establish primary/secondary/tertiary button hierarchy
  - [ ] Ensure consistent sizing and spacing

---

## Phase 2: Information Architecture

Reorganize controls for better cognitive flow

- [ ] **Step 2.1: Sidebar Section Reorganization**
  - [ ] Group related controls logically:
    - View: Hemisphere, Layout, Camera presets
    - Data: Analysis, Statistic, Volume/Contrast
    - Appearance: Colormap, Threshold, Symmetric toggle
    - Advanced: Parcellation, ROI Tools, Accessibility
  - [ ] Add collapsible sections with sensible defaults (Advanced collapsed)

- [ ] **Step 2.2: Improve Volume/Contrast Selection**
  - [ ] Replace "Volume 0" with actual contrast label ("Intercept", "Sex", etc.)
  - [ ] Consider a more visual selector (buttons or segmented control for 5 contrasts)
  - [ ] Show contrast description on selection

- [ ] **Step 2.3: Simplify Camera Presets**
  - [ ] Replace abbreviations with full names or icons with tooltips
  - [ ] Consider a visual "orientation cube" widget instead of buttons
  - [ ] Or: grouped buttons with better labeling ("Left: Lateral | Medial")

- [ ] **Step 2.4: Consolidate Right Panel**
  - [ ] Move Color Legend to be always visible (critical for interpretation)
  - [ ] Group ROI Tools, Vertex Info, and Annotations under collapsible "Analysis Tools"
  - [ ] Move "Top 10 Vertices" table into Analysis Tools section

---

## Phase 3: Progressive Disclosure

Reduce initial complexity, reveal on demand

- [ ] **Step 3.1: Implement View Mode Toggle**
  - [ ] Create "Simple" vs. "Advanced" mode toggle
  - [ ] Simple mode: Only essential controls (Hemisphere, Contrast, Colormap, Threshold)
  - [ ] Advanced mode: Full control set as currently exists

- [ ] **Step 3.2: Default to Simple Mode**
  - [ ] Tutorial readers see clean, focused interface
  - [ ] "Show Advanced Controls" link expands to full toolset
  - [ ] Persist preference in localStorage

- [ ] **Step 3.3: Smart Defaults**
  - [ ] Set sensible defaults based on data (e.g., auto-detect threshold from metadata)
  - [ ] Pre-select first meaningful contrast (not Intercept)
  - [ ] Default to "Both" hemispheres if side-by-side layout works

---

## Phase 4: Canvas Enhancements

Improve the core visualization experience

- [ ] **Step 4.1: Add On-Canvas Controls**
  - [ ] Zoom in/out buttons (bottom-right corner)
  - [ ] Reset view button
  - [ ] Fullscreen toggle

- [ ] **Step 4.2: Interaction Hints**
  - [ ] Add subtle "Drag to rotate" hint on first load
  - [ ] Show cursor changes for different modes (rotate vs. ROI draw)
  - [ ] Toast/tooltip on first vertex click

- [ ] **Step 4.3: Loading States**
  - [ ] Design proper loading skeleton for canvas
  - [ ] Show progress for large data loads
  - [ ] Graceful error states with retry option

- [ ] **Step 4.4: Fullscreen Mode**
  - [ ] Implement fullscreen toggle that hides page chrome
  - [ ] Ensure controls remain accessible in fullscreen
  - [ ] Keyboard shortcut (F or Escape to exit)

---

## Phase 5: Tutorial Integration

Connect viewer to surrounding educational content

- [ ] **Step 5.1: Guided Exploration Presets**
  - [ ] Add "Preset Views" dropdown tied to tutorial content
  - [ ] E.g., "Show Sex Effect (Figure 2)" auto-configures viewer
  - [ ] Link from tutorial text to viewer state

- [ ] **Step 5.2: Contextual Help**
  - [ ] Add (?) icons next to complex controls with explanatory tooltips
  - [ ] Link to glossary for terms like "T-statistic", "Threshold"
  - [ ] Consider inline help mode that highlights and explains each control

- [ ] **Step 5.3: Synchronized State**
  - [ ] URL hash reflects viewer state (shareable links)
  - [ ] "Copy view link" creates link that restores exact configuration
  - [ ] Deep links from tutorial text to specific viewer states

---

## Phase 6: Responsive & Accessibility

Ensure viewer works across devices and for all users

- [ ] **Step 6.1: Responsive Breakpoints**
  - [ ] Define behavior for tablet (stack sidebars below canvas)
  - [ ] Define behavior for mobile (overlay panels, swipe to access)
  - [ ] Ensure touch interactions work (pinch to zoom, swipe to rotate)

- [ ] **Step 6.2: Keyboard Navigation**
  - [ ] Ensure all controls are keyboard accessible
  - [ ] Add keyboard shortcuts for common actions (R=reset, F=fullscreen)
  - [ ] Visible focus indicators

- [ ] **Step 6.3: Screen Reader Support**
  - [ ] Add ARIA labels to all controls
  - [ ] Announce state changes (e.g., "Now showing Sex contrast")
  - [ ] Provide text alternative for visualization (summary stats)

---

## Phase 7: Polish & Performance

Final refinements

- [ ] **Step 7.1: Animation & Transitions**
  - [ ] Smooth transitions when changing views/contrasts
  - [ ] Subtle animations for collapsible sections
  - [ ] Loading shimmer effects

- [ ] **Step 7.2: Performance Optimization**
  - [ ] Lazy load viewer component (don't block page render)
  - [ ] Optimize WebGPU rendering for smooth interaction
  - [ ] Cache loaded data across contrast switches

- [ ] **Step 7.3: User Testing & Iteration**
  - [ ] Gather feedback from target users (researchers, students)
  - [ ] A/B test simplified vs. full interface
  - [ ] Iterate based on findings

---

## Implementation Priority

| Priority | Phase     | Rationale                                       | Status |
|----------|-----------|-------------------------------------------------|--------|
| 1        | Phase 0   | Must decide direction before coding             | ✅ Done |
| 2        | Phase 1   | Quick wins build momentum, low risk             | 🔄 In Progress |
| 3        | Phase 2.2 | Volume→Contrast label is high-impact UX fix     | Pending |
| 4        | Phase 3   | Progressive disclosure addresses core tension   | Pending |
| 5        | Phase 4.4 | Fullscreen is high-value, moderate effort       | Pending |
| 6        | Phase 2   | Full reorganization after basics work           | Pending |
| 7        | Phase 5   | Tutorial integration is site-specific value-add | Pending |
| 8        | Phase 6-7 | Polish after core UX is solid                   | Pending |

---

## User Decisions (from planning session)

1. **Theming**: Use site design tokens for consistency
2. **ROI Tools**: Hide by default (advanced feature)
3. **Timeline**: Middle-ground approach (not minimal, not comprehensive)
4. **Modifications**: Can modify viewer_app freely (with rollback via git tags)
