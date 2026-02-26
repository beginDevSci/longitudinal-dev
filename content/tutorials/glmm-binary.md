---
title: "GLMM: Binary Outcomes"
slug: glmm-binary
author: Biostatistics Working Group
date_iso: 2025-11-06
tags:
  - abcd-study
  - generalized-linear-mixed-model
  - binary-outcomes
  - logistic-regression
family: GLMM
family_label: Generalized Linear Mixed Models (GLMM)
engine: glmmTMB
engines:
  - glmmTMB
  - lme4
covariates: TIC
outcome_type: Binary
difficulty: intermediate
timepoints: 3_5
summary: Apply generalized linear mixed models for binary repeated measures outcomes, modeling the probability of events like substance use initiation over time while accounting for individual heterogeneity.
description: Apply generalized linear mixed models for binary repeated measures outcomes, modeling the probability of events like substance use initiation over time while accounting for individual heterogeneity.
---

# Overview

## Summary {.summary}

Generalized Linear Mixed Models (GLMMs) extend linear mixed models to handle non-continuous outcomes through link functions that connect the linear predictor to the expected value of the outcome. For binary outcomes (yes/no, present/absent), the logit link transforms probabilities into log-odds, enabling modeling of repeated binary measurements while accounting for the correlation structure induced by repeated observations on the same individuals. This tutorial applies GLMM with a logit link to analyze substance use initiation across ABCD assessments, demonstrating how to specify random effects structures, interpret odds ratios, and translate results to the probability scale for substantive interpretation.

## Features {.features}

- **When to Use:** Apply when your outcome is binary (0/1) and measured repeatedly over time, and you want to model the probability of the event while accounting for individual differences in baseline risk and change over time.
- **Key Advantage:** GLMM handles the bounded nature of probabilities correctly, provides odds ratios for interpretation, and allows both random intercepts (individual differences in baseline risk) and random slopes (individual differences in change).
- **What You'll Learn:** How to specify binary GLMMs using glmmTMB, interpret fixed effects as odds ratios, understand marginal vs. conditional effects, and visualize predicted probabilities over time.

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
library(glmmTMB)      # Generalized linear mixed models
library(lme4)         # Alternative GLMM fitting
library(broom.mixed)  # Tidy output for mixed models
library(gt)           # For creating formatted tables
library(emmeans)      # For marginal means and contrasts

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
  "ab_g_dyn__design_site",
  "ab_g_stc__design_id__fam",
  "su_y_tlfb__alc__1mo_ud",       # Alcohol use days (last 30 days)
  "ab_g_stc__cohort_sex"          # Sex (time-invariant covariate)
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

## Data Transformation {.code}

```r
# Prepare data for GLMM analysis
df_long <- abcd_data %>%
  # Filter to available annual assessments using NBDCtools
  filter_events_abcd(conditions = c("annual")) %>%
  # Rename variables for clarity
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    alcohol_use = su_y_tlfb__alc__1mo_ud,
    sex = ab_g_stc__cohort_sex
  ) %>%
  # Create binary outcome: any alcohol use vs. none
  mutate(
    any_use = as.integer(alcohol_use > 0)
  ) %>%
  # Keep only participants with at least 2 observations
  group_by(participant_id) %>%
  filter(sum(!is.na(any_use)) >= 2) %>%
  ungroup() %>%
  drop_na(any_use, sex) %>%
  # Create numeric time variable from session_id
  mutate(
    time = as.numeric(session_id) - 1,
    # Convert sex codes to labeled factor (1=Male, 2=Female in ABCD)
    sex = factor(sex, levels = c(1, 2), labels = c("Male", "Female"))
  )

# Check the prevalence of use at each wave
df_long %>%
  group_by(session_id) %>%
  summarise(
    n = n(),
    n_users = sum(any_use),
    prevalence = mean(any_use),
    .groups = "drop"
  )
```

## Descriptive Statistics {.code}

```r
# Create prevalence table by wave
prevalence_table <- df_long %>%
  mutate(any_use = factor(any_use, levels = c(0, 1), labels = c("No Use", "Any Use"))) %>%
  select(session_id, any_use) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      any_use ~ "Alcohol Use"
    )
  ) %>%
  modify_header(all_stat_cols() ~ "**{level}**<br>N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "**Assessment Wave**") %>%
  bold_labels() %>%
  italicize_levels()

### Apply compact styling
theme_gtsummary_compact()

prevalence_table <- as_gt(prevalence_table)

### Save the table as HTML
gt::gtsave(prevalence_table, filename = "prevalence_table.html")

### Print the table
prevalence_table

```

