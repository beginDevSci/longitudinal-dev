#!/usr/bin/env bash

# Disabled: after switching to gh-pages, the steps below delete every
# top-level entry in the working copy except .git (including dist/ and
# Git-ignored files) and then fail. Keep this guard until the production
# publishing workflow is confirmed.
echo "Manual deployment is disabled until the production publishing workflow is confirmed" >&2
exit 1

set -euo pipefail

echo "🚀 Deploying to GitHub Pages..."

# Safety check: ensure we're not on gh-pages branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" = "gh-pages" ]; then
    echo "❌ Error: Cannot run from gh-pages branch"
    echo "   Switch to main branch first: git checkout main"
    exit 1
fi

# Save current branch
ORIGINAL_BRANCH="$CURRENT_BRANCH"

# Step 1: Ensure working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo "❌ Error: Working directory has uncommitted changes"
    echo "   Commit or stash changes before deploying"
    git status --short
    exit 1
fi

# Step 2: Build with GitHub Pages base path
echo "📦 Building site..."
SITE_BASE_PATH="/longitudinal-dev/" make ssg

# Drop validator artifacts that shouldn't ship
if [ -d "dist/stage4-artifacts" ]; then
    echo "🧹 Removing stage4 artifacts from deploy payload"
    rm -rf dist/stage4-artifacts
fi

# Step 3: Switch to gh-pages branch (create if needed)
echo "🔀 Switching to gh-pages branch..."
if git show-ref --verify --quiet refs/heads/gh-pages; then
    git checkout gh-pages
else
    echo "📝 Creating gh-pages branch..."
    git checkout --orphan gh-pages
    git rm -rf .
fi

# Step 4: Copy dist contents to root
echo "📂 Copying build artifacts..."
# Remove old files (keep .git)
find . -maxdepth 1 ! -name '.git' ! -name '.' ! -name '..' -exec rm -rf {} +

# Copy dist contents to root
cp -r dist/* .
cp -r dist/.* . 2>/dev/null || true

# Step 5: Commit and push
echo "📝 Committing deployment..."
git add -A
if git diff --staged --quiet; then
    echo "✅ No changes to deploy"
    git checkout "$ORIGINAL_BRANCH"
else
    git commit -m "chore: Deploy to GitHub Pages ($(date +'%Y-%m-%d %H:%M'))"

    echo "🚀 Pushing to gh-pages..."
    git push origin gh-pages

    echo "✅ Deployed to GitHub Pages!"
    echo "🌐 Site will be live at: https://swhawes.github.io/longitudinal-dev/"

    # Return to original branch
    git checkout "$ORIGINAL_BRANCH"
fi

echo "✅ Deployment complete!"
