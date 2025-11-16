#!/usr/bin/env Rscript
# Mock Data Generator for Stage 3 Validation - ENHANCED VERSION
#
# This script provides mock implementations of ABCD data functions
# for dry-run validation without requiring the full ABCD dataset.
#
# ENHANCEMENTS (2025-11-06):
# - Fixed sex/race variables to return numeric codes matching ABCD format
# - Added cohort variable patterns (education, income)
# - Added visit_age pattern
# - More specific NIH Toolbox patterns
# - Ensure minimum complete cases function
# - Better stable variable detection for "cohort_" prefix
# - AR(1) autocorrelation for longitudinal mental health variables (r=0.6)
# - Stratified sampling by education levels in complete cases guarantee
#
# Usage: Source this file before executing tutorial R code.
#        It overrides create_dataset_abcd() with mock_create_dataset_abcd().

#' Generate mock values for a single variable based on naming patterns
#'
#' @param var_name Character string of variable name
#' @param n_rows Number of rows to generate
#' @param participant_idx Optional: participant index for deterministic generation
#' @return Vector of appropriate type and range for the variable
generate_mock_var <- function(var_name, n_rows, participant_idx = NULL) {
  # Mental health scores (typically 1-5 Likert scales)
  if (grepl("^mh_", var_name)) {
    return(runif(n_rows, min = 1, max = 5))
  }

  # Participant IDs (family IDs, etc.)
  else if (grepl("_id", var_name) && !grepl("session", var_name)) {
    if (!is.null(participant_idx) && length(participant_idx) > 0) {
      # Create cross-classified structure: multiple participants per family
      # Increased to 200 families (from 50) to preserve sample size after family selection
      # With 300 participants and 200 families, family selection yields ~150 participants
      # instead of 50, providing sufficient sample for complex models
      n_families <- 200
      family_idx <- ((participant_idx - 1) %% n_families) + 1
      return(paste0("FAM_", sprintf("%05d", family_idx)))
    } else {
      return(sample(paste0("ID_", 1:100), n_rows, replace = TRUE))
    }
  }

  # Site variables (22 ABCD study sites)
  else if (grepl("site", var_name)) {
    sites <- paste0("site", sprintf("%02d", 1:22))
    if (!is.null(participant_idx) && length(participant_idx) > 0) {
      # Create cross-classified structure: distribute across sites
      # Use deterministic assignment based on participant index
      # This creates non-perfect nesting: site:family:participant
      # Note: We use modulo arithmetic instead of set.seed() to maintain
      # determinism from the global seed set in mock_create_dataset_abcd()
      site_idx <- ((participant_idx - 1) %% 22) + 1
      return(factor(sites[site_idx]))
    } else {
      return(factor(sample(sites, n_rows, replace = TRUE)))
    }
  }

  # Sex/Gender variables - FIXED to return numeric codes
  # ABCD stores sex as numeric: 1 = Male, 2 = Female
  else if (grepl("(cohort_sex|demo.*sex)", var_name, ignore.case = TRUE)) {
    return(sample(c(1, 2), n_rows, replace = TRUE))
  }

  # Age variables
  else if (grepl("demo.*age", var_name, ignore.case = TRUE)) {
    # Ages 9-16 for ABCD (in years)
    return(runif(n_rows, min = 9, max = 16))
  }

  # Visit age - in months, varies across sessions
  else if (grepl("visit_age", var_name, ignore.case = TRUE)) {
    # Ages in months: typically 108-192 months (9-16 years)
    return(runif(n_rows, min = 108, max = 192))
  }

  # Race variables - FIXED to return numeric codes
  # ABCD race codes: 1=White, 2=Black, 3=Hispanic, 4=Asian, 5=AIAN, 6=NHPI, 7=Other, 8=Refuse
  else if (grepl("(cohort_race|demo.*race)", var_name, ignore.case = TRUE)) {
    return(sample(1:8, n_rows, replace = TRUE))
  }

  else if (grepl("demo.*ethnicity", var_name, ignore.case = TRUE)) {
    ethnicity <- c("Hispanic", "Not Hispanic")
    return(factor(sample(ethnicity, n_rows, replace = TRUE)))
  }

  # Parent/Cohort Education - ADDED
  # ABCD parent education scale: 1-21 (essentially years of education)
  # Ensure good distribution across levels for categorization
  # Typical categories: <=12 (HS), 13-14 (HS+), 15-17 (Some college), 18 (BA), 19-21 (Grad)
  # Matches: cohort_edu, demo__ed, p_demo__ed__slf
  else if (grepl("(cohort_edu|demo.*_ed_|_ed__slf)", var_name, ignore.case = TRUE)) {
    # Weighted sample to create roughly equal categorical distributions
    # Reduced weight for 1-12 to prevent all participants ending in "Less than HS"
    # Weights: 1-12 (20%), 13-14 (20%), 15-17 (30%), 18 (15%), 19-21 (15%)
    return(sample(1:21, n_rows, replace = TRUE,
                  prob = c(rep(0.5, 12), rep(3, 2), rep(3, 3), 4, rep(1.5, 3))))
  }

  # Household Income - ADDED
  # ABCD household income (3-level version): 1-3
  else if (grepl("cohort_income", var_name, ignore.case = TRUE)) {
    return(sample(1:3, n_rows, replace = TRUE))
  }

  # Handedness (Edinburgh Handedness Inventory Score)
  else if (grepl("ehis.*score", var_name, ignore.case = TRUE)) {
    # Return as character strings to match ABCD data format
    # Tutorials will convert to factor with appropriate labels
    return(sample(c("1", "2", "3"), n_rows, replace = TRUE))
  }

  # Physical measures
  else if (grepl("anthro.*height", var_name, ignore.case = TRUE)) {
    return(runif(n_rows, min = 130, max = 180))  # cm
  }

  else if (grepl("anthro.*weight", var_name, ignore.case = TRUE)) {
    return(runif(n_rows, min = 30, max = 90))  # kg
  }

  else if (grepl("anthro.*bmi", var_name, ignore.case = TRUE)) {
    return(runif(n_rows, min = 15, max = 30))
  }

  # NIH Toolbox - ENHANCED with more specific patterns
  # T-scores (mean=50, SD=10)
  else if (grepl("nihtb.*tscore", var_name, ignore.case = TRUE)) {
    return(runif(n_rows, min = 70, max = 130))
  }

  # Age-corrected scores
  else if (grepl("nihtb.*(agecor|age_corr)", var_name, ignore.case = TRUE)) {
    return(runif(n_rows, min = 80, max = 120))
  }

  # Raw scores (catch-all for other NIH Toolbox)
  else if (grepl("nihtb", var_name, ignore.case = TRUE)) {
    return(runif(n_rows, min = 0, max = 100))
  }

  # Binary indicators
  else if (grepl("^(is_|has_|flag_)", var_name)) {
    return(sample(c(0, 1), n_rows, replace = TRUE))
  }

  # Count variables
  else if (grepl("(count|number|total)_", var_name, ignore.case = TRUE)) {
    return(rpois(n_rows, lambda = 3))
  }

  # Default: continuous 0-1
  else {
    return(runif(n_rows))
  }
}

