.PHONY: content test build clippy validate validate-json validate-tutorials validate-tutorials-force all help hook-install ssg serve deploy bootstrap watch

# Default target shows help
help:
	@echo "Available targets:"
	@echo "  make bootstrap              - Check prerequisites and install tools (first-time setup)"
	@echo "  make watch                  - Run development server with live reload"
	@echo "  make content                - Build, test, and lint (main workflow)"
	@echo "  make validate-json          - Validate JSON syntax with Rust validator (fast)"
	@echo "  make validate-tutorials      - Validate all tutorial markdown files (5-stage pipeline)"
	@echo "  make validate-tutorials-force - Force validate all tutorials (bypass cache)"
	@echo "  make validate               - Alias for validate-json"
	@echo "  make build                  - Build the project"
	@echo "  make test                   - Run all tests"
	@echo "  make clippy                 - Run clippy linter"
	@echo "  make all                    - Run validate-json + content"
	@echo "  make hook-install           - Install pre-commit hook"
	@echo "  make ssg                    - Generate static site to ./dist"
	@echo "  make serve                  - Generate and serve static site on http://localhost:8000"
	@echo "  make deploy                 - Build and deploy to GitHub Pages"

# Main content workflow: validate → build → test → clippy
content: build test clippy
	@echo "✅ Content workflow complete!"

# Build the project
build:
	@echo "🔨 Building project..."
	cargo build --locked

# Run tests with SSR feature
test:
	@echo "🧪 Running tests..."
	cargo test --locked --features ssr

# Run clippy with strict warnings
clippy:
	@echo "📎 Running clippy..."
	cargo clippy --locked -- -D warnings

# Fast JSON syntax validation with Rust validator
validate-json:
	@echo "🔍 Validating JSON syntax..."
	@for f in content/posts/*.post.json; do \
		echo "  Checking $$(basename $$f)"; \
		cargo run --locked --quiet --bin validate-json -- "$$f" || exit 1; \
	done
	@echo "✅ All JSON posts are valid!"

# Alias for validate-json (for backward compatibility)
validate: validate-json

# Validate all tutorial markdown files through 5-stage pipeline
validate-tutorials:
	@echo "🔍 Validating all tutorials through 5-stage pipeline..."
	@cargo run --locked -p longitudinal_validator -- content/tutorials/*.md

# Validate all tutorials with force flag (bypass cache)
validate-tutorials-force:
	@echo "🔍 Force validating all tutorials (bypassing cache)..."
	@cargo run --locked -p longitudinal_validator -- --force content/tutorials/*.md

# Complete workflow with validation first
all: validate-json content

# Install pre-commit hook
hook-install:
	@install -d .git/hooks
	@install -m 0755 scripts/pre-commit .git/hooks/pre-commit
	@echo "✅ Pre-commit hook installed"

# Generate static site
ssg:
	@cargo run --locked --features ssr --bin xtask --release -- build-ssg --outdir dist

# Generate and serve static site
serve: ssg
	@echo "🌐 Serving site at http://localhost:8000"
	@echo "Press Ctrl+C to stop"
	@cd dist && python3 -m http.server 8000

# Deploy to GitHub Pages
deploy:
	@./scripts/deploy/deploy_github_pages.sh

# Bootstrap: check prerequisites and install tools
bootstrap:
	@./scripts/dev/bootstrap.sh

# Watch mode: run development server with live reload
watch:
	@echo "🚀 Starting development server with live reload..."
	@echo "This requires 3 terminals. Opening dev script..."
	@./scripts/dev/dev.sh
