# Suggestions API - Cloudflare Worker

Cloudflare Worker for handling page edit suggestions from the frontend. Validates submissions, implements rate limiting, and creates GitHub issues for each suggestion.

## Features

- ✅ **Honeypot spam protection** - Rejects submissions with filled honeypot field
- ✅ **Size limits** - Maximum 50 KB per submission
- ✅ **Rate limiting** - 10 submissions per IP per hour using KV storage
- ✅ **Origin validation** - Only accepts requests from allowed origin
- ✅ **GitHub Issues integration** - Creates labeled issues for each suggestion
- ✅ **CORS support** - Proper CORS headers for preflight and responses
- ✅ **Input validation** - Validates all required fields and formats

## Prerequisites

1. **Cloudflare Account** with Workers enabled
2. **GitHub Personal Access Token** with `repo` scope (for creating issues)
3. **Node.js** 18+ and npm
4. **Wrangler CLI** (Cloudflare's deployment tool)

## Setup Instructions

### 1. Install Dependencies

```bash
cd workers/suggestions
npm install
```

### 2. Create KV Namespace

Create a KV namespace for rate limiting:

```bash
# Production namespace
wrangler kv namespace create RATE_LIMIT

# Preview namespace (for testing)
wrangler kv namespace create RATE_LIMIT --preview
```

This will output namespace IDs like:
```
{ binding = "RATE_LIMIT", id = "abc123..." }
{ binding = "RATE_LIMIT", preview_id = "xyz789..." }
```

### 3. Update wrangler.toml

Update the `wrangler.toml` file with your KV namespace IDs from step 2:

```toml
[[kv_namespaces]]
binding = "RATE_LIMIT"
id = "YOUR_KV_NAMESPACE_ID"        # Replace with production ID
preview_id = "YOUR_PREVIEW_KV_NAMESPACE_ID"  # Replace with preview ID
```

Also verify these settings match your configuration:

```toml
[vars]
GITHUB_OWNER = "swhawes"  # Your GitHub username/org
GITHUB_REPO = "leptos-test"  # Your repository name
ALLOWED_ORIGIN = "https://swhawes.github.io"  # Your site origin
MAX_SIZE_BYTES = 51200  # 50 KB
RATE_LIMIT_PER_HOUR = 10
```

### 4. Create GitHub Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Give it a descriptive name: "Suggestions API Worker"
4. Select scopes:
   - ✅ `repo` (Full control of private repositories)
   - Or at minimum: `public_repo` if working with public repos only
5. Click "Generate token"
6. **Copy the token immediately** (you won't see it again)

### 5. Set GitHub Token Secret

```bash
# Set the secret (will prompt you to paste the token)
wrangler secret put GITHUB_TOKEN

# Paste your GitHub token when prompted
```

### 6. Test Locally (Optional)

```bash
# Start local development server
npm run dev

# Test with curl in another terminal
curl -X POST http://localhost:8787/api/suggestions \
  -H "Content-Type: application/json" \
  -H "Origin: https://swhawes.github.io" \
  -d '{
    "slug": "test-tutorial",
    "page_url": "https://swhawes.github.io/longitudinal-dev/posts/test-tutorial/",
    "edits": "This is a test suggestion",
    "notes": "Testing the API",
    "contact": "test@example.com",
    "baseline_hash": "abc123",
    "website": ""
  }'
```

**Note:** Local testing requires setting the `GITHUB_TOKEN` in your `.dev.vars` file:

```bash
# Create .dev.vars file (DO NOT commit this!)
echo "GITHUB_TOKEN=your_github_token_here" > .dev.vars
```

### 7. Deploy to Cloudflare

```bash
# Deploy to production
npm run deploy

# The output will show your Worker URL, e.g.:
# Published suggestions-api
#   https://suggestions-api.your-subdomain.workers.dev
```

### 8. Configure Routes (Optional)

To use a custom route instead of the default `*.workers.dev` URL:

1. Go to Cloudflare Dashboard → Workers & Pages
2. Click on your worker (`suggestions-api`)
3. Go to Settings → Triggers
4. Add route: `yourdomain.com/api/suggestions`
5. Or configure in `wrangler.toml`:

```toml
routes = [
  { pattern = "swhawes.github.io/api/suggestions", zone_name = "swhawes.github.io" }
]
```

**Note:** For GitHub Pages, you'll likely use the default workers.dev URL and update your frontend code accordingly.

## API Specification

### Endpoint

```
POST /api/suggestions
```

### Request Headers

```
Content-Type: application/json
Origin: https://swhawes.github.io
```

### Request Body

```json
{
  "slug": "tutorial-name",
  "page_url": "https://swhawes.github.io/longitudinal-dev/posts/tutorial-name/",
  "edits": "Suggested changes here (REQUIRED)",
  "notes": "Additional context (optional)",
  "contact": "email@example.com or @github-handle (optional)",
  "baseline_hash": "sha256-hash-of-content (optional)",
  "website": ""  // Honeypot - must be empty
}
```

### Response

**Success (200):**
```json
{
  "ok": true
}
```

**Error (4xx/5xx):**
```json
{
  "error": "Description of what went wrong"
}
```

### Error Codes

- `400` - Invalid JSON, missing required fields, honeypot filled, invalid slug format
- `403` - Origin not allowed
- `404` - Invalid endpoint
- `413` - Request too large (>50 KB)
- `429` - Rate limit exceeded (10 submissions per hour)
- `500` - Server error (GitHub API failure)

## Rate Limiting

- **Limit:** 10 submissions per IP address per hour
- **Storage:** Cloudflare KV (timestamps stored for 2 hours)
- **Reset:** Rolling window (timestamps older than 1 hour are discarded)

## Validation Rules

1. **Honeypot:** `website` field must be empty
2. **Required fields:** `slug`, `page_url`, `edits` must be present and non-empty
3. **Slug format:** Must match `^[a-z0-9-]+$` (lowercase, numbers, hyphens only)
4. **Size limit:** Total request body ≤ 50 KB
5. **Origin:** Must match `ALLOWED_ORIGIN` environment variable

## GitHub Integration

Each valid submission creates a GitHub issue with:

- **Title:** `[Suggestion] {slug}`
- **Labels:** `suggestion`, `user-submitted`
- **Body:** Formatted markdown with all submission details

Example issue body:

```markdown
## Page Suggestion

**Page:** https://swhawes.github.io/longitudinal-dev/posts/gee/
**Slug:** `gee`
**Submitted:** 2025-11-18T12:34:56.789Z
**Contact:** user@example.com
**Baseline Hash:** `5551e2e30ace6706...`

---

## Suggested Changes

Fix typo in section 3: "anlaysis" should be "analysis"

---

## Additional Notes

Found this while reading through the tutorial.
```

## Monitoring

### View Logs

```bash
# Tail live logs
npm run tail

# Or with wrangler directly
wrangler tail suggestions-api
```

### Check KV Storage

```bash
# List all keys in the RATE_LIMIT namespace
wrangler kv key list --binding RATE_LIMIT

# Get a specific rate limit entry
wrangler kv key get "ratelimit:1.2.3.4" --binding RATE_LIMIT
```

### Metrics

View metrics in Cloudflare Dashboard:
1. Workers & Pages → suggestions-api
2. Metrics tab shows:
   - Requests per second
   - Errors
   - Duration (p50, p99)
   - KV operations

## Troubleshooting

### "Origin not allowed" Error

- Verify `ALLOWED_ORIGIN` in `wrangler.toml` matches your site exactly (including https://)
- Check that the frontend is sending the correct `Origin` header
- For local testing, temporarily update `ALLOWED_ORIGIN` to `http://localhost:3000`
- Origin validation is strict - must be an exact match (case-sensitive)

### Rate Limiting Behavior

- Rate limits are tracked per IP address using `CF-Connecting-IP` header (Cloudflare-specific)
- Falls back to `X-Forwarded-For` header if `CF-Connecting-IP` is not available
- If both headers are missing, all requests are grouped under IP `'unknown'` (may cluster many clients)
- For production use behind Cloudflare, `CF-Connecting-IP` will always be present
- During local testing (`wrangler dev`), IP will fallback to `'unknown'`

### "Failed to create GitHub issue" Error

- Verify `GITHUB_TOKEN` secret is set: `wrangler secret list`
- Check token has `repo` scope in GitHub settings
- Verify `GITHUB_OWNER` and `GITHUB_REPO` are correct in `wrangler.toml`
- Check Worker logs for detailed error: `wrangler tail`

### Rate Limit Issues

- Check KV namespace is created and IDs are correct in `wrangler.toml`
- View rate limit entries: `wrangler kv key list --binding RATE_LIMIT`
- Manually delete rate limit for testing: `wrangler kv key delete "ratelimit:IP" --binding RATE_LIMIT`

### CORS Errors in Browser

- Ensure Worker is returning proper CORS headers
- Check browser console for specific CORS error message
- Verify preflight OPTIONS request succeeds (204 status)

## Security Considerations

1. **Never commit `.dev.vars`** - Contains sensitive tokens
2. **Rotate GitHub token periodically** - Update via `wrangler secret put GITHUB_TOKEN`
3. **Monitor rate limits** - Adjust `RATE_LIMIT_PER_HOUR` if abused
4. **Review GitHub issues** - Set up notifications for `suggestion` label
5. **Keep dependencies updated** - Run `npm audit` regularly

## Cost Estimate

Cloudflare Workers Free Tier includes:
- 100,000 requests per day
- 10 GB KV reads per day
- 1 GB KV storage

For typical usage (10-50 suggestions per day), this Worker will stay **well within the free tier**.

## Development Workflow

```bash
# Install dependencies
npm install

# Start local dev server
npm run dev

# Test changes locally
curl -X POST http://localhost:8787/api/suggestions ...

# Deploy to production
npm run deploy

# View live logs
npm run tail
```

## Environment Variables Reference

| Variable | Type | Description | Example |
|----------|------|-------------|---------|
| `GITHUB_TOKEN` | Secret | GitHub PAT with repo scope | `ghp_xxxx...` |
| `GITHUB_OWNER` | Var | GitHub username/org | `swhawes` |
| `GITHUB_REPO` | Var | Repository name | `leptos-test` |
| `ALLOWED_ORIGIN` | Var | Allowed request origin | `https://swhawes.github.io` |
| `MAX_SIZE_BYTES` | Var | Max request size (number) | `51200` (50 KB) |
| `RATE_LIMIT_PER_HOUR` | Var | Submissions per IP/hour (number) | `10` |

**Note on Numeric Variables:** Cloudflare Workers inject environment variables as strings, even when defined as numbers in `wrangler.toml`. The Worker code explicitly parses `MAX_SIZE_BYTES` and `RATE_LIMIT_PER_HOUR` using `Number()` to ensure proper numeric comparison.

## Files

```
workers/suggestions/
├── src/
│   └── index.ts          # Main Worker code
├── wrangler.toml         # Cloudflare configuration
├── package.json          # Dependencies
├── tsconfig.json         # TypeScript config
└── README.md            # This file
```

## Next Steps

After deploying the Worker:

1. **Get Worker URL** from deployment output
2. **Update frontend** to use Worker URL:
   - Edit `src/editor_modal_island.rs:210`
   - Change `/api/suggestions` to your Worker URL
   - Or configure route in Cloudflare to handle `/api/suggestions`
3. **Test end-to-end** from the live site
4. **Monitor GitHub issues** for incoming suggestions
5. **Set up notifications** for the `suggestion` label

## Support

- Cloudflare Workers Docs: https://developers.cloudflare.com/workers/
- Wrangler CLI Docs: https://developers.cloudflare.com/workers/wrangler/
- GitHub Issues API: https://docs.github.com/en/rest/issues/issues
- KV Storage Docs: https://developers.cloudflare.com/kv/

---

**Status:** ✅ Implementation complete and ready for deployment
**Last Updated:** 2025-11-18