#' Generate autocorrelated longitudinal data
#'
#' Creates realistic longitudinal data with temporal autocorrelation
#' Uses AR(1) process: value_t = rho * value_(t-1) + sqrt(1-rho^2) * noise
#'
#' @param n_participants Number of participants
#' @param n_sessions Number of time points
#' @param mean Mean value
#' @param sd Standard deviation
#' @param autocorr Autocorrelation coefficient (0-1, typically 0.5-0.7)
#' @param predictor Optional: vector of predictor values to create cross-sectional correlation
#' @param predictor_effect Strength of predictor effect (0-1, creates cross-correlation)
#' @return Vector of length n_participants * n_sessions with autocorrelated values
generate_autocorrelated_data <- function(n_participants, n_sessions, mean = 3, sd = 1,
                                        autocorr = 0.6, predictor = NULL, predictor_effect = 0) {
  # Create result vector - will be filled by participant
  result <- numeric(n_participants * n_sessions)

  # Calculate innovation SD for AR(1) process
  innovation_sd <- sd * sqrt(1 - autocorr^2)

  # Generate person-level random intercepts to create stable individual differences
  # This is crucial for creating between-person stability
  person_intercepts <- rnorm(n_participants, mean = 0, sd = sd * 0.7)

  for (i in 1:n_participants) {
    # Calculate row indices for this participant in expand.grid output
    # expand.grid cycles: (p1,s1), (p1,s2), (p1,s3), ..., (p2,s1), (p2,s2), ...
    idx_start <- (i - 1) * n_sessions + 1
    idx_end <- i * n_sessions
    indices <- idx_start:idx_end

    # Generate AR(1) process for this participant
    values <- numeric(n_sessions)

    # Person-specific mean (adds stable individual differences)
    person_mean <- mean + person_intercepts[i]

    # Initial value
    if (!is.null(predictor) && predictor_effect > 0) {
      # Start with correlation to predictor
      values[1] <- person_mean + predictor_effect * (predictor[indices[1]] - mean) +
                   rnorm(1, mean = 0, sd = innovation_sd * sqrt(1 - predictor_effect^2) * 0.7)
    } else {
      values[1] <- rnorm(1, mean = person_mean, sd = sd * 0.5)
    }

    # Generate subsequent timepoints with AR(1) structure
    for (t in 2:n_sessions) {
      # Standard AR(1) with person-specific mean: value_t = mean_i*(1-rho) + rho*value_(t-1) + epsilon
      ar_component <- person_mean * (1 - autocorr) + autocorr * values[t-1]

      if (!is.null(predictor) && predictor_effect > 0) {
        # Add predictor effect as an additional component
        # This creates cross-sectional correlation while preserving temporal autocorrelation
        pred_component <- predictor_effect * (predictor[indices[t]] - mean)
        innovation <- rnorm(1, mean = 0, sd = innovation_sd * sqrt(1 - predictor_effect^2) * 0.7)
        values[t] <- ar_component + pred_component + innovation
      } else {
        innovation <- rnorm(1, mean = 0, sd = innovation_sd * 0.7)
        values[t] <- ar_component + innovation
      }
    }

    # Constrain to reasonable range (0-6 for typical scales, prevent extreme values)
    values <- pmax(pmin(values, 6), 0)

    # Insert into result vector at correct positions
    result[indices] <- values
  }

  return(result)
}

