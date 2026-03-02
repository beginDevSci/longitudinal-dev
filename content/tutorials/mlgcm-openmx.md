---
title: "LGCM: Multivariate (OpenMx)"
slug: mlgcm-openmx
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - trajectory
  - parallel-process
  - openmx
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: OpenMx
engines:
  - OpenMx
covariates: None
outcome_type: Continuous
difficulty: advanced
timepoints: 3_5
summary: Fit multivariate latent growth curve models to estimate parallel developmental processes and relate intercepts and slopes of externalizing and internalizing symptoms across ABCD assessments.
description: Fit multivariate latent growth curve models to estimate parallel developmental processes and relate intercepts and slopes.
---

# Overview

## Summary {.summary}

Multivariate Latent Growth Curve Modeling (MLGCM) simultaneously models trajectories of multiple outcomes, revealing how different developmental processes unfold together over time. By estimating intercept and slope factors for each construct in a unified framework, MLGCM captures individual growth patterns while quantifying dynamic interrelationships between developmental trajectories — including baseline comorbidity, co-development, and cross-domain predictive associations. This tutorial examines co-development of externalizing and internalizing symptoms in ABCD youth across four annual assessments (Baseline through Year 3), estimating parallel growth parameters and cross-domain correlations to reveal comorbidity patterns and shared developmental processes.

## Features {.features}

- **When to Use:** Applicable when you want to model parallel growth processes (e.g., externalizing and internalizing symptoms) simultaneously to understand how trajectories are linked across domains.
- **Key Advantage:** Captures covariance between multiple latent intercepts and slopes, revealing whether baseline levels in one domain predict change in another and whether growth across domains is correlated.
- **What You'll Learn:** How to specify multivariate LGCMs, interpret within- and cross-domain covariances among latent factors, and visualize coupled developmental trajectories.

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
library(gtsummary)    # Creating publication-quality tables
library(OpenMx)       # Matrix-based SEM engine
library(broom)        # For tidying model outputs
library(gt)           # For creating formatted tables
library(patchwork)    # For combining ggplots

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
  "ab_g_dyn__design_site",
  "ab_g_stc__design_id__fam",
  "mh_p_cbcl__synd__ext_tscore",
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
### Clean and transform variables for analysis
df_long <- abcd_data %>%
  # Filter to baseline through Year 3
  filter(session_id %in% c("ses-00", "ses-00A", "ses-01A", "ses-02A", "ses-03A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    session_id = factor(
      session_id,
      levels = c("ses-00", "ses-00A", "ses-01A", "ses-02A", "ses-03A"),
      labels = c("Baseline", "Baseline", "Year_1", "Year_2", "Year_3")
    )
  ) %>%
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    externalizing = mh_p_cbcl__synd__ext_tscore,
    internalizing = mh_p_cbcl__synd__int_tscore
  ) %>%
  select(participant_id, session_id, site, family_id, externalizing, internalizing) %>%
  droplevels() %>%
  drop_na(externalizing, internalizing)

### Reshape data from long to wide format
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = c(externalizing, internalizing),
    names_sep = "_"
  ) %>%
  drop_na(starts_with("externalizing_"), starts_with("internalizing_"))
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, externalizing, internalizing) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      externalizing ~ "Externalizing",
      internalizing ~ "Internalizing"
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

/stage4-artifacts/mlgcm-openmx/descriptives_table.html

# Statistical Analysis

## Define and Fit Multivariate LGCM {.code}

