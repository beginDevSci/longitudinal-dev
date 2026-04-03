#!/usr/bin/env bash
#
# Run Stage 4 validation inside the Nix R shell
# This ensures all required R packages are available
#
# Usage: ./scripts/stage4/run-in-nix.sh <tutorial-slug>
# Example: ./scripts/stage4/run-in-nix.sh glmm-binary

set -euo pipefail

TUTORIAL="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DOTFILES_DIR="${HOME}/dotfiles"
LOG_DIR="${PROJECT_DIR}/logs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Validate arguments
if [ -z "$TUTORIAL" ]; then
    log_error "TUTORIAL parameter required"
    echo ""
    echo "Usage: $0 <tutorial-slug>"
    echo "Example: $0 glmm-binary"
    echo ""
    echo "Available tutorials:"
    ls -1 "${PROJECT_DIR}/content/tutorials/"*.md 2>/dev/null | xargs -n1 basename | sed 's/.md$//' | column
    exit 1
fi

TUTORIAL_FILE="${PROJECT_DIR}/content/tutorials/${TUTORIAL}.md"
if [ ! -f "$TUTORIAL_FILE" ]; then
    log_error "Tutorial not found: $TUTORIAL_FILE"
    exit 1
fi

# Check if dotfiles exist
if [ ! -d "$DOTFILES_DIR" ]; then
    log_error "Dotfiles directory not found: $DOTFILES_DIR"
    echo "The Nix R shell is defined in ~/dotfiles/shells/r.nix"
    exit 1
fi

# Check if nix is available
if ! command -v nix &> /dev/null; then
    log_error "Nix is not installed or not in PATH"
    echo "Install Nix: https://nixos.org/download.html"
    exit 1
fi

# Create logs directory
mkdir -p "$LOG_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOGFILE="${LOG_DIR}/stage4_${TUTORIAL}_${TIMESTAMP}.log"

log_info "Running Stage 4 for: $TUTORIAL"
log_info "Log file: $LOGFILE"
echo ""

# Write header to log
{
    echo "=== Stage 4 Run (Nix Shell): $TUTORIAL ==="
    echo "Timestamp: $(date)"
    echo "Project: $PROJECT_DIR"
    echo "Dotfiles: $DOTFILES_DIR"
    echo "ABCD_DATA_PATH: ${ABCD_DATA_PATH:-/Users/shawes/abcd/6_0/phenotype}"
    echo ""
} > "$LOGFILE"

# Check if we're already in a Nix shell with R
if [ -n "${IN_NIX_SHELL:-}" ] && command -v Rscript &> /dev/null; then
    # Check if required packages are available
    if Rscript -e "cat(requireNamespace('lavaan', quietly=TRUE))" 2>/dev/null | grep -q "TRUE"; then
        log_info "Already in Nix R shell with required packages"

        cd "$PROJECT_DIR"
        if cargo run --locked -p longitudinal_validator -- --force "$TUTORIAL_FILE" 2>&1 | tee -a "$LOGFILE"; then
            echo "" >> "$LOGFILE"
            echo "=== SUCCESS ===" >> "$LOGFILE"
            echo ""
            log_success "Stage 4 complete!"
            echo "  Artifacts: public/stage4-artifacts/$TUTORIAL/"
            echo "  Log: $LOGFILE"
            echo ""
            echo "Next step: make refresh-tutorial TUTORIAL=$TUTORIAL"
            exit 0
        else
            echo "" >> "$LOGFILE"
            echo "=== FAILED ===" >> "$LOGFILE"
            echo ""
            log_error "Stage 4 failed"
            echo "  Check log: $LOGFILE"
            echo "  Check /tmp/ for R script to debug manually"
            exit 1
        fi
    fi
fi

# Not in Nix shell or missing packages - need to enter it
log_info "Entering Nix R shell..."
log_info "This may take a moment on first run (downloading packages)..."
echo ""

# Run inside nix develop
# We use --command to run a single command inside the shell
cd "$DOTFILES_DIR"
nix develop .#r --command bash -c "
    cd '$PROJECT_DIR'
    echo 'R environment active:'
    echo '  Rscript: \$(which Rscript)'
    echo '  Version: \$(Rscript --version 2>&1 | head -1)'
    echo ''

    # Verify key packages
    echo 'Verifying packages...'
    if ! Rscript -e \"stopifnot(requireNamespace('lavaan', quietly=TRUE))\" 2>/dev/null; then
        echo 'ERROR: lavaan not available even in Nix shell'
        exit 1
    fi
    echo '  Packages OK'
    echo ''

    # Run the validator
    echo 'Running validator...'
    cargo run --locked -p longitudinal_validator -- --force '$TUTORIAL_FILE'
" 2>&1 | tee -a "$LOGFILE"

# Check exit status
if [ "${PIPESTATUS[0]}" -eq 0 ]; then
    echo "" >> "$LOGFILE"
    echo "=== SUCCESS ===" >> "$LOGFILE"
    echo ""
    log_success "Stage 4 complete!"
    echo "  Artifacts: public/stage4-artifacts/$TUTORIAL/"
    echo "  Log: $LOGFILE"
    echo ""
    echo "Next step: make refresh-tutorial TUTORIAL=$TUTORIAL"
else
    echo "" >> "$LOGFILE"
    echo "=== FAILED ===" >> "$LOGFILE"
    echo ""
    log_error "Stage 4 failed"
    echo "  Check log: $LOGFILE"
    echo "  Check /tmp/ for R script to debug manually"
    exit 1
fi