#' Ensure minimum complete cases for key variables
#'
#' @param data Data frame with mock data
#' @param min_complete Minimum number of participants with complete data
#' @param education_var Optional: name of education variable for stratification
#' @return Data frame with guaranteed complete cases
ensure_complete_cases <- function(data, min_complete = 100, education_var = NULL) {
  # Identify unique participants
  participants <- unique(data$participant_id)
  n_participants <- length(participants)

  if (n_participants < min_complete) {
    min_complete <- n_participants
  }

  # ENHANCEMENT: Stratify by education if variable exists
  complete_participants <- participants[1:min_complete]  # Default

  if (!is.null(education_var) && education_var %in% names(data)) {
    # Get education values for each participant (take first session)
    participant_data <- data[!duplicated(data$participant_id), ]
    edu_col_idx <- which(names(participant_data) == education_var)

    if (length(edu_col_idx) > 0) {
      edu_values <- participant_data[[education_var]]

      if (is.numeric(edu_values) && !all(is.na(edu_values))) {
        # For numeric education (1-21), create 5 equal groups
        edu_breaks <- c(0, 12, 14, 17, 18, 22)  # Less than HS, HS, Some College, Bachelor's, Graduate
        edu_group <- cut(edu_values, breaks = edu_breaks, labels = FALSE, include.lowest = TRUE)

        # Sample proportionally from each group
        n_per_group <- ceiling(min_complete / 5)
        complete_participants <- c()

        for (g in 1:5) {
          group_participants <- participant_data$participant_id[!is.na(edu_group) & edu_group == g]
          if (length(group_participants) > 0) {
            n_sample <- min(n_per_group, length(group_participants))
            complete_participants <- c(complete_participants,
                                      sample(group_participants, n_sample, replace = FALSE))
          }
        }

        # If we don't have enough, sample more from available participants
        if (length(complete_participants) < min_complete) {
          remaining <- setdiff(participants, complete_participants)
          n_needed <- min_complete - length(complete_participants)
          if (length(remaining) >= n_needed) {
            complete_participants <- c(complete_participants, sample(remaining, n_needed))
          } else {
            complete_participants <- c(complete_participants, remaining)
          }
        }

        # Trim to exactly min_complete
        complete_participants <- complete_participants[1:min(min_complete, length(complete_participants))]
      }
    }
  }

  # For these participants, replace any NAs with valid values
  for (col in names(data)) {
    if (col == "participant_id" || col == "session_id") {
      next  # Skip ID columns
    }

    # Find rows for complete participants that have NAs in this column
    needs_filling <- data$participant_id %in% complete_participants & is.na(data[[col]])

    if (any(needs_filling)) {
      if (is.numeric(data[[col]])) {
        # Use median of non-NA values
        fill_value <- median(data[[col]], na.rm = TRUE)
        if (is.na(fill_value)) {
          fill_value <- 50  # Fallback if all NA
        }
        data[needs_filling, col] <- fill_value
      } else if (is.factor(data[[col]]) || is.character(data[[col]])) {
        # Use most common level
        level_counts <- table(data[[col]])
        if (length(level_counts) > 0) {
          fill_value <- names(sort(level_counts, decreasing = TRUE))[1]
          data[needs_filling, col] <- fill_value
        }
      }
    }
  }

  return(data)
}

