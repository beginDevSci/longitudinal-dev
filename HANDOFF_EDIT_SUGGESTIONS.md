# 🔄 HANDOFF: Edit Page Suggestions Feature

**Branch**: `feature/edit-page-suggestions`
**Status**: Steps 1-5 Complete ✅ | Steps 6-8 Remaining
**Last Updated**: 2025-11-18

---

## 📋 Quick Start for Next Session

```bash
git checkout feature/edit-page-suggestions
git log --oneline | head -5
# Should show:
# 4c2de5f Steps 3-5: Complete implementation with fixes
# c526579 Amend Step 2: Use rendered HTML for prefill and baseline hash
# aba4186 Step 2: Implement prefill generation during SSG
# f82313f Step 1: Add prop plumbing for edit page suggestions
```

---

## ✅ Completed Work (Steps 1-5)

### Step 1: Prop Plumbing ✅
**Commit**: `f82313f`

All components wired with necessary props:

- **EditorModalIsland** (`src/editor_modal_island.rs`)
  - Props: `slug`, `page_url`, `prefill_markdown`, `baseline_hash`
  - Signal state for notes and contact fields
  - Prefill logic uses prefill_markdown with localStorage fallback

- **EditPageButton** (`src/edit_page_button.rs`)
  - Prop: `slug`
  - Generates GitHub edit URL
  - Separate button and GitHub link (no nesting)

- **TableOfContents** (`src/layout/table_of_contents.rs`)
  - Passes slug to EditPageButton
  - Gated by `config::ENABLE_SUGGESTIONS`

- **PostLayout** (`src/layout/post_layout.rs:97-103, 306-317`)
  - Generates page_url
  - Receives prefill_markdown and baseline_hash from SSG
  - Passes all props to EditorModalIsland
  - Gated by `config::ENABLE_SUGGESTIONS`

---

### Step 2: Prefill Generation During SSG ✅
**Commits**: `aba4186`, `c526579`

**Implementation** (`src/main.rs:14-16, 148-176`):
- Added dependencies: `sha2`, `pulldown-cmark`
- Reads markdown from `content/tutorials/{slug}.md`
- Renders markdown to HTML using pulldown-cmark
  - Options: `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH`
- Generates SHA-256 hash of rendered HTML
- Passes both to PostLayout during SSG

**Key Files**:
- `Cargo.toml:52-53, 109-111` - Dependencies added
- `src/main.rs:148-176` - SSG generation loop
- `src/lib.rs:43-52` - App component hydration fallback

**Verification**:
```bash
# Hash of gee.md rendered content:
# 5551e2e30ace6706251b1119083489c5dd609d79c5066cdc118bc0c28c1bf002
```

---

### Steps 3-5: Frontend Implementation ✅
**Commit**: `4c2de5f`

#### Step 3: Enable EditPageButton
- ✅ Removed disabled state
- ✅ Added click handler for custom event "open-editor-modal"
- ✅ **GitHub link moved outside button** (was nested, now sibling)
- ✅ Valid HTML structure

**File**: `src/edit_page_button.rs:17-61`

#### Step 4: Modal UI Fields
- ✅ **Edits textarea** (required) - **NOW PREFILLED** with rendered HTML
- ✅ Notes textarea (optional)
- ✅ Contact input (optional)
- ✅ Honeypot field "website" (hidden, must be empty)
- ✅ Display slug and page_url
- ✅ Autosave with slug-specific keys: `edit_suggestion_{slug}_edits/notes/contact`
- ✅ Submit disabled when edits empty
- ✅ Prefill behavior:
  - If localStorage draft exists → use draft
  - Else → use prefill_markdown (rendered HTML from SSG)
  - Status message shows which source

**File**: `src/editor_modal_island.rs:53-88, 290-364`

#### Step 5: Submit Handler
- ✅ POST to `/api/suggestions`
- ✅ JSON payload:
  ```json
  {
    "slug": "...",
    "page_url": "...",
    "edits": "...",
    "notes": "...",
    "contact": "...",
    "baseline_hash": "...",
    "website": ""
  }
  ```
- ✅ Client-side validation:
  - Edits non-empty (trimmed)
  - Honeypot empty
