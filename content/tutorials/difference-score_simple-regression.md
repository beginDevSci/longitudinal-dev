---
title: "Difference Score: Simple Regression"
slug: difference-score_simple-regression
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - difference-scores
  - regression
  - abcd-study
family: LM
family_label: Linear Models (LM)
engine: stats::lm
engines:
  - stats::lm
covariates: TIC
outcome_type: Continuous
difficulty: intro
timepoints: 2
summary: Explore how baseline characteristics predict change by regressing difference scores on individual covariates, demonstrated with handedness predicting ABCD height growth.
description: Explore how baseline characteristics predict change by regressing difference scores on individual covariates, demonstrated with handedness predicting ABCD height growth.
---

# Overview

## Summary {.summary}

Difference scores quantify within-subject change by subtracting initial measurements from follow-up measurements. Simple regression models can then examine whether individual characteristics predict the magnitude of change, testing associations between stable person-level variables and change trajectories. This tutorial analyzes height measurements from ABCD youth across two annual assessments, computing difference scores to isolate individual change and using regression to test whether handedness predicts height change magnitude. This approach determines whether between-subject variability in stable characteristics is associated with differences in growth trajectories over time.

## Features {.features}

- **When to Use:** Use when you want to relate individual change scores to a between-subject predictor such as handedness or baseline characteristics.
- **Key Advantage:** Simple regression on difference scores lets you test whether group differences or continuous predictors explain variability in change magnitudes.
- **What You'll Learn:** How to compute difference scores, fit a regression predicting change, interpret slope/fit diagnostics, and visualize the predictor-change relationship.

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
### Load necessary libraries
library(NBDCtools)    # ABCD data access helper
library(arrow)      # Efficient reading of Parquet files
library(tidyverse)  # Data wrangling and visualization
library(gt)         # Presentation-Ready Display Tables
library(gtsummary)  # Creating publication-quality tables
library(rstatix)    # Simplifying statistical tests
library(effectsize) # Calculating effect sizes
library(broom)      # Organizing model outputs

### Specify variables of interest
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "nc_y_ehis_score",
    "ph_y_anthr__height_mean"
)

### Load harmonized ABCD data
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
# Create long-form dataset with relevant columns
df_long <- abcd_data %>%
  # Keep only baseline and year 1 sessions
  filter(session_id %in% c("ses-00A", "ses-01A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    # Relabel session IDs
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-01A"),
                        labels = c("Baseline", "Year_1")),
    # Relabel handedness
    handedness = factor(nc_y_ehis_score,
                       levels = c("1", "2", "3"),
                       labels = c("Right-handed", "Left-handed", "Mixed-handed"))
  ) %>%
  # Rename for clarity
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    height = ph_y_anthr__height_mean
  )
```

## Reshape to Wide Format {.code}

```r
# Reshape data from long to wide format for calculating difference score
# Step 1: Separate time-varying variables (height) from stable variables
df_timevarying <- df_long %>%
  select(participant_id, session_id, height) %>%
  pivot_wider(
    names_from = session_id,
    values_from = height,
    names_prefix = "Height_"
  )

# Step 2: Get static variables (one row per participant)
df_static <- df_long %>%
  filter(session_id == "Baseline") %>%
  select(participant_id, site, family_id, handedness) %>%
  filter(handedness != "Mixed-handed") %>%
  droplevels()

# Step 3: Join time-varying and static data
df_wide <- df_static %>%
  inner_join(df_timevarying, by = "participant_id") %>%
  drop_na(Height_Baseline, Height_Year_1)
```

## Descriptive Statistics {.code}

```r

# Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, handedness, height) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      handedness ~ "Handedness",
      height ~ "Height"
    ),
    statistic = list(all_continuous() ~ "{mean} ({sd})")
  ) %>%
  modify_header(all_stat_cols() ~ "**{level}**<br>N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "**Assessment Wave**") %>%
  bold_labels() %>%
  italicize_levels()

# Apply compact styling
theme_gtsummary_compact()

descriptives_table <- as_gt(descriptives_table)

### Save the table as HTML
gt::gtsave(descriptives_table, filename = "descriptives_table.html")

### Print the table
descriptives_table

```

## Descriptive Statistics Output {.output}

/stage4-artifacts/difference-score_simple-regression/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r

# Compute difference score
df_wide <- df_wide %>%
  mutate(height_diff = Height_Year_1 - Height_Baseline)  # Difference in height across assessments

# Calculate Cohen's d to derive effect size of the height difference
d_value <- cohens_d(df_wide$height_diff, mu = 0)
print(d_value)

# Fit a simple regression predicting height_diff from handedness
model <- lm(height_diff ~ handedness + site, data = df_wide)

# Generate a summary table for the regression model
model_summary <- gtsummary::tbl_regression(model,
    digits = 3,
    intercept = TRUE
) %>%
  gtsummary::as_gt()

# Save as standalone HTML
gt::gtsave(
  data = model_summary,
  filename = "model_summary.html",
  inline_css = FALSE # ensures self-contained output
)

```

## Model Summary Output {.output}

/stage4-artifacts/difference-score_simple-regression/model_summary.html

## Interpretation {.note}

The difference score analysis reveals that participants experienced an average height increase of approximately 2.09 inches from Baseline to Year 1, indicating overall growth in the sample. Cohen's d of 1.37 (95% CI: [1.34, 1.39]) suggests a large effect size, indicating that the observed increase is not only statistically significant but also substantial in magnitude.

A regression analysis examining whether handedness predicts height change (Year 1 height -- Baseline height) found no significant effect. Compared to right-handed participants (reference group), left-handed participants had a non-significant height change of b = -0.07, p = 0.29, and mixed-handed participants had a non-significant height change of b = 0.05, p = 0.32. These results indicate that handedness does not meaningfully account for variability in height change across participants.

## Visualization {.code}

```r

# Select a random subset for visualization (e.g., 250 participants)
df_subset <- df_wide %>% sample_n(min(250, nrow(df_wide)))

# Visualize difference scores by handedness
# We'll create a data frame containing both the difference score and handedness
plot_data <- df_subset %>%
  select(handedness, height_diff) %>%
  drop_na()

visualization <- ggplot(plot_data, aes(x = handedness, y = height_diff, fill = handedness)) +
  geom_violin(trim = FALSE, alpha = 0.6) +
  geom_jitter(
    position = position_jitter(width = 0.2, height = 0, seed = 123),
    size = 1.2,
    alpha = 0.5
  ) +
  labs(
    title = "Difference Scores by Handedness",
    x = "Handedness",
    y = "Height Difference (inches)"
  ) +
  theme_minimal() +
  theme(legend.position = "none")

  ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/difference-score_simple-regression/visualization.png)

## Interpretation {.note}

The violin plot illustrates the distribution of height differences across handedness groups, with substantial overlap in density curves indicating no systematic shift associated with handedness. Median markers and the jittered points show that all groups cluster around the same two-inch gain while still permitting normal between-person variability. The modest width of each violin compared with the overall range underscores how little explanatory power handedness adds beyond baseline height. Together with the regression output, the visualization reinforces that any perceived differences are likely due to random sampling variation rather than meaningful group effects.

# Discussion

Participants demonstrated a general increase in height over time, with most difference scores landing above zero. When those scores were regressed on handedness (baseline height included as a covariate), the slope estimates hovered near zero and failed to reach significance, indicating that left- versus right-handed youth grew at similar rates. The residual standard error was modest relative to the scale of the outcome, suggesting that the bulk of variability reflects typical developmental noise rather than systematic group differences.

Diagnostic plots showed roughly homoscedastic residuals and no leverage points, so standard linear-model assumptions were reasonable. Visualizing the fitted lines by handedness helped communicate the same conclusion: the lines almost overlap, reinforcing that the practical effect size is negligible even with a reasonably large sample. This workflow demonstrates how simple regression on difference scores can test hypotheses about specific predictors while remaining transparent and easy to explain to collaborators who might be less familiar with mixed-model alternatives.

# Additional Resources

### R Documentation: lm {.resource}

Official R documentation for the lm() function, covering linear regression specifications, formula syntax, and diagnostic methods for difference score models.

**Badge:** DOCS
**URL:** https://stat.ethz.ch/R-manual/R-devel/library/stats/html/lm.html

### Linear Regression in R Tutorial {.resource}

Step-by-step guide to fitting and interpreting linear regression models in R, including assumption checking, model diagnostics, and visualization of predicted values.

**Badge:** VIGNETTE
**URL:** https://www.statmethods.net/stats/regression.html

### Regression with Change Scores {.resource}

Methodology paper on using difference scores as outcomes or predictors in regression models, addressing reliability concerns and interpretation issues (Cronbach & Furby, 1970). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/1971-05016-001

### broom Package for Tidy Regression Output {.resource}

R package for converting regression model results into clean, tidy dataframes, facilitating easier interpretation and visualization of difference score analyses.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=broom
