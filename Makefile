.PHONY: content test build clippy validate validate-json validate-tutorials validate-tutorials-force all help hook-install ssg serve deploy bootstrap watch check-guide-hierarchy stage4 stage4-nix refresh-tutorial generate-placeholders check-r-env check-artifacts

# Default target shows help
help:
	@echo "Available targets:"
	@echo ""
	@echo "  DEVELOPMENT:"
	@echo "  make serve                  - Build and serve site at http://localhost:8000 (with brain viewer)"
	@echo "  make watch                  - Run development server with live reload (cargo-leptos)"
	@echo "  make bootstrap              - Check prerequisites and install tools (first-time setup)"
	@echo ""
	@echo "  BUILD:"
	@echo "  make ssg                    - Generate static site to ./dist (with brain viewer)"
	@echo "  make ssg-minimal            - Generate static site without brain viewer (faster)"
	@echo "  make build                  - Build the project (library and binaries)"
	@echo "  make deploy                 - Build and deploy to GitHub Pages"
	@echo ""
	@echo "  VALIDATION:"
	@echo "  make content                - Build, test, and lint (main workflow)"
	@echo "  make validate-json          - Validate JSON syntax with Rust validator (fast)"
	@echo "  make validate-tutorials     - Validate all tutorial markdown files (5-stage pipeline)"
	@echo "  make validate-tutorials-force - Force validate all tutorials (bypass cache)"
	@echo "  make validate               - Alias for validate-json"
	@echo "  make check-guide-hierarchy  - Verify guide heading counts (max 10 H2s)"
	@echo ""
	@echo "  STAGE 4 (R Execution - requires ABCD data):"
	@echo "  make check-r-env            - Verify R environment has required packages"
	@echo "  make check-artifacts        - Show status of Stage 4 artifacts (real vs placeholder)"
	@echo "  make stage4 TUTORIAL=<slug> - Run Stage 4 to generate artifacts for a tutorial"
	@echo "  make stage4-nix TUTORIAL=<slug> - Run Stage 4 inside Nix R shell (recommended)"
	@echo "  make refresh-tutorial TUTORIAL=<slug> - Regenerate JSON after artifacts update"
	@echo "  make generate-placeholders TUTORIAL=<slug> - Create placeholder artifacts for new tutorial"
	@echo ""
	@echo "  OTHER:"
	@echo "  make test                   - Run all tests"
	@echo "  make clippy                 - Run clippy linter"
	@echo "  make all                    - Run validate-json + content"
	@echo "  make hook-install           - Install pre-commit hook"

# Main content workflow: validate → build → test → clippy
content: build test clippy
	@echo "✅ Content workflow complete!"

# Build the project
build:
	@echo "🔨 Building project..."
	cargo build --locked --lib --bin check-prereqs --bin test_tutorials --bin xtask --features ssr

# Run tests with SSR feature
test:
	@echo "🧪 Running tests..."
	cargo test --locked --features ssr

# Run clippy with strict warnings
clippy:
	@echo "📎 Running clippy..."
	cargo clippy --locked --lib --bin check-prereqs --bin test_tutorials --bin xtask --features ssr -- -D warnings

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
	@WEBGPU_VIEWER=1 cargo run --locked --features ssr --bin xtask --release -- build-ssg --outdir dist

# Generate and serve static site (with interactive brain viewer)
serve: ssg
	@echo "🌐 Serving site at http://localhost:8000"
	@echo "Press Ctrl+C to stop"
	@cd dist && python3 -m http.server 8000

# Generate SSG without brain viewer (smaller WASM, faster build)
ssg-minimal:
	@cargo run --locked --features ssr --bin xtask --release -- build-ssg --outdir dist

# Deploy to GitHub Pages
deploy:
	@./scripts/deploy/deploy_github_pages.sh

# Bootstrap: check prerequisites and install tools
bootstrap:
	@cargo run --features ssr --bin check-prereqs -- --install

# Watch mode: run development server with live reload
watch:
	@echo "🚀 Starting development server with live reload..."
	@echo "This requires 3 terminals. Opening dev script..."
	@./scripts/dev/dev.sh