## Prevalence Table Output {.output}

/stage4-artifacts/glmm-binary/prevalence_table.html

# Statistical Analysis

## Model 1: Random Intercept Only {.code}

```r
# Fit GLMM with random intercept only
# This allows individuals to differ in baseline risk (log-odds)
m1 <- glmmTMB(
  any_use ~ time + (1 | site:family_id:participant_id),
  family = binomial(link = "logit"),
  data = df_long
)

summary(m1)
```

## Model 2: Random Intercept + Random Slope {.code}

```r
# Add random slope for time
# This allows individuals to differ in how their risk changes over time
m2 <- glmmTMB(
  any_use ~ time + (1 + time | site:family_id:participant_id),
  family = binomial(link = "logit"),
  data = df_long
)

summary(m2)
```

## Model 3: Add Time-Invariant Covariate (Sex) {.code}

```r
# Add sex as a predictor
m3 <- glmmTMB(
  any_use ~ time + sex + (1 + time | site:family_id:participant_id),
  family = binomial(link = "logit"),
  data = df_long
)

# Full summary
summary(m3)

# Create formatted coefficient table
model_summary_table <- broom.mixed::tidy(m3, conf.int = TRUE, exponentiate = FALSE) %>%
  filter(effect == "fixed") %>%
  select(term, estimate, std.error, statistic, p.value, conf.low, conf.high) %>%
  gt() %>%
  tab_header(title = "GLMM for Binary Outcomes: Fixed Effects (Log-Odds Scale)") %>%
  fmt_number(columns = c(estimate, std.error, statistic, conf.low, conf.high), decimals = 3) %>%
  fmt_number(columns = p.value, decimals = 4)

gt::gtsave(model_summary_table, filename = "model_summary.html")
```

## Model Summary Output {.output}

/stage4-artifacts/glmm-binary/model_summary.html

## Odds Ratios {.code}

```r
# Convert to odds ratios for interpretation
odds_ratios <- broom.mixed::tidy(m3, conf.int = TRUE, exponentiate = TRUE) %>%
  filter(effect == "fixed") %>%
  select(term, estimate, conf.low, conf.high, p.value) %>%
  rename(
    OR = estimate,
    CI_lower = conf.low,
    CI_upper = conf.high
  )

or_table <- odds_ratios %>%
  gt() %>%
  tab_header(title = "Odds Ratios with 95% Confidence Intervals") %>%
  fmt_number(columns = c(OR, CI_lower, CI_upper), decimals = 3) %>%
  fmt_number(columns = p.value, decimals = 4) %>%
  cols_label(
    term = "Predictor",
    OR = "Odds Ratio",
    CI_lower = "95% CI Lower",
    CI_upper = "95% CI Upper",
    p.value = "p-value"
  )

gt::gtsave(or_table, filename = "odds_ratios.html")
```

## Odds Ratios Output {.output}

/stage4-artifacts/glmm-binary/odds_ratios.html

## Interpretation {.note}

The GLMM coefficients are on the log-odds scale, which can be difficult to interpret directly. Converting to **odds ratios (OR)** provides a more intuitive interpretation:

**Time effect:** The OR for time represents the multiplicative change in odds of alcohol use for each unit increase in time. An OR > 1 indicates increasing probability of use over time. For example, OR = 1.5 means the odds of use increase by 50% with each assessment wave.

**Sex effect:** The OR for sex compares the odds of use between males and females. An OR > 1 for females (with males as reference) means females have higher odds of use; OR < 1 means females have lower odds.

**Conditional vs. Marginal interpretation:** These effects are **conditional** on the random effects (i.e., subject-specific). For a population-averaged (marginal) interpretation, the effects would be attenuated because the random intercept variance is absorbed. The conditional interpretation answers: "For a given individual, how does time affect their odds of use?"

## Model Comparison {.code}

```r
# Compare models using AIC/BIC
model_comparison <- data.frame(
  Model = c("M1: Random Intercept", "M2: + Random Slope", "M3: + Sex Covariate"),
  AIC = c(AIC(m1), AIC(m2), AIC(m3)),
  BIC = c(BIC(m1), BIC(m2), BIC(m3)),
  LogLik = c(logLik(m1), logLik(m2), logLik(m3))
)

comparison_table <- model_comparison %>%
  gt() %>%
  tab_header(title = "Model Comparison") %>%
  fmt_number(columns = c(AIC, BIC, LogLik), decimals = 2)

gt::gtsave(comparison_table, filename = "model_comparison.html")
```

## Model Comparison Output {.output}

/stage4-artifacts/glmm-binary/model_comparison.html

