---
title: "LMM: Time-Varying Covariates"
slug: lmm-time-varying-covariates
author: Biostatistics Working Group
date_iso: 2025-11-06
tags:
  - abcd-study
  - linear-mixed-model
  - time-varying-covariates
family: LMM
family_label: Linear Mixed Models (LMM)
engine: lme4
engines:
  - lme4
covariates: TVC
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Extend linear mixed models to include time-varying covariates, decomposing within-person and between-person effects to understand how changing predictors relate to developmental outcomes.
description: Extend linear mixed models to include time-varying covariates, decomposing within-person and between-person effects to understand how changing predictors relate to developmental outcomes.
---

# Overview

## Summary {.summary}

Time-varying covariates (TVCs) are predictors that change value at each measurement occasion, such as pubertal development, stress levels, or peer relationships. Including TVCs in linear mixed models allows researchers to examine how changes in these predictors relate to changes in outcomes, but requires careful attention to the distinction between within-person and between-person effects. This tutorial demonstrates how to incorporate TVCs in LMMs using ABCD data, including person-mean centering strategies that decompose the total TVC effect into within-person (how deviations from one's own average predict outcomes) and between-person (how individual differences in average levels predict outcomes) components.

## Features {.features}

- **When to Use:** Apply when you have a predictor that changes over time for each participant and you want to understand how fluctuations in this predictor relate to the outcome, separately from stable individual differences.
- **Key Advantage:** Person-mean centering decomposes TVC effects into within-person and between-person components, preventing Simpson's paradox and enabling more accurate causal inference.
- **What You'll Learn:** How to add TVCs to LMMs, implement person-mean centering, interpret within vs. between effects, and compare models with different TVC specifications.

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
library(arrow)        # For reading Parquet files
library(tidyverse)    # For data manipulation & visualization
library(gtsummary)    # For generating publication-quality summary tables
library(lme4)         # Linear mixed-effects models (LMMs)
library(lmerTest)     # P-values for lmer models
library(kableExtra)   # Formatting & styling in HTML/Markdown reports
library(performance)  # Model diagnostics & comparisons
library(gt)           # For creating formatted tables

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
  "ab_g_dyn__design_site",
  "ab_g_stc__design_id__fam",
  "nc_y_nihtb__comp__fluid__fullcorr_tscore",  # Fluid cognition (outcome)
  "ph_y_pds__pds_mean"                          # Pubertal development (TVC)
)

data_dir <- Sys.getenv("ABCD_DATA_PATH", "/path/to/abcd/6_0/phenotype")

abcd_data <- create_dataset(
  dir_data = data_dir,
  study = "abcd",
  vars = requested_vars,
  release = "6.0",
  format = "parquet",
  categ_to_factor = TRUE,
  value_to_na = TRUE,
  add_labels = TRUE
)
```

## Data Transformation with Person-Mean Centering {.code}

```r
# Prepare data for LMM analysis with time-varying covariate
df_long <- abcd_data %>%
  # Filter to available annual assessments using NBDCtools
  filter_events_abcd(conditions = c("annual")) %>%
  # Rename variables for clarity
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    cognition = nc_y_nihtb__comp__fluid__fullcorr_tscore,
    puberty = ph_y_pds__pds_mean
  ) %>%
  # Keep only participants with at least 2 non-missing observations
  group_by(participant_id) %>%
  filter(sum(!is.na(cognition) & !is.na(puberty)) >= 2) %>%
  ungroup() %>%
  drop_na(cognition, puberty) %>%
  # Create numeric time variable from session_id
  mutate(time = as.numeric(session_id) - 1) %>%
  # CRITICAL: Person-mean centering of time-varying covariate
  group_by(participant_id) %>%
  mutate(
    # Between-person component: person's average puberty across all waves
    puberty_pm = mean(puberty, na.rm = TRUE),
    # Within-person component: deviation from person's own average
    puberty_cwc = puberty - puberty_pm
  ) %>%
  ungroup()

