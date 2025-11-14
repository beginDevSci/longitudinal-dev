---
title: "GLMM: Basic"
slug: glmm
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - glmm
  - mixed-model
family: GLMM
family_label: Generalized Linear Mixed Models (GLMM)
engine: glmmTMB
covariates: TIC
outcome_type: Count
description: Build generalized linear mixed models for clustered count data, specify random effects, handle overdispersion, and interpret conditional estimates for ABCD longitudinal outcomes.
---

# Overview

## Summary {.summary}

Generalized Linear Mixed Models (GLMMs) extend linear mixed models to handle non-normally distributed outcomes such as counts or binary responses while modeling random effects to account for individual differences and hierarchical structures. By combining generalized linear model distributions with random intercepts and slopes, GLMMs capture both population-level trends and person-specific variability in longitudinal data. This tutorial examines alcohol use in ABCD youth across four annual assessments using a Poisson GLMM to model drinking frequency, estimating fixed effects for population trends and random effects for individual variability.

## Features {.features}

- **When to Use:** Ideal when you need subject-specific inference for non-Gaussian outcomes collected repeatedly in ABCD.
- **Key Advantage:** GLMMs combine fixed effects with random intercepts/slopes, delivering both population and subject-level insight for generalized outcomes.
- **What You'll Learn:** How to fit GLMMs in `glmmTMB`, interpret fixed/random effects, and evaluate fit/diagnostics for binary/count data.

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

## Additional NBDCtools Resources

For more details on using NBDCtools:

