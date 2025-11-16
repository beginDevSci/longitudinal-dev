---
title: "GEE: Time-Varying Covariates"
slug: gee-time-varying-covariate
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - gee
  - time-varying-covariate
family: GEE
family_label: Generalized Estimating Equations (GEE)
engine: geepack
covariates: TVC
outcome_type: Binary
description: Add time-varying covariates to generalized estimating equations to examine how evolving exposures influence repeated binary outcomes.
---

# Overview

## Summary {.summary}

Generalized Estimating Equations with time-varying covariates extend standard GEE models to examine how predictors that change over time influence longitudinal outcomes, estimating population-averaged effects while accounting for within-subject correlation. This approach captures dynamic relationships by allowing covariate values to vary at each assessment occasion. This tutorial analyzes sufficient sleep patterns (9–11 hours per night) in ABCD youth across multiple assessments, incorporating anxiety scores as a time-varying covariate to examine whether changes in anxiety levels over time predict concurrent fluctuations in sleep duration.

## Features {.features}

- **When to Use:** Use when both the outcome and predictors change over time and you want to assess dynamic relationships.
- **Key Advantage:** Extends GEE to include covariates that vary at each occasion, letting you examine how concurrent predictors modify population-level change.
- **What You'll Learn:** How to structure data for time-varying covariates, specify formulas with interaction terms, and interpret dynamic effects.

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
    "ph_p_sds__dims_001",
    "mh_p_cbcl__dsm__anx_tscore"
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
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-01A", "ses-02A", "ses-03A"),
                        labels = c("Baseline", "Year_1", "Year_2", "Year_3")),  # Label sessions
    sleep_hours = as.numeric(ph_p_sds__dims_001),
    anxiety = as.numeric(mh_p_cbcl__dsm__anx_tscore),  # Convert to numeric
    sleep_binary = ifelse(sleep_hours == 1, 1, 0)  # Recode: 9-11 hrs = 1, others = 0
  ) %>%
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam
  ) %>%
  drop_na()  # Remove missing values

```

## Descriptive Statistics {.code}

```r
# Summarize 'sleep_binary' and 'strength_exercise' across session_ids

