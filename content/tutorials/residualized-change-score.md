---
title: Residualized Change Score
slug: residualized-change-score
author: Samuel Hawes
date_iso: 2025-11-05
tags:
  - abcd-study
  - residualized-change
  - regression
family: LM
family_label: Linear Models (LM)
engine: stats::lm
covariates: None
outcome_type: Continuous
description: Use residualized change score regression to isolate within-person change while adjusting for baseline levels in ABCD longitudinal analyses.
---

# Overview

## Summary {.summary}

Residualized change scores quantify within-subject change while controlling for baseline levels by regressing follow-up values on initial values and extracting residuals that represent deviations from expected change. Unlike simple difference scores, this approach isolates true change from regression-to-the-mean effects. This tutorial analyzes height measurements from ABCD youth across two annual assessments, generating residualized change scores that capture individual deviations from expected growth and testing whether handedness predicts variability in height change beyond what baseline values explain.

## Features {.features}

- **When to Use:** Choose this when you have two timepoints and need to control for baseline levels while examining associations with follow-up outcomes.
- **Key Advantage:** Residualized change separates true change from regression-to-the-mean by regressing follow-up on baseline and analyzing the residuals.
- **What You'll Learn:** How to fit the baseline-adjusted model, extract residualized change scores, test predictors against those residuals, and visualize distributions.

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
library(tidyverse)    # Collection of R packages for data science
library(lavaan)       # Structural Equation Modeling in R
library(gt)           # Presentation-Ready Display Tables
library(gtsummary)    # Creating publication-quality tables
library(broom)        # Organizing model outputs

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "nc_y_ehis_score",
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
# Create a long-form dataset with relevant columns
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site, ab_g_stc__design_id__fam, nc_y_ehis_score, ph_y_anthr__height_mean) %>%
  filter(session_id %in% c("ses-00A", "ses-01A")) %>%   # Keep only baseline and year 1 sessions
  arrange(participant_id, session_id) %>%
  mutate(
    participant_id = factor(participant_id),           # Convert participant_id to a factor
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-01A"),
                        labels = c("Baseline", "Year_1")),  # Label sessions
    ab_g_dyn__design_site = factor(ab_g_dyn__design_site),  # Convert site to a factor
    ab_g_stc__design_id__fam = factor(ab_g_stc__design_id__fam), # Convert family id to a factor
    nc_y_ehis_score = factor(nc_y_ehis_score,
      levels = c("1", "2", "3"),
      labels = c("Right-handed", "Left-handed", "Mixed-handed")),  # Convert handedness to a factor
    ph_y_anthr__height_mean = round(as.numeric(ph_y_anthr__height_mean), 2)  # Specify height as numeric
  ) %>%
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    handedness = nc_y_ehis_score,
    height = ph_y_anthr__height_mean
  )

# Reshape data from long to wide format for calculating difference score
# Step 1: Get static variables (participant-level) from baseline only
df_static <- df_long %>%
  filter(session_id == "Baseline") %>%
  select(participant_id, site, family_id, handedness) %>%
  filter(handedness != "Mixed-handed") %>%  # Remove "Mixed-handed" participants
  droplevels()  # Drop unused factor levels

# Step 2: Pivot time-varying variable (height) to wide format
df_timevarying <- df_long %>%
  select(participant_id, session_id, height) %>%
  pivot_wider(
    names_from = session_id,
    values_from = height,
    names_prefix = "Height_"
  )

# Step 3: Join static and time-varying data
df_wide <- df_static %>%
  inner_join(df_timevarying, by = "participant_id") %>%
  drop_na(Height_Baseline, Height_Year_1)  # Only keep complete cases

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

### Apply compact styling
theme_gtsummary_compact()

descriptives_table <- as_gt(descriptives_table)

### Save the table as HTML
gt::gtsave(descriptives_table, filename = "descriptives_table.html")

### Print the table
descriptives_table

```

## Descriptive Statistics Output {.output}

/stage4-artifacts/residualized-change-score/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r
### Model
# Predict follow-up (Year_1) height from Baseline height
baseline_model <- lm(Height_Year_1 ~ Height_Baseline, data = df_wide)

# Create simple and residualized change scores
df_wide <- df_wide %>%
  mutate(residualized_change = residuals(baseline_model))   # Portion not explained by baseline

# Regress the residualized change scores on handedness
model <- lm(residualized_change ~ handedness + site, data = df_wide)

# 2. Extract and tidy model summary
tidy_model <- broom::tidy(model)

# 3. Format into a gt table
model_summary <- tidy_model %>%
  gt() %>%
  tab_header(title = "Regression Summary Table") %>%
  fmt_number(
    columns = c(estimate, std.error, statistic, p.value),
    decimals = 3
  ) %>%
  cols_label(
    term = "Predictor",
    estimate = "Estimate",
    std.error = "Std. Error",
    statistic = "t-Statistic",
    p.value = "p-Value"
  )

model_summary

# 5. Save as standalone HTML
gt::gtsave(
  data = model_summary,
  filename = "model_summary.html",
  inline_css = FALSE
)

```

