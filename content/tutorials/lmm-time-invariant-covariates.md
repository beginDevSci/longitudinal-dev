---
title: "LMM: Time-Invariant Covariates"
slug: lmm-time-invariant-covariates
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - linear-mixed-model
  - time-invariant-covariate
family: LMM
family_label: Linear Mixed Models (LMM)
engine: lme4
covariates: TIC
outcome_type: Continuous
description: Incorporate time-invariant covariates within mixed models to test whether baseline characteristics explain differences in intercepts or slopes across ABCD participants.
---

# Overview

## Summary {.summary}

Linear mixed models with random intercepts and slopes extended with time-invariant covariates allow examination of how stable individual characteristics predict both baseline levels and rates of change over time. By modeling covariate effects on latent intercept and slope parameters, this approach reveals whether factors like demographics or socioeconomic status influence starting points, developmental trajectories, or both. This tutorial analyzes cognition scores from ABCD youth across four annual assessments, incorporating parental education as a time-invariant covariate to test whether it predicts baseline cognition levels and rates of cognitive change.

## Features {.features}

- **When to Use:** Use when you suspect static characteristics (e.g., parental education, sex) explain differences in both initial levels and longitudinal change in ABCD outcomes.
- **Key Advantage:** Simultaneously estimates the population trajectory and covariate effects, making it easy to see how subject-level factors shift intercepts and slopes.
- **What You'll Learn:** How to add time-invariant covariates to lme4 models, interpret their effects on intercepts/slopes, and visualize stratified trajectories.

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
library(arrow)       # For reading Parquet files
library(tidyverse)   # For data manipulation & visualization
library(gtsummary)   # For generating publication-quality summary tables
library(rstatix)     # Provides tidy-format statistical tests
library(lme4)        # Linear mixed-effects models (LMMs)
library(sjPlot)      # Visualization of mixed models
library(kableExtra)  # Formatting & styling in HTML/Markdown reports
library(performance) # Useful functions for model diagnostics & comparisons
library(ggeffects)   # Adjusted regression predictions

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "ab_g_dyn__cohort_edu__cgs",  # Parent education
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

# Prepare data for LMM analysis with time-invariant covariate
df_long <- abcd_data %>%
    select(participant_id, session_id, ab_g_dyn__design_site, ab_g_stc__design_id__fam, ab_g_dyn__cohort_edu__cgs, nc_y_nihtb__comp__cryst__fullcorr_tscore) %>%
    # Filter to include only 4 relevant measurement occasions
    filter(session_id %in% c("ses-00A", "ses-02A", "ses-04A", "ses-06A")) %>%
    # Forward fill parental education (time-invariant covariate)
    group_by(participant_id) %>%
    fill(ab_g_dyn__cohort_edu__cgs, .direction = "down") %>%
    ungroup() %>%
    # Ensure dataset is sorted by participant and session
    arrange(participant_id, session_id) %>%
    # Convert categorical and numerical variables
    mutate(
        participant_id = factor(participant_id), # Convert IDs to factors
        session_id = factor(session_id, # Convert session to a factors
            levels = c("ses-00A", "ses-02A", "ses-04A", "ses-06A"),
            labels = c("Baseline", "Year_2", "Year_4", "Year_6")
        ),
        time = as.numeric(session_id) - 1, # Convert session_id to numeric
        ab_g_dyn__design_site = factor(ab_g_dyn__design_site),  # Convert site to a factor
        ab_g_stc__design_id__fam = factor(ab_g_stc__design_id__fam), # Convert family id to a factor
        # Convert and clean parent education variable
        parent_education = as.numeric(ab_g_dyn__cohort_edu__cgs),
        parent_education = na_if(parent_education, 777),  # Set "Decline to Answer" as NA
        # Categorize parent education levels
        parent_education_cat = case_when(
          parent_education <= 12 ~ "Less than HS",
          parent_education %in% 13:14 ~ "HS Diploma/GED",
          parent_education %in% 15:17 ~ "Some College/Associate",
          parent_education == 18 ~ "Bachelor's Degree",
          parent_education %in% 19:21 ~ "Graduate Degree",
          TRUE ~ NA_character_
        ) %>% factor(levels = c("Less than HS", "HS Diploma/GED",
                                 "Some College/Associate", "Bachelor's Degree", "Graduate Degree")),
        nc_y_nihtb__comp__cryst__fullcorr_tscore =
            round(as.numeric(nc_y_nihtb__comp__cryst__fullcorr_tscore), 2)
    ) %>%
    # Rename variables for clarity
    rename(
        site = ab_g_dyn__design_site,
        family_id = ab_g_stc__design_id__fam,
        cognition = nc_y_nihtb__comp__cryst__fullcorr_tscore
    ) %>%
    # Remove participants with any missing cognition scores across time points
    group_by(participant_id) %>%
    filter(sum(!is.na(cognition)) >= 2) %>%  # Keep only participants with at least 2 non-missing cognition scores
    ungroup() %>%
    drop_na(site, family_id, participant_id, cognition)  # Ensure all remaining rows have complete cases
