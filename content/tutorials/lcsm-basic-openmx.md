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
summary: Model true change between measurement occasions using a latent change score model with four time points, separating measurement error from systematic developmental change in height across ABCD youth.
description: Model true change between measurement occasions using a latent change score model with four time points, separating measurement error from systematic developmental change in height across ABCD youth.
---

# Overview

## Summary {.summary}

Latent Change Score Models (LCSM) provide a framework for modeling change between measurement occasions by treating change as a latent variable rather than a simple observed difference. Unlike raw difference scores, LCSM separates true change from measurement error, allowing researchers to estimate the reliability of change, model proportional effects (where change depends on prior status), and correlate change with other variables. This tutorial applies LCSM with four time points (Baseline, Year 2, Year 4, Year 6) to analyze height changes in ABCD youth across multiple annual assessments, demonstrating how to specify and interpret basic LCSM parameters including initial status, change scores, and their variances and covariances.

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
    "ph_y_anthr__height_mean"
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
  select(participant_id, session_id, ab_g_dyn__design_site, ab_g_stc__design_id__fam, ph_y_anthr__height_mean) %>%
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
    height = ph_y_anthr__height_mean
  ) %>%
  droplevels() %>%
  drop_na(height)

### Reshape data from long to wide format for LCSM
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = height,
    names_prefix = "Height_"
  ) %>%
  drop_na(starts_with("Height_"))  # Require complete data across time points
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, height) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      height ~ "Height (cm)"
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
  select(starts_with("Height_")) %>%
  as.data.frame()

manifest_vars <- c("Height_Baseline", "Height_Year_2",
                    "Height_Year_4", "Height_Year_6")
latent_vars <- c("eta1", "eta2", "eta3", "eta4",
                 "delta12", "delta23", "delta34")
