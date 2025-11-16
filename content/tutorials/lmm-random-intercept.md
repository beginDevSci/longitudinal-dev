---
title: "LMM: Random Intercept"
slug: lmm-random-intercept
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - linear-mixed-model
  - random-intercept
family: LMM
family_label: Linear Mixed Models (LMM)
engine: lme4
covariates: None
outcome_type: Continuous
description: Estimate linear mixed models with random intercepts to capture person-specific baselines, separating within- and between-subject variance in repeated ABCD measurements.
---

# Overview

## Summary {.summary}

Linear mixed models with random intercepts extend ordinary linear regression by allowing each participant to have a unique baseline level in addition to the overall mean intercept, accounting for individual differences while modeling change over time. This approach estimates population-level trajectories while capturing person-specific deviations in starting values. This tutorial analyzes cognition scores from ABCD youth across four annual assessments using random intercept LMMs, modeling cognitive trajectories while allowing individual variation in baseline performance to estimate overall time effects on cognition while accounting for individual differences in starting scores.

## Features {.features}

- **When to Use:** Apply when you have repeated measures and expect individuals to differ mainly in baseline levels, not rates of change.
- **Key Advantage:** The model absorbs participant-specific intercepts, reducing bias from within-subject correlation and delivering cleaner estimates of the average longitudinal trajectory.
- **What You'll Learn:** How to fit a `lme4::lmer` random-intercept model, interpret the variance components, and visualize predicted trajectories that reflect individual starting points.

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
library(arrow)       # For reading Parquet files
library(tidyverse)   # For data manipulation & visualization
library(gtsummary)   # For generating publication-quality summary tables
library(rstatix)     # Provides tidy-format statistical tests
library(lme4)        # Linear mixed-effects models (LMMs)
library(kableExtra)  # Formatting & styling in HTML/Markdown reports
library(performance) # Useful functions for model diagnostics & comparisons

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
# Prepare data for LMM analysis
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
#  Create descriptive summary table
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

/stage4-artifacts/lmm-random-intercept/descriptives_table.html

# Statistical Analysis

## Model Specification and Fitting {.code}

```r
#  Fit a Linear Mixed Model (LMM) with random intercepts
model <- lmerTest::lmer(
    cognition ~ time + (1 | site:family_id:participant_id), # Fixed effect (time), random intercept (participant_id, site, family_id)
    data = df_long # Dataset containing repeated measures of cognition
)

# Generate a summary table for the LMM model
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

# Generate alternative summary table with variance components (sjPlot format)
# This provides additional details on random effects not shown in gtsummary
sjPlot::tab_model(model,
    show.se = TRUE, show.df = FALSE, show.ci = FALSE,
    digits = 3, pred.labels = c("Intercept", "Time"),
    dv.labels = c("LMM Model (lme4)"), string.se = "SE",
    string.p = "P-Value",
    file = "lmm_model_results.html"
)

```

## Model Summary Output-1 {.output}

/stage4-artifacts/lmm-random-intercept/model_summary.html

## Model Summary Output-2 {.output}

/stage4-artifacts/lmm-random-intercept/lmm_model_results.html

## Interpretation {.note}

The **fixed effects estimates** indicate that the average baseline cognition score is 50.560 (SE = 0.115, p < 0.001), with cognition declining by approximately 0.354 points per biannual assessment (Time coefficient = -0.354, SE = 0.044, p < 0.001), confirming a significant negative trajectory over time.

The **random intercept variance (τ₀₀ = 83.20)** suggests substantial individual differences in baseline cognition scores, reinforcing the need to account for between-person variability. The intraclass correlation (ICC = 0.69) indicates that a large proportion (69%) of the total variance in cognition scores is attributable to differences between-individuals rather than within-person fluctuations over time. Since **no random slope is included**, this model assumes a common rate of cognitive decline across all participants, capturing individual differences only in their starting cognition levels.

## Prepare Data and Generate Predictions {.code}