```

## Descriptive Statistics {.code}

```r
# Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, parent_education_cat, cognition) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      parent_education_cat ~ "Parent Education",
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

/stage4-artifacts/lmm-time-invariant-covariates/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r
# Fit a Linear Mixed Model (LMM) to with random intercepts and slopes
model <- lmerTest::lmer(
    cognition ~ time + parent_education + (1 + time | site:family_id:participant_id) , # Fixed effect (time), random intercept & slope (participant_id)
    data = df_long # Dataset containing repeated measures of cognition
)

# Generate a summary table for the LMM model
model_summary <- gtsummary::tbl_regression(model,
    digits = 3,
    intercept = TRUE
) %>%
  gtsummary::as_gt()

# Display model summary
model_summary

### Save the gt table
gt::gtsave(
  data = model_summary,
  filename = "model_summary_table.html",
  inline_css = FALSE
)

# Generate comprehensive diagnostics with sjPlot
sjPlot::tab_model(model,
    show.se = TRUE, show.df = FALSE, show.ci = FALSE,
    digits = 3,
    pred.labels = c("Intercept", "Time", "Parent Education"),
    dv.labels = c("LMM Model (lme4)"),
    string.se = "SE",
    string.p = "P-Value",
    file = "lmm_model_results.html"
)

# Test interaction between education and time
model_interaction <- lmer(
    cognition ~ time * parent_education + (1 + time | participant_id),
    data = df_long
)

# Compare models
anova_output <- anova(model, model_interaction)

# 1. Convert the data frame to a gt object
anova_gt_table <- gt::gt(
    data = anova_output,
    rownames_to_stub = TRUE # This keeps the model comparison info in the first column
) %>%
    # Optional: Customize the table (e.g., format p-values)
    gt::fmt_number(
        columns = everything(),
        decimals = 3
    ) %>%
    gt::fmt_scientific(
        columns = tidyselect::all_of(c("Chisq", "Pr(>Chisq)")),
        decimals = 4
    ) %>%
    gt::tab_header(
        title = "ANOVA Model Comparison"
    )

# 2. Save the gt table as HTML
gt::gtsave(
    data = anova_gt_table,
    filename = "anova_model_comparison.html",
    inline_css = FALSE
)

```

## Model Summary Output-1 {.output}

/stage4-artifacts/lmm-time-invariant-covariates/model_summary_table.html

## Model Summary Output-2 {.output}

/stage4-artifacts/lmm-time-invariant-covariates/lmm_model_results.html

## Model Summary Output-3 {.output}

/stage4-artifacts/lmm-time-invariant-covariates/anova_model_comparison.html

## Interpretation {.note}

The **fixed effects estimates** indicate that the average cognition score at baseline (time = 0) is 46.1 (SE = 0.32, p < 0.001). Over time, cognition declines by approximately 0.35 points per biannual assessment (SE = 0.05, p < 0.001), suggesting a gradual decline in cognitive performance. Additionally, higher parental education levels are associated with significantly higher cognition scores (β = 1.17, SE = 0.08, p < 0.001), indicating a positive relationship between parental education and cognitive performance.

The **random effects** (see Model Summary Output-2) reveal substantial variability in both baseline cognition scores (τ₀₀ = 87.27) and individual rates of cognitive decline (τ₁₁ = 2.30). The negative correlation (ρ₀₁ = -0.31) between the intercept and slope suggests that individuals with higher initial cognition scores tend to experience steeper declines over time. The intraclass correlation (ICC = 0.71) indicates that 71% of the total variance in cognition scores is attributable to between-individual differences rather than within-person fluctuations over time.

## Visualization {.code}

```r
# Get predicted neurocognition scores by parental education
preds_edu <- ggpredict(model, terms = "parent_education")  # Correct function

