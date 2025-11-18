# 🔄 HANDOFF: Edit Page Suggestions Feature

**Branch**: `feature/edit-page-suggestions`
**Status**: Steps 1-6 Complete ✅ | Steps 7-8 Remaining (Deployment & Testing)
**Last Updated**: 2025-11-18

---

## 📋 Quick Summary

**What's Complete:**
- ✅ Frontend: Modal UI, validation, autosave, prefill from SSG
- ✅ Backend: Cloudflare Worker with validation, rate limiting, GitHub Issues integration
- ✅ Infrastructure: Feature flag, documentation, test scripts

**What's Remaining:**
- ⏳ Deploy Worker to Cloudflare (requires account)
- ⏳ Update frontend with Worker URL
- ⏳ End-to-end testing
- ⏳ Comprehensive test suite (Step 8)

**To Continue:**
1. Deploy Worker: `cd workers/suggestions && npm install && npm run deploy`
2. Get Worker URL from deployment output
3. Update `src/editor_modal_island.rs:210` with Worker URL
4. Rebuild frontend: `make ssg`
5. Test end-to-end from live site

---

## 📋 Quick Start for Next Session

```bash
git checkout feature/edit-page-suggestions
git log --oneline | head -6
# Should show:
# XXXXXX Step 6: Implement Cloudflare Worker for suggestions API
# e392981 Add comprehensive handoff documentation
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
**Commits**: `aba4186`, `c526579`, Updated

**Implementation** (`src/main.rs:14-16, 148-169`):
- Added dependency: `sha2`
- Reads raw markdown from `content/tutorials/{slug}.md`
- Generates SHA-256 hash of raw markdown content
- Passes markdown and hash to PostLayout during SSG
- **Changed**: Now uses raw markdown instead of rendered HTML for better editing UX

**Key Files**:
- `Cargo.toml:52, 115` - SHA-256 dependency
- `src/main.rs:148-169` - SSG generation loop (reads markdown, hashes it)
- `src/lib.rs:43-52` - App component hydration fallback
- `src/editor_modal_island.rs:73` - Status message updated to "Showing page markdown"

**Rationale for Markdown**:
- Users can edit in familiar markdown syntax
- Easier to see structure and make changes
- Baseline hash tracks markdown content changes
- More intuitive than editing rendered HTML

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
- ✅ **Edits textarea** (required) - **NOW PREFILLED** with raw markdown
- ✅ Notes textarea (optional)
- ✅ Contact input (optional)
- ✅ Honeypot field "website" (hidden, must be empty)
- ✅ Display slug and page_url
- ✅ Autosave with slug-specific keys: `edit_suggestion_{slug}_edits/notes/contact`
- ✅ Submit disabled when edits empty
- ✅ Prefill behavior:
  - If localStorage draft exists → use draft
  - Else → use prefill_markdown (raw markdown from SSG)
  - Status message shows which source ("Showing page markdown - edit as needed")

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

---

### Step 6: Cloudflare Worker ✅
**Status**: Complete (Not Deployed)

**Implementation** (`workers/suggestions/`):

✅ **All files created:**
- `src/index.ts` - Main Worker implementation (270 lines)
- `wrangler.toml` - Cloudflare configuration
- `package.json` - Dependencies (Wrangler, TypeScript, Octokit)
- `tsconfig.json` - TypeScript configuration
- `README.md` - Comprehensive setup guide (400+ lines)
- `.gitignore` - Excludes node_modules, .dev.vars, .wrangler
- `test-local.sh` - Local testing script

✅ **Features implemented:**
1. **CORS support** - Preflight handling, origin validation
2. **Request validation**:
   - Honeypot check (`website` field must be empty)
   - Required fields: `slug`, `page_url`, `edits`
   - Slug format: `^[a-z0-9-]+$`
   - Size limit: 50 KB (51,200 bytes)
3. **Rate limiting**:
   - 10 submissions per IP per hour
   - KV storage with rolling window
   - 2-hour TTL on rate limit data
4. **GitHub Issues integration**:
   - Creates issue with title `[Suggestion] {slug}`
   - Labels: `suggestion`, `user-submitted`
   - Formatted markdown body with all fields
5. **Error handling**:
   - Proper HTTP status codes
   - Descriptive error messages
   - CORS headers on all responses

**Key Files**:
- `workers/suggestions/src/index.ts:1-270` - Worker logic
- `workers/suggestions/wrangler.toml:1-20` - Configuration
- `workers/suggestions/README.md:1-400+` - Setup documentation

**Deployment Steps** (from README):
1. Install dependencies: `npm install`
2. Create KV namespace: `wrangler kv:namespace create "RATE_LIMIT"`
3. Update `wrangler.toml` with KV namespace IDs
4. Create GitHub token with `repo` scope
5. Set secret: `wrangler secret put GITHUB_TOKEN`
6. Deploy: `npm run deploy`
7. Update frontend with Worker URL (see Step 7 below)

**Next Actions**:
- [ ] Deploy Worker to Cloudflare (requires Cloudflare account)
- [ ] Set up KV namespace
- [ ] Configure GitHub token secret
- [ ] Get Worker URL from deployment
- [ ] Update frontend to use Worker URL

---

## 🚧 Remaining Work (Steps 7-8)

### Step 7: Frontend Integration (Not Started)

**After deploying the Worker**, update the frontend to use the Worker URL:

**File to modify**: `src/editor_modal_island.rs:210`

**Current code**:
```rust
let request = leptos::web_sys::Request::new_with_str_and_init("/api/suggestions", &opts)?;
```

**Update to**:
```rust
// Option A: Use your Worker URL directly
let request = leptos::web_sys::Request::new_with_str_and_init(
    "https://suggestions-api.YOUR-SUBDOMAIN.workers.dev/api/suggestions",
    &opts
)?;

