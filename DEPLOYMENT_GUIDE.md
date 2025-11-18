# Step 7: Deployment & Integration Guide

Complete step-by-step guide to deploy the Cloudflare Worker and integrate it with your frontend.

---

## Prerequisites

Before starting, ensure you have:

- [ ] **Cloudflare Account** (free tier is sufficient)
  - Sign up at: https://dash.cloudflare.com/sign-up
- [ ] **Node.js 18+** installed
  - Check: `node --version`
- [ ] **npm** installed
  - Check: `npm --version`
- [ ] **GitHub Account** (you already have this)
- [ ] **Wrangler CLI** installed globally
  - Install: `npm install -g wrangler`
  - Check: `wrangler --version`

---

## Part 1: Deploy Cloudflare Worker

### Step 1.1: Authenticate Wrangler with Cloudflare

```bash
# Log in to Cloudflare (opens browser for OAuth)
wrangler login

# This will:
# 1. Open your browser
# 2. Ask you to log in to Cloudflare
# 3. Grant wrangler permission to manage your Workers
```

**Expected output:**
```
Successfully logged in.
```

---

### Step 1.2: Create KV Namespaces

```bash
# Navigate to the Worker directory
cd workers/suggestions

# Create production KV namespace
wrangler kv:namespace create "RATE_LIMIT"

# Create preview KV namespace (for local testing)
wrangler kv:namespace create "RATE_LIMIT" --preview
```

**Expected output:**
```
🌀 Creating namespace with title "suggestions-api-RATE_LIMIT"
✨ Success!
Add the following to your configuration file in your kv_namespaces array:
{ binding = "RATE_LIMIT", id = "abc123def456..." }

🌀 Creating namespace with title "suggestions-api-RATE_LIMIT_preview"
✨ Success!
Add the following to your configuration file in your kv_namespaces array:
{ binding = "RATE_LIMIT", preview_id = "xyz789abc123..." }
```

**⚠️ IMPORTANT:** Copy these IDs! You'll need them in the next step.

---

### Step 1.3: Update wrangler.toml with KV IDs

Open `workers/suggestions/wrangler.toml` and replace the placeholders:

**Before:**
```toml
[[kv_namespaces]]
binding = "RATE_LIMIT"
id = "YOUR_KV_NAMESPACE_ID"              # Replace with production KV namespace ID
preview_id = "YOUR_PREVIEW_KV_NAMESPACE_ID"  # Replace with preview KV namespace ID
```

**After:** (using your actual IDs from Step 1.2)
```toml
[[kv_namespaces]]
binding = "RATE_LIMIT"
id = "abc123def456..."              # Your production ID
preview_id = "xyz789abc123..."      # Your preview ID
```

Save the file.

---

### Step 1.4: Install Worker Dependencies

```bash
# Make sure you're in workers/suggestions
cd workers/suggestions

# Install dependencies
npm install
```

**Expected output:**
```
added 2 packages, and audited 3 packages in 2s
```

---

### Step 1.5: Create GitHub Personal Access Token

1. Go to: https://github.com/settings/tokens
2. Click **"Generate new token"** → **"Generate new token (classic)"**
3. Give it a name: `Suggestions API Worker`
4. Set expiration: `No expiration` (or 1 year if you prefer)
5. Select scopes:
   - ✅ **`repo`** (Full control of private repositories)
     - This gives access to create issues
6. Click **"Generate token"**
7. **⚠️ COPY THE TOKEN IMMEDIATELY** - You won't see it again!
   - It looks like: `ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`

---

### Step 1.6: Set GitHub Token as Secret

```bash
# Still in workers/suggestions directory
wrangler secret put GITHUB_TOKEN
```

**Prompt will appear:**
```
Enter a secret value: █
```