```r
### Prepare data for OpenMx
mx_data <- df_wide %>%
  select(starts_with("externalizing_"), starts_with("internalizing_")) %>%
  as.data.frame()

ext_vars <- c("externalizing_Baseline", "externalizing_Year_1",
              "externalizing_Year_2", "externalizing_Year_3")
int_vars <- c("internalizing_Baseline", "internalizing_Year_1",
              "internalizing_Year_2", "internalizing_Year_3")
manifest_vars <- c(ext_vars, int_vars)
latent_vars <- c("i_ext", "s_ext", "i_int", "s_int")

### Build the multivariate LGCM in OpenMx RAM notation
# Two parallel growth processes with correlated latent factors
model <- mxModel(
  "MLGCM",
  type = "RAM",
  manifestVars = manifest_vars,
  latentVars = latent_vars,

  # Data
  mxData(observed = mx_data, type = "raw"),

  # ===== EXTERNALIZING GROWTH MODEL =====

  # Factor loadings: intercept (all 1s)
  mxPath(from = "i_ext", to = ext_vars,
         free = FALSE, values = c(1, 1, 1, 1)),

  # Factor loadings: slope (0, 1, 2, 3)
  mxPath(from = "s_ext", to = ext_vars,
         free = FALSE, values = c(0, 1, 2, 3)),

  # ===== INTERNALIZING GROWTH MODEL =====

  # Factor loadings: intercept (all 1s)
  mxPath(from = "i_int", to = int_vars,
         free = FALSE, values = c(1, 1, 1, 1)),

  # Factor loadings: slope (0, 1, 2, 3)
  mxPath(from = "s_int", to = int_vars,
         free = FALSE, values = c(0, 1, 2, 3)),

  # ===== LATENT MEANS =====

  mxPath(from = "one", to = c("i_ext", "s_ext", "i_int", "s_int"),
         free = TRUE, values = c(45, -0.5, 48, -0.3),
         labels = c("mean_i_ext", "mean_s_ext", "mean_i_int", "mean_s_int")),

  # Fix manifest intercepts to zero
  mxPath(from = "one", to = manifest_vars,
         free = FALSE, values = 0),

  # ===== WITHIN-DOMAIN VARIANCES AND COVARIANCES =====

  # Externalizing: intercept and slope variances + covariance
  mxPath(from = "i_ext", arrows = 2, free = TRUE, values = 80,
         labels = "var_i_ext"),
  mxPath(from = "s_ext", arrows = 2, free = TRUE, values = 3,
         labels = "var_s_ext"),
  mxPath(from = "i_ext", to = "s_ext", arrows = 2, free = TRUE, values = -5,
         labels = "cov_is_ext"),

  # Internalizing: intercept and slope variances + covariance
  mxPath(from = "i_int", arrows = 2, free = TRUE, values = 80,
         labels = "var_i_int"),
  mxPath(from = "s_int", arrows = 2, free = TRUE, values = 4,
         labels = "var_s_int"),
  mxPath(from = "i_int", to = "s_int", arrows = 2, free = TRUE, values = -5,
         labels = "cov_is_int"),

  # ===== CROSS-DOMAIN COVARIANCES =====

  # Intercept-intercept: do youth high on ext also start high on int?
  mxPath(from = "i_ext", to = "i_int", arrows = 2, free = TRUE, values = 50,
         labels = "cov_i_ext_i_int"),

  # Slope-slope: do changes in ext track with changes in int?
  mxPath(from = "s_ext", to = "s_int", arrows = 2, free = TRUE, values = 3,
         labels = "cov_s_ext_s_int"),

  # Intercept-slope cross-domain associations
  mxPath(from = "i_ext", to = "s_int", arrows = 2, free = TRUE, values = -2,
         labels = "cov_i_ext_s_int"),
  mxPath(from = "i_int", to = "s_ext", arrows = 2, free = TRUE, values = -2,
         labels = "cov_i_int_s_ext"),

  # ===== RESIDUAL VARIANCES =====

  # Externalizing residuals (freely estimated per wave)
  mxPath(from = ext_vars, arrows = 2,
         free = TRUE, values = c(15, 15, 15, 15),
         labels = c("resvar_ext1", "resvar_ext2", "resvar_ext3", "resvar_ext4")),

  # Internalizing residuals (freely estimated per wave)
  mxPath(from = int_vars, arrows = 2,
         free = TRUE, values = c(15, 15, 15, 15),
         labels = c("resvar_int1", "resvar_int2", "resvar_int3", "resvar_int4"))
)

### Fit the model
fit <- mxRun(model)

### Display model summary
summary(fit)
```

## Format Model Summary Table {.code}

```r
### Extract parameter estimates into a tidy table
param_table <- summary(fit)$parameters

### Focus on substantive parameters (exclude individual residual variances)
resvar_labels <- c("resvar_ext1", "resvar_ext2", "resvar_ext3", "resvar_ext4",
                    "resvar_int1", "resvar_int2", "resvar_int3", "resvar_int4")

substantive_params <- param_table %>%
  filter(!(name %in% resvar_labels)) %>%
  select(name, Estimate, Std.Error) %>%
  mutate(
    z_value = Estimate / Std.Error,
    p_value = 2 * pnorm(-abs(z_value))
  ) %>%
  gt() %>%
  tab_header(title = "Multivariate LGCM Results (OpenMx)") %>%
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
  data = substantive_params,
  filename = "model_summary.html",
  inline_css = FALSE
)
```

## Format Model Fit Indices Table {.code}

```r
### Compute reference models for incremental fit indices
ref_models <- mxRefModels(fit, run = TRUE)
mx_summary <- summary(fit, refModels = ref_models)

### Extract fit indices
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

/stage4-artifacts/mlgcm-openmx/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/mlgcm-openmx/model_fit_indices.html

## Interpretation {.note}

Externalizing symptoms averaged 45.4 at Baseline and declined by 0.416 points per year (both p < .001); internalizing symptoms started at 48.4 and fell by 0.274 points annually. Thus, both domains improved over the study window. Intercept variances of roughly 80 and slope variances of 3-4 confirmed large between-person differences, so the population mean hides substantial heterogeneity.

Model fit was mixed — RMSEA (0.144) and TLI (0.899) suggested that additional cross-domain links might further reduce misfit, though CFI (0.921) was acceptable. Even so, the estimated covariances captured the core comorbidity pattern: youth who started high on externalizing also tended to start high on internalizing (covariance = 58.6, p < .001), and declines in one domain generally coincided with declines in the other (slope covariance = 4.32, p < .001). Negative intercept-slope covariances within each domain (-6.34 for externalizing, -5.56 for internalizing) imply diminishing returns — youth with the highest baseline symptoms improved, but at a slower rate than peers with milder presentations.

## Visualization {.code}

```r
### Calculate means for each time point
mean_externalizing <- df_wide %>%
    summarise(across(starts_with("externalizing"), mean, na.rm = TRUE))