// Option B: Use custom route (if configured in Cloudflare)
let request = leptos::web_sys::Request::new_with_str_and_init("/api/suggestions", &opts)?;
```

**Also update CORS origin** in `workers/suggestions/wrangler.toml`:
```toml
ALLOWED_ORIGIN = "https://swhawes.github.io"
```

**Testing checklist**:
- [ ] Deploy Worker and get URL
- [ ] Update frontend with Worker URL
- [ ] Rebuild and deploy frontend: `make ssg`
- [ ] Test end-to-end from live site
- [ ] Verify GitHub issue created
- [ ] Test rate limiting (11+ submissions)
- [ ] Test honeypot protection
- [ ] Test modal auto-close on success

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

### Backend (Cloudflare Worker)

| File | Purpose | Status |
|------|---------|--------|
| `workers/suggestions/wrangler.toml` | Cloudflare config | ✅ Created |
| `workers/suggestions/src/index.ts` | Worker logic | ✅ Created (270 lines) |
| `workers/suggestions/package.json` | Dependencies | ✅ Created |
| `workers/suggestions/tsconfig.json` | TypeScript config | ✅ Created |
| `workers/suggestions/README.md` | Setup docs | ✅ Created (400+ lines) |
| `workers/suggestions/test-local.sh` | Test script | ✅ Created |
| `workers/suggestions/.gitignore` | Git ignore | ✅ Created |

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

1. **Prefill Content**: Raw markdown (for user-friendly editing)
2. **Baseline Hash**: SHA-256 of raw markdown content
3. **Storage**: GitHub Issues (recommended) or `suggestions/{slug}/` branch
4. **Rate Limits**: 50 KB max, 10 submissions/IP/hour
5. **Honeypot**: Field named `website`, must be empty
6. **No Email Notifications**: GitHub issues/files only
7. **GitHub Edit**: `https://github.com/swhawes/leptos-test/edit/main/content/tutorials/{slug}.md`
8. **Auto-close**: Modal closes 2s after success
9. **Prefill Priority**: localStorage draft > prefill_markdown

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