# Verify centering worked correctly
df_long %>%
  group_by(participant_id) %>%
  summarise(
    mean_cwc = mean(puberty_cwc, na.rm = TRUE),
    .groups = "drop"
  ) %>%
  summary()  # Mean of cwc should be ~0 for each person
```

## Descriptive Statistics {.code}

```r
# Create descriptive summary table by wave
descriptives_table <- df_long %>%
  select(session_id, cognition, puberty) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      cognition ~ "Fluid Cognition",
      puberty ~ "Pubertal Development"
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

/stage4-artifacts/lmm-time-varying-covariates/descriptives_table.html

# Statistical Analysis

## Model 1: Baseline Model (No TVC) {.code}

```r
# Fit baseline model with just time
m1 <- lmer(
  cognition ~ time + (1 + time | site:family_id:participant_id),
  data = df_long,
  REML = TRUE
)

summary(m1)
```

## Model 2: Raw TVC (Uncentered) {.code}

```r
# Add raw (uncentered) puberty as predictor
# WARNING: This conflates within- and between-person effects
m2 <- lmer(
  cognition ~ time + puberty + (1 + time | site:family_id:participant_id),
  data = df_long,
  REML = TRUE
)

summary(m2)
```

## Model 3: Person-Mean Centered TVC (Decomposed Effects) {.code}

```r
# Add decomposed TVC: within-person (cwc) + between-person (pm)
# This is the recommended approach for causal inference
m3 <- lmer(
  cognition ~ time + puberty_cwc + puberty_pm + (1 + time | site:family_id:participant_id),
  data = df_long,
  REML = TRUE
)

# Generate summary
summary(m3)

# Create formatted output table
model_summary_table <- gtsummary::tbl_regression(m3,
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

# Generate alternative summary with variance components
sjPlot::tab_model(m3,
    show.se = TRUE, show.df = FALSE, show.ci = FALSE,
    digits = 3,
    pred.labels = c("Intercept", "Time", "Puberty (Within)", "Puberty (Between)"),
    dv.labels = c("LMM with Time-Varying Covariate"),
    string.se = "SE",
    string.p = "P-Value",
    file = "lmm_tvc_results.html"
)
```

## Model Summary Output-1 {.output}

/stage4-artifacts/lmm-time-varying-covariates/model_summary.html

## Model Summary Output-2 {.output}

/stage4-artifacts/lmm-time-varying-covariates/lmm_tvc_results.html

## Model Comparison {.code}

```r
# Compare models using likelihood ratio test and information criteria
model_comparison <- anova(m1, m2, m3)

# Create comparison table
comparison_df <- data.frame(
  Model = c("M1: Time Only", "M2: Time + Raw TVC", "M3: Time + Decomposed TVC"),
  AIC = c(AIC(m1), AIC(m2), AIC(m3)),
  BIC = c(BIC(m1), BIC(m2), BIC(m3)),
  LogLik = c(logLik(m1), logLik(m2), logLik(m3))
)

comparison_table <- comparison_df %>%
  gt() %>%
  tab_header(title = "Model Comparison: Time-Varying Covariate Specifications") %>%
  fmt_number(columns = c(AIC, BIC, LogLik), decimals = 2)

gt::gtsave(comparison_table, filename = "model_comparison.html")
```

## Model Comparison Output {.output}

/stage4-artifacts/lmm-time-varying-covariates/model_comparison.html

## Interpretation {.note}

The key distinction in Model 3 is between the **within-person effect** (puberty_cwc) and the **between-person effect** (puberty_pm) of pubertal development on cognition:

**Within-person effect (puberty_cwc):** This coefficient answers: "When a participant's pubertal development is higher than their own average, how does their cognition compare to when they are at their average puberty level?" A negative coefficient would suggest that at times when a participant is more pubertally advanced than usual, their cognition tends to be lower. This effect controls for all stable between-person differences.

**Between-person effect (puberty_pm):** This coefficient answers: "Do participants who are, on average, more pubertally advanced tend to have different cognition scores?" This compares different people rather than the same person at different times.

If these two effects differ substantially, it indicates that the relationship between puberty and cognition operates differently at the within-person vs. between-person level - a critical insight for understanding developmental processes.

