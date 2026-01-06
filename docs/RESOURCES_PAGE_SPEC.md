# R Resources Page Specification

> Extracted from the original Nuxt prototype at `/Users/shawes/git/beginDevSci/longitudinal-dev/`

---

## 1. Scope & Explicit Non-Goals

### In Scope

- R learning resources only (books, videos, tutorials, cheatsheets)
- Browse-first catalog (four sections, anchor navigation)
- Faithful reproduction of prototype content and layout intent
- Maintainable, text-first card design

### Explicit Non-Goals

- Tools page (not part of this effort)
- Learning paths or "Start Here" flows
- New categories beyond the four in the prototype
- Difficulty levels, sorting, filtering, or recommendation systems
- Tag taxonomies or search functionality

---

## 2. Page Information Architecture

### Section Order (From Prototype)

1. **Header** — Page title and brief description
2. **Navigation** — Horizontal anchor links
3. **Books** — 4-column responsive grid
4. **Videos** — 3-column responsive grid
5. **Tutorials** — 3-column responsive grid
6. **Cheatsheets** — Grid layout (section exists in prototype, content provisional)

### Navigation Pattern

```
[Books]  [Videos]  [Tutorials]  [Cheatsheets]
```

- Anchor links only (`#books`, `#videos`, `#tutorials`, `#cheatsheets`)
- No sticky behavior
- No floating action buttons
- No scroll-to-top buttons

### Grid Layouts (From Prototype)

| Section | Mobile | Tablet | Desktop | Large |
|---------|--------|--------|---------|-------|
| Books | 1 col | 2 col | 3 col | 4 col |
| Videos | 1 col | 2 col | 3 col | 3 col |
| Tutorials | 1 col | 2 col | 3 col | 3 col |
| Cheatsheets | 1 col | 2 col | 3 col | 3 col |

---

## 3. Card Types & Required Fields

### BookCard

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Book title |
| `author` | Yes | Author name(s) |
| `url` | Yes | External link (opens new tab) |
| `blurb` | Yes | 1-2 sentence description |
| `image` | No | Cover image path (text-first design must work without) |

### VideoCard

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Video/playlist title |
| `source` | Yes | Channel or institution name |
| `url` | Yes | YouTube link (not embed) |
| `blurb` | Yes | 1-2 sentence description |

### TutorialCard

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Tutorial or platform name |
| `url` | Yes | External link |
| `blurb` | Yes | 1-2 sentence description |
| `platform` | No | Platform name if distinct from title |

### CheatsheetCard

| Field | Required | Notes |
|-------|----------|-------|
| `title` | Yes | Cheatsheet name |
| `url` | Yes | PDF or page link |
| `blurb` | Yes | Brief description |
| `format` | No | File format (e.g., "pdf") |

---

## 4. Content Rules

### Annotations

- 1-2 sentences maximum
- Neutral, academic tone
- Describe what the resource covers, not why it's good
- No superlatives ("best", "essential", "must-read")

### Ordering

- No implied difficulty progression
- No recommendation hierarchy
- List order matches prototype source order
- Future additions append to end of section

### Authority

- Content from `resources.vue` is authoritative
- Cheatsheets section: prototype has placeholder only; content in data file is a **provisional seed set** (not authoritative)

---

## 5. Data File Format

### Location

`content/resources.yaml`

### Schema

```yaml
books:
  - title: string       # Required
    author: string      # Required
    url: string         # Required
    blurb: string       # Required (1-2 sentences)
    image: string       # Optional (filename only)

videos:
  - title: string       # Required
    source: string      # Required
    url: string         # Required
    blurb: string       # Required

tutorials:
  - title: string       # Required
    url: string         # Required
    blurb: string       # Required
    platform: string    # Optional

cheatsheets:
  - title: string       # Required
    url: string         # Required
    blurb: string       # Required
    format: string      # Optional
```

### Example Entry

```yaml
books:
  - title: "R for Data Science"
    author: "Hadley Wickham & Garrett Grolemund"
    url: "https://r4ds.hadley.nz/"
    blurb: "Data science with R, covering the entire data analysis workflow."
    image: "r-for-data-science-cover.jpg"
```

---

## 6. UI Alignment Checklist

### Preserve

- [ ] Four-section taxonomy from prototype
- [ ] Section titles ("Open Source R Books", etc.)
- [ ] Card information density (title + blurb + link)
- [ ] Responsive grid breakpoints
- [ ] External links open in new tab

### Constraints

- [ ] Use existing site design tokens
- [ ] Do not introduce prototype-only colors or fonts
- [ ] Text-first cards must render without images
- [ ] No animations beyond standard hover states
- [ ] No glassmorphism or glow effects

### Images

- Images are optional enhancement, not required
- Implementation must not block on missing cover assets
- Cards degrade gracefully to text-only

---

## 7. Card Schema Guardrails

### Do Not Add