- [NBDCtools Getting Started Guide](https://software.nbdc-datahub.org/NBDCtools/articles/NBDCtools.html) - Complete package overview
- [Joining Data](https://software.nbdc-datahub.org/NBDCtools/articles/join.html) - Advanced data merging strategies
- [Filtering Events](https://software.nbdc-datahub.org/NBDCtools/articles/filter.html) - Selecting specific assessment waves
- [Data Transformations](https://software.nbdc-datahub.org/NBDCtools/articles/transformation.html) - Preprocessing and cleaning

# Data Preparation

## Loading and Initial Processing {.code}

```r
### Load necessary libraries
library(NBDCtools)    # ABCD data access helper
library(arrow)      # Reading Parquet files
library(tidyverse)  # Data wrangling & visualization
library(gtsummary)  # Summary tables
library(rstatix)    # Tidy-format statistical tests
library(lme4)       # Linear mixed-effects models (GLMMs)
library(ggeffects)  # Extract & visualize model predictions
library(broom)      # Organizing model outputs
library(broom.mixed)  # Organizing mixed model outputs

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "su_y_lowuse__isip_001__l"
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

# Data wrangling: clean, restructure, and filter alcohol use variable
df_long <- abcd_data %>%
  filter(session_id %in% c("ses-01A", "ses-02A", "ses-03A", "ses-04A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    id = factor(participant_id),  # Convert participant ID to factor
    session_id = factor(session_id, levels = c("ses-01A", "ses-02A", "ses-03A", "ses-04A"),
                   labels = c("Year_1", "Year_2", "Year_3", "Year_4")),  # Rename sessions for clarity
    time = as.numeric(session_id) - 1,  # Converts factor to 0,1,2,3
    ab_g_dyn__design_site = factor(ab_g_dyn__design_site),  # Convert site to a factor
    ab_g_stc__design_id__fam = factor(ab_g_stc__design_id__fam), # Convert family id to a factor
    alcohol_use = as.numeric(su_y_lowuse__isip_001__l)  # Ensure alcohol use is numeric
  ) %>%
  filter(alcohol_use >= 0 & alcohol_use <= 10) %>%  # Keep only valid alcohol use values
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
  ) %>%
  # Remove participants with any missing substance use reporting across time points
  filter(sum(!is.na(alcohol_use)) >= 2) %>%  # Keep only participants with at least 2 non-missing cognition scores
  ungroup() %>%
  drop_na(site, family_id, participant_id, alcohol_use)  # Ensure all remaining rows have complete cases
```

## Descriptive Statistics {.code}

```r
# Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, alcohol_use) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(alcohol_use ~ "Alcohol Use"),
    statistic = list(all_continuous() ~ "{mean} ({sd})")
  ) %>%
  modify_header(all_stat_cols() ~ "**{level}**N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "**Assessment Wave**") %>%
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

/stage4-artifacts/glmm/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r
# Fit a Poisson GLMM with random intercepts for site, family, and participant
model <- glmer(
    alcohol_use ~ time + (1 | site:family_id:participant_id),
    data = df_long,
    family = poisson(link = "log"),
    control = glmerControl(optimizer = "bobyqa")
)

# Generate a summary table for the GLMM model
model_summary_table <- gtsummary::tbl_regression(model,
    digits = 3,
    intercept = TRUE
) %>%
  gtsummary::as_gt()

# Display model summary (optional)
model_summary_table

### Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)

```

## Model Summary Output {.output}

/stage4-artifacts/glmm/model_summary.html

## Model Diagnostics {.code}

```r
# Extract random effects variance and model diagnostics
re_var <- as.data.frame(VarCorr(model))

# Create diagnostics table
diagnostics_data <- data.frame(
  Diagnostic = c(
    "Family",
    "Link Function",
    "Random Intercept Variance (σ²)",
    "Number of Participants",
    "Total Observations"
  ),
  Value = c(
    "Poisson",
    "log",
    sprintf("%.4f", re_var$vcov[1]),
    sprintf("%d", length(unique(df_long$participant_id))),
    sprintf("%d", nrow(df_long))
  )
)

# Create gt table
diagnostics_table <- diagnostics_data %>%
  gt::gt() %>%
  gt::tab_header(title = "GLMM Model Diagnostics") %>%
  gt::cols_label(
    Diagnostic = gt::md("**Diagnostic**"),
    Value = gt::md("**Value**")
  ) %>%
  gt::tab_options(table.font.size = 12)

# Save diagnostics table
gt::gtsave(diagnostics_table, filename = "model_diagnostics.html")

# Display table
diagnostics_table
```

## Model Diagnostics Output {.output}

/stage4-artifacts/glmm/model_diagnostics.html

## Interpretation {.note}

The Poisson GLMM results indicate a significant increase in alcohol use over time, with the time coefficient of 0.16 (log-scale, 95% CI: 0.14, 0.18, p < 0.001) suggesting an upward trend in consumption across assessments. This corresponds to an incidence rate ratio (IRR) of approximately 1.17 (exp(0.16) ≈ 1.17), meaning alcohol use increases by approximately 17% per assessment wave.

The random intercept variance (σ² = 0.1868) highlights moderate individual differences in baseline alcohol use, reinforcing the importance of accounting for between-person variability.

Model fit metrics, including the log-likelihood value, suggest that the GLMM provides a well-suited framework for capturing both population-wide trends and subject-specific differences in alcohol consumption over time.

## Visualize {.code}

```r
# Get predicted alcohol use by time
preds <- ggeffect(model, terms = "time")

# Plot predicted probabilities with confidence intervals
ggplot(preds, aes(x = x, y = predicted)) +
  geom_point(size = 3, color = "blue") +
  geom_line(group = 1, color = "blue") +
  geom_errorbar(aes(ymin = conf.low, ymax = conf.high), width = 0.2) +
  labs(title = "Predicted Alcohol Use by time",
       x = "time (Timepoint)",
       y = "Predicted Alcohol Use") +
  theme_minimal()

  df_long$predicted <- predict(model, type = "response")

visualization <- ggplot(df_long, aes(x = predicted, y = alcohol_use)) +
  geom_point(alpha = 0.5) +
  geom_smooth(method = "lm", color = "red", se = FALSE) +
  labs(title = "Predicted vs. Observed Alcohol Use",
       x = "Predicted Alcohol Use",
       y = "Observed Alcohol Use") +
  theme_minimal()

# Display the plot
visualization

# Save the plot
ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/glmm/visualization.png)

## Visualization Notes {.note}

The predicted trajectories indicate a steady increase in alcohol use across assessment waves, reflecting an overall upward trend in consumption over time. While the population-level pattern suggests consistent growth, individual trajectories vary, highlighting differences in baseline alcohol use and rates of change. This variability underscores the importance of accounting for both fixed effects (overall trends) and random effects (subject-specific deviations) in modeling alcohol use trajectories.

# Discussion

The Poisson GLMM indicated a clear upward shift in alcohol use: the fixed effect for time was 0.16 on the log scale (p < .001), which translates to an incidence-rate ratio of roughly 1.17 per wave. In practical terms, self-reported drinking frequency increased about 17% each assessment, even after adjusting for repeated measures. Visualizations of the fitted trajectories mirrored this monotonic rise.

Random intercept variance (σ² = 0.187) remained sizable, indicating that youth entered the study with very different baseline propensities that persisted after conditioning on time. Inspecting predicted versus observed counts showed no systemic bias, suggesting the Poisson mean-variance assumption was adequate for these data. Together, the fixed and random effects illustrate how GLMMs can capture both the population trend and the heterogeneity around it, offering a richer story than either a simple Poisson regression or subject-specific regressions could provide.

# Additional Resources

### lme4 Package Documentation {.resource}

Official CRAN documentation for the lme4 package, covering the glmer() function for fitting generalized linear mixed models with detailed specifications for family distributions and link functions.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=lme4

### Fitting GLMMs in R with lme4 {.resource}

Comprehensive vignette on implementing generalized linear mixed models using lme4, including binary, count, and proportion outcomes with random effects specifications.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/lme4/vignettes/lmer.pdf

### Data Analysis Using Regression and Multilevel Models {.resource}

Foundational textbook by Gelman & Hill covering hierarchical models for non-normal outcomes. Chapters 13-14 focus on GLMMs with practical examples and interpretation guidance.

**Badge:** BOOK
**URL:** https://www.cambridge.org/core/books/data-analysis-using-regression-and-multilevelhierarchical-models/

### sjPlot for GLMM Visualization {.resource}

R package for creating publication-quality tables and plots from mixed models, including predicted probabilities, marginal effects, and random effects visualization.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=sjPlot
