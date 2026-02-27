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
summary: Specify a latent change score model in OpenMx using explicit RAM path notation with four time points, producing a properly identified model with testable fit indices for height changes in ABCD youth.
description: Specify a latent change score model in OpenMx using explicit RAM path notation with four time points, producing a properly identified model with testable fit indices for height changes in ABCD youth.
---

# Overview

## Summary {.summary}

This tutorial specifies a Latent Change Score Model (LCSM) in OpenMx's RAM path notation using four time points. LCSMs treat change between measurement occasions as a latent variable, separating true change from measurement error. By specifying each latent true score, change score, and autoregressive path as explicit `mxPath` declarations, the tutorial makes the algebra of the LCSM fully transparent: how observed scores decompose into latent true scores plus error, and how true scores at each occasion equal the prior true score plus a latent change factor. Using four waves (Baseline, Year 2, Year 4, Year 6) with equal residual variances produces a properly identified model with testable fit indices. This tutorial analyzes height changes in ABCD youth across four assessment waves.

## Features {.features}

- **When to Use:** Choose OpenMx when you want explicit matrix control over the LCSM structure, plan to add proportional effects or coupled change processes, or want the model algebra to be fully visible in the code.
- **Key Advantage:** Every path in the LCSM — factor loadings, autoregressive carryover, change score definitions, variances, and covariances — is declared as a named `mxPath`, making the model equation `eta(t) = eta(t-1) + delta(t)` directly readable in the code.
- **What You'll Learn:** How to specify a basic LCSM in OpenMx using `mxModel` and `mxPath`; how to interpret latent means, variances, and covariances of initial status and change; and how to compute fit indices using reference models.

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

### Build the LCSM in OpenMx RAM notation
# The model structure: eta(t) = eta(t-1) + delta(t)
# Each observed score = latent true score + measurement error
#
# With 4 time points we have 14 observed moments (4 means + 10 unique
# covariance elements). Constraining both change score variances and
# residual variances equal gives 13 free parameters and df = 1.
lcsm_model <- mxModel(
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
lcsm_model <- mxModel(lcsm_model,
  mxBounds(c("var_eta1", "var_delta", "resvar"), min = 0.001))

### Fit the model
fit_mx <- mxRun(lcsm_model)

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

/stage4-artifacts/lcsm-basic-openmx/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lcsm-basic-openmx/model_fit_indices.html

## Interpretation {.note}

This OpenMx specification uses four time points (Baseline, Year 2, Year 4, Year 6) with two equality constraints — equal change score variances and equal residual variances — yielding a properly identified model with df = 1 and testable fit indices.

The **manifest intercepts** capture the observed mean height at each wave. The **variance of initial status (var_eta1)** captures individual differences in baseline height. The **change score variance (var_delta)**, constrained equal across all three periods, captures the overall degree of individual differences in growth rate — the homogeneous change assumption posits that growth heterogeneity is stable across developmental stages. The **covariances between initial status and change** test whether taller youth grow more or less: negative values suggest compensatory growth, positive values suggest cumulative advantage. The **covariances between successive change scores** indicate whether faster growth in one period predicts faster or slower growth in the next.

The equal residual variance constraint assumes that measurement precision is stable across assessment waves, and the equal change variance constraint assumes that the degree of individual variability in growth is constant across periods. Both are mild assumptions for a physical measure like height. These constraints can be relaxed by adding additional time points to provide the degrees of freedom needed for identification.

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
    subtitle = "Basic LCSM (OpenMx engine)",
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

This tutorial demonstrates how to specify a Latent Change Score Model using OpenMx's RAM path notation, making the underlying algebra — `eta(t) = eta(t-1) + delta(t)` — directly visible in the code. Each path in the model corresponds to an explicit element of the LCSM framework: unit-weighted factor loadings link true scores to observations, autoregressive paths carry forward prior status, and change score paths define the latent difference.

Using four time points with equality constraints on change score variances and residual variances produces a properly identified model (df = 1) with testable fit indices and trustworthy standard errors. Adding further time points would allow relaxing the homogeneity constraints to examine whether growth heterogeneity varies across developmental stages.

The RAM specification makes it natural to extend the basic model. Proportional effects — where change depends on prior status — require only adding a single `mxPath(from = "eta1", to = "delta12")` with a free parameter. Bivariate LCSMs with cross-domain coupling, which test whether change in one construct drives change in another, extend the same logic by adding coupling paths between the two sets of latent variables. These extensions are straightforward in the RAM framework because each new hypothesis corresponds to a new path.

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
