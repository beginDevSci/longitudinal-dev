/**
 * Cloudflare Worker for handling page edit suggestions
 *
 * Validates submissions, implements rate limiting, and creates GitHub issues
 * for user-submitted page edit suggestions.
 */

interface Env {
  RATE_LIMIT: KVNamespace;
  GITHUB_TOKEN: string;
  GITHUB_OWNER: string;
  GITHUB_REPO: string;
  ALLOWED_ORIGIN: string;
  MAX_SIZE_BYTES: number;
  RATE_LIMIT_PER_HOUR: number;
}

interface SuggestionPayload {
  slug: string;
  page_url: string;
  edits: string;
  notes?: string;
  contact?: string;
  baseline_hash?: string;
  website?: string; // Honeypot field
}

/**
 * CORS headers for preflight and responses
 */
function getCorsHeaders(origin: string, allowedOrigin: string): Record<string, string> {
  const headers: Record<string, string> = {
    'Access-Control-Allow-Methods': 'POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type',
    'Access-Control-Max-Age': '86400',
  };

  if (origin === allowedOrigin) {
    headers['Access-Control-Allow-Origin'] = origin;
  }

  return headers;
}

/**
 * Create error response with CORS headers
 */
function errorResponse(
  message: string,
  status: number,
  corsHeaders: Record<string, string>
): Response {
  return new Response(
    JSON.stringify({ error: message }),
    {
      status,
      headers: {
        'Content-Type': 'application/json',
        ...corsHeaders
      }
    }
  );
}

/**
 * Create success response with CORS headers
 */
function successResponse(corsHeaders: Record<string, string>): Response {
  return new Response(
    JSON.stringify({ ok: true }),
    {
      status: 200,
      headers: {
        'Content-Type': 'application/json',
        ...corsHeaders
      }
    }
  );
}

/**
 * Check rate limit using KV storage
 * Returns true if rate limit exceeded
 */
async function checkRateLimit(
  env: Env,
  ip: string
): Promise<boolean> {
  const now = Date.now();
  const hourAgo = now - 3600000; // 1 hour in milliseconds
  const key = `ratelimit:${ip}`;

  // Get existing timestamps
  const existing = await env.RATE_LIMIT.get(key);
  let timestamps: number[] = existing ? JSON.parse(existing) : [];

  // Filter out timestamps older than 1 hour
  timestamps = timestamps.filter(ts => ts > hourAgo);

  // Check if limit exceeded
  if (timestamps.length >= env.RATE_LIMIT_PER_HOUR) {
    return true;
  }

  // Add current timestamp
  timestamps.push(now);

  // Store updated timestamps with 2-hour expiration
  await env.RATE_LIMIT.put(key, JSON.stringify(timestamps), {
    expirationTtl: 7200, // 2 hours
  });

  return false;
}

/**
 * Validate the suggestion payload
 */
function validatePayload(payload: SuggestionPayload): string | null {
  // Check honeypot
  if (payload.website && payload.website.trim() !== '') {
    return 'Invalid submission';
  }

  // Check required fields
  if (!payload.slug || typeof payload.slug !== 'string') {
    return 'Missing or invalid slug';
  }

  if (!payload.page_url || typeof payload.page_url !== 'string') {
    return 'Missing or invalid page_url';
  }

  if (!payload.edits || typeof payload.edits !== 'string' || payload.edits.trim() === '') {
    return 'Edits field is required';
  }

  // Validate slug format (alphanumeric and hyphens only)
  if (!/^[a-z0-9-]+$/.test(payload.slug)) {
    return 'Invalid slug format';
  }

  return null;
}

/**
 * Create a GitHub issue with the suggestion
 */
async function createGitHubIssue(
  env: Env,
  payload: SuggestionPayload
): Promise<void> {
  const timestamp = new Date().toISOString();

  // Format the issue body
  const issueBody = `## Page Suggestion

**Page:** ${payload.page_url}
**Slug:** \`${payload.slug}\`
**Submitted:** ${timestamp}
${payload.contact ? `**Contact:** ${payload.contact}  ` : ''}
${payload.baseline_hash ? `**Baseline Hash:** \`${payload.baseline_hash}\`  ` : ''}

---

## Suggested Changes

${payload.edits}

${payload.notes ? `\n---\n\n## Additional Notes\n\n${payload.notes}` : ''}
`;

  const issueTitle = `[Suggestion] ${payload.slug}`;

  // Create GitHub issue using REST API
  const response = await fetch(
    `https://api.github.com/repos/${env.GITHUB_OWNER}/${env.GITHUB_REPO}/issues`,
    {
      method: 'POST',
      headers: {
        'Authorization': `token ${env.GITHUB_TOKEN}`,
        'Accept': 'application/vnd.github.v3+json',
        'Content-Type': 'application/json',
        'User-Agent': 'Cloudflare-Worker-Suggestions',
      },
      body: JSON.stringify({
        title: issueTitle,
        body: issueBody,
        labels: ['suggestion', 'user-submitted'],
      }),
    }
  );

  if (!response.ok) {
    const errorText = await response.text();
    console.error('GitHub API error:', response.status, errorText);
    throw new Error(`Failed to create GitHub issue: ${response.status}`);
  }
}

/**
 * Main request handler
 */
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const origin = request.headers.get('Origin') || '';
    const corsHeaders = getCorsHeaders(origin, env.ALLOWED_ORIGIN);

    // Handle CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: corsHeaders });
    }

    // Only accept POST requests to /api/suggestions
    if (request.method !== 'POST' || url.pathname !== '/api/suggestions') {
      return errorResponse('Not found', 404, corsHeaders);
    }

    // Verify origin
    if (origin !== env.ALLOWED_ORIGIN) {
      return errorResponse('Origin not allowed', 403, corsHeaders);
    }

    // Get client IP for rate limiting
    const ip = request.headers.get('CF-Connecting-IP') ||
               request.headers.get('X-Forwarded-For') ||
               'unknown';

    // Check rate limit
    const rateLimitExceeded = await checkRateLimit(env, ip);
    if (rateLimitExceeded) {
      return errorResponse(
        'Rate limit exceeded. Please try again later.',
        429,
        corsHeaders
      );
    }

    // Parse and validate request body
    let payload: SuggestionPayload;
    try {
      const text = await request.text();

      // Check size limit
      const sizeBytes = new TextEncoder().encode(text).length;
      if (sizeBytes > env.MAX_SIZE_BYTES) {
        return errorResponse(
          'Request too large (max 50 KB)',
          413,
          corsHeaders
        );
      }

      payload = JSON.parse(text);
    } catch (e) {
      return errorResponse('Invalid JSON', 400, corsHeaders);
    }

    // Validate payload
    const validationError = validatePayload(payload);
    if (validationError) {
      return errorResponse(validationError, 400, corsHeaders);
    }

    // Create GitHub issue
    try {
      await createGitHubIssue(env, payload);
      return successResponse(corsHeaders);
    } catch (error) {
      console.error('Error creating GitHub issue:', error);
      return errorResponse(
        'Failed to submit suggestion. Please try again.',
        500,
        corsHeaders
      );
    }
  },
};