- ✅ Loading state: "Submitting...", button disabled
- ✅ Success state: Clear form + localStorage, **auto-close after 2s**
- ✅ Error state: Show error, preserve draft
- ✅ Uses Action pattern (avoids FnOnce issues)

**File**: `src/editor_modal_island.rs:141-250, 408-415`

#### Step 7: Feature Flag (completed early)
- ✅ Created `src/config.rs`
- ✅ `ENABLE_SUGGESTIONS` constant (default: `true`)
- ✅ Gates EditPageButton visibility
- ✅ Gates EditorModalIsland rendering

**Files**:
- `src/config.rs:1-9`
- `src/lib.rs:6`
- `src/layout/table_of_contents.rs:110-118`
- `src/layout/post_layout.rs:306-317`

---

## 🚧 Remaining Work (Steps 6-8)

### Step 6: Cloudflare Worker (Not Started)

**Location**: `workers/suggestions/` (to be created)

**Requirements**:

1. **Route**: `POST /api/suggestions`

2. **Validation**:
   - Origin check (only allow from your domain)
   - Honeypot: `website` field must be empty
   - Size limit: ≤ 50 KB
   - Required: `edits` field non-empty

3. **Rate Limiting**:
   - 10 submissions per IP per hour
   - Consider using Cloudflare KV or Durable Objects

4. **File Storage**:
   - Path: `suggestions/{slug}/{timestamp}-{short-hash}.md`
   - Format:
     ```markdown
     slug: {slug}
     page_url: {url}
     submitted_at: {iso8601}
     contact: {optional}
     baseline_hash: {optional}
     user_agent: {from Worker}
     ---
     ## Suggested changes
     {edits}

     ## Notes
     {notes}
     ```

5. **GitHub Integration**:
   - **Option A (Recommended)**: Create GitHub Issue
     - POST to `/repos/swhawes/leptos-test/issues`
     - Label: `suggestion`
     - Body: formatted suggestion
   - **Option B**: Direct commit to `suggestions/` branch
     - PUT to `/repos/swhawes/leptos-test/contents/{path}`
     - Commit message: `chore: suggestion {slug} {timestamp}`

6. **Secrets Needed**:
   - `GITHUB_TOKEN` with `contents:write` and `issues:write` scopes

7. **Response**:
   - Success: `{"ok": true}`
   - Error: `{"error": "descriptive message"}`

**Deliverables**:
- `workers/suggestions/wrangler.toml`
- `workers/suggestions/src/index.ts` (or `.js`)
- `workers/suggestions/README.md` (setup instructions)
- Secret configuration notes

---

### Step 8: Testing (Not Started)

**Unit Tests**:
- [ ] Prop propagation in components
- [ ] Modal open/close functionality
- [ ] Submit button disabled state logic
- [ ] Autosave to localStorage
- [ ] Prefill vs. draft priority

**Integration Tests**:
- [ ] Worker validation (honeypot, size limits)
- [ ] Rate limiting behavior
- [ ] GitHub API integration
- [ ] Error handling and responses

**E2E Testing**:
- [ ] Open tutorial page
- [ ] Click "Suggest edit" button
- [ ] Verify modal opens with prefilled content
- [ ] Edit content, add notes/contact
- [ ] Submit suggestion
- [ ] Verify success message and auto-close
- [ ] Check GitHub (issue or file in `suggestions/`)

---

## 📁 Key Files Reference

### Frontend

| File | Purpose | Lines of Interest |
|------|---------|-------------------|
| `src/config.rs` | Feature flag | 9 (ENABLE_SUGGESTIONS) |
| `src/edit_page_button.rs` | Trigger button | 17-61 (click handler, GitHub link) |
| `src/editor_modal_island.rs` | Modal UI & logic | 53-88 (prefill), 141-250 (submit) |
| `src/layout/post_layout.rs` | Props integration | 97-103 (gen props), 306-317 (render) |
| `src/main.rs` | SSG generation | 148-176 (read & hash markdown) |
| `Cargo.toml` | Dependencies | 52-53, 109-111 (sha2, pulldown-cmark) |

### Backend (To Be Created)

| File | Purpose | Status |
|------|---------|--------|
| `workers/suggestions/wrangler.toml` | Cloudflare config | ❌ Not created |
| `workers/suggestions/src/index.ts` | Worker logic | ❌ Not created |
| `workers/suggestions/README.md` | Setup docs | ❌ Not created |

