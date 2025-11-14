#!/usr/bin/env bash
# Preflight check script - runs all quality checks before committing
# Usage: ./scripts/dev/check.sh [tutorial-slug]
#   If tutorial-slug provided, validates that specific tutorial
#   If omitted, runs all checks except tutorial validation

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track failures
FAILED_CHECKS=()

# Helper function to print section headers
print_header() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Helper function to run a check
run_check() {
    local check_name="$1"
    local check_cmd="$2"

    echo -e "\n${YELLOW}▶ $check_name${NC}"

    if eval "$check_cmd"; then
        echo -e "${GREEN}✓ $check_name passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $check_name failed${NC}"
        FAILED_CHECKS+=("$check_name")
        return 1
    fi
}

# Parse arguments
TUTORIAL_SLUG=""
if [ $# -gt 0 ]; then
    TUTORIAL_SLUG="$1"
fi

print_header "🚀 Preflight Checks"
echo "Running quality checks before commit..."

# Check 1: Code Formatting
print_header "📝 Step 1/5: Code Formatting"
run_check "cargo fmt --check" "cargo fmt --check" || true

# Check 2: Clippy Linting (Main Workspace)
print_header "📎 Step 2/5: Clippy Linting (Main Workspace)"
run_check "clippy (main workspace)" \
    "cargo clippy --lib --bin check-prereqs --bin test_tutorials --bin xtask --features ssr -- -D warnings" || true

# Check 3: Clippy Linting (Parser, Writer & Validator)
print_header "📎 Step 3/5: Clippy Linting (Parser, Writer & Validator)"
run_check "clippy (parser, writer & validator)" \
    "cargo clippy -p longitudinal_parser -p longitudinal_writer -p longitudinal_validator --all-targets -- -D warnings" || true

# Check 4: Rust Tests
print_header "🧪 Step 4/5: Rust Tests"
run_check "cargo test" "cargo test --features ssr" || true

# Check 5: Tutorial Validation (conditional)
if [ -n "$TUTORIAL_SLUG" ]; then
    print_header "✅ Step 5/5: Tutorial Validation"

    TUTORIAL_PATH="content/tutorials/${TUTORIAL_SLUG}.md"

    if [ ! -f "$TUTORIAL_PATH" ]; then
        echo -e "${RED}✗ Tutorial not found: $TUTORIAL_PATH${NC}"
        FAILED_CHECKS+=("tutorial validation")
    else
        echo -e "${YELLOW}Validating tutorial: $TUTORIAL_SLUG${NC}"
        echo -e "${YELLOW}Note: ABCD_DATA_PATH must be set for full validation${NC}"

        if [ -z "${ABCD_DATA_PATH:-}" ]; then
            echo -e "${YELLOW}⚠ Warning: ABCD_DATA_PATH not set, skipping tutorial validation${NC}"
            echo -e "${YELLOW}  Set ABCD_DATA_PATH to enable full validation${NC}"
        else
            run_check "validator ($TUTORIAL_SLUG)" \
                "cargo run -p longitudinal_validator -- '$TUTORIAL_PATH' --force" || true
        fi
    fi
else
    print_header "⏭️  Step 5/5: Tutorial Validation (Skipped)"
    echo -e "${YELLOW}No tutorial specified${NC}"
    echo -e "To validate a specific tutorial, run:"
    echo -e "  ${BLUE}./scripts/dev/check.sh <tutorial-slug>${NC}"
    echo -e "Example:"
    echo -e "  ${BLUE}./scripts/dev/check.sh lgcm-basic${NC}"
fi

# Optional: SSG Build (disabled by default - can be slow)
if [ "${CHECK_SSG:-}" = "true" ]; then
    print_header "🏗️  Bonus: Static Site Generation"
    run_check "make ssg" "make ssg" || true
fi

# Summary
print_header "📊 Summary"

if [ ${#FAILED_CHECKS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! 🎉${NC}"
    echo -e "${GREEN}Ready to commit.${NC}"
    exit 0
else
    echo -e "${RED}✗ ${#FAILED_CHECKS[@]} check(s) failed:${NC}"
    for check in "${FAILED_CHECKS[@]}"; do
        echo -e "${RED}  - $check${NC}"
    done
    echo ""
    echo -e "${YELLOW}Fix the issues above before committing.${NC}"
    echo -e "See ${BLUE}TESTING.md${NC} for detailed guidance."
    exit 1
fi