## Predicted Probabilities Visualization {.code}

```r
# Generate predicted probabilities across time by sex
# Create prediction grid
newdata <- expand.grid(
  time = seq(0, 3, by = 0.5),
  sex = c("Male", "Female")
)

# Get marginal predictions (averaging over random effects)
# Using emmeans for marginal means
emm <- emmeans(m3, ~ time + sex, at = list(time = seq(0, 3, by = 0.5)),
               type = "response")
pred_df <- as.data.frame(emm)

# Create probability plot
prob_plot <- ggplot(pred_df, aes(x = time, y = prob, color = sex, fill = sex)) +
  geom_ribbon(aes(ymin = asymp.LCL, ymax = asymp.UCL), alpha = 0.2, color = NA) +
  geom_line(linewidth = 1.2) +
  geom_point(size = 2) +
  scale_x_continuous(
    breaks = 0:3,
    labels = c("Baseline", "Year 2", "Year 4", "Year 6")
  ) +
  scale_y_continuous(
    limits = c(0, 1),
    labels = scales::percent_format()
  ) +
  scale_color_manual(values = c("Male" = "#2E86AB", "Female" = "#A23B72")) +
  scale_fill_manual(values = c("Male" = "#2E86AB", "Female" = "#A23B72")) +
  labs(
    title = "Predicted Probability of Any Alcohol Use Over Time",
    subtitle = "Marginal predictions from GLMM with 95% confidence intervals",
    x = "Assessment Wave",
    y = "Probability of Any Use",
    color = "Sex",
    fill = "Sex"
  ) +
  theme_minimal() +
  theme(legend.position = "bottom")

prob_plot

ggsave(
  filename = "visualization.png",
  plot = prob_plot,
  width = 8, height = 6, dpi = 300
)
```

## Predicted Probabilities Plot {.output}

![Predicted Probabilities of Alcohol Use](stage4-artifacts/glmm-binary/visualization.png)

## Visualization Notes {.note}

The plot displays **marginal predicted probabilities** - the average probability of alcohol use at each time point, accounting for the random effects distribution. The shaded regions represent 95% confidence intervals. This visualization makes the time trend and sex differences more interpretable than log-odds or odds ratios, as probabilities are bounded between 0 and 1 and directly represent the likelihood of the outcome.

Note that these marginal probabilities will be closer to 0.5 than the conditional (subject-specific) probabilities would be, because averaging over the random effects distribution compresses the probability scale. Individual participants may have substantially higher or lower probabilities depending on their random intercept value.

# Discussion

This analysis demonstrates the application of GLMMs to binary longitudinal outcomes, a common scenario in developmental and clinical research where the outcome is whether an event has occurred (e.g., substance use initiation, symptom presence, milestone achievement). The logit link function ensures that predicted probabilities remain bounded between 0 and 1, while the random effects structure accounts for the correlation among repeated measures from the same individual.

Key findings typically include: (1) The time effect captures the developmental trajectory of risk - whether and how fast the probability of the outcome changes over time; (2) The random intercept variance quantifies individual differences in baseline susceptibility; (3) The random slope variance (if included) captures heterogeneity in developmental trajectories - some individuals may show steeper increases in risk than others.

Several extensions are possible: (1) Time-varying covariates can be added to examine how changes in other factors relate to changes in outcome probability; (2) Interaction terms (e.g., time × sex) can test whether trajectories differ by group; (3) Alternative link functions (probit, complementary log-log) may better suit certain data patterns; (4) Zero-inflation or hurdle models can handle outcomes with excess zeros.

# Additional Resources

### glmmTMB Package Documentation {.resource}

Comprehensive documentation for the glmmTMB package, covering model specification for binary and other non-Gaussian outcomes, including random effects structures and model diagnostics.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=glmmTMB

### Agresti: Categorical Data Analysis {.resource}

Foundational textbook covering generalized linear models for categorical outcomes, including detailed treatment of logistic regression, random effects models, and interpretation of odds ratios.

**Badge:** BOOK
**URL:** https://www.wiley.com/en-us/Categorical+Data+Analysis%2C+3rd+Edition-p-9780470463635

### emmeans Package for Marginal Means {.resource}

R package for computing estimated marginal means from fitted models, essential for obtaining predicted probabilities and pairwise comparisons from GLMMs.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=emmeans

### Bolker et al. (2009): GLMMs in Ecology {.resource}

Influential paper providing practical guidance on fitting and interpreting GLMMs, including discussion of random effects interpretation and model selection for binary outcomes.

**Badge:** PAPER
**URL:** https://doi.org/10.1016/j.tree.2008.10.008
