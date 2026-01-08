# Brain Viewer Design Token Mapping

This document maps the site's design tokens to the brain viewer components for the UI redesign.

## Site Design System Summary

### Color Palette

```css
/* Neutral (slate grays) */
--color-neutral-50:  #f8fafc   /* Lightest background */
--color-neutral-100: #f1f5f9   /* Subtle background */
--color-neutral-200: #e2e8f0   /* Border subtle */
--color-neutral-300: #cbd5e1   /* Border default */
--color-neutral-400: #94a3b8   /* Muted text (dark bg) */
--color-neutral-500: #64748b   /* Tertiary text */
--color-neutral-600: #475569   /* Secondary text */
--color-neutral-700: #334155   /* Dark border */
--color-neutral-800: #1e293b   /* Dark background */
--color-neutral-900: #0f172a   /* Darkest background */

/* Accent (teal/cyan) */
--color-accent-400: #22d3ee    /* Accent on dark backgrounds */
--color-accent-500: #06b6d4    /* Primary accent */
--color-accent-600: #0891b2    /* Accent text on light */
```

### Semantic Tokens (Light Mode)

```css
/* Text */
--color-text-primary:   var(--color-neutral-900)
--color-text-secondary: var(--color-neutral-600)
--color-text-muted:     var(--color-neutral-600)
--color-text-accent:    var(--color-accent-600)

/* Backgrounds */
--color-bg-surface:     #ffffff
--color-bg-subtle:      var(--color-neutral-50)

/* Borders */
--color-border-default: var(--color-neutral-300)
--color-border-subtle:  var(--color-neutral-200)
--color-border-accent:  var(--color-accent-500)

/* Focus */
--color-focus-ring:     var(--color-accent-500)
```

### Spacing Scale

```css
--spacing-1:  0.25rem   /* 4px  */
--spacing-2:  0.5rem    /* 8px  */
--spacing-3:  0.75rem   /* 12px */
--spacing-4:  1rem      /* 16px */
--spacing-6:  1.5rem    /* 24px */
--spacing-8:  2rem      /* 32px */
```

### Border Radius

```css
--radius-sm:   0.375rem  /* 6px  */
--radius-md:   0.5rem    /* 8px  */
--radius-lg:   0.75rem   /* 12px */
--radius-xl:   1rem      /* 16px */
--radius-2xl:  1.5rem    /* 24px */

/* Semantic */
--radius-container: var(--radius-2xl)
--radius-panel:     var(--radius-xl)
```

### Shadows

```css
--shadow-xs: 0 1px 2px 0 rgb(0 0 0 / 0.05)
--shadow-sm: 0 1px 3px 0 rgb(0 0 0 / 0.1)
--shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1)
```

### Card Tokens

```css
--card-radius:       var(--radius-2xl)
--card-padding:      var(--spacing-4)
--card-bg:           var(--color-bg-surface)
--card-border:       var(--color-border-default)
--card-border-hover: var(--color-accent-500)
--card-shadow:       var(--shadow-xs)
--card-shadow-hover: 0 10px 25px -5px rgb(0 0 0 / 0.1)
```

---

## Current Viewer Styling (viewer_app)

The viewer uses inline Tailwind classes. Here's what needs to change:

### Main Container (`brain_viewer.rs:1483`)
```rust
// Current
class="brain-viewer-container focus:outline-none focus:ring-2 focus:ring-blue-500"

// Target: Use site focus ring
class="brain-viewer-container focus:outline-none focus:ring-2 focus:ring-[var(--color-focus-ring)]"
```

### Control Panel (`control_panel.rs:139`)
```rust
// Current
class="w-64 bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden"

// Target: Use site card tokens
class="w-64 bg-[var(--color-bg-surface)] rounded-[var(--radius-panel)] shadow-[var(--shadow-sm)] border border-[var(--color-border-default)] overflow-hidden"
```

### Select/Dropdown Controls
```rust
// Current
class="w-full border border-gray-300 rounded px-2 py-1.5 text-sm focus:ring-1 focus:ring-blue-500 focus:border-blue-500"

// Target
class="w-full border border-[var(--color-border-default)] rounded-[var(--radius-md)] px-2 py-1.5 text-sm text-[var(--color-text-primary)] bg-[var(--color-bg-surface)] focus:ring-1 focus:ring-[var(--color-focus-ring)] focus:border-[var(--color-border-accent)]"
```

### Labels
```rust
// Current
class="block text-xs font-medium text-gray-600 mb-1"

// Target
class="block text-xs font-medium text-[var(--color-text-secondary)] mb-1"
```

### Buttons (Secondary Style)
```rust
// Current
class="px-2 py-1 text-xs border border-gray-300 rounded hover:bg-gray-100"

// Target: Match site's .btn-secondary
class="px-2 py-1 text-xs border border-[var(--color-border-default)] rounded-[var(--radius-md)] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)] hover:bg-[var(--color-bg-subtle)] hover:border-[var(--color-border-accent)] hover:text-[var(--color-text-accent)] transition-colors"
```