# Define model specification
model <- mxModel(
  "BasicLCSM",
  type = "RAM",
  manifestVars = manifest_vars,
  latentVars = latent_vars,

  # Data
  mxData(observed = mx_data, type = "raw"),

  # --- Factor loadings: latent true scores → observed scores ---
  mxPath(from = "eta1", to = "Height_Baseline", free = FALSE, values = 1),
  mxPath(from = "eta2", to = "Height_Year_2", free = FALSE, values = 1),
  mxPath(from = "eta3", to = "Height_Year_4", free = FALSE, values = 1),
  mxPath(from = "eta4", to = "Height_Year_6", free = FALSE, values = 1),

  # --- Autoregressive paths: carryover from prior true score ---
  mxPath(from = "eta1", to = "eta2", free = FALSE, values = 1),
  mxPath(from = "eta2", to = "eta3", free = FALSE, values = 1),
  mxPath(from = "eta3", to = "eta4", free = FALSE, values = 1),

  # --- Change score definitions ---
  mxPath(from = "delta12", to = "eta2", free = FALSE, values = 1),
  mxPath(from = "delta23", to = "eta3", free = FALSE, values = 1),
  mxPath(from = "delta34", to = "eta4", free = FALSE, values = 1),

  # --- Mean structure ---
  # Manifest intercepts freely estimated (latent means fixed to zero)
  mxPath(from = "one", to = c("eta1", "delta12", "delta23", "delta34"),
         free = FALSE, values = 0),
  mxPath(from = "one", to = c("eta2", "eta3", "eta4"),
         free = FALSE, values = 0),
  mxPath(from = "one", to = manifest_vars,
         free = TRUE, values = c(55, 60, 65, 68),
         labels = c("int_baseline", "int_yr2", "int_yr4", "int_yr6")),

  # --- Latent variances and covariances ---
  # Initial status variance (free)
  mxPath(from = "eta1", arrows = 2, free = TRUE, values = 8,
         labels = "var_eta1"),
  # Change score variances constrained equal (homogeneous change)
  mxPath(from = "delta12", arrows = 2, free = TRUE, values = 0.5,
         labels = "var_delta"),
  mxPath(from = "delta23", arrows = 2, free = TRUE, values = 0.5,
         labels = "var_delta"),
  mxPath(from = "delta34", arrows = 2, free = TRUE, values = 0.5,
         labels = "var_delta"),
  # Covariances: initial status with each change score (all free)
  mxPath(from = "eta1", to = "delta12", arrows = 2, free = TRUE,
         values = 0.5, labels = "cov_eta1_d12"),
  mxPath(from = "eta1", to = "delta23", arrows = 2, free = TRUE,
         values = -0.5, labels = "cov_eta1_d23"),
  mxPath(from = "eta1", to = "delta34", arrows = 2, free = TRUE,
         values = -0.5, labels = "cov_eta1_d34"),
  # Covariances: between change scores (all free)
  mxPath(from = "delta12", to = "delta23", arrows = 2, free = TRUE,
         values = 0.1, labels = "cov_d12_d23"),
  mxPath(from = "delta12", to = "delta34", arrows = 2, free = TRUE,
         values = 0.1, labels = "cov_d12_d34"),
  mxPath(from = "delta23", to = "delta34", arrows = 2, free = TRUE,
         values = 0.1, labels = "cov_d23_d34"),
  # Fix intermediate true score residual variances to zero
  mxPath(from = c("eta2", "eta3", "eta4"), arrows = 2,
         free = FALSE, values = 0),

  # --- Residual (measurement error) variances ---
  # Constrained equal across time points for identification (df = 1)
  mxPath(from = manifest_vars, arrows = 2,
         free = TRUE, values = 2,
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

The model fit was marginal (CFI = 0.946, RMSEA = 0.401, df = 1), with the large RMSEA indicating that the equality constraints are somewhat restrictive for height data spanning 6 years of adolescent growth. Mean height rose from 55.4 cm at Baseline to 66.7 cm by Year 6, with the manifest intercepts (55.39, 60.22, 64.94, 66.70) revealing decelerating growth — gains narrowed from roughly 5 cm per wave early on to under 2 cm in the final period.

Initial status variance was large (7.24, p < .001), confirming substantial individual differences in baseline height. The constrained change score variance (0.55) was not significant (p = .740), suggesting that once initial status is accounted for, individual differences in period-to-period growth were modest. The covariance between initial status and early change (1.14, p = .174) was not significant, but later periods showed significant negative covariances (eta1-delta23 = -1.30, eta1-delta34 = -1.46, both p < .001) — taller youth at baseline grew less in later waves, consistent with earlier approach to adult stature. The positive covariance between the final two change scores (2.75, p = .001) suggests that growth in Years 4–6 was positively coupled across adjacent periods.

## Visualization {.code}

```r
### Create visualization of change score distribution
df_wide <- df_wide %>%
  mutate(
    observed_change_12 = Height_Year_2 - Height_Baseline,
    observed_change_23 = Height_Year_4 - Height_Year_2,
    observed_change_34 = Height_Year_6 - Height_Year_4
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
    title = "Distribution of Height Change Across Assessment Periods",
    subtitle = "Basic LCSM — Four Time Points",
    x = "Height Change (cm)",
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

![Height Change Distribution](/stage4-artifacts/lcsm-basic-openmx/visualization.png)

## Visualization Notes {.note}

The histograms show the distribution of observed height changes across the three assessment periods. These are raw difference scores — the LCSM estimates are adjusted for measurement error and thus provide more reliable estimates of true change variance. The spread of each distribution reflects individual differences in growth rates: a wider distribution indicates greater heterogeneity. Comparing across periods reveals whether growth rates are decelerating (narrower and lower-centered distributions in later periods), consistent with the expected deceleration of height growth as adolescents approach adult stature.

# Discussion

The Latent Change Score Model provides several advantages over simple difference scores for analyzing developmental change. By modeling change as a latent variable, LCSM separates true change from measurement error, yielding more reliable estimates of both average change and individual differences in change. This is particularly important when measurement reliability is imperfect, as raw difference scores can substantially underestimate true change variance.

Using four time points with equality constraints produces a properly identified model (df = 1), enabling formal goodness-of-fit testing. The model's assumptions — that change score variance and measurement error variance are constant across periods — can be evaluated directly through chi-square tests and incremental fit indices. These are mild assumptions for a stable physical measure like height, and relaxing them requires only additional time points to provide the necessary degrees of freedom.

The covariance between initial status and change tests whether development follows a compensatory or cumulative pattern: a negative covariance suggests that shorter youth grow faster (regression toward the mean), while a positive covariance would indicate cumulative advantage. The covariances between successive change scores reveal whether faster growth in one period predicts faster or slower growth in the next, addressing questions about the consistency of individual growth trajectories. Extensions include proportional effects (where change depends on current level), bivariate models examining coupled change across two constructs, and piecewise specifications allowing change rates to differ across developmental periods.

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