- Difficulty levels
- Learning order or sequence numbers
- Recommendation flags or ratings
- Sorting or filtering logic
- Tag systems (beyond single optional lightweight field)
- Embed URLs for videos (link only)

### Keep Minimal

Schemas match prototype card props exactly. Any field not in prototype components should not be added without explicit approval.

---

## Appendix A: Content Inventory

### Books (12 items, from prototype)

| Title | Author | URL |
|-------|--------|-----|
| R for Data Science | Hadley Wickham & Garrett Grolemund | https://r4ds.hadley.nz/ |
| Advanced R | Hadley Wickham | https://adv-r.hadley.nz/ |
| Efficient R Programming | Colin Gillespie & Robin Lovelace | https://csgillespie.github.io/efficientR/ |
| R Programming for Data Science | Roger D. Peng | https://bookdown.org/rdpeng/RProgDA/ |
| Hands-On Programming with R | Garrett Grolemund | https://rstudio-education.github.io/hopr/ |
| R Inferno | Patrick Burns | https://www.burns-stat.com/pages/Tutor/R_inferno.pdf |
| Software for Data Analysis with R | John M. Chambers | https://bookdown.org/rdpeng/RProgDA/ |
| The Pirate's Guide to R (YaRrr!) | Nathaniel D. Phillips | https://bookdown.org/ndphillips/YaRrr/ |
| The Art of R Programming | Norman Matloff | https://web.cs.ucdavis.edu/~matloff/matloff/public_html/145/PLN/RMaterials/NSPpart.pdf |
| Deep R Programming | Roger D. Peng | https://deepr.gagolewski.com/ |
| R Cookbook, 2nd Edition | James (JD) Long | https://rc2e.com/ |
| ggplot2: Elegant Graphics for Data Analysis (3e) | Hadley Wickham | https://ggplot2-book.org/ |

### Videos (6 items, from prototype)

| Title | Source | URL |
|-------|--------|-----|
| R Programming Tutorial | freeCodeCamp.org | https://www.youtube.com/watch?v=_V8eKsto3Ug |
| CS50's Introduction to Programming with R | Harvard/CS50 | https://www.youtube.com/playlist?list=PLhQjrBD2T382yfNp_-xzX244d-O9W6YmD |
| R Programming For Beginners | R Programming 101 | https://www.youtube.com/playlist?list=PLtL57Fdbwb_Chn-dNR0qBjH3esKS2MXY3 |
| An R Tutorial For Beginners | Simplilearn | https://www.youtube.com/playlist?list=PLm-R300a1uRfH-cy2-KuOD7gHzSRFnX-h |
| Full data analyses with R | Equitable Equations | https://www.youtube.com/playlist?list=PLKBUk9FL4nBbnAFHU3AZfsGHShsMC2AJC |
| Statistical Learning with R | Stanford | https://www.youtube.com/playlist?list=PLoROMvodv4rOzrYsAxzQyHb8n_RWNuS1e |

### Tutorials (4 items, from prototype)

| Title | URL |
|-------|-----|
| DataCamp R Programming Tutorials | https://www.datacamp.com/tutorial/category/r-programming |
| {swirl} Learn R, in R | https://swirlstats.com/ |
| HarvardX: CS50's Introduction to Programming with R | https://cs50.harvard.edu/r/2024/ |
| fasteR | https://github.com/matloff/fasteR |

### Cheatsheets (6 items, provisional seed set)

> **Note:** The prototype contains a placeholder section for cheatsheets with no content data. The following is a provisional seed set based on commonly referenced R cheatsheets. These are not authoritative and may be revised.

| Title | URL |
|-------|-----|
| Base R Cheatsheet | https://rstudio.github.io/cheatsheets/base-r.pdf |
| Data Transformation with dplyr | https://rstudio.github.io/cheatsheets/data-transformation.pdf |
| Data Visualization with ggplot2 | https://rstudio.github.io/cheatsheets/data-visualization.pdf |
| R Markdown Cheatsheet | https://rstudio.github.io/cheatsheets/rmarkdown.pdf |
| Shiny Cheatsheet | https://rstudio.github.io/cheatsheets/shiny.pdf |
| String Manipulation with stringr | https://rstudio.github.io/cheatsheets/strings.pdf |

---

## Appendix B: Implementation-Agnostic Data Model

The data model maps 1:1 to card props from the prototype components.

```
Resource
├── title: String (required)
├── url: String (required)
├── blurb: String (required, 1-2 sentences)
└── [type-specific fields]

Book extends Resource
├── author: String (required)
└── image: String (optional)

Video extends Resource
└── source: String (required)

Tutorial extends Resource
└── platform: String (optional)

Cheatsheet extends Resource
└── format: String (optional)
```

Implementation may use any format (YAML, JSON, embedded structs) as long as the schema is preserved.

---

*Specification extracted from Nuxt prototype — January 2026*
