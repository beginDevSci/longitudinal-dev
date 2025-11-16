---
title: "LMM: Random Slopes"
slug: lmm-random-slopes
author: Biostatistics Working Group
date_iso: 2025-11-04
tags:
  - abcd-study
  - linear-mixed-model
  - random-slopes
family: LMM
family_label: Linear Mixed Models (LMM)
engine: lme4
covariates: None
outcome_type: Continuous
description: Extend random-intercept LMMs by adding random slopes, enabling individualized change trajectories and richer inferences for ABCD longitudinal outcomes.
---

# Overview

## Summary {.summary}

Linear Mixed Models with random slopes allow each individual to have a unique trajectory of change over time, recognizing heterogeneous developmental patterns across participants. By estimating both random intercepts and random slopes, this approach captures individual differences in baseline levels and rates of change, providing more accurate representations than models assuming identical trajectories. This tutorial demonstrates random slope LMM using cognitive data from ABCD youth across four assessments, modeling individual differences in baseline cognition and change rates to capture both population-level trends and participant-specific deviations.

## Features {.features}

- **When to Use:** Choose this approach when both baselines and rates of change vary across participants—typical in ABCD longitudinal cognition or behavioral measures.
- **Key Advantage:** Random slopes capture heterogeneous growth rates, improving prediction accuracy and highlighting how change differs across individuals.
- **What You'll Learn:** How to specify random intercept + slope structures in `lme4`, interpret intercept-slope covariance, and plot individual trajectories.

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
library(NBDCtools)  # ABCD data access helper
library(arrow)       # For reading Parquet files
library(tidyverse)   # For data manipulation & visualization
library(gtsummary)   # For generating publication-quality summary tables
library(rstatix)     # Provides tidy-format statistical tests
library(lme4)        # Linear mixed-effects models (LMMs)
library(kableExtra)  # Formatting & styling in HTML/Markdown reports
library(performance) # Useful functions for model diagnostics & comparisons
library(lmerTest)    # Provides p-values for lmer models
library(sjPlot)      # Visualization of mixed models

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
  "ab_g_dyn__design_site",
  "ab_g_stc__design_id__fam",
  "nc_y_nihtb__comp__cryst__fullcorr_tscore"
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
  # Filter to available annual assessments using NBDCtools
  filter_events_abcd(conditions = c("annual")) %>%
  # Rename variables for clarity
  rename(
    site = ab_g_dyn__design_site,              # site already a factor from NBDCtools
    family_id = ab_g_stc__design_id__fam,
    cognition = nc_y_nihtb__comp__cryst__fullcorr_tscore  # cognition already numeric from NBDCtools
  ) %>%
  # Keep only participants with at least 2 non-missing cognition scores
  group_by(participant_id) %>%
  filter(sum(!is.na(cognition)) >= 2) %>%
  ungroup() %>%
  drop_na(cognition) %>%  # Remove rows with missing outcome only (lmer handles missing predictors)
  # Create numeric time variable from session_id
  mutate(time = as.numeric(session_id) - 1)  # Convert session to numeric time (0, 1, 2, 3...)
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, cognition) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      cognition ~ "Cognition"
    ),
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

/stage4-artifacts/lmm-random-slopes/descriptives_table.html

# Statistical Analysis

## Model Specification and Fitting {.code}

```r
### Fit a Linear Mixed Model (LMM) with random intercepts and slopes
model <- lmerTest::lmer(
    cognition ~ time + (1 + time | participant_id), # Fixed effect (time), random intercept & slope (participant_id)
    data = df_long # Dataset containing repeated measures of cognition
)

### Generate a summary table for the LMM model
model_summary_table <- gtsummary::tbl_regression(model,
    digits = 3,
    intercept = TRUE
) %>%
  gtsummary::as_gt()

### Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)

# Generate a summary table for the LMM model
sjPlot::tab_model(model,
    show.se = TRUE, show.df = FALSE, show.ci = FALSE,
    digits = 3,
    pred.labels = c("Intercept", "Time"),  # Adjust predictor labels
    dv.labels = c("Random Intercept & Slope LMM"),  # Update model label
    string.se = "SE",
    string.p = "P-Value",
    file = "lmm_model_results.html"
)

```

## Model Summary Output-1 {.output}

/stage4-artifacts/lmm-random-slopes/model_summary.html

## Model Summary Output-2 {.output}

/stage4-artifacts/lmm-random-slopes/lmm_model_results.html

## Interpretation {.note}

The fixed effects estimates suggest that the average cognition score at baseline (time = 0) is 50.545 (SE = 0.117, p < 0.001), with cognition declining by 0.188 points per biannual assessment (SE = 0.023, p < 0.001).

Examining the random effects, we find considerable variability in both baseline cognition scores (random intercept variance τ₀₀ = 89.96) and individual rates of cognitive decline (random slope variance τ₁₁ = 0.56). The negative correlation (ρ₀₁ = -0.29) between the intercept and slope suggests that individuals with higher initial cognition scores tend to experience steeper declines over time. This model better accounts for individual differences in both starting cognition levels and their rate of change, making it a more flexible approach compared to the random-intercept-only model.

## Generate Quick Diagnostic Plot and Predictions {.code}

