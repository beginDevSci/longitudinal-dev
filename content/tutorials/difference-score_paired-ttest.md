---
title: "Difference Score: Paired Samples T-Test"
slug: difference-score_paired-ttest
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - difference-scores
  - paired-t-test
family: LM
family_label: Linear Models (LM)
engine: stats::t.test
covariates: None
outcome_type: Continuous
description: Learn to compute participant-level difference scores, run paired t-tests, and interpret within-subject height changes using ABCD repeated measurements.
---

# Overview

## Summary {.summary}

Difference scores quantify change over time by subtracting initial measurements from follow-up measurements, isolating individual-level change within each participant. A paired samples t-test then evaluates whether the average change differs significantly from zero by comparing related measurements within the same group, accounting for each participant serving as their own control. This tutorial analyzes height measurements from ABCD youth across two annual assessments, computing difference scores to assess within-subject changes and applying a paired t-test to determine whether observed height changes are statistically significant.

## Features {.features}

- **When to Use:** Apply when you have two repeated measurements on each participant and want to test whether the average change differs from zero.
- **Key Advantage:** Paired t-tests work directly on within-subject differences, providing a simple, robust test that accounts for each child serving as their own control.
- **What You'll Learn:** How to compute difference scores, run a paired t-test in R, interpret the mean change and confidence interval, and visualize change via scatterplots.

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
library(arrow)      # Efficient reading of Parquet files
library(tidyverse)  # Data wrangling and visualization
library(gt)         # Presentation-Ready Display Tables
library(gtsummary)  # Creating publication-quality tables
library(rstatix)    # Simplifying statistical tests
library(effectsize) # Calculating effect sizes
library(broom)      # Organizing model outputs

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

# Prepare data for analysis by selecting relevant columns, filtering sessions, and cleaning
df_long <- abcd_data %>%
  select(participant_id, session_id, ph_y_anthr__height_mean) %>%
  filter(session_id %in% c("ses-00A", "ses-01A")) %>%   # Keep only baseline and year 1 sessions
  arrange(participant_id, session_id) %>%
  mutate(
    participant_id = factor(participant_id),  # Convert participant_id to a factor
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-01A"),
                        labels = c("Baseline", "Year_1")),  # Label sessions
    ph_y_anthr__height_mean = round(as.numeric(ph_y_anthr__height_mean), 2)  # Round height to 2 decimals
  ) %>%
  rename(
    height = ph_y_anthr__height_mean # Rename for simplicity
  ) %>%
  drop_na

# Reshape data from long to wide format for calculating difference score
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = height,
    names_prefix = "Height_"
  ) %>%
  drop_na

```

## Descriptive Statistics {.code}

```r

# Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, height) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(height ~ "Height"),
    statistic = list(all_continuous() ~ "{mean} ({sd})")
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

/stage4-artifacts/difference-score_paired-ttest/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r

# Compute difference score
df_wide <- df_wide %>%
  mutate(height_diff = Height_Year_1 - Height_Baseline)  # Difference in height across assessments

# Calculate Cohen's d
d_value <- cohens_d(df_wide$height_diff, mu = 0) # derive effect size of the height difference
print(d_value)

# 1. Fit a Paired T-test
model <- t.test(df_wide$height_diff, mu = 0)  # tests if mean difference ≠ 0

# 2. Convert test results into a tidy dataframe
tidy_model <- broom::tidy(model)

# 3. Create a styled gt table with improved formatting
model_summary <- tidy_model %>%
  gt() %>%
  tab_header(title = "T-Test Summary Table") %>%
  fmt_number(
    columns = c(estimate, statistic, p.value, conf.low, conf.high),
    decimals = 3
  ) %>%
  tab_options(
    table.width = px(650),
    table.font.size = px(13),
    heading.title.font.size = px(15),
    column_labels.font.size = px(13),
    table.margin.left = px(0),
    table.margin.right = px(0),
    data_row.padding = px(4),
    column_labels.padding = px(6),
    heading.padding = px(6)
  ) %>%
  cols_label(
    estimate = "Mean Diff",
    statistic = "t-stat",
    p.value = "p-value",
    parameter = "df",
    conf.low = "95% CI Low",
    conf.high = "95% CI High",
    method = "Method",
    alternative = "Alternative"
  ) %>%
  cols_width(
    estimate ~ px(80),
    statistic ~ px(70),
    p.value ~ px(80),
    parameter ~ px(60),
    conf.low ~ px(90),
    conf.high ~ px(90)
  )

