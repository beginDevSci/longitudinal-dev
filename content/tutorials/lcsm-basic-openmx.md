---
title: "LCSM: Basic (OpenMx)"
slug: lcsm-basic-openmx
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - latent-change
  - structural-equation-modeling
  - openmx
family: LCSM
family_label: Latent Change Score Models (LCSM)
engine: OpenMx
engines:
  - OpenMx
covariates: None
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Model true change between measurement occasions using a latent change score model with four time points, separating measurement error from systematic developmental change in internalizing problems across ABCD youth.
description: Model true change between measurement occasions using a latent change score model with four time points, separating measurement error from systematic developmental change in internalizing problems across ABCD youth.
---

# Overview

## Summary {.summary}

Latent Change Score Models (LCSM) provide a framework for modeling change between measurement occasions by treating change as a latent variable rather than a simple observed difference. Unlike raw difference scores, LCSM separates true change from measurement error, allowing researchers to estimate the reliability of change, model proportional effects (where change depends on prior status), and correlate change with other variables. This tutorial applies LCSM with four time points (Baseline, Year 2, Year 4, Year 6) to analyze changes in CBCL Internalizing T-scores in ABCD youth, demonstrating how to specify and interpret basic LCSM parameters including initial status, change scores, and their variances and covariances.

## Features {.features}

- **When to Use:** Apply when you want to model change as an explicit latent construct with four or more time points, especially when measurement error is a concern or when you need a properly identified model with testable fit indices.
- **Key Advantage:** LCSM separates true score variance from error variance in the change score, providing more reliable estimates of individual differences in change than simple difference scores.
- **What You'll Learn:** How to specify a basic LCSM in OpenMx, interpret the mean and variance of latent change, assess whether initial status predicts subsequent change, and evaluate model fit.

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
library(OpenMx)       # Matrix-based SEM engine
library(broom)        # For tidying model outputs
library(gt)           # For creating formatted tables

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "mh_p_cbcl__synd__int_tscore"
)

data_dir <- Sys.getenv("ABCD_DATA_PATH", "/path/to/abcd/6_0/phenotype")

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

## Data Transformation {.code}

```r
### Create a long-form dataset with relevant columns
# Uses 4 time points (Baseline, Year 2, Year 4, Year 6) for model identification
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site,
         ab_g_stc__design_id__fam, mh_p_cbcl__synd__int_tscore) %>%
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
    internalizing = mh_p_cbcl__synd__int_tscore
  ) %>%
  droplevels() %>%
  drop_na(internalizing)

### Reshape data from long to wide format for LCSM
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = internalizing,
    names_prefix = "Int_"
  ) %>%
  drop_na(starts_with("Int_"))  # Require complete data across time points
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, internalizing) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      internalizing ~ "Internalizing (T-score)"
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

/stage4-artifacts/lcsm-basic-openmx/descriptives_table.html

# Statistical Analysis

## Define and Fit Basic LCSM with OpenMx {.code}

```r
### Prepare data for OpenMx
mx_data <- df_wide %>%
  select(starts_with("Int_")) %>%
  as.data.frame()

manifest_vars <- c("Int_Baseline", "Int_Year_2",
                    "Int_Year_4", "Int_Year_6")
latent_vars <- c("eta1", "eta2", "eta3", "eta4",
                 "delta12", "delta23", "delta34")

