---
title: "LGCM: Time-Invariant Covariates (OpenMx)"
slug: lgcm-tic-openmx
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - latent-growth-curve-model
  - time-invariant-covariate
  - openmx
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: OpenMx
engines:
  - OpenMx
covariates: TIC
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Add time-invariant covariates to latent growth curve models to evaluate how stable demographic and socioeconomic characteristics shift baseline levels and growth rates of ABCD emotional suppression.
description: Add time-invariant covariates to latent growth curve models to evaluate how stable demographic and socioeconomic characteristics shift baseline levels and growth rates of ABCD emotional suppression.
---

# Overview

## Summary {.summary}

Latent Growth Curve Modeling with time-invariant covariates extends basic growth modeling by explaining why individuals differ in initial levels and rates of change. By incorporating predictors like demographics or socioeconomic factors as covariates of latent intercept and slope factors, this approach reveals how stable characteristics shape developmental trajectories. This tutorial examines emotional suppression in ABCD youth across four annual assessments (Years 3–6), modeling how age, sex, parental education, and household income predict both baseline suppression levels and individual rates of change over time.

## Features {.features}

- **When to Use:** Use when baseline demographics or socioeconomic factors may predict both initial levels and individual growth trajectories, and you want to understand what drives developmental heterogeneity.
- **Key Advantage:** Integrates covariates directly into the latent intercept and slope, revealing how each predictor shifts starting points and trajectories simultaneously while accounting for measurement error in the outcome.
- **What You'll Learn:** How to add covariates to an LGCM, interpret their effects on intercept and slope factors, and evaluate whether covariates explain meaningful portions of growth heterogeneity.

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

### Set random seed for reproducible family member selection
set.seed(123)

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "ab_g_dyn__visit_age",
    "ab_g_stc__cohort_sex",
    "ab_g_stc__cohort_race__nih",
    "ab_g_dyn__cohort_edu__cgs",
    "ab_g_dyn__cohort_income__hhold__3lvl",
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
### Create longitudinal dataset
# Filter to ERQ assessment waves (Years 3-6)
df_long <- abcd_data %>%
  filter(session_id %in% c("ses-03A", "ses-04A", "ses-05A", "ses-06A")) %>%
  arrange(participant_id, session_id)

### Clean and transform variables
df_long <- df_long %>%
  mutate(
    participant_id = factor(participant_id),
    session_id = factor(session_id,
                       levels = c("ses-03A", "ses-04A", "ses-05A", "ses-06A"),
                       labels = c("Year_3", "Year_4", "Year_5", "Year_6")),
    site = factor(ab_g_dyn__design_site),
    family_id = factor(ab_g_stc__design_id__fam),
    age = as.numeric(ab_g_dyn__visit_age),
    sex = factor(ab_g_stc__cohort_sex,
                 levels = c("1", "2"),
                 labels = c("Male", "Female")),
    race = factor(ab_g_stc__cohort_race__nih,
                  levels = c("2", "3", "4", "5", "6", "7", "8"),
                  labels = c("White", "Black", "Asian", "AI/AN", "NH/PI", "Multi-Race", "Other")),
    education = as.numeric(ab_g_dyn__cohort_edu__cgs),
    income = as.numeric(ab_g_dyn__cohort_income__hhold__3lvl),
    suppression = round(as.numeric(mh_y_erq__suppr_mean), 2)
  ) %>%
  select(participant_id, session_id, site, family_id, age, sex, race, education, income, suppression) %>%
  drop_na()

### Get baseline covariates (Year 3)
baseline_covariates <- df_long %>%
  filter(session_id == "Year_3") %>%
  select(participant_id, age, sex, education, income) %>%
  mutate(
    age_c = age - mean(age, na.rm = TRUE),
    female = ifelse(sex == "Female", 1, 0),
    education_c = education - mean(education, na.rm = TRUE),
    income_c = income - mean(income, na.rm = TRUE)
  ) %>%
  select(participant_id, age_c, female, education_c, income_c)
```

## Reshape to Wide Format {.code}

```r
### Reshape suppression to wide format
df_wide <- df_long %>%
  select(participant_id, session_id, suppression, site) %>%
  pivot_wider(
    names_from = session_id,
    values_from = suppression,
    names_prefix = "Suppression_"
  ) %>%
  left_join(baseline_covariates, by = "participant_id") %>%
  drop_na()
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

/stage4-artifacts/lgcm-tic-openmx/descriptives_table.html

# Statistical Analysis

## Define and Fit LGCM with Covariates in OpenMx {.code}