mean_internalizing <- df_wide %>%
    summarise(across(starts_with("internalizing"), mean, na.rm = TRUE))

### Reshape the data for plotting
mean_externalizing_long <- pivot_longer(mean_externalizing, cols = everything(),
                                         names_to = "Time", values_to = "Mean_externalizing")
mean_internalizing_long <- pivot_longer(mean_internalizing, cols = everything(),
                                         names_to = "Time", values_to = "Mean_internalizing")

### Plot the mean trajectories for externalizing
externalizing_plot <- ggplot(mean_externalizing_long,
                             aes(x = Time, y = Mean_externalizing, group = 1)) +
    geom_line(color = "blue", linewidth = 1.2) +
    geom_point(color = "blue") +
    labs(title = "Mean Externalizing Trajectory",
         subtitle = "Baseline to Year 3",
         y = "Mean Externalizing", x = "Time Point") +
    theme_minimal() +
    theme(axis.text.x = element_text(angle = 45, hjust = 1))

### Plot the mean trajectories for internalizing
internalizing_plot <- ggplot(mean_internalizing_long,
                              aes(x = Time, y = Mean_internalizing, group = 1)) +
    geom_line(color = "red", linewidth = 1.2) +
    geom_point(color = "red") +
    labs(title = "Mean Internalizing Trajectory",
         subtitle = "Baseline to Year 3",
         y = "Mean Internalizing", x = "Time Point") +
    theme_minimal() +
    theme(axis.text.x = element_text(angle = 45, hjust = 1))

### Combine the plots side by side using patchwork
combined_plot <- externalizing_plot + internalizing_plot
print(combined_plot)

### Save as a high-resolution PNG file
ggsave(filename = "visualization.png",
       plot = combined_plot,
       width = 10, height = 5, dpi = 300)
```

## Visualization {.output}

![Parallel Growth Trajectories](/stage4-artifacts/mlgcm-openmx/visualization.png)

## Visualization Notes {.note}

The side-by-side panels display the mean growth trajectories for externalizing (blue, left) and internalizing (red, right) symptoms across four annual assessments. Both domains show declining trends at the population level, with externalizing symptoms starting lower (~45) and declining slightly faster than internalizing symptoms (~48). The parallel visualization format facilitates comparison of trajectory shapes and highlights the co-development captured by the model's cross-domain covariances.

# Discussion

The multivariate growth model captures parallel trends in both externalizing and internalizing symptoms. Intercept and slope variances are significant for each domain, confirming that youth enter adolescence with different symptom levels and follow heterogeneous pathways even when the overall population trend points in the same direction.

Cross-domain associations reveal the core comorbidity pattern. A positive intercept-intercept covariance indicates that youth with elevated baseline externalizing symptoms also tend to start high on internalizing problems. A positive slope-slope covariance indicates that changes in one domain track with changes in the other — improvements or worsening tend to co-occur across domains. Negative intercept-slope covariances within each domain suggest diminishing returns: adolescents with the highest baseline symptoms show the most change, but at a slower rate than peers with milder presentations.

Extensions include adding directed coupling paths to test whether baseline levels in one domain predict rates of change in the other (converting the correlational MLGCM into a cross-lagged model), incorporating time-invariant covariates to explain cross-domain associations, and adding time-specific residual covariances to capture occasion-specific comorbidity beyond the latent factors.

# Additional Resources

### OpenMx Growth Curve Tutorial {.resource}

Official OpenMx documentation covering growth curve models in the RAM framework, including multivariate specifications and cross-domain associations.

**Badge:** DOCS
**URL:** https://openmx.ssri.psu.edu/docs/OpenMx/latest/

### Parallel Process LGCM Tutorial {.resource}

Step-by-step quantitative methods tutorial on modeling correlated growth trajectories between two or more outcomes, including interpretation of cross-domain associations.

**Badge:** VIGNETTE
**URL:** https://quantdev.ssri.psu.edu/tutorials/parallel-process-latent-growth-curve-models

### Bollen & Curran (2006): Latent Curve Models {.resource}

Comprehensive textbook on latent growth curve modeling including multivariate and parallel-process specifications, with extensive treatment of cross-domain associations and model extensions.

**Badge:** BOOK
**URL:** https://doi.org/10.1002/0471746096
