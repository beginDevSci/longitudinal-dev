# Method Guides v1.1 Polish Roadmap

This document tracks potential UX enhancements for a future polish pass on the Method Guides system. These items are explicitly out of scope for v1 but represent good improvements for v1.1.

## TOC & Navigation Polish

- [ ] **Sticky TOC sidebar** - Add a table of contents sidebar for longer guides (mirrors tutorial system)
- [ ] **Progress indicator** - Show reading progress through the guide
- [ ] **Section jump links** - TOC links that scroll to sections smoothly

## Sharing & Anchors

- [ ] **Heading anchor links** - Add hover-reveal `#` links on H2/H3 for easy sharing
- [ ] **Auto-copy anchor** - Click heading anchor → copy URL to clipboard
- [ ] **Deep link support** - Ensure anchor links work from external sources

## Power-User Features

- [ ] **"Copy all code" button** - Single button to copy all code blocks from Worked Example section
- [ ] **Code block filenames** - Show suggested filename above code blocks (e.g., `lgcm_analysis.R`)
- [ ] **Run in RStudio Cloud** - Deep link to open code in cloud R environment (stretch goal)

## Print/Export Improvements

- [ ] **Print stylesheet** - Add `@media print` styles for clean PDF export
- [ ] **Expand all for print** - Auto-expand collapsible sections when printing
- [ ] **Hide interactive elements** - Remove copy buttons, theme toggles in print view

## Visual Theme Alignment

- [ ] **Syntax highlighting theme** - Match code block highlighting to Dracula theme
- [ ] **Callout icon polish** - Consider custom SVG icons for each callout type
- [ ] **Math font consistency** - Ensure KaTeX fonts blend with body typography

## Content Enhancements

- [ ] **"When to use" comparison** - Add section comparing related methods (e.g., LMM vs LGCM)
- [ ] **Cross-links to tutorials** - Link guides to corresponding ABCD tutorial implementations
- [ ] **Difficulty badges** - Add "Beginner / Intermediate / Advanced" indicators on index cards
- [ ] **Prerequisites display** - Show prerequisite knowledge on guide cards

## Technical Debt

- [ ] **Pipeline-first math escaping** - Move math detection before emphasis parsing to eliminate underscore escape requirement
- [ ] **Test coverage** - Add integration tests for guide rendering pipeline
- [ ] **Accessibility audit** - Screen reader testing for collapsible modules and math

---

## Prioritization Notes

**Quick wins (low effort, high value):**
1. Heading anchor links
2. Print stylesheet
3. Difficulty badges on index

**Medium effort:**
1. Sticky TOC sidebar
2. Copy all code button
3. Cross-links to tutorials

**Higher effort (v1.2+):**
1. Pipeline-first math escaping
2. Run in RStudio Cloud integration
3. Full accessibility audit

---

## Browser QA Checklist (v1 Release)

Manual testing checklist for `/guides/lgcm` and `/guides/lmm` before declaring v1 ready.

### Structure & Rendering

- [ ] **H2 sections render correctly** - All 6 sections visible (Overview, Conceptual Foundations, Model Specification & Fit, Interpretation, Worked Example, Reference & Resources)
- [ ] **Collapsible modules work** - "Worked Example" and "Reference & Resources" are in `<details>` elements, toggle open/closed
- [ ] **Tables render with wrapper** - Tables have horizontal scroll on narrow viewports
- [ ] **Code blocks have syntax highlighting** - R code blocks styled with language-appropriate colors

### Callouts

- [ ] **All callout types render** - tip (green), warning (amber), note (blue), caution (red), info (cyan)
- [ ] **Callout structure** - Title bar visible, content properly contained
- [ ] **Callout accessibility** - `role="note"` and `aria-label` present

### Math Rendering

- [ ] **Display math renders** - Centered, formatted LaTeX equations (not raw `$...$`)
- [ ] **Inline math renders** - `\(y_{it}\)` appears as formatted math within text
- [ ] **Subscripts work** - `\(\eta_{0i}\)`, `\(\beta_1\)` render correctly
- [ ] **Math in callouts** - Math inside tip/warning blocks renders

### Interactivity

- [ ] **Copy code buttons work** - Click copies code to clipboard, shows feedback
- [ ] **Anchor navigation** - Direct links to sections (`#overview`, `#worked-example`) scroll to target
- [ ] **Auto-expand on anchor** - Linking to `#worked-example` expands that section if collapsed

### Accessibility & Motion

- [ ] **Reduced motion respected** - With `prefers-reduced-motion: reduce`, animations are suppressed
- [ ] **Smooth scrolling** - Anchor navigation uses smooth scroll (unless reduced motion enabled)
- [ ] **Keyboard navigation** - Tab through interactive elements, Enter/Space toggles details

### Responsive Design

- [ ] **Mobile layout** - Content readable at 375px width
- [ ] **Tablet layout** - Proper spacing at 768px width
- [ ] **Table overflow** - Wide tables scroll horizontally, don't break layout
- [ ] **Math overflow** - Long equations don't overflow container

### No-JS Fallback

- [ ] **Content visible without JS** - All text, math, code visible with JavaScript disabled
- [ ] **Details work natively** - `<details>` elements still toggle without hydration
- [ ] **Copy buttons graceful** - Button present but may not function (acceptable)

### Cross-Browser

- [ ] **Safari** - All features work in Safari 17+
- [ ] **Chrome** - All features work in Chrome 120+
- [ ] **Firefox** - All features work in Firefox 120+
- [ ] **Edge** - All features work in Edge 120+

---

*Last updated: December 2024*
*Status: Parked for post-v1 consideration*
