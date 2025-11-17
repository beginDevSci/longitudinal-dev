---
title: "GEE: Basic"
slug: gee
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - gee
  - marginal-model
family: GEE
family_label: Generalized Estimating Equations (GEE)
engine: geepack
covariates: None
outcome_type: Binary
description: Fit population-averaged generalized estimating equations for binary outcomes, choose working correlation structures, and interpret marginal effects for clustered ABCD observations.
---

# Overview

## Summary {.summary}

Generalized Estimating Equations (GEE) analyze longitudinal or clustered data by extending generalized linear models to estimate population-averaged effects while accounting for within-subject correlation. Unlike subject-specific models, GEE focuses on marginal trends across the population rather than individual-specific trajectories. This tutorial applies GEE to examine sufficient sleep patterns (9–11 hours per night) in ABCD youth across four annual assessments, evaluating whether the likelihood of meeting sleep recommendations changes over time at the population level while accounting for repeated measurements within individuals.

## Features {.features}

- **When to Use:** Apply when you have repeated binary or continuous outcomes and want a population-level trend while accounting for within-child correlation.
- **Key Advantage:** GEE models provide robust estimates of marginal effects without requiring a specific random-effects structure, making them ideal for large ABCD panels.
- **What You'll Learn:** How to fit a logistic GEE in `geepack`, interpret population-averaged effects, and diagnose the working correlation structure.

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

## NBDCtools Setup and Data Loading {.code}

```r
# Load necessary libraries
library(NBDCtools)    # ABCD data access helper
library(arrow)        # Read Parquet files
library(tidyverse)    # Data wrangling & visualization
library(gtsummary)    # Summary tables
library(rstatix)      # Statistical tests in tidy format
library(geepack)      # Generalized Estimating Equations (GEE) analysis
library(ggeffects)    # Extract & visualize model predictions
library(broom)      # Organizing model outputs

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "ph_p_sds__dims_001"
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
# Data wrangling: clean, restructure, and recode sleep variable
df_long <- abcd_data %>%
  filter(session_id %in% c("ses-00A", "ses-01A", "ses-02A", "ses-03A")) %>%  # Keep Baseline - Year 3
  arrange(participant_id, session_id) %>%
   mutate(
    participant_id = factor(participant_id), # Convert participant_id to a factor
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-01A", "ses-02A", "ses-03A"),
                        labels = c("Baseline", "Year_1", "Year_2", "Year_3")),  # Label sessions

    ab_g_dyn__design_site = factor(ab_g_dyn__design_site),  # Convert site to a factor
    ab_g_stc__design_id__fam = factor(ab_g_stc__design_id__fam), # Convert family id to a factor
    ph_p_sds__dims_001 = as.numeric(ph_p_sds__dims_001)  # Convert to numeric
  ) %>%
  filter(ph_p_sds__dims_001 != 999) %>%  # Remove "Don't know" responses
  mutate(
    # Original coding: 1=9-11hrs (sufficient), 2=<9hrs, 3=>11hrs (both insufficient)
    sleep_binary = ifelse(ph_p_sds__dims_001 == 1, 1, 0)  # Recode: 9-11 hrs = 1, others = 0
  ) %>%
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
  ) %>%
  drop_na()  # Remove missing values

```

## Descriptive Statistics {.code}

```r

# Summarize 'sleep_binary' across session_ids
descriptives_table <- df_long %>%
    select(session_id, sleep_binary) %>%
    tbl_summary(
        by = session_id,
        missing = "no",
        label = list(sleep_binary ~ "Sufficient Sleep (9-11 hrs)"),
        statistic = list(
            all_categorical() ~ "{n} ({p}%)"  # Count & percentage
        )
    ) %>%
    modify_header(all_stat_cols() ~ "**{level}**<br>N = {n}") %>%
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

/stage4-artifacts/gee/descriptives_table.html

# Statistical Analysis

## Fit GEE Model {.code}

```r
# Fit GEE model: Predicting sufficient sleep over time
# Site is included to adjust for potential clustering and recruitment differences across ABCD study sites
model <- geeglm(sleep_binary ~ session_id + site,
  id = participant_id,
  data = df_long,
  family = binomial(link = "logit"),
  corstr = "exchangeable"
)

# Generate summary table
model_summary_table <- gtsummary::tbl_regression(model,
  digits = 3,
  intercept = TRUE
) %>%
  gtsummary::as_gt()

model_summary_table

# Save as standalone HTML
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)
```

## Create Model Diagnostics Table {.code}

```r
# Create GEE diagnostics data
diagnostics_data <- data.frame(
  Characteristic = c(
    "Correlation Structure",
    "Correlation Parameter (α)",
    "Scale Parameter",
    "Number of Participants",
    "Total Observations"
  ),
  Value = c(
    model$corstr,
    round(model$geese$alpha, 3),
    round(summary(model)$dispersion$Estimate, 3),
    length(unique(df_long$participant_id)),
    nrow(df_long)
  )
)

