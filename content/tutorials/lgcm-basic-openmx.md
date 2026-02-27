---
title: "LGCM: Basic (OpenMx)"
slug: lgcm-basic-openmx
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - trajectory
  - growth
  - openmx
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: OpenMx
engines:
  - OpenMx
covariates: None
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Estimate average emotional suppression trajectories, growth rates, and individual variability across repeated ABCD assessments using latent growth curve modeling in OpenMx.
description: Estimate average emotional suppression trajectories, growth rates, and individual variability across repeated ABCD assessments using latent growth curve modeling in OpenMx.
---

# Overview

## Summary {.summary}

Latent Growth Curve Modeling (LGCM) analyzes longitudinal change by estimating growth trajectories as latent factors while distinguishing systematic development from measurement error. Using intercept and slope parameters, LGCM captures both population-average patterns and individual differences in developmental processes, providing more accurate estimates than traditional repeated measures approaches. This tutorial applies LGCM to examine emotional suppression in ABCD youth across four annual assessments (Years 3–6), estimating the average trajectory and individual variation in initial levels and rates of change. The OpenMx implementation specifies each model element — factor loadings, means, variances, and covariances — as named paths, making the underlying algebra directly readable in the code.

## Features {.features}

- **When to Use:** Ideal when you have repeated measures and want to model the average growth trajectory plus individual deviations, with measurement error accounted for.
- **Key Advantage:** LGCM provides latent intercept and slope factors, so you can quantify both initial status and change over time while separating true developmental change from measurement noise.
- **What You'll Learn:** How to specify a basic LGCM in OpenMx, interpret intercept and slope estimates (means, variances, and their covariance), and assess overall model fit.

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
    "mh_y_erq__suppr_mean"
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
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site, ab_g_stc__design_id__fam, mh_y_erq__suppr_mean) %>%
  # Filter to Years 3-6 annual assessments using NBDCtools
  filter_events_abcd(conditions = c("annual", ">=3", "<=6")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    session_id = factor(session_id,
                        levels = c("ses-03A", "ses-04A", "ses-05A", "ses-06A"),
                        labels = c("Year_3", "Year_4", "Year_5", "Year_6"))  # Relabel sessions for clarity
  ) %>%
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    suppression = mh_y_erq__suppr_mean
  ) %>%
  droplevels() %>%                                     # Drop unused factor levels
  drop_na(suppression)                                 # Remove rows with missing outcome data

### Reshape data from long to wide format
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = suppression,
    names_prefix = "Suppression_"
  ) %>%
  drop_na(starts_with("Suppression_"))  # Require complete data across all time points
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, suppression) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      suppression ~ "Suppression"
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

/stage4-artifacts/lgcm-basic-openmx/descriptives_table.html

# Statistical Analysis

## Define and Fit Basic LGCM with OpenMx {.code}

```r
### Prepare data for OpenMx
# OpenMx expects a plain data.frame with only the manifest variables
mx_data <- df_wide %>%
  select(starts_with("Suppression_")) %>%
  as.data.frame()

manifest_vars <- c("Suppression_Year_3", "Suppression_Year_4",
                    "Suppression_Year_5", "Suppression_Year_6")
latent_vars <- c("intercept", "slope")

### Build the OpenMx growth model
growth_model <- mxModel(
  "BasicLGCM",
  type = "RAM",
  manifestVars = manifest_vars,
  latentVars = latent_vars,

  # Data
  mxData(observed = mx_data, type = "raw"),

  # Factor loadings: intercept loads 1 on all indicators
  mxPath(from = "intercept", to = manifest_vars,
         free = FALSE, values = c(1, 1, 1, 1)),

  # Factor loadings: slope loads 0, 1, 2, 3 (linear time coding)
  mxPath(from = "slope", to = manifest_vars,
         free = FALSE, values = c(0, 1, 2, 3)),

  # Latent means (intercept and slope means)
  mxPath(from = "one", to = latent_vars,
         free = TRUE, values = c(3.0, 0.1),
         labels = c("mean_intercept", "mean_slope")),

  # Latent variances and covariance
  mxPath(from = latent_vars, arrows = 2,
         connect = "unique.pairs", free = TRUE,
         values = c(0.5, -0.02, 0.05),
         labels = c("var_intercept", "cov_i_s", "var_slope")),

  # Residual variances (freely estimated per time point)
  mxPath(from = manifest_vars, arrows = 2,
         free = TRUE, values = rep(0.3, 4),
         labels = c("resvar_yr3", "resvar_yr4", "resvar_yr5", "resvar_yr6")),

  # Zero manifest means (means captured by latent factors)
  mxPath(from = "one", to = manifest_vars,
         free = FALSE, values = 0)
)

### Fit the model
fit_mx <- mxRun(growth_model)

### Display model summary
summary(fit_mx)
```