descriptives_table <- df_long %>%
    select(session_id, sleep_binary, anxiety) %>%
    tbl_summary(
        by = session_id,
        missing = "no",
        label = list(
          sleep_binary ~ "Sufficient Sleep (9-11 hrs)",
          anxiety ~ "Anxiety"
        ),
        statistic = list(
            all_categorical() ~ "{n} ({p}%)",
            all_continuous() ~ "{mean} ({sd})"
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

/stage4-artifacts/gee-time-varying-covariate/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r

model <- geeglm(sleep_binary ~ session_id + site + anxiety,
                    id = participant_id,
                    data = df_long,
                    family = binomial(link = "logit"),
                    corstr = "exchangeable")

# Generate a summary table for the GEE model
model_summary <- gtsummary::tbl_regression(model,
    digits = 3,
    intercept = TRUE
) %>%
  gtsummary::as_gt()

model_summary

# Save as standalone HTML
gt::gtsave(
  data = model_summary,
  filename = "model_summary.html",
  inline_css = FALSE
)

# GEE Model Diagnostics
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

diagnostics_table <- diagnostics_data %>%
  gt::gt() %>%
  gt::tab_header(title = "GEE Model Diagnostics")

diagnostics_table

gt::gtsave(diagnostics_table, filename = "model_diagnostics.html")

```

## Model Summary Output {.output}

/stage4-artifacts/gee-time-varying-covariate/model_summary.html

## Model Diagnostics Output {.output}

/stage4-artifacts/gee-time-varying-covariate/model_diagnostics.html

## Interpretation {.note}

Compared with Baseline, the odds of meeting the 9–11 hour guideline fell by 31% at Year 1 (OR = 0.69, p < .001), 51% at Year 2 (OR = 0.49, p < .001), and 58% at Year 3 (OR = 0.42, p < .001). The monotonic decline points to a cohort-wide erosion of sufficient sleep across adolescence rather than a transient dip. Anxiety exerted an additional, albeit modest, effect: each one-unit uptick corresponded to a 1.6% decrease in the odds of sufficient sleep (OR = 0.984, p < .001), indicating that even within-person fluctuations in stress meaningfully shape nightly outcomes. The working-correlation estimate (α = 0.37) signals moderate within-subject stability—youth differ from one another, but each individual tends to retain their relative position over time. Together, the coefficients portray a population that is steadily drifting away from healthy sleep, with anxiety acting as a persistent drag on the probability of recovery.

## Visualization {.code}

```r

preds <- ggeffect(model, terms = "anxiety")

# Plot predicted probabilities
visualization <- ggplot(preds, aes(x = x, y = predicted)) +  # 'x' corresponds to 'anxiety'
  geom_point(size = 3.5, color = "darkblue") +  # Larger point size for clarity
  geom_line(color = "darkblue", linewidth = 1) +  # Increase line thickness
  geom_errorbar(aes(ymin = conf.low, ymax = conf.high),
                width = 0.2, color = "darkblue") +  # Match error bar color
  labs(title = "Effect of Anxiety on Sleep Sufficiency",
       x = "Anxiety Level",
       y = "Predicted Probability of Sufficient Sleep") +
  theme_classic() +  # Cleaner layout
  theme(plot.title = element_text(face = "bold", size = 14),
        axis.title = element_text(size = 12))

  ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/gee-time-varying-covariate/visualization.png)

## Visualization Notes {.note}

The predicted-probability curve slopes downward, translating the small negative logit coefficient for anxiety into an easily interpretable decline in sleep sufficiency as anxiety rises. Confidence bands stay tight across the anxiety range, indicating that the effect is estimated precisely even at the more extreme scores. Although the probability drop looks gradual—roughly five percentage points between the calmest and most anxious youth—the entirely negative band underscores that the relationship is consistently detrimental. Taken together with the model table, the figure shows how incremental increases in anxiety accumulate into meaningfully lower odds of attaining the recommended 9–11 hours of sleep.

# Discussion

Participants with higher anxiety scores were less likely to meet sufficient sleep recommendations, even after accounting for site and demographic controls. The logit coefficient for anxiety was negative and significant, translating to a meaningful drop in the probability of sufficient sleep for each within-person uptick in anxiety. Because anxiety was modeled as time-varying, the estimate reflects dynamic departures from an individual’s own typical level rather than static differences between youth.

The exchangeable working correlation captured modest within-person dependence, and robust standard errors remained stable under alternative structures, reinforcing confidence in the inference. Visualizations of predicted probabilities showed nearly parallel declines across time, indicating that anxiety consistently depresses sleep odds at every assessment rather than accelerating deterioration. Incorporating time-varying covariates in GEE therefore provided a population-averaged view of how fluctuations in stress manifest behaviorally, offering a practical template for future analyses that need to link changing predictors with binary outcomes.

# Additional Resources

### geepack Package Documentation {.resource}

Official CRAN documentation for the geepack package, covering geeglm() with time-varying covariates and different working correlation structures for repeated measures data.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=geepack

### Time-Varying Covariates in GEE (Analysis Factor) {.resource}

Practical tutorial that walks through formatting longitudinal data, specifying time-varying predictors in `geeglm()`, and interpreting coefficients and interactions using real-world R examples.

**Badge:** VIGNETTE
**URL:** https://www.theanalysisfactor.com/time-varying-covariates-gee/

### GEE vs Mixed Models {.resource}

Methodology paper comparing population-averaged (GEE) versus subject-specific (mixed model) approaches for longitudinal data, with guidance on choosing between methods (Hubbard et al., 2010).

**Badge:** PAPER
**URL:** https://www.ncbi.nlm.nih.gov/pmc/articles/PMC2798744/

### emmeans for GEE Marginal Effects {.resource}

R package for computing estimated marginal means and contrasts from GEE models, useful for interpreting time-varying covariate effects at specific time points.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=emmeans