# Parsimonious specification: equal change variances, equal residual variances,
# initial-to-first-change covariance only (all other latent covariances zero)
model <- mxModel(
  "BasicLCSM",
  type = "RAM",
  manifestVars = manifest_vars,
  latentVars = latent_vars,

  # Data
  mxData(observed = mx_data, type = "raw"),

  # --- Factor loadings: latent true scores → observed scores ---
  mxPath(from = "eta1", to = "Int_Baseline", free = FALSE, values = 1),
  mxPath(from = "eta2", to = "Int_Year_2", free = FALSE, values = 1),
  mxPath(from = "eta3", to = "Int_Year_4", free = FALSE, values = 1),
  mxPath(from = "eta4", to = "Int_Year_6", free = FALSE, values = 1),

  # --- Autoregressive paths: carryover from prior true score ---
  mxPath(from = "eta1", to = "eta2", free = FALSE, values = 1),
  mxPath(from = "eta2", to = "eta3", free = FALSE, values = 1),
  mxPath(from = "eta3", to = "eta4", free = FALSE, values = 1),

  # --- Change score definitions ---
  mxPath(from = "delta12", to = "eta2", free = FALSE, values = 1),
  mxPath(from = "delta23", to = "eta3", free = FALSE, values = 1),
  mxPath(from = "delta34", to = "eta4", free = FALSE, values = 1),

  # --- Mean structure ---
  # Latent means carry the mean structure; manifest intercepts fixed to zero
  mxPath(from = "one", to = c("eta1", "delta12", "delta23", "delta34"),
         free = TRUE, values = c(50, -1, -1, -1),
         labels = c("mean_eta1", "mean_d12", "mean_d23", "mean_d34")),
  mxPath(from = "one", to = c("eta2", "eta3", "eta4"),
         free = FALSE, values = 0),
  mxPath(from = "one", to = manifest_vars,
         free = FALSE, values = 0),

  # --- Latent variances ---
  # Initial status variance (free)
  mxPath(from = "eta1", arrows = 2, free = TRUE, values = 80,
         labels = "var_eta1"),
  # Change score variances constrained equal (homogeneous change)
  mxPath(from = "delta12", arrows = 2, free = TRUE, values = 30,
         labels = "var_delta"),
  mxPath(from = "delta23", arrows = 2, free = TRUE, values = 30,
         labels = "var_delta"),
  mxPath(from = "delta34", arrows = 2, free = TRUE, values = 30,
         labels = "var_delta"),

  # --- Latent covariances ---
  # Initial status to first change covariance only
  mxPath(from = "eta1", to = "delta12", arrows = 2, free = TRUE,
         values = -5, labels = "cov_eta1_d12"),

  # Fix all other latent covariances to zero
  mxPath(from = "eta1", to = "delta23", arrows = 2, free = FALSE, values = 0),
  mxPath(from = "eta1", to = "delta34", arrows = 2, free = FALSE, values = 0),
  mxPath(from = "delta12", to = "delta23", arrows = 2, free = FALSE, values = 0),
  mxPath(from = "delta12", to = "delta34", arrows = 2, free = FALSE, values = 0),
  mxPath(from = "delta23", to = "delta34", arrows = 2, free = FALSE, values = 0),

  # Fix intermediate true score residual variances to zero
  mxPath(from = c("eta2", "eta3", "eta4"), arrows = 2,
         free = FALSE, values = 0),

  # --- Residual (measurement error) variances ---
  # Constrained equal across time points
  mxPath(from = manifest_vars, arrows = 2,
         free = TRUE, values = 25,
         labels = c("resvar", "resvar", "resvar", "resvar"))
)

### Add non-negativity bounds on variance parameters
model <- mxModel(model,
  mxBounds(c("var_eta1", "var_delta", "resvar"), min = 0.001))

### Fit the model
fit <- mxRun(model)

### Display model summary
summary(fit)
```

## Format Model Summary Table {.code}

```r
### Extract parameter estimates into a tidy table
param_table <- summary(fit)$parameters

model_summary_table <- param_table %>%
  select(name, Estimate, Std.Error) %>%
  mutate(
    z_value = Estimate / Std.Error,
    p_value = 2 * pnorm(-abs(z_value))
  ) %>%
  gt() %>%
  tab_header(title = "Latent Change Score Model Results (OpenMx)") %>%
  fmt_number(columns = c(Estimate, Std.Error, z_value, p_value), decimals = 3) %>%
  cols_label(
    name = "Parameter",
    Estimate = "Estimate",
    Std.Error = "Std. Error",
    z_value = "z",
    p_value = "p"
  )

### Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)
```

## Format Model Fit Indices Table {.code}

```r
### Compute reference models for incremental fit indices
ref_models <- mxRefModels(fit, run = TRUE)
mx_summary <- summary(fit, refModels = ref_models)

# Extract fit indices
fit_data <- data.frame(
  Metric = c("chi-squared", "df", "p-value", "CFI", "TLI", "RMSEA", "AIC", "BIC"),
  Value = c(
    mx_summary$Chi,
    mx_summary$ChiDoF,
    mx_summary$p,
    mx_summary$CFI,
    mx_summary$TLI,
    mx_summary$RMSEA,
    mx_summary$AIC.Mx,
    mx_summary$BIC.Mx
  )
)

fit_indices_table <- fit_data %>%
  gt() %>%
  tab_header(title = "Model Fit Indices (OpenMx)") %>%
  fmt_number(columns = Value, decimals = 3) %>%
  cols_label(
    Metric = "Fit Measure",
    Value = "Value"
  )

### Save fit indices table
gt::gtsave(
  data = fit_indices_table,
  filename = "model_fit_indices.html",
  inline_css = FALSE
)
```

## Model Summary Output {.output}

/stage4-artifacts/lcsm-basic-openmx/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lcsm-basic-openmx/model_fit_indices.html

## Interpretation {.note}

The parsimonious LCSM fit the data well (CFI = 0.985, TLI = 0.985, RMSEA = 0.065, df = 6), confirming that the equality constraints on change variances and residual variances are appropriate for CBCL Internalizing T-scores across biennial intervals. OpenMx estimates converge to the same values as the lavaan specification, as expected for equivalent model parameterizations.

Mean internalizing at baseline was 48.32 (SE = 0.155, p < .001), close to the normed T-score mean of 50. The mean latent change scores were -0.72 (Baseline to Year 2, p < .001), -0.09 (Year 2 to Year 4, p = .514), and -0.40 (Year 4 to Year 6, p = .003), indicating modest average decreases in internalizing problems over time, with the largest decline in the first period and a non-significant change in the middle period.

Initial status variance was 74.90 (p < .001), confirming substantial individual differences in baseline internalizing levels. The constrained change score variance was 15.81 (p < .001), indicating meaningful individual differences in biennial change — some youth increased while others decreased in internalizing symptoms within each period. The covariance between initial status and the first change score was negative (-12.17, p < .001), suggesting a compensatory pattern: youth with higher baseline internalizing levels tended to show greater decreases (or smaller increases) in the first period, consistent with regression toward the mean. Residual variance was 33.78 (constrained equal across waves), representing approximately 31% of observed variance — reasonable measurement error for a parent-reported broadband syndrome scale.

## Visualization {.code}

```r
### Create visualization of change score distribution
df_wide <- df_wide %>%
  mutate(
    observed_change_12 = Int_Year_2 - Int_Baseline,
    observed_change_23 = Int_Year_4 - Int_Year_2,
    observed_change_34 = Int_Year_6 - Int_Year_4
  )