```r
# Generate quick diagnostic plot using sjPlot
sjPlot::plot_model(model,
  type = "pred",
  terms = c("time"),
  title = "Predicted Cognition Scores Over Time",
  axis.title = c("Time", "Predicted Cognition"),
  show.data = TRUE,
  ci.lvl = 0.95,
  dot.size = 2,
  line.size = 1.2,
  jitter = 0.2,
  file = "lmm_sjPlot_results.html"
)

# Generate model predictions for custom visualization
df_long <- df_long %>%
  mutate(
    predicted_fixed = predict(model, re.form = NA),
    predicted_random = predict(model, re.form = ~ (1 + time | participant_id))
  )

# Select a subset of participant IDs for visualization
set.seed(124)
sample_ids <- sample(unique(df_long$participant_id),
                     size = min(250, length(unique(df_long$participant_id))))

# Filter dataset to include only sampled participants
df_long_sub <- df_long %>%
  filter(participant_id %in% sample_ids)
```

## Create Custom Trajectory Visualization {.code}

```r
# Create the visualization of individual and overall cognition trajectories
visualization <- ggplot(df_long_sub, aes(x = time, y = cognition, group = participant_id)) +

  # Plot observed data (individual trajectories)
  geom_line(aes(color = "Observed Data"), alpha = 0.3) +
  geom_point(alpha = 0.3) +

  # Plot model-predicted values with random intercepts and slopes
  geom_line(aes(y = predicted_random, color = "Model Predictions"), alpha = 0.7) +
  geom_line(aes(y = predicted_fixed, group = 1, color = "Population Mean"), linewidth = 1.2) +

  # Customize colors for clarity
  scale_color_manual(values = c(
    "Observed Data" = "red",
    "Model Predictions" = "grey40",
    "Population Mean" = "blue"
  )) +

  # Format x-axis labels
  scale_x_continuous(
    breaks = 0:3,
    labels = c("Baseline", "Year 2", "Year 4", "Year 6")
  ) +

  # Add labels and title
  labs(
    title = "Random Intercept-Slope Model: Individual Trajectories",
    x = "Assessment Wave",
    y = "Cognition",
    color = "Trajectory Type"
  ) +

  # Apply theme
  theme_minimal() +
  theme(legend.position = "bottom")

# Display the plot
visualization

# Save the plot
ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization-1 {.output}

![Visualization](stage4-artifacts/lmm-random-slopes/lmm_sjPlot_results.html)

## Visualization-2 {.output}

![Visualization](stage4-artifacts/lmm-random-slopes/visualization.png)

## Interpretation {.note}

The plot illustrates individual and overall cognition trajectories over time. Red lines represent observed cognition trajectories for each participant, while grey lines depict their model-estimated trajectories incorporating both random intercepts and random slopes. The blue line represents the overall fixed-effect mean trajectory, summarizing the population-average trend in cognition from Baseline to Year 6.

Compared to the random intercept-only model, the random slopes component allows for individual differences in the rate of cognitive change over time. This results in diverging trajectories, where some individuals experience steeper declines while others remain relatively stable. The negative intercept-slope correlation (-0.29) suggests that those with higher initial cognition tend to decline faster, a pattern captured by the spread of grey lines becoming wider over time.

# Discussion

The random-slope LMM revealed substantial heterogeneity in cognitive trajectories despite an average decline of 0.188 points per biennial assessment. Random intercept variance (τ₀₀ = 89.96) indicated wide dispersion in baseline cognition, and slope variance (τ₁₁ = 0.56) showed that participants diverged in how quickly they changed. Together, these estimates justify the added complexity of allowing participant-specific slopes instead of forcing one common trajectory.

The negative intercept–slope correlation (ρ₀₁ = −0.29) suggested regression-to-the-mean dynamics: children who started high tended to drop faster, whereas lower-performing peers often declined slowly or even stabilized. Residual diagnostics showed no major violations of normality or homoscedasticity, and the conditional R² improved noticeably relative to a random-intercept-only model. Practically, these findings emphasize that interventions targeting cognitive decline should be personalized, because youth begin at different levels and respond differently over time even when exposed to the same macro-level experiences.

# Additional Resources

### lme4 Package Documentation {.resource}

Official CRAN documentation for the lme4 package, with detailed coverage of random slopes models using the (1 + time|subject) syntax. Explains variance-covariance matrix estimation for random effects and interpretation of intercept-slope correlations.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=lme4

### Fitting Linear Mixed-Effects Models Using lme4 {.resource}

Comprehensive guide to linear mixed-effects modeling with lme4, including random slopes specification, convergence troubleshooting, and diagnostic procedures. Section 2.3 specifically addresses modeling individual differences in change trajectories with correlated random effects.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/lme4/vignettes/lmer.pdf

### Data Analysis Using Regression and Multilevel Models {.resource}

Gelman & Hill (2007). Chapters 11-13 cover varying-intercept and varying-slope models with excellent practical guidance on model building, interpretation of random effects correlations, and when to include random slopes. Includes discussion of the bias-variance tradeoff in complex random effects structures.

**Badge:** BOOK
**URL:** https://www.cambridge.org/core/books/data-analysis-using-regression-and-multilevelhierarchical-models/

### sjPlot for Model Visualization {.resource}

R package providing publication-ready tables and plots for lme4 models, including plot_model() for visualizing random slopes and tab_model() for formatted regression tables with variance components. Particularly useful for showing individual trajectories with confidence bands.

**Badge:** TOOL
**URL:** https://strengejacke.github.io/sjPlot/
