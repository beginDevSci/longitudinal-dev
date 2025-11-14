/*!
 * Shared R script prelude components
 *
 * Contains reusable R code snippets that are injected into generated scripts
 * to ensure consistent, robust execution across all validation stages.
 */

/// Generate CPU count sanitization prelude for R scripts
///
/// This prevents lavaan lav_options_checkinterval failures when:
/// - parallel::detectCores() returns NA (common on macOS)
/// - Options contain NULL, "", or invalid Ncpus values
/// - Thread count environment variables are misconfigured
///
/// # Returns
/// R code string that:
/// 1. Defines safe_ncpus() function with robust fallback chain
/// 2. Logs RAW options before sanitization (diagnostic)
/// 3. Sanitizes Ncpus, lavaan.ncpus, mc.cores options
/// 4. Sets thread environment variables for BLAS/MKL
/// 5. Logs SANITIZED options after fix (diagnostic)
///
/// # Usage
/// ```rust
/// let mut script = String::new();
/// script.push_str(&r_prelude::cpu_sanitization_prelude());
/// script.push_str("library(lavaan)\n");
/// // ... rest of script
/// ```
pub fn cpu_sanitization_prelude() -> String {
    let mut prelude = String::new();

    prelude.push_str("# ===== CPU COUNT SANITIZATION =====\n");
    prelude.push_str("# Prevent lavaan lav_options_checkinterval failures from NA/blank Ncpus\n");
    prelude.push_str("# This fixes issues on macOS where detectCores() returns NA\n");
    prelude.push_str("safe_ncpus <- function(x = NULL) {\n");
    prelude.push_str("  # Handle NULL first\n");
    prelude.push_str("  if (is.null(x)) x <- NA\n");
    prelude.push_str("  val <- suppressWarnings(as.integer(x))\n");
    prelude.push_str("  # Try detectCores() if x is NA/invalid\n");
    prelude.push_str("  if (length(val) == 0 || is.na(val[1]) || val[1] < 1L) {\n");
    prelude.push_str("    dc <- tryCatch(parallel::detectCores(logical = TRUE), error = function(e) NA_integer_)\n");
    prelude.push_str("    val <- suppressWarnings(as.integer(dc))\n");
    prelude.push_str("  }\n");
    prelude.push_str("  # Final fallback to 1 CPU\n");
    prelude.push_str("  if (length(val) == 0 || is.na(val[1]) || val[1] < 1L) val <- 1L\n");
    prelude.push_str("  val[1]\n");
    prelude.push_str("}\n\n");

    prelude.push_str("# Log RAW options before sanitization\n");
    prelude.push_str("cat('DEBUG: RAW CPU options before sanitization:\\n')\n");
    prelude.push_str("cat(sprintf('  Ncpus = %s\\n', deparse(getOption('Ncpus'))))\n");
    prelude
        .push_str("cat(sprintf('  lavaan.ncpus = %s\\n', deparse(getOption('lavaan.ncpus'))))\n");
    prelude.push_str("cat(sprintf('  mc.cores = %s\\n', deparse(getOption('mc.cores'))))\n");
    prelude.push_str("cat(sprintf('  detectCores() = %s\\n', tryCatch(parallel::detectCores(), error=function(e) 'NA')))\n\n");

    prelude.push_str("# Sanitize all CPU-related options\n");
    prelude.push_str("options(\n");
    prelude.push_str("  Ncpus        = safe_ncpus(getOption('Ncpus')),\n");
    prelude.push_str("  lavaan.ncpus = safe_ncpus(getOption('lavaan.ncpus')),\n");
    prelude.push_str("  mc.cores     = safe_ncpus(getOption('mc.cores'))\n");
    prelude.push_str(")\n\n");

    prelude.push_str("# Also set thread env vars for math libraries (OpenBLAS, MKL, etc.)\n");
    prelude.push_str("Sys.setenv(\n");
    prelude.push_str("  OMP_NUM_THREADS      = as.character(getOption('Ncpus')),\n");
    prelude.push_str("  OPENBLAS_NUM_THREADS = as.character(getOption('Ncpus')),\n");
    prelude.push_str("  MKL_NUM_THREADS      = as.character(getOption('Ncpus'))\n");
    prelude.push_str(")\n\n");

    prelude.push_str("# Log SANITIZED options after fix\n");
    prelude.push_str("cat('DEBUG: SANITIZED CPU options after fix:\\n')\n");
    prelude.push_str("cat(sprintf('  Ncpus = %s\\n', getOption('Ncpus')))\n");
    prelude.push_str("cat(sprintf('  lavaan.ncpus = %s\\n', getOption('lavaan.ncpus')))\n");
    prelude.push_str("cat(sprintf('  mc.cores = %s\\n', getOption('mc.cores')))\n");
    prelude.push_str("cat('========================\\n\\n')\n\n");

    prelude
}

/// Generate session info diagnostic logging for R scripts
///
/// Logs R version, detectCores() result, and loaded package versions.
/// Useful for debugging environment-specific issues.
///
/// # Returns
/// R code string that logs session diagnostics
pub fn session_info_logging() -> String {
    let mut logging = String::new();

    logging.push_str("# ===== SESSION INFO =====\n");
    logging.push_str("cat('===== SESSION INFO =====\\n')\n");
    logging.push_str("cat(sprintf('R version: %s\\n', R.version.string))\n");
    logging.push_str("cat(sprintf('detectCores(): %s\\n', tryCatch(parallel::detectCores(), error=function(e) 'NA')))\n");
    logging.push_str("if ('lavaan' %in% loadedNamespaces()) {\n");
    logging.push_str("  cat(sprintf('lavaan version: %s\\n', packageVersion('lavaan')))\n");
    logging.push_str("}\n");
    logging.push_str("cat('========================\\n\\n')\n\n");

    logging
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_sanitization_prelude_contains_safe_ncpus() {
        let prelude = cpu_sanitization_prelude();
        assert!(prelude.contains("safe_ncpus <- function"));
        assert!(prelude.contains("options("));
        assert!(prelude.contains("Sys.setenv("));
    }

    #[test]
    fn test_session_info_logging_contains_version_check() {
        let logging = session_info_logging();
        assert!(logging.contains("R.version.string"));
        assert!(logging.contains("detectCores()"));
        assert!(logging.contains("lavaan"));
    }
}
