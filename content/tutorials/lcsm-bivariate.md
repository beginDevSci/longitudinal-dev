---
title: "LCSM: Bivariate"
slug: lcsm-bivariate
author: Biostatistics Working Group
date_iso: 2026-03-01
tags:
  - abcd-study
  - latent-change
  - structural-equation-modeling
  - bivariate
family: LCSM
family_label: Latent Change Score Models (LCSM)
engine: lavaan
engines:
  - lavaan
covariates: None
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Examine coupled developmental change across two constructs using a bivariate latent change score model with four time points (Baseline, Year 2, Year 4, Year 6), estimating cross-domain coupling parameters for internalizing and externalizing problems in ABCD youth.
description: Examine coupled developmental change across two constructs using a bivariate latent change score model with four time points (Baseline, Year 2, Year 4, Year 6), estimating cross-domain coupling parameters for internalizing and externalizing problems in ABCD youth.
---

# Overview

## Summary {.summary}

Bivariate Latent Change Score Models (BLCSM) extend the univariate LCSM to examine coupled change across two developmental processes. Rather than modeling each construct in isolation, BLCSM estimates cross-domain coupling parameters that test whether the level of one process predicts subsequent change in the other. This tutorial applies BLCSM with four time points (Baseline, Year 2, Year 4, Year 6) to analyze coupled internalizing and externalizing problem changes in ABCD youth, using CBCL syndrome scale T-scores. These behavioral constructs have moderate adjacent-wave stability and meaningful within-person variability — properties well-suited to latent change score modeling.

## Features {.features}

- **When to Use:** Apply when you have repeated measures on two constructs and hypothesize that change in one domain depends on the level or change in another domain (e.g., internalizing symptoms driving externalizing change, or vice versa).
- **Key Advantage:** BLCSM separates within-domain dynamics (constant change) from cross-domain coupling, allowing direct tests of developmental leading-lagging relationships between constructs.
- **What You'll Learn:** How to specify univariate LCSMs for two processes, combine them into a bivariate model with cross-domain coupling, compare uncoupled versus coupled models, and interpret coupling parameters.

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
library(tidyverse)    # Collection of R packages for data science
library(arrow)        # For reading Parquet files
library(gtsummary)    # Creating publication-quality tables
library(lavaan)       # Structural Equation Modeling in R
library(broom)        # For tidying model outputs
library(gt)           # For creating formatted tables

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "mh_p_cbcl__synd__int_tscore",
    "mh_p_cbcl__synd__ext_tscore"
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
### Create a long-form dataset with relevant columns
# Uses 4 biennial time points for model identification (matching basic LCSM)
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site,
         ab_g_stc__design_id__fam, mh_p_cbcl__synd__int_tscore,
         mh_p_cbcl__synd__ext_tscore) %>%
  filter_events_abcd(conditions = c("annual")) %>%
  filter(session_id %in% c("ses-00A", "ses-02A", "ses-04A", "ses-06A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-02A", "ses-04A", "ses-06A"),
                        labels = c("Baseline", "Year_2", "Year_4", "Year_6"))
  ) %>%
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    internalizing = mh_p_cbcl__synd__int_tscore,
    externalizing = mh_p_cbcl__synd__ext_tscore
  ) %>%
  droplevels() %>%
  drop_na(internalizing, externalizing)

