#!/bin/bash
# Test script for local Worker development
# Usage: ./test-local.sh [worker-url] [origin]
#
# Examples:
#   ./test-local.sh                                    # Uses defaults
#   ./test-local.sh http://localhost:8787              # Custom worker URL
#   ./test-local.sh http://localhost:8787 http://localhost:3000  # Custom worker and origin

# Default values
WORKER_URL="${1:-http://localhost:8787}/api/suggestions"
ORIGIN="${2:-https://swhawes.github.io}"

# Remove duplicate /api/suggestions if provided in URL
WORKER_URL="${WORKER_URL%/api/suggestions}/api/suggestions"

echo "Testing Suggestions API Worker"
echo "================================"
echo "Worker URL: $WORKER_URL"
echo "Origin: $ORIGIN"
echo ""
echo "NOTE: For local testing, you may need to temporarily update"
echo "ALLOWED_ORIGIN in wrangler.toml to match your origin."
echo ""

# Test 1: Valid submission
echo "Test 1: Valid submission"
curl -X POST "$WORKER_URL" \
  -H "Content-Type: application/json" \
  -H "Origin: $ORIGIN" \
  -d '{
    "slug": "test-tutorial",
    "page_url": "https://swhawes.github.io/longitudinal-dev/posts/test-tutorial/",
    "edits": "This is a test suggestion with some proposed changes.",
    "notes": "Just testing the API endpoint",
    "contact": "test@example.com",
    "baseline_hash": "abc123",
    "website": ""
  }'
echo -e "\n"

# Test 2: Honeypot filled (should fail)
echo "Test 2: Honeypot filled (should reject)"
curl -X POST "$WORKER_URL" \
  -H "Content-Type: application/json" \
  -H "Origin: $ORIGIN" \
  -d '{
    "slug": "test-tutorial",
    "page_url": "https://swhawes.github.io/longitudinal-dev/posts/test-tutorial/",
    "edits": "Some edits",
    "website": "https://spam.com"
  }'
echo -e "\n"

# Test 3: Missing required field (should fail)
echo "Test 3: Missing edits field (should reject)"
curl -X POST "$WORKER_URL" \
  -H "Content-Type: application/json" \
  -H "Origin: $ORIGIN" \
  -d '{
    "slug": "test-tutorial",
    "page_url": "https://swhawes.github.io/longitudinal-dev/posts/test-tutorial/",
    "website": ""
  }'
echo -e "\n"

# Test 4: Invalid origin (should fail)
echo "Test 4: Invalid origin (should reject)"
curl -X POST "$WORKER_URL" \
  -H "Content-Type: application/json" \
  -H "Origin: https://evil.com" \
  -d '{
    "slug": "test-tutorial",
    "page_url": "https://swhawes.github.io/longitudinal-dev/posts/test-tutorial/",
    "edits": "Some edits",
    "website": ""
  }'
echo -e "\n"

# Test 5: CORS preflight
echo "Test 5: CORS preflight (OPTIONS request)"
curl -X OPTIONS "$WORKER_URL" \
  -H "Origin: $ORIGIN" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: Content-Type" \
  -v
echo -e "\n"

echo "================================"
echo "Tests complete"