# Check guide heading hierarchy (max 10 H2s per guide)
MAX_H2_COUNT := 10
check-guide-hierarchy:
	@echo "🔍 Checking guide heading hierarchy..."
	@failed=0; \
	for guide in content/guides/*.md; do \
		count=$$(grep -c '^## ' "$$guide" 2>/dev/null || echo 0); \
		name=$$(basename "$$guide"); \
		if [ "$$count" -gt $(MAX_H2_COUNT) ]; then \
			echo "  ❌ $$name: $$count H2s (max $(MAX_H2_COUNT))"; \
			failed=1; \
		else \
			echo "  ✅ $$name: $$count H2s"; \
		fi; \
	done; \
	if [ "$$failed" -eq 1 ]; then \
		echo ""; \
		echo "Guide hierarchy check failed. Run:"; \
		echo "  python3 scripts/demote_guide_headings.py content/guides/<guide>.md"; \
		exit 1; \
	fi
	@echo "✅ All guides have proper heading hierarchy!"

# =============================================================================
# STAGE 4: R Execution (requires local ABCD data access)
# =============================================================================

# Run Stage 4 for a specific tutorial to generate real artifacts
# Usage: make stage4 TUTORIAL=glmm-binary
# Note: Checks R environment first. Use 'make stage4-nix' for automatic Nix shell.
stage4: check-r-env
	@if [ -z "$(TUTORIAL)" ]; then \
		echo "❌ Error: TUTORIAL parameter required"; \
		echo "Usage: make stage4 TUTORIAL=<tutorial-slug>"; \
		echo ""; \
		echo "Available tutorials:"; \
		ls -1 content/tutorials/*.md | xargs -n1 basename | sed 's/.md$$/  /' | column; \
		exit 1; \
	fi
	@if [ ! -f "content/tutorials/$(TUTORIAL).md" ]; then \
		echo "❌ Error: Tutorial not found: content/tutorials/$(TUTORIAL).md"; \
		exit 1; \
	fi
	@mkdir -p logs
	@timestamp=$$(date +%Y%m%d_%H%M%S); \
	logfile="logs/stage4_$(TUTORIAL)_$$timestamp.log"; \
	echo "🔬 Running Stage 4 (R execution) for $(TUTORIAL)..."; \
	echo "   ABCD data: $${ABCD_DATA_PATH:-/Users/shawes/abcd/6_0/phenotype}"; \
	echo "   Log file: $$logfile"; \
	echo ""; \
	echo "=== Stage 4 Run: $(TUTORIAL) ===" > "$$logfile"; \
	echo "Timestamp: $$(date)" >> "$$logfile"; \
	echo "ABCD_DATA_PATH: $${ABCD_DATA_PATH:-/Users/shawes/abcd/6_0/phenotype}" >> "$$logfile"; \
	echo "" >> "$$logfile"; \
	if cargo run --locked -p longitudinal_validator -- --force content/tutorials/$(TUTORIAL).md 2>&1 | tee -a "$$logfile"; then \
		echo "" >> "$$logfile"; \
		echo "=== SUCCESS ===" >> "$$logfile"; \
		echo ""; \
		echo "✅ Stage 4 complete! Artifacts generated in public/stage4-artifacts/$(TUTORIAL)/"; \
		echo "   Log saved: $$logfile"; \
		echo "   Run 'make refresh-tutorial TUTORIAL=$(TUTORIAL)' to update JSON"; \
	else \
		echo "" >> "$$logfile"; \
		echo "=== FAILED ===" >> "$$logfile"; \
		echo ""; \
		echo "❌ Stage 4 failed for $(TUTORIAL)"; \
		echo "   Check log file: $$logfile"; \
		echo "   Check temp R script in /tmp/ for debugging"; \
		exit 1; \
	fi

# Regenerate JSON for a tutorial after Stage 4 artifacts are updated
# Usage: make refresh-tutorial TUTORIAL=glmm-binary
refresh-tutorial:
	@if [ -z "$(TUTORIAL)" ]; then \
		echo "❌ Error: TUTORIAL parameter required"; \
		echo "Usage: make refresh-tutorial TUTORIAL=<tutorial-slug>"; \
		exit 1; \
	fi
	@if [ ! -f "content/tutorials/$(TUTORIAL).md" ]; then \
		echo "❌ Error: Tutorial not found: content/tutorials/$(TUTORIAL).md"; \
		exit 1; \
	fi
	@echo "🔄 Regenerating JSON for $(TUTORIAL)..."
	@cargo run --locked -p longitudinal_parser --bin md2json -- \
		content/tutorials/$(TUTORIAL).md \
		--output content/posts/$(TUTORIAL).post.json
	@echo "✅ JSON updated: content/posts/$(TUTORIAL).post.json"

# Generate placeholder artifacts for a new tutorial
# Usage: make generate-placeholders TUTORIAL=new-tutorial
generate-placeholders:
	@if [ -z "$(TUTORIAL)" ]; then \
		echo "❌ Error: TUTORIAL parameter required"; \
		echo "Usage: make generate-placeholders TUTORIAL=<tutorial-slug>"; \
		exit 1; \
	fi
	@if [ ! -f "content/tutorials/$(TUTORIAL).md" ]; then \
		echo "❌ Error: Tutorial not found: content/tutorials/$(TUTORIAL).md"; \
		exit 1; \
	fi
	@echo "📝 Generating placeholder artifacts for $(TUTORIAL)..."
	@mkdir -p "public/stage4-artifacts/$(TUTORIAL)"
	@# Extract artifact filenames from markdown and create placeholders
	@grep -oE 'stage4-artifacts/$(TUTORIAL)/[^")\s]+' "content/tutorials/$(TUTORIAL).md" | \
		sed 's|stage4-artifacts/$(TUTORIAL)/||' | sort -u | while read artifact; do \
		filepath="public/stage4-artifacts/$(TUTORIAL)/$$artifact"; \
		if [ ! -f "$$filepath" ]; then \
			case "$$artifact" in \
				*.html) \
					echo '<div class="placeholder-output">' > "$$filepath"; \
					echo '  <p><em>Output will be generated when R code is executed.</em></p>' >> "$$filepath"; \
					echo '</div>' >> "$$filepath"; \
					echo "  Created: $$filepath"; \
					;; \
				*.png) \
					printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\nIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > "$$filepath"; \
					echo "  Created: $$filepath (1x1 placeholder)"; \
					;; \
				*) \
					echo "Placeholder content" > "$$filepath"; \
					echo "  Created: $$filepath"; \
					;; \
			esac \
		else \
			echo "  Exists: $$filepath"; \
		fi \
	done
	@echo "✅ Placeholders ready in public/stage4-artifacts/$(TUTORIAL)/"

# Check R environment for required packages
# This validates that all packages needed for Stage 4 are available
REQUIRED_R_PACKAGES := tidyverse lavaan NBDCtools lme4 lmerTest glmmTMB gtsummary broom broom.mixed gt arrow geepack semPlot sjPlot performance OpenMx rstatix ggeffects patchwork effectsize emmeans corrplot
check-r-env:
	@echo "🔍 Checking R environment..."
	@echo ""
	@rscript_path=$$(which Rscript 2>/dev/null); \
	if [ -z "$$rscript_path" ]; then \
		echo "❌ Rscript not found in PATH"; \
		echo ""; \
		echo "To fix: Enter the Nix R shell first:"; \
		echo "  cd ~/dotfiles && nix develop .#r"; \
		exit 1; \
	fi; \
	echo "  Rscript: $$rscript_path"; \
	r_version=$$(Rscript --version 2>&1 | head -1); \
	echo "  Version: $$r_version"; \
	echo ""; \
	echo "  Checking required packages..."; \
	missing=""; \
	for pkg in $(REQUIRED_R_PACKAGES); do \
		result=$$(Rscript -e "cat(requireNamespace('$$pkg', quietly=TRUE))" 2>/dev/null); \
		if [ "$$result" = "TRUE" ]; then \
			echo "    ✅ $$pkg"; \
		else \
			echo "    ❌ $$pkg (missing)"; \
			missing="$$missing $$pkg"; \
		fi; \
	done; \
	echo ""; \
	if [ -n "$$missing" ]; then \
		echo "❌ Missing R packages:$$missing"; \
		echo ""; \
		echo "To fix: Enter the Nix R shell which has all packages:"; \
		echo "  cd ~/dotfiles && nix develop .#r"; \
		echo ""; \
		echo "Then return to this directory and run Stage 4:"; \
		echo "  cd $(CURDIR) && make stage4 TUTORIAL=<slug>"; \
		exit 1; \
	fi; \
	echo "✅ R environment ready for Stage 4!"

# Check status of Stage 4 artifacts across all tutorials
# Shows which have real output vs placeholders
check-artifacts:
	@echo "🔍 Checking Stage 4 artifact status..."
	@echo ""
	@echo "  Tutorial                          Status       Size      Date"
	@echo "  ────────────────────────────────  ───────────  ────────  ──────────"
	@for dir in public/stage4-artifacts/*/; do \
		name=$$(basename "$$dir"); \
		if [ -f "$$dir/visualization.png" ]; then \
			size=$$(wc -c < "$$dir/visualization.png" | tr -d ' '); \
			date=$$(stat -f "%Sm" -t "%Y-%m-%d" "$$dir/visualization.png" 2>/dev/null || stat -c "%y" "$$dir/visualization.png" 2>/dev/null | cut -d' ' -f1); \
			if [ "$$size" -lt 100 ]; then \
				printf "  %-34s  ⚠ placeholder  %6s B  %s\n" "$$name" "$$size" "$$date"; \
			else \
				printf "  %-34s  ✅ real        %6s B  %s\n" "$$name" "$$size" "$$date"; \
			fi; \
		else \
			printf "  %-34s  ❌ missing     -         -\n" "$$name"; \
		fi; \
	done
	@echo ""
	@echo "Legend: ✅ real = has actual R output, ⚠ placeholder = needs Stage 4 run"

# Run Stage 4 inside Nix R shell (recommended approach)
# Usage: make stage4-nix TUTORIAL=glmm-binary
stage4-nix:
	@if [ -z "$(TUTORIAL)" ]; then \
		echo "❌ Error: TUTORIAL parameter required"; \
		echo "Usage: make stage4-nix TUTORIAL=<tutorial-slug>"; \
		exit 1; \
	fi
	@if [ ! -f "content/tutorials/$(TUTORIAL).md" ]; then \
		echo "❌ Error: Tutorial not found: content/tutorials/$(TUTORIAL).md"; \
		exit 1; \
	fi
	@echo "🔬 Running Stage 4 for $(TUTORIAL) inside Nix R shell..."
	@echo ""
	@./scripts/stage4/run-in-nix.sh $(TUTORIAL)