# Plot
visualization <- ggplot(preds_edu, aes(x = x, y = predicted)) +
  geom_point(size = 3, color = "darkred") +
  geom_line(group = 1, color = "darkred") +
  geom_errorbar(aes(ymin = conf.low, ymax = conf.high), width = 0.2) +
  labs(title = "Effect of Parental Education on Neurocognition",
       x = "Parental Education Level",
       y = "Predicted Neurocognition Score") +
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

## Plot {.output}

![Plot](/stage4-artifacts/lmm-time-invariant-covariates/visualization.png)

## Visualization Notes {.note}

The plot illustrates the effect of parental education on predicted neurocognition scores. Each point represents the model-estimated neurocognition score for a given level of parental education, with error bars showing the 95% confidence intervals. The visualization demonstrates a clear positive relationship: as parental education increases, predicted neurocognition scores also increase, controlling for time and individual variability.

This pattern confirms the significant positive effect of parental education (β = 1.17, p < 0.001) observed in the fixed effects, showing that higher parental education levels are associated with better cognitive performance across all time points.

# Discussion

The longitudinal analysis revealed significant insights into the influence of parental education on cognitive performance. Participants with higher parental education demonstrated significantly higher baseline cognition scores (β = 1.17, p < 0.001), indicating that parental education is a strong predictor of initial cognitive ability.

The ANOVA model comparison (Model Summary Output-3) tested whether parental education also moderates the rate of cognitive change by comparing the main effects model to an interaction model (time × parent_education). The significant chi-square test (χ² = 101.09, p < 0.001) indicates that the interaction model provides a significantly better fit, suggesting that the effect of parental education on cognition may vary across time. This finding warrants further investigation into how parental education influences not only baseline cognitive performance but also developmental trajectories.

By including parental education as a time-invariant covariate in the linear mixed model, we were better able to capture and account for individual differences in cognitive development, demonstrating that stable background characteristics significantly influence longitudinal cognitive outcomes.

# Additional Resources

### lme4 Package Documentation {.resource}

Official CRAN documentation for the lme4 package, with examples of incorporating time-invariant covariates into linear mixed models. Covers model specification with covariates predicting both intercepts and slopes, including interaction terms with time.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=lme4

### Fitting Linear Mixed-Effects Models Using lme4 {.resource}

Comprehensive lme4 vignette with Section 2.4 covering conditional models with time-invariant predictors. Demonstrates centering strategies for covariates, interpretation of cross-level interactions, and testing whether covariates predict individual differences in change rates.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/lme4/vignettes/lmer.pdf

### Longitudinal Data Analysis {.resource}

Singer & Willett (2003). Applied Longitudinal Data Analysis: Modeling Change and Event Occurrence. Chapters 5-6 provide detailed coverage of time-invariant covariates in growth models, including centering decisions, interpretation of covariate-by-time interactions, and distinguishing between effects on initial status versus change. Note: access may require institutional or paid subscription.

**Badge:** BOOK
**URL:** https://oxford.universitypressscholarship.com/view/10.1093/acprof:oso/9780195152968.001.0001/acprof-9780195152968

### Centering Predictor Variables in Mixed Models {.resource}

Enders & Tofighi (2007). Methodological paper addressing the critical question of when and how to center time-invariant covariates in multilevel models. Explains grand-mean versus group-mean centering and how centering choices affect the interpretation of fixed and random effects. Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://doi.org/10.1037/1082-989X.12.2.121
