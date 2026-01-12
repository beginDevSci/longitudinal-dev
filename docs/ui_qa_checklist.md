# UI QA Checklist

Quality assurance checklist for Resources and Tools page refinements.

## Breakpoints to Test

| Breakpoint | Width | Device Example |
|------------|-------|----------------|
| Mobile     | 375px | iPhone SE      |
| Mobile+    | 414px | iPhone 14      |
| Tablet     | 768px | iPad Mini      |
| Desktop    | 1024px| Laptop         |
| Desktop+   | 1280px| Large Monitor  |
| Wide       | 1536px| Ultrawide      |

## Visual Checks

### Cards
- [ ] Consistent border-radius across all card types
- [ ] Uniform padding inside cards
- [ ] Cards in same row have equal heights
- [ ] Hover states visible (shadow, border color, scale)
- [ ] Images/logos don't overflow containers
- [ ] Text truncation (line-clamp) works consistently
- [ ] Fallback states render correctly (missing images/logos)

### Grids
- [ ] Gap spacing consistent across sections
- [ ] Column count appropriate for breakpoint
- [ ] No orphan cards (single card on final row looks intentional)
- [ ] Grid alignment clean at all breakpoints

### Typography
- [ ] Section headers hierarchy clear (H1 > H2 > H3)
- [ ] Body text readable (adequate line-height, font-size)
- [ ] Muted/tertiary text still legible
- [ ] Line clamping doesn't cut text awkwardly

### Color/Theme
- [ ] Light mode: all elements visible and contrasting
- [ ] Dark mode: all elements visible and contrasting
- [ ] Accent colors consistent across hover states
- [ ] No "washed out" or invisible elements

## Keyboard Navigation

- [ ] All interactive cards reachable via Tab
- [ ] Focus ring clearly visible on all focusable elements
- [ ] Focus order logical (top-to-bottom, left-to-right)
- [ ] Enter/Space activates links/buttons
- [ ] No keyboard traps
- [ ] Skip navigation works (if applicable)

## Accessibility

### Focus Visibility
- [ ] Focus ring has sufficient contrast (3:1 minimum)
- [ ] Focus ring not obscured by card styling
- [ ] Focus offset adequate (not touching text)

### Contrast (WCAG AA)
- [ ] Primary text: 4.5:1 minimum
- [ ] Secondary/muted text: 4.5:1 minimum
- [ ] Interactive elements: 3:1 minimum
- [ ] Check in both light and dark modes

### Screen Reader
- [ ] Images have meaningful alt text
- [ ] Links have descriptive text (not just "click here")
- [ ] Section headings create logical outline
- [ ] External links indicated (visually and for AT)

## Motion & Interaction

- [ ] Hover transitions smooth (200-300ms typical)
- [ ] No jarring layout shifts on hover
- [ ] Animations respect `prefers-reduced-motion`
- [ ] Loading states don't cause flicker

## Cross-Browser

- [ ] Chrome
- [ ] Firefox
- [ ] Safari
- [ ] Edge

## Performance

- [ ] Images lazy-loaded (`loading="lazy"`)
- [ ] No layout shift from images loading
- [ ] Smooth scrolling to anchor links
- [ ] Interactive elements respond immediately

---

## Test Pages

1. **Learning Resources** (`/toolkit/learning/`)
   - Hero section
   - Books grid (4 columns max)
   - Videos grid (3 columns max)
   - Tutorials grid (2 columns max)
   - Cheatsheets grid (3 columns max)

2. **Software Tools** (`/toolkit/software/`)
   - Hero section
   - Languages grid (5 columns max)
   - IDEs grid (6 columns max)
   - Version Control grid (5 columns max)
   - Data Formats grid (6 columns max)
   - Notebooks grid (3 columns max)
   - Databases grid (5 columns max)

## Screenshot Locations

Before screenshots: `docs/ui_baseline_screenshots/`
After screenshots: `docs/ui_after_screenshots/`