### Plot distribution of change scores across three periods
change_plot <- df_wide %>%
  select(observed_change_12, observed_change_23, observed_change_34) %>%
  pivot_longer(cols = everything(), names_to = "Period", values_to = "Change") %>%
  mutate(Period = factor(Period,
                         levels = c("observed_change_12", "observed_change_23",
                                    "observed_change_34"),
                         labels = c("Baseline to Year 2", "Year 2 to Year 4",
                                    "Year 4 to Year 6"))) %>%
  ggplot(aes(x = Change, fill = Period)) +
  geom_histogram(bins = 30, alpha = 0.7, position = "identity") +
  facet_wrap(~Period, scales = "free_y") +
  labs(
    title = "Distribution of Internalizing Change Across Assessment Periods",
    subtitle = "Basic LCSM — Four Time Points (Biennial Intervals)",
    x = "Internalizing Change (T-score)",
    y = "Count"
  ) +
  theme_minimal() +
  theme(legend.position = "none")

ggsave(
  filename = "visualization.png",
  plot = change_plot,
  width = 10, height = 5, dpi = 300
)
```

## Visualization {.output}

![Internalizing Change Distribution](/stage4-artifacts/lcsm-basic-openmx/visualization.png)

## Visualization Notes {.note}

The histograms show the distribution of observed internalizing changes across the three biennial assessment periods. These are raw difference scores — the LCSM estimates are adjusted for measurement error and thus provide more reliable estimates of true change variance. The spread of each distribution reflects individual differences in change rates: a wider distribution indicates greater heterogeneity. Comparing across periods reveals whether change patterns are stable or shifting over time. Note that CBCL T-scores are normed with a mean of 50, so changes centered near zero indicate stable average levels, while systematic shifts suggest developmental trends in internalizing symptoms.

# Discussion

The Latent Change Score Model provides several advantages over simple difference scores for analyzing developmental change. By modeling change as a latent variable, LCSM separates true change from measurement error, yielding more reliable estimates of both average change and individual differences in change. This is particularly important for behavioral measures like the CBCL, where measurement error can attenuate difference score reliability.

This tutorial uses a parsimonious constraint strategy designed for stable estimation with four time points: change variances and residual variances are each constrained equal across periods, and the only within-domain covariance estimated is between initial status and the first change score. This provides adequate degrees of freedom for formal goodness-of-fit testing while avoiding the Heywood cases (negative variance estimates) that can arise with more complex covariance structures when inter-wave stability is high. The zero-covariance constraints on non-adjacent latent variables are substantively motivated — they assume that once initial status is accounted for, change scores in non-adjacent periods are conditionally independent.

The covariance between initial status and the first change score tests whether development follows a compensatory or cumulative pattern: a negative covariance suggests that youth with higher initial internalizing levels show less increase (or more decrease) in symptoms, while a positive covariance would indicate cumulative risk. This parameter is particularly informative for understanding whether regression toward the mean operates in internalizing symptom trajectories. Extensions include proportional effects (where change depends on current level), bivariate models examining coupled change across internalizing and externalizing domains (see the Bivariate LCSM tutorial), and piecewise specifications allowing change rates to differ across developmental periods.

# Additional Resources

### OpenMx Latent Change Score Tutorial {.resource}

Official OpenMx documentation covering latent change score models in the RAM framework, including univariate and bivariate specifications with worked examples.

**Badge:** DOCS
**URL:** https://openmx.ssri.psu.edu/docs/OpenMx/latest/

### McArdle (2009): Latent Variable Modeling of Differences and Changes {.resource}

Foundational paper by John McArdle introducing the latent change score framework, explaining the mathematical basis and advantages over traditional difference score approaches.

**Badge:** PAPER
**URL:** https://doi.org/10.1146/annurev.psych.60.110707.163612

### Grimm, Ram & Estabrook: Growth Modeling {.resource}

Textbook covering structural equation modeling approaches to longitudinal data, with detailed chapters on latent change score models including OpenMx code examples.

**Badge:** BOOK
**URL:** https://www.guilford.com/books/Growth-Modeling/Grimm-Ram-Estabrook/9781462526062