### Camera Preset Buttons
```rust
// Current
class="px-2 py-1 text-xs font-medium bg-gray-100 hover:bg-gray-200 rounded border border-gray-300 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"

// Target
class="px-2 py-1 text-xs font-medium bg-[var(--color-bg-subtle)] hover:bg-[var(--color-neutral-200)] rounded-[var(--radius-md)] border border-[var(--color-border-default)] text-[var(--color-text-primary)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
```

### Canvas Container (`brain_viewer.rs:1524`)
```rust
// Current
class="w-full h-96 border rounded-lg shadow-lg bg-gray-900"

// Target: Keep dark bg for contrast, but use site border
class="w-full h-96 border border-[var(--color-border-default)] rounded-[var(--radius-panel)] shadow-[var(--shadow-md)] bg-[var(--color-neutral-900)]"
```

### Right Info Panel (`brain_viewer.rs:1562`)
```rust
// Current
class="w-full lg:w-64 flex flex-col gap-4 mt-4 lg:mt-0 bg-white/95 text-gray-800 p-3 rounded-lg shadow-md border border-gray-200"

// Target
class="w-full lg:w-64 flex flex-col gap-4 mt-4 lg:mt-0 bg-[var(--color-bg-surface)]/95 text-[var(--color-text-primary)] p-[var(--spacing-3)] rounded-[var(--radius-panel)] shadow-[var(--shadow-md)] border border-[var(--color-border-default)]"
```

### Color Legend (`color_legend.rs:77`)
```rust
// Current
class="text-xs p-3 border rounded-lg bg-white shadow-sm"

// Target
class="text-xs p-[var(--spacing-3)] border border-[var(--color-border-default)] rounded-[var(--radius-lg)] bg-[var(--color-bg-surface)] shadow-[var(--shadow-sm)]"
```

### Collapsible Sections (`collapsible_section.rs:64-67`)
```rust
// Current
class="border-b border-gray-200 last:border-b-0"
class="w-full flex items-center justify-between py-2 px-1 text-left text-xs font-semibold text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:ring-inset"

// Target
class="border-b border-[var(--color-border-subtle)] last:border-b-0"
class="w-full flex items-center justify-between py-2 px-1 text-left text-xs font-semibold text-[var(--color-text-primary)] hover:bg-[var(--color-bg-subtle)] focus:outline-none focus:ring-1 focus:ring-[var(--color-focus-ring)] focus:ring-inset"
```

### Muted/Helper Text
```rust
// Current
class="text-gray-500" or class="text-gray-600"

// Target
class="text-[var(--color-text-muted)]"
```

### Primary Action Buttons (Export, etc.)
```rust
// Current (in brain_viewer_facade)
class="px-6 py-3 bg-emerald-600 hover:bg-emerald-700 text-white font-medium rounded-lg shadow-lg transition-colors"

// Target: Use site accent
class="px-6 py-3 bg-[var(--color-accent-500)] hover:bg-[var(--color-accent-600)] text-white font-medium rounded-[var(--radius-lg)] shadow-[var(--shadow-md)] transition-colors"
```

---

## Implementation Strategy

### Option A: CSS Custom Properties (Recommended)
Add CSS variables to viewer components that reference site tokens. This allows:
- Automatic theme switching (light/dark/dracula)
- Easy maintenance
- Single source of truth

### Option B: Tailwind Config Extension
Extend Tailwind config to include site's semantic tokens:
```js
// tailwind.config.js
theme: {
  extend: {
    colors: {
      surface: 'var(--color-bg-surface)',
      subtle: 'var(--color-bg-subtle)',
      // etc.
    }
  }
}
```

### Option C: Direct Class Replacement
Replace Tailwind classes with site's utility classes where they exist (e.g., `.btn-secondary`).

---

## Files to Modify

### In viewer_app (blmm_demo/crates/viewer_app/src/components/):
1. `brain_viewer.rs` - Main container, canvas, right panel
2. `control_panel.rs` - Left sidebar, all form controls
3. `collapsible_section.rs` - Section headers
4. `color_legend.rs` - Legend card
5. `camera_presets.rs` - Preset buttons
6. `roi_panel.rs` - ROI tools (to be hidden by default)
7. `vertex_info_panel.rs` - Vertex info display
8. `annotation_panel.rs` - Annotations (to be hidden by default)
9. `vertex_summary_table.rs` - Top 10 table

### In brain_viewer_facade (longitudinal-dev/crates/brain_viewer_facade/src/):
1. `lib.rs` - Fallback/placeholder styling, "Load viewer" button

---

## Rollback Points

Tag before each phase:
- `viewer-redesign-phase0-complete` - After token mapping
- `viewer-redesign-phase1-complete` - After theme alignment
- `viewer-redesign-phase2-complete` - After simplification
- etc.