**Paste your GitHub token** (it won't display as you type) and press Enter.

**Expected output:**
```
🌀 Creating the secret for the Worker "suggestions-api"
✨ Success! Uploaded secret GITHUB_TOKEN
```

---

### Step 1.7: Deploy the Worker

```bash
# Deploy to Cloudflare
npm run deploy

# Or directly with wrangler
wrangler deploy
```

**Expected output:**
```
Total Upload: 15.xx KiB / gzip: 5.xx KiB
Uploaded suggestions-api (2.xx sec)
Published suggestions-api (0.xx sec)
  https://suggestions-api.YOUR-SUBDOMAIN.workers.dev
Current Deployment ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

**⚠️ IMPORTANT:** Copy your Worker URL! You'll need it in Part 2.

Example: `https://suggestions-api.your-account.workers.dev`

---

### Step 1.8: Test the Worker

```bash
# Test with the provided script
./test-local.sh https://suggestions-api.YOUR-SUBDOMAIN.workers.dev https://swhawes.github.io

# Or test with curl
curl -X POST "https://suggestions-api.YOUR-SUBDOMAIN.workers.dev/api/suggestions" \
  -H "Content-Type: application/json" \
  -H "Origin: https://swhawes.github.io" \
  -d '{
    "slug": "test-deployment",
    "page_url": "https://swhawes.github.io/longitudinal-dev/posts/test/",
    "edits": "Testing Worker deployment",
    "notes": "This is a test from deployment",
    "contact": "test@example.com",
    "baseline_hash": "test123",
    "website": ""
  }'
```

**Expected response:**
```json
{"ok":true}
```

**Verify:** Check your GitHub repository for a new issue with label `suggestion`!
- Go to: https://github.com/swhawes/leptos-test/issues
- You should see: `[Suggestion] test-deployment`

---

## Part 2: Update Frontend with Worker URL

### Step 2.1: Update the API Endpoint

Navigate back to the project root:

```bash
cd /Users/shawes/git/swhawes/leptos-test/longitudinal-dev
```

**Option A: Use Worker URL Directly (Recommended for testing)**

Edit `src/editor_modal_island.rs` around line 210:

**Before:**
```rust
let request = leptos::web_sys::Request::new_with_str_and_init("/api/suggestions", &opts)?;
```

**After:** (replace with YOUR Worker URL)
```rust
let request = leptos::web_sys::Request::new_with_str_and_init(
    "https://suggestions-api.YOUR-SUBDOMAIN.workers.dev/api/suggestions",
    &opts
)?;
```

**Option B: Use Environment Variable (Recommended for production)**

If you want to keep the code cleaner, add to `src/config.rs`:

```rust
/// API endpoint for suggestions
pub const SUGGESTIONS_API_URL: &str = env!("SUGGESTIONS_API_URL", "https://suggestions-api.YOUR-SUBDOMAIN.workers.dev/api/suggestions");
```

Then in `src/editor_modal_island.rs`:

```rust
let request = leptos::web_sys::Request::new_with_str_and_init(
    crate::config::SUGGESTIONS_API_URL,
    &opts
)?;
```

**For now, use Option A** (direct URL) - it's simpler for testing.

---

### Step 2.2: Rebuild the Frontend

```bash
# Clean previous build
make clean

# Build the static site with the new Worker URL
make ssg
```

**Expected output:**
```
✅ SSG build complete! Output in dist/
📊 Asset sizes:
  WASM assets:
    blog_bg.wasm.br          263.5K  — Brotli compressed
  ...
  Tutorial pages:
    16 tutorials totaling 2.3M
```

---

### Step 2.3: Test Locally (Optional)

Before deploying to GitHub Pages, test locally:

```bash
# Serve the dist directory
cd dist
python3 -m http.server 8000
```

Open browser to: http://localhost:8000

1. Navigate to any tutorial page
2. Click **"Suggest edit"** button
3. Modal should open with markdown content
4. Fill in some test edits
5. Click **"Submit Suggestion"**
6. Should see success message and modal closes after 2s
7. Check GitHub for new issue!

**Note:** CORS may block this if `ALLOWED_ORIGIN` is set to production. To test locally, you'd need to temporarily change `ALLOWED_ORIGIN` in `wrangler.toml` to `http://localhost:8000` and redeploy the Worker.

---

### Step 2.4: Deploy to GitHub Pages

```bash
# Make sure you're in the project root
cd /Users/shawes/git/swhawes/leptos-test/longitudinal-dev

# Commit the frontend changes
git add src/editor_modal_island.rs
git commit -m "Update suggestions API to use deployed Worker URL

Connect frontend to Cloudflare Worker endpoint for production use.

Worker URL: https://suggestions-api.YOUR-SUBDOMAIN.workers.dev
"

# Deploy to GitHub Pages (adjust based on your deployment method)
# Option 1: If you have a deploy script
./deploy.sh

# Option 2: Manual deployment
# Copy dist/ contents to your GitHub Pages repository
# OR push to gh-pages branch
git subtree push --prefix dist origin gh-pages

# Option 3: If using GitHub Actions, just push
git push origin feature/edit-page-suggestions
```

---

## Part 3: End-to-End Testing

### Step 3.1: Test from Live Site

1. Go to your live site: https://swhawes.github.io/longitudinal-dev/
2. Navigate to any tutorial (e.g., `/posts/gee/`)
3. Look for **"Suggest edit"** button in the sidebar
4. Click the button
5. Modal should open with markdown content prefilled
6. Make a test edit in the markdown
7. Add optional notes and contact info
8. Click **"Submit Suggestion"**
9. Wait for success message
10. Modal should auto-close after 2 seconds

### Step 3.2: Verify GitHub Issue Created

1. Go to: https://github.com/swhawes/leptos-test/issues
2. Look for new issue with title: `[Suggestion] {slug}`
3. Verify it has labels: `suggestion`, `user-submitted`
4. Check the issue body contains:
   - Page URL
   - Slug
   - Timestamp
   - Your suggested edits
   - Optional notes and contact

### Step 3.3: Test Rate Limiting

1. Submit 10 suggestions quickly from the same browser
2. On the 11th attempt, you should get an error:
   - "Rate limit exceeded. Please try again later."
3. Wait 1 hour and try again (should work)

### Step 3.4: Test Honeypot Protection

**This is an internal test - users won't see this field:**

1. Open browser DevTools (F12)
2. Open the suggestion modal
3. In console, run:
   ```javascript
   document.querySelector('input[name="website"]').value = "spam";
   ```
4. Try to submit
5. Should fail with "Invalid submission"

---

## Part 4: Monitoring & Maintenance

### Monitor Worker Logs

```bash
# View live logs from the Worker
cd workers/suggestions
npm run tail

# Or with wrangler
wrangler tail suggestions-api
```

This shows real-time logs of all requests hitting your Worker.

### Check Rate Limiting Data

```bash
# List all rate limit keys
wrangler kv:key list --binding RATE_LIMIT

# Check a specific IP's rate limit
wrangler kv:key get "ratelimit:1.2.3.4" --binding RATE_LIMIT
```

### View Worker Metrics

1. Go to: https://dash.cloudflare.com/
2. Click **Workers & Pages**
3. Click **suggestions-api**
4. Click **Metrics** tab
5. View:
   - Requests per second
   - Error rate
   - Success rate
   - Duration (p50, p99)

### Update Worker

If you need to make changes to the Worker:

```bash
cd workers/suggestions

# Edit src/index.ts
# ...make your changes...

# Deploy updated version
npm run deploy
```

**Frontend doesn't need to be rebuilt** unless you change the API contract.

---

## Part 5: Optional Configuration

### Custom Route (Advanced)

Instead of using `suggestions-api.YOUR-SUBDOMAIN.workers.dev`, you can configure a custom route.

**If you own a domain managed by Cloudflare:**

1. Add to `wrangler.toml`:
   ```toml
   routes = [
     { pattern = "yourdomain.com/api/suggestions", zone_name = "yourdomain.com" }
   ]
   ```

2. Deploy: `npm run deploy`

3. Update frontend to use `/api/suggestions` (relative URL)

**For GitHub Pages** (swhawes.github.io), you cannot use custom routes. Stick with the Worker URL.

---

## Troubleshooting

### Error: "Origin not allowed"

**Problem:** Frontend gets 403 error when submitting suggestions.

**Solution:** Verify `ALLOWED_ORIGIN` in `wrangler.toml` matches your site exactly:
```toml
ALLOWED_ORIGIN = "https://swhawes.github.io"
```

Must include `https://` and no trailing slash.

Redeploy Worker: `npm run deploy`

---

### Error: "Failed to create GitHub issue"

**Problem:** Worker returns 500 error.

**Solutions:**

1. Check GitHub token is set:
   ```bash
   wrangler secret list
   ```
   Should show: `GITHUB_TOKEN`

2. Verify token has `repo` scope in GitHub settings

3. Check Worker logs:
   ```bash
   npm run tail
   ```
   Look for detailed error messages

4. Test GitHub token manually:
   ```bash
   curl -H "Authorization: token YOUR_GITHUB_TOKEN" \
        https://api.github.com/repos/swhawes/leptos-test/issues
   ```

---

### Error: "Rate limit exceeded" on first try

**Problem:** Too many test submissions grouped under 'unknown' IP.

**Solution:** Clear rate limit data:
```bash
wrangler kv:key delete "ratelimit:unknown" --binding RATE_LIMIT
```

---

### Worker not updating after deploy

**Problem:** Changes to Worker code not reflected in production.

**Solutions:**

1. Hard deploy:
   ```bash
   wrangler deploy --force
   ```

2. Clear Cloudflare cache:
   - Dashboard → Workers & Pages → suggestions-api → Settings → Purge Cache

3. Verify deployment ID changed:
   ```bash
   wrangler deployments list
   ```

---

## Security Notes

1. **Never commit `.dev.vars`** - Contains secrets for local testing
2. **Rotate GitHub token periodically** - Every 6-12 months
3. **Monitor rate limits** - Check for abuse patterns
4. **Review GitHub issues regularly** - Set up notifications for `suggestion` label
5. **Keep dependencies updated**:
   ```bash
   cd workers/suggestions
   npm audit
   npm update
   ```

---

## Cost Estimate

**Cloudflare Workers Free Tier:**
- 100,000 requests/day
- 10 GB KV reads/day
- 1 GB KV storage

**Expected usage** (10-50 suggestions/day):
- Worker requests: ~50/day
- KV reads: ~100/day
- KV storage: <1 MB

**Total cost: $0/month** (well within free tier)

Even with 1,000 suggestions/day, you'd stay in free tier.

---

## Quick Reference Commands

```bash
# Deploy Worker
cd workers/suggestions && npm run deploy

# View Worker logs
npm run tail

# Test Worker
curl -X POST "YOUR_WORKER_URL/api/suggestions" \
  -H "Content-Type: application/json" \
  -H "Origin: https://swhawes.github.io" \
  -d '{"slug":"test","page_url":"https://test.com","edits":"test","website":""}'

# Rebuild frontend
cd ../../ && make ssg

# Clear rate limit (for testing)
wrangler kv:key delete "ratelimit:unknown" --binding RATE_LIMIT

# Check secrets
wrangler secret list

# Update secret
wrangler secret put GITHUB_TOKEN
```

---

## Summary Checklist

Before marking Step 7 complete, verify:

- [ ] Wrangler authenticated with Cloudflare
- [ ] KV namespaces created (production + preview)
- [ ] wrangler.toml updated with KV IDs
- [ ] GitHub token created with `repo` scope
- [ ] GitHub token set as Worker secret
- [ ] Worker deployed successfully
- [ ] Worker URL obtained
- [ ] Test submission to Worker succeeded
- [ ] GitHub issue created from test
- [ ] Frontend updated with Worker URL
- [ ] Frontend rebuilt with `make ssg`
- [ ] Frontend deployed to GitHub Pages
- [ ] End-to-end test from live site succeeded
- [ ] Verified GitHub issue created from live site
- [ ] Rate limiting tested (optional)
- [ ] Worker monitoring set up

---

## Next Steps

After completing deployment:

1. **Create a PR** to merge `feature/edit-page-suggestions` to main
2. **Update main branch** documentation with Worker URL
3. **Set up GitHub notifications** for `suggestion` label
4. **Monitor for a week** to ensure everything works smoothly
5. **Consider Step 8** (comprehensive testing) for production confidence

---

**Need help?** Check:
- Workers README: `workers/suggestions/README.md`
- Handoff doc: `HANDOFF_EDIT_SUGGESTIONS.md`
- Cloudflare Docs: https://developers.cloudflare.com/workers/
- Wrangler Docs: https://developers.cloudflare.com/workers/wrangler/

---

**Deployment Guide Last Updated:** 2025-11-18