```r
### Prepare data for OpenMx
mx_data <- df_wide %>%
  select(starts_with("Suppression_"), age_c, female, education_c, income_c) %>%
  as.data.frame()

outcome_vars <- c("Suppression_Year_3", "Suppression_Year_4",
                   "Suppression_Year_5", "Suppression_Year_6")
covariate_vars <- c("age_c", "female", "education_c", "income_c")
manifest_vars <- c(outcome_vars, covariate_vars)
latent_vars <- c("i", "s")

### Build the LGCM with time-invariant covariates in OpenMx RAM notation
# The model: outcomes = intercept + slope*time + error
# Covariates predict latent intercept and slope via regression paths
model <- mxModel(
  "LGCM_TIC",
  type = "RAM",
  manifestVars = manifest_vars,
  latentVars = latent_vars,

  # Data
  mxData(observed = mx_data, type = "raw"),

  # --- Factor loadings: intercept (all 1s) ---
  mxPath(from = "i", to = outcome_vars,
         free = FALSE, values = c(1, 1, 1, 1)),

  # --- Factor loadings: slope (0, 1, 2, 3) ---
  mxPath(from = "s", to = outcome_vars,
         free = FALSE, values = c(0, 1, 2, 3)),

  # --- Latent means (conditional on covariates) ---
  mxPath(from = "one", to = c("i", "s"),
         free = TRUE, values = c(3.0, 0.1),
         labels = c("mean_i", "mean_s")),

  # --- Manifest outcome intercepts fixed to zero ---
  mxPath(from = "one", to = outcome_vars,
         free = FALSE, values = 0),

  # --- Covariate means (freely estimated, saturated) ---
  mxPath(from = "one", to = covariate_vars,
         free = TRUE, values = c(0, 0.5, 0, 0),
         labels = c("mean_age_c", "mean_female",
                     "mean_edu_c", "mean_inc_c")),

  # --- Latent residual variances and covariance ---
  mxPath(from = "i", arrows = 2, free = TRUE, values = 0.3,
         labels = "var_i"),
  mxPath(from = "s", arrows = 2, free = TRUE, values = 0.05,
         labels = "var_s"),
  mxPath(from = "i", to = "s", arrows = 2, free = TRUE, values = -0.03,
         labels = "cov_is"),

  # --- Residual variances (constrained equal across waves) ---
  mxPath(from = outcome_vars, arrows = 2,
         free = TRUE, values = 0.3,
         labels = c("resvar", "resvar", "resvar", "resvar")),

  # --- Covariate variances and covariances (saturated) ---
  mxPath(from = covariate_vars, arrows = 2, free = TRUE,
         values = c(1, 0.25, 1, 0.5),
         labels = c("var_age_c", "var_female",
                     "var_edu_c", "var_inc_c")),
  mxPath(from = "age_c", to = "female", arrows = 2, free = TRUE,
         values = 0, labels = "cov_age_fem"),
  mxPath(from = "age_c", to = "education_c", arrows = 2, free = TRUE,
         values = 0, labels = "cov_age_edu"),
  mxPath(from = "age_c", to = "income_c", arrows = 2, free = TRUE,
         values = 0, labels = "cov_age_inc"),
  mxPath(from = "female", to = "education_c", arrows = 2, free = TRUE,
         values = 0, labels = "cov_fem_edu"),
  mxPath(from = "female", to = "income_c", arrows = 2, free = TRUE,
         values = 0, labels = "cov_fem_inc"),
  mxPath(from = "education_c", to = "income_c", arrows = 2, free = TRUE,
         values = 0, labels = "cov_edu_inc"),

  # --- Regression paths: covariates → intercept ---
  mxPath(from = "age_c", to = "i", free = TRUE, values = 0.05,
         labels = "b_age_i"),
  mxPath(from = "female", to = "i", free = TRUE, values = 0,
         labels = "b_fem_i"),
  mxPath(from = "education_c", to = "i", free = TRUE, values = 0,
         labels = "b_edu_i"),
  mxPath(from = "income_c", to = "i", free = TRUE, values = 0,
         labels = "b_inc_i"),

  # --- Regression paths: covariates → slope ---
  mxPath(from = "age_c", to = "s", free = TRUE, values = -0.02,
         labels = "b_age_s"),
  mxPath(from = "female", to = "s", free = TRUE, values = 0,
         labels = "b_fem_s"),
  mxPath(from = "education_c", to = "s", free = TRUE, values = 0,
         labels = "b_edu_s"),
  mxPath(from = "income_c", to = "s", free = TRUE, values = 0,
         labels = "b_inc_s")
)

### Add non-negativity bounds on variance parameters
model <- mxModel(model,
  mxBounds(c("var_i", "var_s", "resvar"), min = 0.001))

### Fit the model
fit <- mxRun(model)

### Display model summary
summary(fit)
```

## Format Model Summary Table {.code}