```r
# Generate model predictions for visualization
df_long <- df_long %>%
  mutate(
    predicted_fixed = predict(model, re.form = NA),
    predicted_random = predict(model, re.form = ~ (1 | site:family_id:participant_id),
                                allow.new.levels = TRUE)
  )

# Select a subset of participant IDs for visualization
set.seed(321)
sample_ids <- sample(unique(df_long$participant_id),
                     size = min(250, length(unique(df_long$participant_id))))

# Filter dataset to include only sampled participants
df_long_sub <- df_long %>%
  filter(participant_id %in% sample_ids)
```

## Create Trajectory Visualization {.code}

```r
# Create the visualization of individual and overall cognition trajectories
visualization <- ggplot(df_long_sub, aes(x = time, y = cognition, group = participant_id)) +

  # Plot observed data (individual trajectories)
  geom_line(aes(color = "Observed Data"), alpha = 0.3) +
  geom_point(alpha = 0.3) +

  # Plot model-predicted values with random intercepts
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
    title = "Random-Intercept Model: Individual Trajectories",
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

## Visualization {.output}

![Visualization](stage4-artifacts/lmm-random-intercept/visualization.png)

## Interpretation {.note}

The plot illustrates individual and overall cognition trajectories over time. **Red lines represent observed cognition trajectories for each participant**, while **grey lines** show their model-estimated trajectories incorporating **random intercepts**. The **blue line** represents the overall **fixed-effect mean trajectory**, reflecting the population-average trend in cognition from **Baseline to Year 6**.

The **vertical spread of lines at each time point** highlights substantial individual variability in **baseline cognition scores**, as participants start at different levels. However, since this model assumes a **common rate of change** over time, all individuals follow **parallel** trajectories with different starting points, reinforcing the importance of accounting for between-person differences in initial cognition.

# Discussion

The fixed-effects portion of the random-intercept LMM estimated an average baseline cognition score of 50.6 points (SE = 0.12, p < .001) and a decline of 0.354 points per biennial assessment (SE = 0.044, p < .001). Although that slope is modest, it represents a consistent downward shift across the cohort, matching expectations for this age band.

Random intercept variance (τ₀₀ = 83.20) and the resulting ICC of 0.69 revealed substantial between-person differences in starting cognition. Allowing each youth to have a unique intercept therefore captured the wide vertical spread visible in the trajectory plot, while the shared slope reflected our modeling choice to keep change rates parallel. Diagnostics showed well-behaved residuals, indicating that a simple random-intercept structure is adequate when interest centers on population-level decline and differences in baseline performance rather than individualized slopes.

# Additional Resources

### lme4 Package Documentation {.resource}

Official CRAN documentation for the lme4 package, covering the lmer() function for fitting linear mixed-effects models with random intercepts. Includes detailed explanations of model specification syntax, variance component estimation, and interpretation of random effects.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=lme4

### Fitting Linear Mixed-Effects Models Using lme4 {.resource}

Comprehensive vignette by Douglas Bates et al. introducing linear mixed models with random intercepts, including the (1|group) formula syntax, intraclass correlation calculation, and comparison with fixed-effects-only models. Essential reading for understanding how lme4 estimates variance components.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/lme4/vignettes/lmer.pdf

### Multilevel Analysis: An Introduction {.resource}

Snijders & Bosker (2011). Foundational textbook on multilevel modeling, with Chapters 2-4 covering random intercept models, interpretation of variance components, intraclass correlation coefficients (ICC), and the distinction between within-person and between-person effects.

**Badge:** BOOK
**URL:** https://uk.sagepub.com/en-gb/eur/multilevel-analysis/book241434

### performance Package for Model Diagnostics {.resource}

R package for checking linear mixed model assumptions including normality of random effects, homoscedasticity, and influential observations. The check_model() function provides comprehensive diagnostic plots specifically designed for lme4 models.

**Badge:** TOOL
**URL:** https://easystats.github.io/performance/