## Model Output {.output}

/stage4-artifacts/residualized-change-score/model_summary.html

## Interpretation {.note}

Handedness does not significantly predict residualized height change: Compared to right-handed participants (the reference group), left-handed participants had a non-significant change in height (b = -0.05, p = 0.40). Mixed-handed participants also showed no significant difference in height change relative to right-handers (b = 0.05, p = 0.31). These results suggest that handedness does not meaningfully contribute to variability in height change over the one-year period.

## Visualization {.code}

```r
# Select a random subset for visualization (e.g., 250 participants)
df_subset <- df_wide %>% sample_n(250)

# Create a violin plot to visualize residualized change scores by handedness
violin_plot <- ggplot(df_subset, aes(x = handedness, y = residualized_change, fill = handedness)) +
  geom_violin(trim = FALSE, alpha = 0.7) +  # Violin plot without trimming the tails
  geom_jitter(position = position_jitter(width = 0.2), size = 1.2, alpha = 0.5) +  # Add jittered points for individual observations
  scale_fill_brewer(palette = "Set2") +    # Use a color palette from RColorBrewer
  labs(
    title = "Residualized Change in Height by Handedness",
    x = "Handedness",
    y = "Height Residuals"
  ) +
  theme_minimal() +     # Apply a minimal theme for a clean look
  theme(
    axis.text.x = element_text(angle = 45, hjust = 1),  # Rotate x-axis labels for better readability
    legend.position = "none"                               # Remove the legend as it's redundant
  )

print(violin_plot)

# Save as a high-resolution PNG file
ggsave(filename = "visualization.png",
       plot = violin_plot,
       width = 10,  # Specify width in inches (or units)
       height = 5, # Specify height in inches (or units)
       units = "in", # Specify units (e.g., "in", "cm", "mm")
       dpi = 300) # Specify resolution (e.g., 300 for good quality)

```

## Visualization {.output}

![Visualization](/stage4-artifacts/residualized-change-score/visualization.png)

## Interpretation {.note}

The violin plot displays the distribution of residualized height-change scores after baseline adjustment. Each group centers near zero and shows comparable spread, reinforcing that the regression residuals contain no systematic differences by handedness. Overlayed jittered points make it easy to spot individual outliers—none deviate meaningfully from the main mass—so the null findings are not being driven by a handful of unusual observations. Taken together, the figure and model output show that once baseline stature is controlled, subsequent growth is effectively independent of handedness.

# Discussion

Residualized change scores allowed us to focus on deviations in growth that could not be explained by baseline height alone. After regressing Year 1 height on baseline and taking the residuals, we fit a linear model that included handedness along with site indicators. The resulting coefficients showed that neither left nor right-handed youth exhibited systematic departures from the expected growth curve once initial stature was accounted for, and site-level contrasts were similarly nonsignificant.

This null finding is informative: when the baseline covariate absorbs most predictable variability, remaining change reflects idiosyncratic influences or measurement noise. The analysis therefore demonstrates how residualized scores can guard against spurious associations that sometimes arise in raw change-score models. Researchers interested in other predictors can drop them into the same framework, interpret effects on the adjusted outcome, and still benefit from the familiar lm tooling for diagnostics and assumption checks.

# Additional Resources

### R Documentation: lm and residuals {.resource}

Official R documentation for the lm() function and residuals() method, essential for computing residualized change scores that control for baseline values.

**Badge:** DOCS
**URL:** https://stat.ethz.ch/R-manual/R-devel/library/stats/html/lm.html

### Residualized Change Score Tutorial {.resource}

Tutorial on computing and interpreting residualized change scores in R, including comparison with raw difference scores and appropriate use cases.

**Badge:** VIGNETTE
**URL:** https://www.statmethods.net/stats/regression.html

### Lord's Paradox and Change Scores {.resource}

Classic methodology paper discussing problems with change score analysis and advantages of residualized change scores for controlling baseline differences (Lord, 1967). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/1968-03916-001

### performance Package for Model Diagnostics {.resource}

R package for assessing regression model quality, checking assumptions, and computing diagnostic statistics for residualized change score models.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=performance