```r
### Extract parameter estimates into a tidy table
param_table <- summary(fit)$parameters

### Focus on substantive parameters (exclude saturated covariate moments)
covariate_moment_labels <- c("mean_age_c", "mean_female", "mean_edu_c",
                              "mean_inc_c", "var_age_c", "var_female",
                              "var_edu_c", "var_inc_c", "cov_age_fem",
                              "cov_age_edu", "cov_age_inc", "cov_fem_edu",
                              "cov_fem_inc", "cov_edu_inc")

substantive_params <- param_table %>%
  filter(!(name %in% covariate_moment_labels)) %>%
  select(name, Estimate, Std.Error) %>%
  mutate(
    z_value = Estimate / Std.Error,
    p_value = 2 * pnorm(-abs(z_value))
  ) %>%
  gt() %>%
  tab_header(title = "LGCM with Covariates Results (OpenMx)") %>%
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

/stage4-artifacts/lgcm-tic-openmx/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lgcm-tic-openmx/model_fit_indices.html

## Interpretation {.note}

Average suppression at Year 3 was 3.074 (SE = 0.019, p < .001) and climbed by 0.134 points per year (SE = 0.008, p < .001). Intercept and slope variances (0.336 and 0.040, both p < .001) confirmed sizable between-person differences, while the negative intercept-slope covariance (-0.041, p < .001) implied that adolescents starting high tended to rise more slowly. Fit indices (CFI = 0.948, TLI = 0.909, SRMR not available, RMSEA = 0.063) indicate the model captures the main longitudinal signal with only minor approximation error.

Time-invariant covariates added interpretable structure. Older youth entered with higher suppression (b = 0.071, p = .001) yet showed flatter growth (b = -0.027, p = .004). Females increased more slowly than males (b = -0.042, p < .001). Lower household income related to higher intercepts (b = -0.058, p = .005) and marginally steeper slopes (b = 0.016, p = .079), whereas higher parental education predicted lower starting suppression (b = -0.041, p = .007). Collectively, the results highlight heterogeneous developmental courses shaped by demographic context as well as idiosyncratic factors.

## Visualization {.code}

```r
### Select a subset of participants for plotting
selected_ids <- sample(unique(df_long$participant_id), 150)
df_long_selected <- df_long %>% filter(participant_id %in% selected_ids)

### Plot suppression growth trajectories
visualization <- ggplot(df_long_selected,
                        aes(x = session_id, y = suppression,
                            group = participant_id)) +
    geom_line(alpha = 0.3, color = "gray") +
    geom_point(size = 1.5, color = "blue") +
    geom_smooth(aes(group = 1), method = "lm", color = "red",
                linewidth = 1.2, se = TRUE, fill = "lightpink") +
    labs(
        title = "Suppression Growth with Confidence Intervals",
        subtitle = "LGCM with Time-Invariant Covariates",
        x = "Assessment Wave",
        y = "Suppression Score"
    ) +
    theme_minimal()

print(visualization)

### Save the plot
ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Suppression Growth Trajectory](/stage4-artifacts/lgcm-tic-openmx/visualization.png)

## Visualization Notes {.note}

The plot displays individual and overall trends in emotional suppression across four annual assessments. Gray lines trace randomly selected individual trajectories, illustrating the heterogeneity in suppression changes. Blue points show observed measurements and the red line with shaded confidence band represents the estimated mean trajectory. The upward trend reflects the general increase in suppression over time, while the spread of individual trajectories underscores the importance of modeling both between-person predictors and residual individual variability.

# Discussion

Suppression generally increased across assessments, yet youth followed noticeably different paths. Significant residual variance in both intercepts and slopes — after accounting for covariates — shows that demographic and socioeconomic factors explain only part of the heterogeneity, with substantial individual variability remaining.

Age, sex, and socioeconomic markers each contributed to understanding trajectory differences. Older youth tended to enter with higher suppression but progress more slowly thereafter, suggesting a ceiling effect or developmental plateau. Sex differences appeared primarily in growth rates rather than starting levels. Household income and parental education each predicted baseline suppression in expected directions, with lower socioeconomic resources associated with higher initial levels.

These patterns highlight meaningful stratification in both starting points and growth rates, while the remaining residual variance underscores that unmeasured factors also shape suppression development. Extensions include adding interaction terms to test whether covariate effects differ across subgroups, mediation models where initial status predicts subsequent change, and multiple-group specifications for formal cross-population comparisons.

# Additional Resources

### OpenMx Growth Curve Tutorial {.resource}

Official OpenMx documentation covering latent growth curve models in the RAM framework, including specifications with covariates and multiple-group extensions.

**Badge:** DOCS
**URL:** https://openmx.ssri.psu.edu/docs/OpenMx/latest/

### McArdle & Nesselroade (2014): Longitudinal Data Analysis {.resource}

Comprehensive textbook on latent variable approaches to longitudinal data, including growth curve models with covariates specified in the Mx/OpenMx tradition.

**Badge:** BOOK
**URL:** https://doi.org/10.1037/14440-000

### Centering in Growth Models {.resource}

Best practices for centering time-invariant predictors in latent growth curve models. Discusses grand-mean centering versus group-mean centering and their interpretational implications (Enders & Tofighi, 2007). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/2007-10421-007