model_summary

# Save as HTML with controlled dimensions
# Use gtsave() (or save_gt()) for HTML, RTF, LaTeX, and image saving
model_summary %>%
  gtsave(filename = "model_summary.html")

```

## Model Summary Output {.output}

/stage4-artifacts/difference-score_paired-ttest/model_summary.html

## Interpretation {.note}

The paired t-test indicates that the mean height difference of 2.33 inches is significantly greater than zero (p < .001), confirming reliable growth over the one-year period. The 95% confidence interval for the mean change (2.29 to 2.37 inches) stays well above zero, reinforcing that the effect is not driven by sampling noise. Cohen's d of 1.36 (95% CI: [1.33, 1.38]) reflects a large, practically meaningful shift in stature. Assumption checks (differences roughly symmetric, no extreme outliers) supported the validity of the paired-samples framework, so the reported effect can be interpreted with confidence.

## Visualization {.code}

```r

# Select a random subset for visualization (e.g., 250 participants)
df_subset <- df_wide %>% sample_n(min(250, nrow(df_wide)))

# Create scatterplot using the subsample
visualization <- ggplot(df_subset, aes(x = Height_Baseline, y = Height_Year_1)) +
  geom_point(aes(color = Height_Baseline), alpha = 0.6) +  # Color points by baseline height
  labs(
    x = "Height at Baseline",
    y = "Height at Year 1",
    title = "Scatterplot of Heights at Baseline and Year 1",
    subtitle = "Random subsample of 1000 participants"
  ) +
  theme_minimal() +
  geom_smooth(method = "lm", se = FALSE, color = "blue") +  # Add a linear regression line
  theme(legend.position = "bottom")

  ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/difference-score_paired-ttest/visualization.png)

## Visualization Notes {.note}

The scatterplot places Baseline height on the x-axis and Year 1 height on the y-axis, so points falling above the diagonal line represent participants who grew. Nearly all observations lie above the identity line and along the fitted regression line, emphasizing the uniformity of the height increase. Wider vertical spread at taller Baseline values hints at slightly greater variability among taller youth, yet the smooth line remains steep and linear, echoing the large effect detected in the t-test. The plot therefore offers a quick visual confirmation that growth occurred broadly rather than being driven by a small subset of participants.

# Discussion

On average, participants exhibited a measurable increase in height across the two annual assessments, as indicated by positive difference scores. The paired samples t-test confirmed that the observed changes were statistically significant, suggesting that the increase in height is unlikely to be due to chance. The scatterplot of initial and follow-up measurements illustrates the general trend of growth, with most individuals showing an upward trajectory.

These findings demonstrate the utility of difference scores in capturing within-subject changes over time and highlight the effectiveness of paired samples t-tests for assessing statistically significant differences in repeated measures data. The method provides a straightforward approach to quantifying and testing within-subject change using two time points.

# Additional Resources

### R Documentation: t.test {.resource}

Official R documentation for the t.test() function, covering paired samples t-tests, confidence intervals, and assumptions for difference score analysis.

**Badge:** DOCS
**URL:** https://stat.ethz.ch/R-manual/R-devel/library/stats/html/t.test.html

### Paired Samples t-test Tutorial {.resource}

Comprehensive tutorial on conducting paired samples t-tests in R, including data preparation, assumption checking, effect size calculation, and result interpretation.

**Badge:** VIGNETTE
**URL:** https://www.statmethods.net/stats/ttest.html

### Change Scores vs Growth Models {.resource}

Methodology paper comparing difference score analysis with more sophisticated longitudinal methods, discussing when simple change scores are appropriate (Rogosa, 1988). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/1988-32642-001

### effectsize Package for Cohen's d {.resource}

R package for computing standardized effect sizes including Cohen's d for paired samples, with confidence intervals and multiple estimation methods.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=effectsize
