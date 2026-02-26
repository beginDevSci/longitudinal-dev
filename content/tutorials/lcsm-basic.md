---
title: "LCSM: Basic"
slug: lcsm-basic
author: Biostatistics Working Group
date_iso: 2025-11-06
tags:
  - abcd-study
  - latent-change
  - structural-equation-modeling
family: LCSM
family_label: Latent Change Score Models (LCSM)
engine: lavaan
engines:
  - lavaan
covariates: None
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Model true change between measurement occasions using latent change score models that separate measurement error from systematic developmental change in ABCD data.
description: Model true change between measurement occasions using latent change score models that separate measurement error from systematic developmental change in ABCD data.
---

# Overview

## Summary {.summary}

Latent Change Score Models (LCSM) provide a powerful framework for modeling change between measurement occasions by treating change as a latent variable rather than a simple observed difference. Unlike raw difference scores, LCSM separates true change from measurement error, allowing researchers to estimate the reliability of change, model proportional effects (where change depends on prior status), and correlate change with other variables. This tutorial applies LCSM to analyze height changes in ABCD youth across multiple annual assessments, demonstrating how to specify and interpret basic LCSM parameters including initial status, change scores, and their variances and covariances.

## Features {.features}

- **When to Use:** Apply when you want to model change as an explicit latent construct, especially when measurement error is a concern or when you hypothesize that change depends on initial status (proportional effects).
- **Key Advantage:** LCSM separates true score variance from error variance in the change score, providing more reliable estimates of individual differences in change than simple difference scores.
- **What You'll Learn:** How to specify a basic LCSM in lavaan, interpret the mean and variance of latent change, and assess whether change is related to initial status through proportional effects.

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
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site, ab_g_stc__design_id__fam, ph_y_anthr__height_mean) %>%
  # Filter to annual assessments, then keep only baseline, year 2, and year 4
  filter_events_abcd(conditions = c("annual")) %>%
  filter(session_id %in% c("ses-00A", "ses-02A", "ses-04A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-02A", "ses-04A"),
                        labels = c("Baseline", "Year_2", "Year_4"))
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

/stage4-artifacts/lcsm-basic/descriptives_table.html

# Statistical Analysis

## Define and Fit Basic LCSM {.code}

```r
# Define the Latent Change Score Model
# This model treats change between T1 and T2 as a latent variable
lcsm_model <- '
  # Define latent true scores at each time point
  # Latent true score T1 is measured by observed Height_Baseline
  eta1 =~ 1*Height_Baseline

  # Latent true score T2 is measured by observed Height_Year_2
  eta2 =~ 1*Height_Year_2

  # Latent true score T3 is measured by observed Height_Year_4
  eta3 =~ 1*Height_Year_4

  # Define latent change scores
  # Change from T1 to T2
  delta12 =~ 1*eta2
  eta2 ~ 1*eta1  # Autoregressive path (carryover)

  # Change from T2 to T3
  delta23 =~ 1*eta3
  eta3 ~ 1*eta2  # Autoregressive path (carryover)

  # Means of latent variables
  eta1 ~ 1           # Mean of initial status
  delta12 ~ 1        # Mean change T1 to T2
  delta23 ~ 1        # Mean change T2 to T3

  # Variances
  eta1 ~~ eta1       # Variance of initial status
  delta12 ~~ delta12 # Variance of change T1-T2
  delta23 ~~ delta23 # Variance of change T2-T3

  # Covariances between initial status and change
  eta1 ~~ delta12
  eta1 ~~ delta23
  delta12 ~~ delta23

  # Residual variances (measurement error)
  Height_Baseline ~~ e1*Height_Baseline
  Height_Year_2 ~~ e2*Height_Year_2
  Height_Year_4 ~~ e3*Height_Year_4

  # Proportional effects (optional: change depends on prior level)
  # delta12 ~ beta1*eta1
  # delta23 ~ beta2*eta2
'

# Fit the LCSM model
fit <- sem(lcsm_model, data = df_wide, missing = "ml")

# Display model summary
summary(fit, fit.measures = TRUE, standardized = TRUE)
```

## Format Model Summary Table {.code}

```r
# Extract model summary
model_summary <- summary(fit)

# Convert lavaan output to a tidy dataframe and then to gt table
model_summary_table <- broom::tidy(fit) %>%
  filter(op %in% c("~1", "~~", "~")) %>%  # Focus on key parameters
  select(term, estimate, std.error, statistic, p.value) %>%
  gt() %>%
  tab_header(title = "Latent Change Score Model Results") %>%
  fmt_number(columns = c(estimate, std.error, statistic, p.value), decimals = 3)

# Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)
```

## Format Model Fit Indices Table {.code}

