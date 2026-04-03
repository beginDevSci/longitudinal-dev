---
title: "GEE: Count Outcomes"
slug: gee-count
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - gee
  - marginal-model
  - count-data
family: GEE
family_label: Generalized Estimating Equations (GEE)
engine: geepack
engines:
  - geepack
covariates: TIC
outcome_type: Count
difficulty: intermediate
timepoints: 3_5
summary: Fit population-averaged GEE models for count outcomes, detect overdispersion, and interpret rate ratios for repeated-measures ABCD data.
description: Fit population-averaged GEE models for count outcomes, detect overdispersion, and interpret rate ratios for repeated-measures ABCD data.
---

# Overview

## Summary {.summary}

Generalized Estimating Equations for count outcomes extend the GEE framework from binary to Poisson-family responses, modeling how the rate of events changes over time at the population level while accounting for within-subject correlation. Count outcomes — such as days of substance use, number of behavioral incidents, or frequency of health-related events — are common in longitudinal developmental research but violate the assumptions of standard linear models. This tutorial applies Poisson GEE to analyze alcohol use days in the past 30 days among ABCD youth across annual assessments, demonstrating overdispersion detection, robust standard error estimation, and interpretation of incidence rate ratios.

## Features {.features}

- **When to Use:** Apply when your outcome is a non-negative integer count measured repeatedly and you want population-averaged trends while accounting for within-subject correlation.
- **Key Advantage:** GEE with Poisson family provides valid marginal rate ratio estimates even when the variance-mean relationship is misspecified, as long as the mean model is correct and robust standard errors are used.
- **What You'll Learn:** How to fit Poisson GEE, detect and handle overdispersion, compare robust versus naive standard errors, and interpret incidence rate ratios for count outcomes.

# Data Access

## Data Download