## Visualization {.code}

```r
# Create visualization comparing within and between effects
# First, get predicted values
df_long <- df_long %>%
  mutate(predicted = predict(m3))

# Sample participants for visualization
set.seed(123)
sample_ids <- sample(unique(df_long$participant_id), min(100, length(unique(df_long$participant_id))))
df_plot <- df_long %>% filter(participant_id %in% sample_ids)

# Create spaghetti plot colored by puberty level
visualization <- ggplot(df_plot, aes(x = time, y = cognition, group = participant_id)) +
  geom_line(aes(color = puberty_pm), alpha = 0.5) +
  geom_point(aes(color = puberty_pm), alpha = 0.3, size = 1) +
  scale_color_viridis_c(name = "Person-Mean\nPuberty", option = "plasma") +
  geom_smooth(aes(group = 1), method = "lm", color = "black", linewidth = 1.5, se = TRUE) +
  scale_x_continuous(
    breaks = 0:3,
    labels = c("Baseline", "Year 2", "Year 4", "Year 6")
  ) +
  labs(
    title = "Cognition Trajectories by Average Pubertal Development",
    subtitle = "Line color indicates person-mean puberty level (between-person effect)",
    x = "Assessment Wave",
    y = "Fluid Cognition"
  ) +
  theme_minimal() +
  theme(legend.position = "right")

visualization

ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 10, height = 6, dpi = 300
)
```

## Visualization {.output}

![Cognition Trajectories by Puberty Level](stage4-artifacts/lmm-time-varying-covariates/visualization.png)

## Visualization Notes {.note}

The plot shows individual cognition trajectories colored by each participant's average pubertal development level. This visualization captures the **between-person effect** - whether participants with higher average puberty (warmer colors) tend to show systematically different cognition trajectories than those with lower average puberty (cooler colors). The black line shows the overall population trend. Note that the **within-person effect** is harder to visualize directly, as it represents how deviations from one's own average puberty relate to deviations from one's own expected cognition trajectory.

# Discussion

This analysis demonstrates the importance of properly decomposing time-varying covariates in longitudinal models. The raw (uncentered) TVC coefficient in Model 2 represents a blend of within-person and between-person effects that can be misleading for causal inference. By contrast, Model 3's decomposition reveals how the puberty-cognition relationship operates at different levels.

The within-person effect is particularly valuable for causal inference because it controls for all stable individual differences (observed or unobserved) that might confound the relationship. If pubertal development causally affects cognition, we would expect to see a within-person effect: at times when a participant is more pubertally advanced than their own average, their cognition should differ in a predictable direction.

However, even the within-person effect can be confounded by time-varying factors that change alongside puberty. Future analyses might address this through instrumental variables, lagged predictors, or cross-lagged panel models. Additionally, the current model assumes that the within-person effect is constant across individuals; random slopes for the TVC could be added to test whether the effect varies across participants.

# Additional Resources

### Curran & Bauer (2011): The Disaggregation of Within-Person and Between-Person Effects {.resource}

Foundational paper explaining why and how to decompose time-varying predictors in longitudinal models, with clear guidance on person-mean centering and interpretation of effects.

**Badge:** PAPER
**URL:** https://doi.org/10.1146/annurev-psych-042716-051139

### lme4 Package Documentation {.resource}

Official CRAN documentation for the lme4 package, covering advanced model specifications including time-varying covariates, random slopes, and model comparison.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=lme4

### Singer & Willett: Applied Longitudinal Data Analysis {.resource}

Comprehensive textbook on longitudinal modeling with detailed chapters on time-varying covariates, centering strategies, and interpretation of within-person vs. between-person effects.

**Badge:** BOOK
**URL:** https://oxford.universitypressscholarship.com/view/10.1093/acprof:oso/9780195152968.001.0001

### performance Package for Model Diagnostics {.resource}

R package for model comparison and diagnostics, including tools for comparing nested models with different covariate specifications and checking model assumptions.

**Badge:** TOOL
**URL:** https://easystats.github.io/performance/