#' Mock implementation of create_dataset() and create_dataset_abcd()
#'
#' Generates synthetic data matching the ABCD Study structure
#' without requiring access to real participant data.
#'
#' @param dir_data Path to data directory (ignored in mock)
#' @param study Study name (e.g., "abcd", "hbcd") - used to determine session structure
#' @param vars Character vector of variable names to include
#' @param sessions Character vector of session IDs (e.g., c("ses-03A", "ses-04A"))
#' @param release Release version (ignored in mock)
#' @param ... Additional arguments (ignored in mock)
#' @return tibble with mock ABCD data structure
mock_create_dataset <- function(dir_data, study = "abcd", vars, sessions = NULL, release = "latest", ...) {
  # Capture additional arguments
  extra_args <- list(...)
  categ_to_factor <- if (!is.null(extra_args$categ_to_factor)) extra_args$categ_to_factor else FALSE
  value_to_na <- if (!is.null(extra_args$value_to_na)) extra_args$value_to_na else FALSE

  # If sessions not provided, infer from study type
  # For ABCD: default to annual assessments from years 0-6
  if (is.null(sessions)) {
    if (tolower(study) == "abcd") {
      sessions <- c("ses-00A", "ses-01A", "ses-02A", "ses-03A", "ses-04A", "ses-05A", "ses-06A")
    } else if (tolower(study) == "hbcd") {
      sessions <- c("ses-V01", "ses-V02", "ses-V03", "ses-V04")
    } else {
      # Default fallback
      sessions <- c("ses-01", "ses-02", "ses-03", "ses-04")
    }
  }
  # Set fixed seed for reproducible mock data generation
  # This ensures deterministic behavior across different environments/runs
  # and prevents occasional ill-conditioned covariance matrices in lavaan
  set.seed(12345)

  # Load required packages
  if (!requireNamespace("tibble", quietly = TRUE)) {
    # Fall back to data.frame if tibble not available
    use_tibble <- FALSE
  } else {
    use_tibble <- TRUE
  }

  # Configuration: read from environment variable or default to 500
  # Increased from 300 to 500 to ensure sufficient sample after filtering
  n_participants <- as.integer(Sys.getenv("MOCK_N_PARTICIPANTS", "500"))

  # Generate participant IDs
  participant_ids <- paste0("NDAR_", sprintf("%05d", 1:n_participants))

  # Create base data frame with all session combinations
  # This creates a long-format dataset
  # IMPORTANT: session_id must be a factor to match real ABCD data structure
  # Real create_dataset_abcd() returns session_id as factor, which allows
  # as.numeric(session_id) to extract integer codes (1, 2, 3, 4...)
  data <- expand.grid(
    participant_id = participant_ids,
    session_id = factor(sessions, levels = sessions),
    stringsAsFactors = FALSE
  )

  # Add requested variables with appropriate mock values
  # For variables that should be stable across sessions (participant-level),
  # generate once per participant and replicate
  stable_vars <- c()
  autocorr_vars <- c()
  generated_vars <- c()  # Track which variables have been generated

  for (var in vars) {
    # Skip if already generated as part of a correlated pair
    if (var %in% generated_vars) {
      next
    }

    # Check if variable should be stable (design variables, demographics, IDs, handedness)
    # ENHANCED: Added "cohort_" to stable patterns
    is_stable <- grepl("(design_|demo_|cohort_|_id|site|sex|race|ethnicity|family|ehis)", var, ignore.case = TRUE)

    # ENHANCEMENT: Check if variable should have autocorrelation (mental health, behavioral)
    is_autocorr <- grepl("^mh_", var, ignore.case = TRUE)

    if (is_stable) {
      # Generate once per participant and replicate across sessions
      per_participant <- sapply(1:n_participants, function(i) {
        generate_mock_var(var, 1, participant_idx = i)
      })
      # expand.grid cycles through participants first, so use 'times' not 'each'
      data[[var]] <- rep(per_participant, times = length(sessions))
      stable_vars <- c(stable_vars, var)
      generated_vars <- c(generated_vars, var)
    } else if (is_autocorr) {
      # ENHANCEMENT: Check for correlated pairs (e.g., suppression & victimization, depression & anxiety)
      # Look for another mh_ variable in the list that hasn't been generated yet
      remaining_mh_vars <- setdiff(grep("^mh_", vars, value = TRUE), generated_vars)
      paired_var <- NULL

      if (length(remaining_mh_vars) > 1) {
        # Find if there's a natural pair (vict/suppr, dep/anx, etc.)
        for (candidate in remaining_mh_vars[remaining_mh_vars != var]) {
          # Check for common paired patterns
          if ((grepl("vict", var, ignore.case = TRUE) && grepl("suppr|erq", candidate, ignore.case = TRUE)) ||
              (grepl("suppr|erq", var, ignore.case = TRUE) && grepl("vict", candidate, ignore.case = TRUE)) ||
              (grepl("dep", var, ignore.case = TRUE) && grepl("anx", candidate, ignore.case = TRUE)) ||
              (grepl("anx", var, ignore.case = TRUE) && grepl("dep", candidate, ignore.case = TRUE))) {
            paired_var <- candidate
            break
          }
        }
      }

      if (!is.null(paired_var)) {
        # Generate correlated pair - simplified approach for better stability
        # Create person-level random effects (stable across time)
        person_effects_var1 <- rnorm(n_participants, mean = 3, sd = 1)
        person_effects_var2 <- 0.5 * person_effects_var1 + rnorm(n_participants, mean = 1.5, sd = 0.8)

        # Expand to all rows with small time-specific noise
        # expand.grid cycles participants first, so use 'times' not 'each'
        var1_values <- rep(person_effects_var1, times = length(sessions)) +
                       rnorm(nrow(data), mean = 0, sd = 0.3)
        var2_values <- rep(person_effects_var2, times = length(sessions)) +
                       rnorm(nrow(data), mean = 0, sd = 0.3)

        # Constrain to 0-6 range
        data[[var]] <- pmax(pmin(var1_values, 6), 0)
        data[[paired_var]] <- pmax(pmin(var2_values, 6), 0)

        autocorr_vars <- c(autocorr_vars, var, paired_var)
        generated_vars <- c(generated_vars, var, paired_var)
      } else {
        # Single variable - person-level effects with small noise
        person_effects <- rnorm(n_participants, mean = 3, sd = 1)
        values <- rep(person_effects, times = length(sessions)) +
                  rnorm(nrow(data), mean = 0, sd = 0.3)
        data[[var]] <- pmax(pmin(values, 6), 0)

        autocorr_vars <- c(autocorr_vars, var)
        generated_vars <- c(generated_vars, var)
      }
    } else {
      # Time-varying: generate independently for each row
      data[[var]] <- generate_mock_var(var, nrow(data), participant_idx = NULL)
      generated_vars <- c(generated_vars, var)
    }
  }

  # ENHANCEMENT: Ensure minimum complete cases with education stratification
  # Increased to 250 (from 150) to ensure sufficient data after filtering/categorization
  # Many tutorials do family selection which reduces sample by ~50%
  # Detect education variable for stratification
  education_var <- NULL
  edu_patterns <- c("_ed_", "education", "cohort_edu")
  for (pattern in edu_patterns) {
    matching_vars <- grep(pattern, vars, value = TRUE, ignore.case = TRUE)
    if (length(matching_vars) > 0) {
      education_var <- matching_vars[1]  # Use first match
      break
    }
  }

  data <- ensure_complete_cases(data, min_complete = 250, education_var = education_var)

  # Apply categ_to_factor transformation if requested
  # This mimics NBDCtools behavior: convert categorical variables (IDs, site, etc.) to factors
  if (categ_to_factor) {
    # Convert participant_id to factor (matches real NBDCtools behavior)
    if ("participant_id" %in% names(data)) {
      data$participant_id <- factor(data$participant_id)
    }

    # Convert family ID variables to factors
    for (col in names(data)) {
      if (grepl("_id__fam|family_id", col, ignore.case = TRUE) && is.character(data[[col]])) {
        data[[col]] <- factor(data[[col]])
      }
    }

    # Note: session_id and site variables are already factors from generate_mock_var()
    # so no additional conversion needed
  }

  # Convert to tibble if available
  if (use_tibble) {
    data <- tibble::as_tibble(data)
  }

  # Print info message (helpful for debugging)
  cat(sprintf(
    "Mock data: %d participants × %d sessions = %d rows, %d variables\n",
    n_participants,
    length(sessions),
    nrow(data),
    ncol(data)
  ))
  cat(sprintf("  → %d guaranteed complete participants\n", min(250, n_participants)))
  if (length(autocorr_vars) > 0) {
    cat(sprintf("  → %d variables with AR(1) autocorrelation (r=0.6)\n", length(autocorr_vars)))
  }
  if (!is.null(education_var)) {
    cat(sprintf("  → Stratified by education variable: %s\n", education_var))
  }

  return(data)
}

# Alias for backward compatibility
mock_create_dataset_abcd <- mock_create_dataset

# Store the mock functions in global environment
# This ensures they persist even after packages are loaded
assign("create_dataset", mock_create_dataset, envir = .GlobalEnv)
assign("create_dataset_abcd", mock_create_dataset_abcd, envir = .GlobalEnv)

# Create a wrapper to re-apply mock after package loads
.mock_post_load <- function(pkgname, pkgpath) {
  if (pkgname == "NBDCtools") {
    assign("create_dataset", mock_create_dataset, envir = .GlobalEnv)
    assign("create_dataset_abcd", mock_create_dataset_abcd, envir = .GlobalEnv)
    cat("  → Re-applied mock create_dataset() and create_dataset_abcd() after NBDCtools load\n")
  }
}

# Hook into package loading
setHook(packageEvent("NBDCtools", "onLoad"), .mock_post_load)
setHook(packageEvent("NBDCtools", "attach"), .mock_post_load)

# Print confirmation message
cat("✓ Mock data generator loaded (ENHANCED VERSION)\n")
cat("  create_dataset() and create_dataset_abcd() will override NBDCtools versions\n")
cat("  Enhancements: numeric sex/race codes, cohort variables, complete cases guarantee\n\n")