## Format Model Summary Table {.code}

```r
### Extract parameter estimates into a tidy table
param_table <- summary(fit_mx)$parameters

model_summary_table <- param_table %>%
  select(name, Estimate, Std.Error) %>%
  mutate(
    z_value = Estimate / Std.Error,
    p_value = 2 * pnorm(-abs(z_value))
  ) %>%
  gt() %>%
  tab_header(title = "Latent Growth Curve Model Results (OpenMx)") %>%
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
# OpenMx requires explicit saturated and independence models to compute
# chi-squared, CFI, TLI, and RMSEA
ref_models <- mxRefModels(fit_mx, run = TRUE)
mx_summary <- summary(fit_mx, refModels = ref_models)

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

/stage4-artifacts/lgcm-basic-openmx/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lgcm-basic-openmx/model_fit_indices.html

## Interpretation {.note}

The LGCM provides estimates of both population-average change and individual differences. The **mean intercept** represents the average suppression level at the first assessment wave, while the **mean slope** captures the average annual rate of change. The **intercept variance** quantifies individual differences in starting levels, and the **slope variance** captures individual differences in growth rates — a significant slope variance confirms that participants follow meaningfully different trajectories. The **intercept-slope covariance** tests whether youth who start higher tend to change faster or slower: a negative value suggests compensatory patterns where higher-starting individuals show slower growth, while a positive value would indicate cumulative advantage. Residual variances capture measurement error at each wave.

## Visualization {.code}

```r
### Select a subset of participants
n_sample <- min(150, length(unique(df_long$participant_id)))
selected_ids <- sample(unique(df_long$participant_id), n_sample)
df_long_selected <- df_long %>% filter(participant_id %in% selected_ids)

### Plot Suppression Growth
visualization <- ggplot(df_long_selected, aes(x = session_id, y = suppression, group = participant_id)) +
    geom_line(alpha = 0.3, color = "gray") +
    geom_point(size = 1.5, color = "blue") +
    geom_smooth(aes(group = 1), method = "lm", color = "red", linewidth = 1.2, se = TRUE, fill = "lightpink") +
    labs(
        title = "Emotional Suppression Trajectories Over Time",
        subtitle = "Basic LGCM — Years 3 to 6",
        x = "Time (Years from Baseline)",
        y = "Suppression Score"
    ) +
    theme_minimal()

ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Emotional Suppression Trajectory Plot](/stage4-artifacts/lgcm-basic-openmx/visualization.png)

## Visualization Notes {.note}

Each gray line shows a participant's suppression trajectory across the four assessments, while blue points mark the observed scores and the red line traces the sample-wide mean. The upward tilt of the red line confirms the cohort-level increase in suppression, and the fan of gray lines illustrates the individual heterogeneity that the latent growth curve model is designed to capture.

# Discussion

The analysis reveals heterogeneous suppression trajectories, with the overall trend indicating increasing suppression over time while individual trajectories vary substantially. The significant slope variance confirms that adolescents follow meaningfully different developmental paths — some rise steeply, others remain stable or even decline. The negative intercept-slope covariance indicates that youth who begin with higher suppression tend to show slower growth, consistent with a regression-to-the-mean or ceiling-effect pattern.

The inclusion of both random intercepts and slopes provides a flexible framework for understanding variability in initial suppression levels and growth rates. Compared to a model with only fixed effects, the random-slope specification captures the heterogeneity visible in individual trajectory plots and yields more realistic standard errors for the population-average trend.

Extensions of the basic LGCM include conditional models with time-invariant covariates (to explain why individuals differ), piecewise specifications that allow change rates to differ across developmental periods, and multivariate models that examine co-development across multiple constructs. These extensions build directly on the intercept-slope framework established here.

# Additional Resources

### OpenMx Growth Model Tutorial {.resource}

Official OpenMx documentation for latent growth curve modeling, covering RAM-type specification, path diagrams, and model comparison in the matrix-based SEM framework.

**Badge:** DOCS
**URL:** https://openmx.ssri.psu.edu/docs/OpenMx/latest/GrowthMixtureModel_Matrix.html

### OpenMx User Guide {.resource}

Comprehensive user guide for the OpenMx package, including detailed coverage of RAM models, LISREL specification, data handling, and optimization options.

**Badge:** DOCS
**URL:** https://openmx.ssri.psu.edu/docs/OpenMx/latest/

### Neale et al. (2016) — OpenMx 2.0 {.resource}

The primary citation for OpenMx, describing the structural equation modeling framework, matrix algebra approach, and full information maximum likelihood estimation used in this tutorial.

**Badge:** PAPER
**URL:** https://doi.org/10.1007/s11336-014-9435-8