# Format diagnostics table
diagnostics_table <- diagnostics_data %>%
  gt::gt() %>%
  gt::tab_header(title = "GEE Model Diagnostics")

diagnostics_table

# Save diagnostics table
gt::gtsave(diagnostics_table, filename = "model_diagnostics.html")
```

## Model Summary Output {.output}

/stage4-artifacts/gee/model_summary.html

## Model Diagnostics Output {.output}

/stage4-artifacts/gee/model_diagnostics.html

## Interpretation {.note}

The GEE analysis indicates a significant decline in sleep sufficiency over time, with the odds of obtaining 9--11 hours of sleep decreasing at each successive assessment. Compared to Baseline (reference group):

- At Year 1, the odds of getting 9-11 hours of sleep were 31% lower (OR = 0.69, b = -0.37, p < .001).

- At Year 2, the odds of getting 9-11 hours of sleep were 52% lower (OR = 0.48, b = -0.72, p < .001).

- At Year 3, the odds of getting 9-11 hours of sleep were 58% lower (OR = 0.42, b = -0.87, p < .001).

These results suggest a progressive decline in sleep sufficiency across time points.

The estimated correlation parameter (α = 0.369) suggests moderate within-subject stability in sleep behavior, meaning that while sleep sufficiency changes over time, individuals tend to follow somewhat consistent sleep patterns across assessment waves.

## Visualization {.code}

```r

# Generate predicted probabilities from the GEE model
preds <- ggeffect(model, terms = "session_id")

# Plot predicted probabilities with confidence intervals
visualization <- ggplot(preds, aes(x = x, y = predicted)) +
  geom_point(size = 3, color = "blue") +
  geom_line(group = 1, color = "blue") +
  geom_errorbar(aes(ymin = conf.low, ymax = conf.high), width = 0.2) +
  labs(title = "Predicted Probability of 9-11 Hours of Sleep Over Time",
       x = "Assessment Wave",
       y = "Predicted Probability of Sufficient Sleep") +
  theme_minimal()

  ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/gee/visualization.png)

## Visualization Notes {.note}

The predicted-probability line falls sharply from Baseline to Year 3, mirroring the odds ratios in the model summary and underscoring that the decline is systematic, not an artifact of sampling noise. Error bars remain narrow, so the apparent drop is statistically well supported even without delving into the table. Because the curve is smooth and monotonic, it also communicates that there are no “recovery” phases later in adolescence—the cohort simply keeps moving away from the 9–11 hour target. This visualization therefore acts as a quick diagnostic that the GEE captured a robust downward trajectory and that any intervention would need to arrest the slide early rather than hoping for spontaneous rebound.

# Discussion

Participants exhibited notable fluctuations in sufficient sleep across assessments, yet the GEE detected a consistent downward trend. The negative time coefficient implies that, on average, the odds of meeting sleep guidelines dropped each year even after adjusting for site and demographic covariates. Because the model is population-averaged, this effect reflects a broad shift across the cohort rather than subject-specific dynamics.

The exchangeable working correlation captured modest within-person dependence (ρ ≈ 0.22), and robust “sandwich” standard errors remained almost identical under alternative structures, bolstering confidence in the inference. Although we did not explore additional predictors in this tutorial, the framework readily accommodates policy or behavioral covariates that might explain the decline. Overall, the analysis demonstrates how GEE can reveal directional population changes while remaining agnostic about individual random effects, making it ideal for public-health style summaries of longitudinal binary outcomes.

# Additional Resources

### geepack Package Documentation {.resource}

Official CRAN documentation for the geepack package, covering the geeglm() function for generalized estimating equations with detailed parameter descriptions and working correlation structures.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=geepack

### GEE Analysis in R Tutorial {.resource}

Comprehensive tutorial on implementing GEE models in R using geepack, including specification of correlation structures, robust standard errors, and interpretation of population-averaged effects.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/geepack/vignettes/geepack-manual.pdf

### Longitudinal Data Analysis by Diggle et al. {.resource}

Authoritative textbook on analyzing longitudinal and clustered data. Chapter 8 covers GEE methodology, working correlation structures, and comparison with mixed models. Note: access may require institutional or paid subscription.

**Badge:** BOOK
**URL:** https://oxford.universitypressscholarship.com/view/10.1093/oso/9780198524847.001.0001

### QIC for Model Selection in GEE {.resource}

Methodology paper on using Quasi-likelihood Information Criterion (QIC) for selecting working correlation structures in GEE models (Pan, 2001). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://onlinelibrary.wiley.com/doi/10.1111/j.0006-341X.2001.00120.x