```r
# Extract and save model fit indices
fit_indices <- fitMeasures(fit, c("chisq", "df", "pvalue", "cfi", "tli", "rmsea", "srmr", "aic", "bic"))

fit_indices_table <- data.frame(
  Metric = names(fit_indices),
  Value = as.numeric(fit_indices)
) %>%
  gt() %>%
  tab_header(title = "Model Fit Indices") %>%
  fmt_number(columns = Value, decimals = 3) %>%
  cols_label(
    Metric = "Fit Measure",
    Value = "Value"
  )

# Save fit indices table
gt::gtsave(
  data = fit_indices_table,
  filename = "model_fit_indices.html",
  inline_css = FALSE
)
```

## Model Summary Output {.output}

/stage4-artifacts/lcsm-basic/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lcsm-basic/model_fit_indices.html

## Interpretation {.note}

The LCSM provides estimates of both the average change and individual differences in change. The **mean of the initial status (eta1)** represents the average height at baseline. The **mean of the latent change scores (delta12, delta23)** represent the average true change between assessment waves, adjusted for measurement error. These values should be interpreted as the population-average growth in height.

The **variance of initial status** captures individual differences in baseline height, while the **variance of the change scores** captures individual differences in growth rates. A significant variance indicates that participants differ meaningfully in how much they changed. The **covariance between initial status and change** tests whether those who started higher grew more or less than those who started lower - a negative covariance suggests regression to the mean or compensatory growth patterns.

## Visualization {.code}

```r
### Create visualization of change score distribution
# Calculate observed change scores for comparison
df_wide <- df_wide %>%
  mutate(
    observed_change_12 = Height_Year_2 - Height_Baseline,
    observed_change_23 = Height_Year_4 - Height_Year_2
  )

### Plot distribution of change scores
change_plot <- df_wide %>%
  select(observed_change_12, observed_change_23) %>%
  pivot_longer(cols = everything(), names_to = "Period", values_to = "Change") %>%
  mutate(Period = factor(Period,
                         levels = c("observed_change_12", "observed_change_23"),
                         labels = c("Baseline to Year 2", "Year 2 to Year 4"))) %>%
  ggplot(aes(x = Change, fill = Period)) +
  geom_histogram(bins = 30, alpha = 0.7, position = "identity") +
  facet_wrap(~Period, scales = "free_y") +
  labs(
    title = "Distribution of Height Change Across Assessment Periods",
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

![Height Change Distribution](stage4-artifacts/lcsm-basic/visualization.png)

## Visualization Notes {.note}

The histograms display the distribution of observed height changes across the two assessment periods. The spread of each distribution reflects individual differences in growth rates - a wider distribution indicates greater heterogeneity in developmental trajectories. Note that these are observed difference scores; the LCSM estimates adjust for measurement error to provide more reliable estimates of true change variance. Comparing the two periods can reveal whether growth rates are constant (similar distributions) or accelerating/decelerating (shifting distributions).

# Discussion

The Latent Change Score Model provides several advantages over simple difference scores for analyzing developmental change. By modeling change as a latent variable, LCSM separates true change from measurement error, yielding more reliable estimates of both average change and individual differences in change. This is particularly important when measurement reliability is imperfect, as raw difference scores can substantially underestimate true change variance.

The model also allows for direct hypothesis testing about the relationship between initial status and subsequent change. The covariance between initial status and change tests whether development is compensatory (negative covariance: those starting lower catch up) or cumulative (positive covariance: initial advantages compound over time). Additionally, proportional effects can be added to test whether the rate of change depends on the current level - a common pattern in biological and psychological development.

Extensions of the basic LCSM include bivariate models that examine coupled change across two constructs (e.g., height and weight), models with time-invariant or time-varying covariates, and piecewise models that allow change rates to differ across developmental periods. These extensions make LCSM a flexible framework for understanding complex developmental processes.

# Additional Resources

### lavaan Tutorial on Latent Change Score Models {.resource}

Comprehensive guide to specifying LCSM and related longitudinal models in lavaan, including univariate and bivariate change score models with worked examples.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/growth.html

### McArdle (2009): Latent Variable Modeling of Differences and Changes {.resource}

Foundational paper by John McArdle introducing the latent change score framework, explaining the mathematical basis and advantages over traditional difference score approaches.

**Badge:** PAPER
**URL:** https://doi.org/10.1146/annurev.psych.60.110707.163612

### Grimm, Ram & Estabrook: Growth Modeling {.resource}

Textbook covering structural equation modeling approaches to longitudinal data, with detailed chapters on latent change score models, including R code examples.

**Badge:** BOOK
**URL:** https://www.guilford.com/books/Growth-Modeling/Grimm-Ram-Estabrook/9781462526062

### semPlot Package for Model Visualization {.resource}

R package for creating publication-quality path diagrams of structural equation models including LCSM, helping to visualize the model structure and parameter estimates.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=semPlot
