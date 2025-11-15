#!/usr/bin/env bash
set -euo pipefail

# Bootstrap script for longitudinal_dev
# Checks prerequisites and installs required tools
# Usage: ./scripts/dev/bootstrap.sh [--dry-run]

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Log file
readonly LOG_FILE="bootstrap.log"

# Dry run flag
DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
fi

# Logging functions
log() {
    echo -e "${BLUE}ℹ${NC}  $*" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}✓${NC}  $*" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}⚠${NC}  $*" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}✗${NC}  $*" | tee -a "$LOG_FILE"
}

check_command() {
    local cmd=$1
    if command -v "$cmd" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

get_version() {
    local cmd=$1
    shift
    "$cmd" "$@" 2>&1 || echo "unknown"
}

# Clear previous log
> "$LOG_FILE"

log "Starting bootstrap process..."
log "Log file: $LOG_FILE"
log "Dry run: $DRY_RUN"
echo ""

# Track missing prerequisites
MISSING_PREREQS=()

# ============================================================================
# Check 1: Rust toolchain
# ============================================================================
log "Checking Rust toolchain..."

if check_command rustc && check_command cargo; then
    RUST_VERSION=$(get_version rustc --version)
    CARGO_VERSION=$(get_version cargo --version)
    success "Rust installed: $RUST_VERSION"
    success "Cargo installed: $CARGO_VERSION"

    # Check if it's the right channel (stable)
    if echo "$RUST_VERSION" | grep -q "stable"; then
        success "Using stable Rust (as configured in rust-toolchain.toml)"
    elif echo "$RUST_VERSION" | grep -q "nightly"; then
        warning "Using nightly Rust - rust-toolchain.toml specifies stable"
        warning "Run: rustup default stable"
    fi
else
    error "Rust not found"
    MISSING_PREREQS+=("rust")
    log "Install from: https://rustup.rs/"
fi

# ============================================================================
# Check 2: WASM target
# ============================================================================
log "Checking wasm32-unknown-unknown target..."

if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    success "wasm32-unknown-unknown target installed"
else
    warning "wasm32-unknown-unknown target not installed"
    if [[ "$DRY_RUN" == "true" ]]; then
        log "Would run: rustup target add wasm32-unknown-unknown"
    else
        log "Installing wasm32-unknown-unknown target..."
        if rustup target add wasm32-unknown-unknown >> "$LOG_FILE" 2>&1; then
            success "wasm32-unknown-unknown target installed"
        else
            error "Failed to install wasm32-unknown-unknown target"
            MISSING_PREREQS+=("wasm-target")
        fi
    fi
fi

# ============================================================================
# Check 3: wasm-bindgen-cli
# ============================================================================
log "Checking wasm-bindgen-cli..."

REQUIRED_WASM_BINDGEN="0.2.104"
if check_command wasm-bindgen; then
    WASM_BINDGEN_VERSION=$(wasm-bindgen --version | awk '{print $2}')
    if [[ "$WASM_BINDGEN_VERSION" == "$REQUIRED_WASM_BINDGEN" ]]; then
        success "wasm-bindgen-cli $WASM_BINDGEN_VERSION installed (matches Cargo.toml)"
    else
        warning "wasm-bindgen-cli $WASM_BINDGEN_VERSION found, but $REQUIRED_WASM_BINDGEN required"
        warning "Version mismatch can cause build errors"
        if [[ "$DRY_RUN" == "true" ]]; then
            log "Would run: cargo install wasm-bindgen-cli --version $REQUIRED_WASM_BINDGEN"
        else
            log "Installing wasm-bindgen-cli $REQUIRED_WASM_BINDGEN..."
            if cargo install wasm-bindgen-cli --version "$REQUIRED_WASM_BINDGEN" --force >> "$LOG_FILE" 2>&1; then
                success "wasm-bindgen-cli $REQUIRED_WASM_BINDGEN installed"
            else
                error "Failed to install wasm-bindgen-cli"
                MISSING_PREREQS+=("wasm-bindgen-cli")
            fi
        fi
    fi
else
    warning "wasm-bindgen-cli not installed"
    if [[ "$DRY_RUN" == "true" ]]; then
        log "Would run: cargo install wasm-bindgen-cli --version $REQUIRED_WASM_BINDGEN"
    else
        log "Installing wasm-bindgen-cli $REQUIRED_WASM_BINDGEN..."
        if cargo install wasm-bindgen-cli --version "$REQUIRED_WASM_BINDGEN" >> "$LOG_FILE" 2>&1; then
            success "wasm-bindgen-cli $REQUIRED_WASM_BINDGEN installed"
        else
            error "Failed to install wasm-bindgen-cli"
            MISSING_PREREQS+=("wasm-bindgen-cli")
        fi
    fi
fi

# ============================================================================
# Check 4: Node.js and npm
# ============================================================================
log "Checking Node.js and npm..."

if check_command node && check_command npm; then
    NODE_VERSION=$(get_version node --version)
    NPM_VERSION=$(get_version npm --version)
    success "Node.js installed: $NODE_VERSION"
    success "npm installed: $NPM_VERSION"
else
    error "Node.js/npm not found"
    MISSING_PREREQS+=("nodejs")
    log "Install from: https://nodejs.org/"
fi

# ============================================================================
# Check 5: npm dependencies
# ============================================================================
if check_command npm; then
    log "Checking npm dependencies..."

    if [[ -f "package-lock.json" ]]; then
        success "package-lock.json found (committed for determinism)"
    else
        warning "package-lock.json not found"
    fi

    if [[ "$DRY_RUN" == "true" ]]; then
        log "Would run: npm install"
    else
        log "Installing npm dependencies..."
        if npm install >> "$LOG_FILE" 2>&1; then
            success "npm dependencies installed"
        else
            error "Failed to install npm dependencies"
            MISSING_PREREQS+=("npm-deps")
        fi
    fi
fi

# ============================================================================
# Check 6: R (optional, needed for validation)
# ============================================================================
log "Checking R (optional for tutorial validation)..."

if check_command Rscript; then
    R_VERSION=$(get_version Rscript --version)
    success "R installed: $R_VERSION"

    # Check for required packages (from config/validation.toml)
    log "Checking R packages..."
    REQUIRED_PACKAGES=("tidyverse" "lavaan" "NBDCtools" "geepack" "lme4")
    MISSING_R_PACKAGES=()

    for pkg in "${REQUIRED_PACKAGES[@]}"; do
        if Rscript -e "if (!require('$pkg', quietly=TRUE)) quit(status=1)" &> /dev/null; then
            success "R package '$pkg' installed"
        else
            warning "R package '$pkg' not installed"
            MISSING_R_PACKAGES+=("$pkg")
        fi
    done

    if [[ ${#MISSING_R_PACKAGES[@]} -gt 0 ]]; then
        warning "Missing R packages: ${MISSING_R_PACKAGES[*]}"
        log "Install in R: install.packages(c('${MISSING_R_PACKAGES[*]}'))"
        log "Or see: config/validation.toml"
    fi
else
    warning "R not found (optional - only needed for tutorial validation)"
    log "Install from: https://www.r-project.org/"
fi

# ============================================================================
# Check 7: Build the project
# ============================================================================
if [[ "$DRY_RUN" == "false" && ${#MISSING_PREREQS[@]} -eq 0 ]]; then
    log "Running initial build check..."

    log "Checking cargo build..."
    if cargo check --lib --features ssr >> "$LOG_FILE" 2>&1; then
        success "cargo check passed (SSR)"
    else
        error "cargo check failed - see $LOG_FILE"
        MISSING_PREREQS+=("build-check")
    fi

    log "Checking cargo clippy..."
    if cargo clippy --lib --features ssr -- -D warnings >> "$LOG_FILE" 2>&1; then
        success "cargo clippy passed"
    else
        warning "cargo clippy warnings found - see $LOG_FILE"
    fi
fi

# ============================================================================
# Summary
# ============================================================================
echo ""
log "========================================"
log "Bootstrap Summary"
log "========================================"

if [[ ${#MISSING_PREREQS[@]} -eq 0 ]]; then
    success "All required prerequisites are installed!"
    log ""
    log "Next steps:"
    log "  1. Generate and serve the site: make serve"
    log "  2. Run preflight checks: ./scripts/dev/check.sh"
    log "  3. See docs/PERSONAS.md for workflow by role"
    log ""
    log "For live development, see: make watch"
else
    error "Missing prerequisites: ${MISSING_PREREQS[*]}"
    log ""
    log "Please install the missing prerequisites and run bootstrap again."
    log "See $LOG_FILE for details."
    exit 1
fi

log "Bootstrap complete! ✨"