### Reshape data from long to wide format for LCSM
# No drop_na on wide data — lavaan's missing = "ml" (FIML) uses all available data
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = c(internalizing, externalizing),
    names_sep = "_"
  )
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table for both constructs
descriptives_table <- df_long %>%
  select(session_id, internalizing, externalizing) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      internalizing ~ "Internalizing (T-score)",
      externalizing ~ "Externalizing (T-score)"
    ),
    statistic = list(all_continuous() ~ "{mean} ({sd})")
  ) %>%
  modify_header(all_stat_cols() ~ "{level}<br>N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "Assessment Wave") %>%
  bold_labels() %>%
  italicize_levels()

### Apply compact styling
theme_gtsummary_compact()

descriptives_table <- as_gt(descriptives_table)

### Save the table as HTML
gt::gtsave(descriptives_table, filename = "descriptives_table.html")

### Print the table
descriptives_table
```

## Descriptive Statistics Output {.output}

/stage4-artifacts/lcsm-bivariate/descriptives_table.html

# Statistical Analysis

## Step 1: Univariate LCSM for Internalizing {.code}

```r
### Fit univariate LCSM for internalizing to establish baseline dynamics
# Parsimonious specification: equal change variances, equal residual variances,
# initial-to-first-change covariance only (all other latent covariances zero)
int_model <- '
  # Latent true scores for internalizing (4 time points)
  i1 =~ 1*internalizing_Baseline
  i2 =~ 1*internalizing_Year_2
  i3 =~ 1*internalizing_Year_4
  i4 =~ 1*internalizing_Year_6

  # Latent change scores for internalizing (3 periods)
  di12 =~ 1*i2
  di23 =~ 1*i3
  di34 =~ 1*i4

  # Autoregressive paths (carry forward prior true score)
  i2 ~ 1*i1
  i3 ~ 1*i2
  i4 ~ 1*i3

  # Means (1 initial + 3 change means)
  i1 ~ 1
  di12 ~ 1
  di23 ~ 1
  di34 ~ 1

  # Variances
  i1 ~~ i1                    # Initial status variance
  di12 ~~ d_int*di12          # Change variances constrained equal
  di23 ~~ d_int*di23
  di34 ~~ d_int*di34

  # Initial status to first change covariance only
  i1 ~~ di12

  # Fix all other latent covariances to zero
  i1 ~~ 0*di23
  i1 ~~ 0*di34
  di12 ~~ 0*di23
  di12 ~~ 0*di34
  di23 ~~ 0*di34

  # Fix manifest intercepts to zero
  internalizing_Baseline ~ 0*1
  internalizing_Year_2 ~ 0*1
  internalizing_Year_4 ~ 0*1
  internalizing_Year_6 ~ 0*1

  # Residual variances constrained equal (measurement error)
  internalizing_Baseline ~~ e_int*internalizing_Baseline
  internalizing_Year_2 ~~ e_int*internalizing_Year_2
  internalizing_Year_4 ~~ e_int*internalizing_Year_4
  internalizing_Year_6 ~~ e_int*internalizing_Year_6

  # Fix intermediate true score residual variances to zero
  i2 ~~ 0*i2
  i3 ~~ 0*i3
  i4 ~~ 0*i4
'

fit_int <- sem(int_model, data = df_wide, missing = "ml")
```

## Step 2: Univariate LCSM for Externalizing {.code}

```r
### Fit univariate LCSM for externalizing
# Same parsimonious specification
ext_model <- '
  # Latent true scores for externalizing (4 time points)
  e1 =~ 1*externalizing_Baseline
  e2 =~ 1*externalizing_Year_2
  e3 =~ 1*externalizing_Year_4
  e4 =~ 1*externalizing_Year_6

  # Latent change scores for externalizing (3 periods)
  de12 =~ 1*e2
  de23 =~ 1*e3
  de34 =~ 1*e4

  # Autoregressive paths
  e2 ~ 1*e1
  e3 ~ 1*e2
  e4 ~ 1*e3

  # Means (1 initial + 3 change means)
  e1 ~ 1
  de12 ~ 1
  de23 ~ 1
  de34 ~ 1

  # Variances
  e1 ~~ e1                    # Initial status variance
  de12 ~~ d_ext*de12          # Change variances constrained equal
  de23 ~~ d_ext*de23
  de34 ~~ d_ext*de34

  # Initial status to first change covariance only
  e1 ~~ de12

  # Fix all other latent covariances to zero
  e1 ~~ 0*de23
  e1 ~~ 0*de34
  de12 ~~ 0*de23
  de12 ~~ 0*de34
  de23 ~~ 0*de34

  # Fix manifest intercepts to zero
  externalizing_Baseline ~ 0*1
  externalizing_Year_2 ~ 0*1
  externalizing_Year_4 ~ 0*1
  externalizing_Year_6 ~ 0*1

  # Residual variances constrained equal (measurement error)
  externalizing_Baseline ~~ e_ext*externalizing_Baseline
  externalizing_Year_2 ~~ e_ext*externalizing_Year_2
  externalizing_Year_4 ~~ e_ext*externalizing_Year_4
  externalizing_Year_6 ~~ e_ext*externalizing_Year_6

  # Fix intermediate true score residual variances to zero
  e2 ~~ 0*e2
  e3 ~~ 0*e3
  e4 ~~ 0*e4
'

fit_ext <- sem(ext_model, data = df_wide, missing = "ml")
```

## Step 3: Bivariate LCSM with Cross-Domain Coupling {.code}

```r
### Bivariate LCSM: coupled internalizing and externalizing change (4 time points)
# Equal change variances per domain, equal residual variances per domain,
# initial-to-first-change covariance only, concurrent cross-domain change covariances
bivariate_model <- '
  # --- Internalizing Process ---
  i1 =~ 1*internalizing_Baseline
  i2 =~ 1*internalizing_Year_2
  i3 =~ 1*internalizing_Year_4
  i4 =~ 1*internalizing_Year_6

  di12 =~ 1*i2
  di23 =~ 1*i3
  di34 =~ 1*i4

  i2 ~ 1*i1
  i3 ~ 1*i2
  i4 ~ 1*i3

  i1 ~ 1
  di12 ~ 1
  di23 ~ 1
  di34 ~ 1

  i1 ~~ i1
  di12 ~~ d_int*di12
  di23 ~~ d_int*di23
  di34 ~~ d_int*di34

  i1 ~~ di12
  i1 ~~ 0*di23
  i1 ~~ 0*di34
  di12 ~~ 0*di23
  di12 ~~ 0*di34
  di23 ~~ 0*di34

  internalizing_Baseline ~ 0*1
  internalizing_Year_2 ~ 0*1
  internalizing_Year_4 ~ 0*1
  internalizing_Year_6 ~ 0*1

  internalizing_Baseline ~~ e_int*internalizing_Baseline
  internalizing_Year_2 ~~ e_int*internalizing_Year_2
  internalizing_Year_4 ~~ e_int*internalizing_Year_4
  internalizing_Year_6 ~~ e_int*internalizing_Year_6

  i2 ~~ 0*i2
  i3 ~~ 0*i3
  i4 ~~ 0*i4

  # --- Externalizing Process ---
  e1 =~ 1*externalizing_Baseline
  e2 =~ 1*externalizing_Year_2
  e3 =~ 1*externalizing_Year_4
  e4 =~ 1*externalizing_Year_6

  de12 =~ 1*e2
  de23 =~ 1*e3
  de34 =~ 1*e4

  e2 ~ 1*e1
  e3 ~ 1*e2
  e4 ~ 1*e3

  e1 ~ 1
  de12 ~ 1
  de23 ~ 1
  de34 ~ 1

  e1 ~~ e1
  de12 ~~ d_ext*de12
  de23 ~~ d_ext*de23
  de34 ~~ d_ext*de34

  e1 ~~ de12
  e1 ~~ 0*de23
  e1 ~~ 0*de34
  de12 ~~ 0*de23
  de12 ~~ 0*de34
  de23 ~~ 0*de34

  externalizing_Baseline ~ 0*1
  externalizing_Year_2 ~ 0*1
  externalizing_Year_4 ~ 0*1
  externalizing_Year_6 ~ 0*1

  externalizing_Baseline ~~ e_ext*externalizing_Baseline
  externalizing_Year_2 ~~ e_ext*externalizing_Year_2
  externalizing_Year_4 ~~ e_ext*externalizing_Year_4
  externalizing_Year_6 ~~ e_ext*externalizing_Year_6

  e2 ~~ 0*e2
  e3 ~~ 0*e3
  e4 ~~ 0*e4

  # --- Cross-Domain Coupling ---
  # Does internalizing level predict subsequent externalizing change?
  de12 ~ gamma_ie*i1
  de23 ~ gamma_ie*i2
  de34 ~ gamma_ie*i3

  # Does externalizing level predict subsequent internalizing change?
  di12 ~ gamma_ei*e1
  di23 ~ gamma_ei*e2
  di34 ~ gamma_ei*e3

  # --- Cross-Domain Covariances ---
  i1 ~~ e1        # Initial status covariance
  di12 ~~ de12    # Concurrent change covariance period 1
  di23 ~~ de23    # Concurrent change covariance period 2
  di34 ~~ de34    # Concurrent change covariance period 3

  # Fix non-concurrent cross-domain covariances to zero
  i1 ~~ 0*de12 + 0*de23 + 0*de34
  e1 ~~ 0*di12 + 0*di23 + 0*di34
  di12 ~~ 0*de23 + 0*de34
  di23 ~~ 0*de12 + 0*de34
  di34 ~~ 0*de12 + 0*de23
'

fit_bivariate <- sem(bivariate_model, data = df_wide, missing = "ml")

summary(fit_bivariate, fit.measures = TRUE, standardized = TRUE)
```

## Step 4: Compare Uncoupled vs Coupled Models {.code}

```r
### Fit an uncoupled bivariate model (no cross-domain paths) for comparison
uncoupled_model <- '
  # --- Internalizing Process ---
  i1 =~ 1*internalizing_Baseline
  i2 =~ 1*internalizing_Year_2
  i3 =~ 1*internalizing_Year_4
  i4 =~ 1*internalizing_Year_6

  di12 =~ 1*i2
  di23 =~ 1*i3
  di34 =~ 1*i4

  i2 ~ 1*i1
  i3 ~ 1*i2
  i4 ~ 1*i3

  i1 ~ 1
  di12 ~ 1
  di23 ~ 1
  di34 ~ 1

  i1 ~~ i1
  di12 ~~ d_int*di12
  di23 ~~ d_int*di23
  di34 ~~ d_int*di34

  i1 ~~ di12
  i1 ~~ 0*di23
  i1 ~~ 0*di34
  di12 ~~ 0*di23
  di12 ~~ 0*di34
  di23 ~~ 0*di34

  internalizing_Baseline ~ 0*1
  internalizing_Year_2 ~ 0*1
  internalizing_Year_4 ~ 0*1
  internalizing_Year_6 ~ 0*1

  internalizing_Baseline ~~ e_int*internalizing_Baseline
  internalizing_Year_2 ~~ e_int*internalizing_Year_2
  internalizing_Year_4 ~~ e_int*internalizing_Year_4
  internalizing_Year_6 ~~ e_int*internalizing_Year_6

  i2 ~~ 0*i2
  i3 ~~ 0*i3
  i4 ~~ 0*i4

  # --- Externalizing Process ---
  e1 =~ 1*externalizing_Baseline
  e2 =~ 1*externalizing_Year_2
  e3 =~ 1*externalizing_Year_4
  e4 =~ 1*externalizing_Year_6

  de12 =~ 1*e2
  de23 =~ 1*e3
  de34 =~ 1*e4

  e2 ~ 1*e1
  e3 ~ 1*e2
  e4 ~ 1*e3

  e1 ~ 1
  de12 ~ 1
  de23 ~ 1
  de34 ~ 1

  e1 ~~ e1
  de12 ~~ d_ext*de12
  de23 ~~ d_ext*de23
  de34 ~~ d_ext*de34

  e1 ~~ de12
  e1 ~~ 0*de23
  e1 ~~ 0*de34
  de12 ~~ 0*de23
  de12 ~~ 0*de34
  de23 ~~ 0*de34

  externalizing_Baseline ~ 0*1
  externalizing_Year_2 ~ 0*1
  externalizing_Year_4 ~ 0*1
  externalizing_Year_6 ~ 0*1

  externalizing_Baseline ~~ e_ext*externalizing_Baseline
  externalizing_Year_2 ~~ e_ext*externalizing_Year_2
  externalizing_Year_4 ~~ e_ext*externalizing_Year_4
  externalizing_Year_6 ~~ e_ext*externalizing_Year_6

  e2 ~~ 0*e2
  e3 ~~ 0*e3
  e4 ~~ 0*e4

  # --- Cross-Domain Covariances (but no coupling) ---
  i1 ~~ e1
  di12 ~~ de12
  di23 ~~ de23
  di34 ~~ de34

  # Fix non-concurrent cross-domain covariances to zero
  i1 ~~ 0*de12 + 0*de23 + 0*de34
  e1 ~~ 0*di12 + 0*di23 + 0*di34
  di12 ~~ 0*de23 + 0*de34
  di23 ~~ 0*de12 + 0*de34
  di34 ~~ 0*de12 + 0*de23
'

fit_uncoupled <- sem(uncoupled_model, data = df_wide, missing = "ml")

### Format comparison table
comparison_table <- data.frame(
  Model = c("Uncoupled", "Coupled"),
  df = c(fitMeasures(fit_uncoupled, "df"),
         fitMeasures(fit_bivariate, "df")),
  AIC = c(AIC(fit_uncoupled), AIC(fit_bivariate)),
  BIC = c(BIC(fit_uncoupled), BIC(fit_bivariate)),
  CFI = c(fitMeasures(fit_uncoupled, "cfi"),
          fitMeasures(fit_bivariate, "cfi")),
  RMSEA = c(fitMeasures(fit_uncoupled, "rmsea"),
            fitMeasures(fit_bivariate, "rmsea"))
) %>%
  gt() %>%
  tab_header(title = "Model Comparison: Uncoupled vs. Coupled BLCSM") %>%
  fmt_number(columns = c(AIC, BIC, CFI, RMSEA), decimals = 3) %>%
  fmt_number(columns = df, decimals = 0)

gt::gtsave(comparison_table, filename = "model_comparison.html")
comparison_table
```

## Model Comparison Output {.output}

/stage4-artifacts/lcsm-bivariate/model_comparison.html

## Format Coupled Model Results {.code}

```r
### Extract and format key parameters from the coupled model
coupled_params <- broom::tidy(fit_bivariate) %>%
  filter(op %in% c("~1", "~~", "~")) %>%
  select(term, estimate, std.error, statistic, p.value) %>%
  gt() %>%
  tab_header(title = "Bivariate Latent Change Score Model: Coupled Results") %>%
  fmt_number(columns = c(estimate, std.error, statistic, p.value), decimals = 3)

gt::gtsave(coupled_params, filename = "model_summary.html")

### Extract fit indices
fit_indices <- fitMeasures(fit_bivariate,
  c("chisq", "df", "pvalue", "cfi", "tli", "rmsea", "srmr", "aic", "bic"))

fit_indices_table <- data.frame(
  Metric = names(fit_indices),
  Value = as.numeric(fit_indices)
) %>%
  gt() %>%
  tab_header(title = "Model Fit Indices: Coupled BLCSM") %>%
  fmt_number(columns = Value, decimals = 3) %>%
  cols_label(Metric = "Fit Measure", Value = "Value")

gt::gtsave(fit_indices_table, filename = "model_fit_indices.html")
```

## Model Summary Output {.output}

/stage4-artifacts/lcsm-bivariate/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lcsm-bivariate/model_fit_indices.html

## Interpretation {.note}

The coupled bivariate LCSM fit the data well (CFI = 0.970, TLI = 0.962, RMSEA = 0.069, SRMR = 0.057, df = 22) and substantially outperformed the uncoupled model (CFI = 0.934, RMSEA = 0.098, AIC difference = 1,496), confirming that cross-domain coupling improves model fit beyond concurrent covariances alone.

**Within-domain dynamics.** Mean internalizing at baseline was approximately 50 T-score points, consistent with the normed CBCL scale. The initial status variance for internalizing (87.83, p < .001) and externalizing (86.82, p < .001) indicated substantial individual differences at baseline. Change score variances were 31.59 (internalizing) and 22.99 (externalizing), both significant (p < .001), confirming meaningful individual differences in biennial change. The initial-to-first-change covariance was negative for both domains (internalizing: -9.81; externalizing: -7.40, both p < .001), indicating that youth with higher initial symptom levels tended to show less increase (or more decrease) in the first period — a regression-toward-the-mean pattern.

**Cross-domain coupling.** Both coupling parameters were significant and negative. The internalizing-to-externalizing coupling (gamma_ie = -0.164, SE = 0.005, p < .001) indicates that higher internalizing levels predict subsequent decreases in externalizing problems. The externalizing-to-internalizing coupling (gamma_ei = -0.132, SE = 0.005, p < .001) indicates the reverse: higher externalizing levels predict subsequent decreases in internalizing problems. This bidirectional negative coupling suggests a compensatory dynamic — as one symptom domain is elevated, the other tends to decrease in the next period, consistent with a regulatory or resource-competition account rather than a cascading escalation model.

**Cross-domain change covariances.** The concurrent change covariances were positive and significant across all three periods (22.00, 25.95, and 26.42, all p < .001), indicating that within each biennial interval, internalizing and externalizing changes were positively correlated — youth who increased in one domain also tended to increase in the other. This concurrent positive association, combined with the negative temporal coupling, suggests that shared environmental influences produce correlated within-period changes, but the lagged effect of elevated symptoms in one domain is a reduction in the other.

**Residual variances.** Measurement error variances were 24.80 (internalizing) and 17.77 (externalizing), both constrained equal across waves. These represent approximately 22% and 17% of the total observed variance, respectively, indicating good reliability for the CBCL T-scores.

## Visualization {.code}

```r
### Visualize coupled change across both constructs (3 biennial periods)
df_wide <- df_wide %>%
  mutate(
    int_change_12 = internalizing_Year_2 - internalizing_Baseline,
    int_change_23 = internalizing_Year_4 - internalizing_Year_2,
    int_change_34 = internalizing_Year_6 - internalizing_Year_4,
    ext_change_12 = externalizing_Year_2 - externalizing_Baseline,
    ext_change_23 = externalizing_Year_4 - externalizing_Year_2,
    ext_change_34 = externalizing_Year_6 - externalizing_Year_4
  )

### Scatter plots of coupled changes across three biennial periods
plot_data <- bind_rows(
  df_wide %>% select(int = int_change_12, ext = ext_change_12) %>%
    mutate(Period = "Baseline to Year 2"),
  df_wide %>% select(int = int_change_23, ext = ext_change_23) %>%
    mutate(Period = "Year 2 to Year 4"),
  df_wide %>% select(int = int_change_34, ext = ext_change_34) %>%
    mutate(Period = "Year 4 to Year 6")
) %>%
  mutate(Period = factor(Period, levels = c("Baseline to Year 2",
                                             "Year 2 to Year 4",
                                             "Year 4 to Year 6")))

coupled_plot <- ggplot(plot_data, aes(x = int, y = ext)) +
  geom_point(alpha = 0.10, size = 0.6, color = "#4A90D9") +
  geom_smooth(method = "lm", se = TRUE, color = "#E74C3C", linewidth = 1) +
  facet_wrap(~Period) +
  labs(
    title = "Coupled Change: Internalizing vs. Externalizing Across Assessment Periods",
    subtitle = "Bivariate LCSM — Four Time Points (Biennial Intervals)",
    x = "Internalizing Change (T-score)",
    y = "Externalizing Change (T-score)"
  ) +
  theme_minimal()

ggsave(
  filename = "visualization.png",
  plot = coupled_plot,
  width = 12, height = 5, dpi = 300
)
```

## Visualization {.output}

![Coupled Change Scatter Plot](/stage4-artifacts/lcsm-bivariate/visualization.png)

## Visualization Notes {.note}

The scatter plots display the association between observed internalizing change and externalizing change across the three biennial assessment periods. The regression lines summarize the direction and strength of the concurrent association within each period. A positive slope indicates that youth whose internalizing problems increased also tended to show increases in externalizing problems, consistent with correlated developmental change across the two broadband dimensions of psychopathology. Note that these are observed associations — the BLCSM coupling parameters provide the latent (measurement-error-adjusted) estimates of cross-domain predictive relationships that operate with a temporal lag, which is a stronger test of directional influence than concurrent correlation.

# Discussion

The Bivariate Latent Change Score Model extends the univariate LCSM to address a fundamental question in developmental psychopathology: how do changes in one symptom domain relate to changes in another? By estimating cross-domain coupling parameters, the BLCSM tests whether the level of one construct predicts subsequent change in the other, providing evidence for temporal precedence that goes beyond simple concurrent correlations.

Internalizing and externalizing problems are well-suited to bivariate LCSM because they exhibit moderate adjacent-wave stability and meaningful within-person variability — properties that provide sufficient latent change variance for coupling parameters to be estimated reliably. The model comparison between uncoupled and coupled specifications provides a formal test of whether cross-domain dynamics improve model fit beyond what is captured by concurrent covariances alone. When coupling parameters are significant, they suggest that knowing a youth's internalizing (or externalizing) level at one time point improves prediction of their externalizing (or internalizing) change in the next period. Asymmetric coupling — where one direction is significant but not the other — is particularly informative, as it identifies which symptom domain leads and which follows.

The model uses a parsimonious constraint strategy designed for stable estimation with four time points: change variances and residual variances are each constrained equal within domain, the only within-domain covariance is between initial status and the first change score, and cross-domain covariances are limited to concurrent change periods. This provides adequate degrees of freedom (df = 22) for formal goodness-of-fit testing while avoiding the Heywood cases (negative variance estimates) that can arise with more complex covariance structures when inter-wave stability is high.

Important limitations include the assumption that coupling effects are constant across all three periods and that the zero-covariance constraints on non-adjacent and non-concurrent latent variables may be overly restrictive. The CBCL is parent-reported, so shared method variance may inflate within-domain stability and cross-domain covariances. Extensions include models with proportional effects (change depending on own prior level in addition to the other domain), time-varying coupling strengths, and three-process models that incorporate a third developmental domain such as attention problems.

# Additional Resources

### McArdle (2009): Latent Variable Modeling of Differences and Changes {.resource}

Foundational paper introducing the latent change score framework and its bivariate extension for modeling coupled developmental processes across multiple domains.

**Badge:** PAPER
**URL:** https://doi.org/10.1146/annurev.psych.60.110707.163612

### Grimm, Ram & Estabrook: Growth Modeling {.resource}

Textbook covering structural equation modeling approaches to longitudinal data, with detailed chapters on bivariate latent change score models including coupled dynamics and R code.

**Badge:** BOOK
**URL:** https://www.guilford.com/books/Growth-Modeling/Grimm-Ram-Estabrook/9781462526062

### Kievit et al. (2018): Mutualism in Cognitive Development {.resource}

Applied example of bivariate latent change score models demonstrating coupled cognitive development, illustrating how BLCSM can reveal mutualistic relationships between developing abilities.

**Badge:** PAPER
**URL:** https://doi.org/10.1177/0956797617710785

### Usami, Murayama & Hamaker (2019): Unified Framework for Longitudinal Models {.resource}

Comprehensive framework paper comparing latent change score models with cross-lagged panel models, clarifying when bivariate LCSM is preferred and how coupling parameters relate to other approaches for studying reciprocal developmental processes.

**Badge:** PAPER
**URL:** https://doi.org/10.1037/met0000210