---

## 🏗️ Build & Test Commands

```bash
# Development build
make build

# Static site generation
make ssg

# Lint
make clippy

# Clean
make clean
```

**Current Build Status**:
- ✅ SSR build succeeds
- ✅ SSG build succeeds
- ✅ WASM: 263.6K Brotli compressed
- ⚠️ Expected warnings for unused variables in SSR-only code

---

## 🔧 Configuration

### Enable/Disable Feature

Edit `src/config.rs`:
```rust
pub const ENABLE_SUGGESTIONS: bool = false; // Disable feature
```

Then rebuild:
```bash
make ssg
```

### Modify Suggestion Endpoint

Edit `src/editor_modal_island.rs:203`:
```rust
let request = leptos::web_sys::Request::new_with_str_and_init(
    "/api/suggestions", // ← Change this
    &opts
)?;
```

---

## 🎯 Design Decisions Locked In

1. **Prefill Content**: Rendered HTML (not raw markdown)
2. **Storage**: `suggestions/{slug}/` or GitHub Issues
3. **Rate Limits**: 50 KB max, 10 submissions/IP/hour
4. **Honeypot**: Field named `website`, must be empty
5. **No Email Notifications**: GitHub issues/files only
6. **GitHub Edit**: `https://github.com/swhawes/leptos-test/edit/main/content/tutorials/{slug}.md`
7. **Auto-close**: Modal closes 2s after success
8. **Prefill Priority**: localStorage draft > prefill_markdown

---

## 📊 Current State Summary

### What Works ✅
- Button opens modal via custom event
- Modal displays with prefilled rendered HTML
- Slug-specific localStorage autosave
- Form validation (edits required, honeypot check)
- Submit button state management
- Success/error messaging
- Auto-close on success
- Feature flag for quick disable
- Clean separation of button and GitHub link

### What's Missing ❌
- Cloudflare Worker (Step 6)
- Backend validation and storage
- GitHub integration
- Rate limiting implementation
- Comprehensive testing (Step 8)

### Dependencies Added
- `sha2 = "0.10"` (SSR only)
- `pulldown-cmark = "0.11.3"` (SSR only)
- `wasm-bindgen-futures = "0.4"` (WASM only)
- Web-sys features: Request, RequestInit, RequestMode, Response, Headers

---

## 🚀 Next Session Instructions

**To continue this work:**

1. **Checkout the branch**:
   ```bash
   cd /Users/shawes/git/swhawes/leptos-test/longitudinal-dev
   git checkout feature/edit-page-suggestions
   git pull origin feature/edit-page-suggestions  # if working across machines
   ```

2. **Verify current state**:
   ```bash
   git log --oneline | head -3
   # Should show: 4c2de5f Steps 3-5: Complete implementation with fixes

   make build  # Should succeed with expected warnings
   ```

3. **Begin Step 6**: Create Cloudflare Worker
   ```bash
   mkdir -p workers/suggestions
   cd workers/suggestions
   # Follow Cloudflare Workers setup
   ```

4. **Reference this document**: All design decisions and implementation details are here

---

## 📝 Important Notes

- **DO NOT** push to main until Step 8 (testing) is complete
- **Feature flag** allows safe merging if needed (set to `false`)
- **Prefill content** is rendered HTML, not raw markdown (important for UX)
- **baseline_hash** reflects the rendered content hash
- **GitHub link** is now properly separated from button (valid HTML)
- **Auto-close** provides better UX after successful submission

---

## 🔗 Related Documentation

- Original handoff summary: Start of this file
- Cloudflare Workers: https://developers.cloudflare.com/workers/
- GitHub API (Issues): https://docs.github.com/en/rest/issues/issues
- GitHub API (Contents): https://docs.github.com/en/rest/repos/contents

---

**Questions or Issues?**
Refer to commit messages for detailed context:
- `git show 4c2de5f` - Steps 3-5 implementation
- `git show c526579` - Rendered HTML for prefill
- `git show aba4186` - Initial SSG implementation
- `git show f82313f` - Prop plumbing

**Ready to proceed with Step 6!** 🚀