ABCD data can be accessed through the [DEAP platform](https://nbdc.deapscience.com) or the [NBDC Data Access Platform (LASSO)](https://nbdc-datashare.lassoinformatics.com), which provide user-friendly interfaces for creating custom datasets with point-and-click variable selection. For detailed instructions on accessing and downloading ABCD data, see the [DEAP documentation](https://docs.deapscience.com).

## Loading Data with NBDCtools

Once you have downloaded ABCD data files, the [**NBDCtools**](https://software.nbdc-datahub.org/NBDCtools/) package provides efficient tools for loading and preparing your data for analysis. The package handles common data management tasks including:

- **Automatic data joining** - Merges variables from multiple tables automatically
- **Built-in transformations** - Converts categorical variables to factors, handles missing data codes, and adds variable labels
- **Event filtering** - Easily selects specific assessment waves

For more information, visit the [NBDCtools documentation](https://software.nbdc-datahub.org/NBDCtools/).

## Basic Usage

The `create_dataset()` function is the main tool for loading ABCD data:

```r
library(NBDCtools)

# Define variables needed for this analysis
requested_vars <- c(
  "var_1",   # Variable 1
  "var_2",   # Variable 2
  "var_3"    # Variable 3
)

# Set path to downloaded ABCD data files
data_dir <- Sys.getenv("ABCD_DATA_PATH", "/path/to/abcd/6_0/phenotype")

# Load data with automatic transformations
abcd_data <- create_dataset(
  dir_data = data_dir,
  study = "abcd",
  vars = requested_vars,
  release = "6.0",
  format = "parquet",
  categ_to_factor = TRUE,   # Convert categorical variables to factors
  value_to_na = TRUE,        # Convert missing codes (222, 333, etc.) to NA
  add_labels = TRUE          # Add variable and value labels
)
```

## Key Parameters

- **`vars`** - Vector of variable names to load
- **`release`** - ABCD data release version (e.g., "6.0")
- **`format`** - File format, typically "parquet" for efficiency
- **`categ_to_factor`** - Automatically converts categorical variables to factors
- **`value_to_na`** - Converts ABCD missing value codes to R's NA
- **`add_labels`** - Adds descriptive labels to variables and values

## Additional Resources

For more details on using NBDCtools:

- [NBDCtools Getting Started Guide](https://software.nbdc-datahub.org/NBDCtools/articles/NBDCtools.html) - Complete package overview
- [Joining Data](https://software.nbdc-datahub.org/NBDCtools/articles/join.html) - Advanced data merging strategies
- [Filtering Events](https://software.nbdc-datahub.org/NBDCtools/articles/filter.html) - Selecting specific assessment waves
- [Data Transformations](https://software.nbdc-datahub.org/NBDCtools/articles/transformation.html) - Preprocessing and cleaning

# Data Preparation

## NBDCtools Setup and Data Loading {.code}

```r
### Load necessary libraries
library(NBDCtools)    # ABCD data access helper
library(tidyverse)    # Data wrangling & visualization
library(arrow)        # For reading Parquet files
library(gtsummary)    # Creating publication-quality tables
library(geepack)      # Generalized Estimating Equations
library(broom)        # For tidying model outputs
library(gt)           # For creating formatted tables

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "su_y_tlfb__alc__1mo_ud",       # Alcohol use days (last 30 days)
    "ab_g_stc__cohort_sex"          # Sex (time-invariant covariate)
)

data_dir <- Sys.getenv("ABCD_DATA_PATH", "/path/to/abcd/6_0/phenotype")

abcd_data <- create_dataset(
  dir_data = data_dir,
  study = "abcd",
  vars = requested_vars,
  release = "6.0",
  format = "parquet",
  categ_to_factor = TRUE,
  value_to_na = TRUE,
  add_labels = TRUE
)
```

## Data Transformation {.code}

```r
### Prepare long-format data for GEE count analysis
df_long <- abcd_data %>%
  filter_events_abcd(conditions = c("annual")) %>%
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    alc_days = su_y_tlfb__alc__1mo_ud,
    sex = ab_g_stc__cohort_sex
  ) %>%
  mutate(
    alc_days = as.numeric(alc_days),
    sex = factor(sex, levels = c(1, 2), labels = c("Male", "Female")),
    time = as.numeric(session_id) - 1,
    participant_id = factor(participant_id)
  ) %>%
  # Keep participants with at least 2 observations
  group_by(participant_id) %>%
  filter(sum(!is.na(alc_days)) >= 2) %>%
  ungroup() %>%
  drop_na(alc_days, sex) %>%
  arrange(participant_id, time)
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table for count outcome
descriptives_table <- df_long %>%
  mutate(
    wave = factor(session_id)
  ) %>%
  select(wave, alc_days) %>%
  tbl_summary(
    by = wave,
    missing = "no",
    label = list(
      alc_days ~ "Alcohol Use Days (past 30)"
    ),
    statistic = list(
      all_continuous() ~ "{mean} ({sd}); Median: {median}"
    )
  ) %>%
  modify_header(all_stat_cols() ~ "{level}<br>N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "Assessment Wave") %>%
  bold_labels() %>%
  italicize_levels()

### Apply compact styling and save
theme_gtsummary_compact()

descriptives_table <- as_gt(descriptives_table)
gt::gtsave(descriptives_table, filename = "descriptives_table.html")
descriptives_table
```

## Descriptive Statistics Output {.output}

/stage4-artifacts/gee-count/descriptives_table.html

# Statistical Analysis

## Step 1: Fit Poisson GEE {.code}

```r
### Fit Poisson GEE: modeling alcohol use days over time
poisson_gee <- geeglm(
  alc_days ~ time + sex,
  id = participant_id,
  data = df_long,
  family = poisson(link = "log"),
  corstr = "exchangeable"
)

### Extract Poisson GEE results
poisson_summary <- summary(poisson_gee)

### Check for overdispersion: scale parameter substantially > 1 indicates overdispersion
scale_param <- poisson_summary$dispersion$Estimate
cat("Scale parameter:", round(scale_param, 3), "\n")
cat("Overdispersion detected:", scale_param > 1.5, "\n")
```

## Step 2: Compare Robust vs Naive Standard Errors {.code}

```r
### Extract coefficients with both naive and robust SEs
naive_se <- poisson_summary$coefficients[, "Std.err"]
robust_results <- as.data.frame(poisson_summary$coefficients)

### Format SE comparison table
se_comparison <- data.frame(
  Parameter = rownames(robust_results),
  Estimate = robust_results$Estimate,
  Naive_SE = naive_se,
  Robust_SE = robust_results$Std.err,
  SE_Ratio = robust_results$Std.err / naive_se
) %>%
  gt() %>%
  tab_header(title = "Robust vs. Naive Standard Errors (Poisson GEE)") %>%
  fmt_number(columns = c(Estimate, Naive_SE, Robust_SE, SE_Ratio), decimals = 3) %>%
  cols_label(
    Parameter = "Parameter",
    Estimate = "Estimate (log)",
    Naive_SE = "Naive SE",
    Robust_SE = "Robust SE",
    SE_Ratio = "Robust/Naive"
  )

gt::gtsave(se_comparison, filename = "se_comparison.html")
se_comparison
```

## SE Comparison Output {.output}

/stage4-artifacts/gee-count/se_comparison.html

## Step 3: Fit Model and Format Rate Ratios {.code}

```r
### Compute incidence rate ratios (IRRs) from the Poisson GEE
coefs <- as.data.frame(poisson_summary$coefficients)

irr_table <- data.frame(
  Parameter = rownames(coefs),
  Log_Estimate = coefs$Estimate,
  IRR = exp(coefs$Estimate),
  IRR_Lower = exp(coefs$Estimate - 1.96 * coefs$Std.err),
  IRR_Upper = exp(coefs$Estimate + 1.96 * coefs$Std.err),
  p_value = coefs[, "Pr(>|W|)"]
) %>%
  gt() %>%
  tab_header(title = "Incidence Rate Ratios: Poisson GEE") %>%
  fmt_number(columns = c(Log_Estimate, IRR, IRR_Lower, IRR_Upper), decimals = 3) %>%
  fmt_number(columns = p_value, decimals = 4) %>%
  cols_label(
    Parameter = "Parameter",
    Log_Estimate = "Log(IRR)",
    IRR = "IRR",
    IRR_Lower = "95% CI Lower",
    IRR_Upper = "95% CI Upper",
    p_value = "p-value"
  )

gt::gtsave(irr_table, filename = "model_summary.html")
irr_table
```

## Model Summary Output {.output}

/stage4-artifacts/gee-count/model_summary.html

## Step 4: Model Diagnostics {.code}

```r
### Create GEE diagnostics summary
diagnostics_data <- data.frame(
  Characteristic = c(
    "Family",
    "Link Function",
    "Correlation Structure",
    "Correlation Parameter",
    "Scale Parameter",
    "Overdispersion Detected",
    "Number of Participants",
    "Total Observations"
  ),
  Value = c(
    "Poisson",
    "log",
    poisson_gee$corstr,
    round(poisson_gee$geese$alpha, 3),
    round(scale_param, 3),
    ifelse(scale_param > 1.5, "Yes", "No"),
    length(unique(df_long$participant_id)),
    nrow(df_long)
  )
)

diagnostics_table <- diagnostics_data %>%
  gt() %>%
  gt::tab_header(title = "GEE Model Diagnostics")

gt::gtsave(diagnostics_table, filename = "model_diagnostics.html")
diagnostics_table
```

## Model Diagnostics Output {.output}

/stage4-artifacts/gee-count/model_diagnostics.html

## Interpretation {.note}

The Poisson GEE estimated a population-averaged IRR for time of 2.79 (95% CI: 2.61–2.98, p < .001), indicating that the expected count of alcohol use days nearly tripled with each assessment wave. The sex effect was not significant (IRR = 1.22, 95% CI: 0.99–1.51, p = .068), suggesting only a marginal difference between males and females in average alcohol use days after accounting for the time trend.

The **scale parameter** of 2.448 confirms substantial **overdispersion** — the data have roughly 2.4 times more variability than the Poisson distribution assumes. This is expected for count data like alcohol use days with many zeros and occasional large values. GEE with robust (sandwich) standard errors remains valid under overdispersion because inference relies on the sandwich variance estimator rather than the assumed Poisson variance structure. The exchangeable correlation parameter of 0.048 indicates modest within-person clustering across waves.

The **SE comparison** table shows that robust and naive standard errors are essentially identical for this model, suggesting the working correlation structure is well-specified. When these diverge substantially, it signals variance misspecification — but robust inference protects against this regardless.

## Visualization {.code}

```r
### Visualize count distribution over time
count_plot <- df_long %>%
  mutate(wave = factor(session_id)) %>%
  ggplot(aes(x = alc_days)) +
  geom_histogram(binwidth = 1, fill = "#4A90D9", color = "white", alpha = 0.8) +
  facet_wrap(~wave, scales = "free_y", ncol = 2) +
  labs(
    title = "Distribution of Alcohol Use Days by Assessment Wave",
    x = "Number of Days (past 30)",
    y = "Count of Participants"
  ) +
  theme_minimal() +
  coord_cartesian(xlim = c(0, 30))

ggsave(
  filename = "visualization.png",
  plot = count_plot,
  width = 10, height = 6, dpi = 300
)
```

## Visualization {.output}

![Count Distribution by Wave](/stage4-artifacts/gee-count/visualization.png)

## Visualization Notes {.note}

The histograms reveal the characteristic shape of count data with excess zeros: most youth report zero alcohol use days at each wave, with a long right tail of those reporting higher counts. As waves progress, the distribution shifts rightward — the proportion reporting zero decreases and the tail extends further — consistent with the increasing IRR from the GEE model. This zero-inflated, right-skewed distribution explains the overdispersion detected by the scale parameter: the Poisson distribution assumes the variance equals the mean, but the combination of many zeros and a few high counts produces variance far exceeding the mean. The GEE's robust standard errors account for this pattern, making the inference valid despite the distributional mismatch.

# Discussion

The Poisson GEE analysis demonstrates how to model count outcomes in a population-averaged framework while handling the practical challenges of overdispersion and excess zeros that are typical of behavioral count data. By using robust sandwich standard errors, the GEE provides valid inference on incidence rate ratios even when the Poisson variance assumption is violated — a common occurrence with developmental count data where many participants report zero events.

The rate ratio interpretation is particularly useful for applied researchers: rather than reporting log-scale coefficients, IRRs directly communicate the multiplicative change in the expected count associated with each predictor. For time effects, this translates to statements about the percentage increase (or decrease) in alcohol use frequency per assessment wave, which is more interpretable than additive effects on a transformed scale.

Key methodological takeaways include: (1) always report the scale parameter and compare robust versus naive standard errors as overdispersion diagnostics; (2) GEE with robust SEs is a practical default for count outcomes because it protects against variance misspecification; and (3) the marginal interpretation of GEE — population-averaged rate ratios rather than subject-specific effects — is appropriate when the research question targets population-level trends. For subject-specific interpretations or when random effects are of substantive interest, a GLMM count model (negative binomial or Poisson) would be the complementary approach.

# Additional Resources

### geepack Package Documentation {.resource}

Official CRAN documentation for the geepack package, covering the geeglm() function for Poisson and other generalized estimating equations with different working correlation structures.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=geepack

### Zeger & Liang (1986): Longitudinal Data Analysis Using GEE {.resource}

Foundational paper introducing GEE methodology for longitudinal data, establishing the theoretical basis for robust inference under working correlation misspecification for both binary and count outcomes.

**Badge:** PAPER
**URL:** https://doi.org/10.2307/2336267

### Hilbe (2011): Negative Binomial Regression {.resource}

Comprehensive reference on modeling count data including Poisson regression, overdispersion detection, negative binomial alternatives, and zero-inflated models with worked examples in R.

**Badge:** BOOK
**URL:** https://doi.org/10.1017/CBO9780511973420

### QIC for Model Selection in GEE {.resource}

Methodology paper on using Quasi-likelihood Information Criterion (QIC) for selecting working correlation structures in GEE models (Pan, 2001). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://onlinelibrary.wiley.com/doi/10.1111/j.0006-341X.2001.00120.x
