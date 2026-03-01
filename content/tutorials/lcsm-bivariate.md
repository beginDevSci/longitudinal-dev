---
title: "LCSM: Bivariate"
slug: lcsm-bivariate
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - latent-change
  - structural-equation-modeling
  - bivariate
family: LCSM
family_label: Latent Change Score Models (LCSM)
engine: lavaan
engines:
  - lavaan
covariates: None
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Examine coupled developmental change across two constructs using a bivariate latent change score model with four time points, estimating cross-domain coupling parameters for height and weight in ABCD youth.
description: Examine coupled developmental change across two constructs using a bivariate latent change score model with four time points, estimating cross-domain coupling parameters for height and weight in ABCD youth.
---

# Overview

## Summary {.summary}

Bivariate Latent Change Score Models (BLCSM) extend the univariate LCSM to examine coupled change across two developmental processes. Rather than modeling each construct in isolation, BLCSM estimates cross-domain coupling parameters that test whether the level of one process predicts subsequent change in the other. This tutorial applies BLCSM with four time points (Baseline, Year 2, Year 4, Year 6) to analyze coupled height and weight changes in ABCD youth, demonstrating how to specify univariate models for each process, combine them into a bivariate framework, and test whether height-weight coupling is bidirectional or asymmetric.

## Features {.features}

- **When to Use:** Apply when you have repeated measures on two constructs and hypothesize that change in one domain depends on the level or change in another domain (e.g., height driving weight change, or vice versa).
- **Key Advantage:** BLCSM separates within-domain dynamics (proportional change, constant change) from cross-domain coupling, allowing direct tests of developmental leading-lagging relationships between constructs.
- **What You'll Learn:** How to specify univariate LCSMs for two processes, combine them into a bivariate model with cross-domain coupling, compare uncoupled versus coupled models, and interpret coupling parameters.

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
    "ph_y_anthr__height_mean",
    "ph_y_anthr__weight_mean"
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
### Create a long-form dataset with relevant columns
# Uses 4 time points (Baseline, Year 2, Year 4, Year 6) for model identification
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site,
         ab_g_stc__design_id__fam, ph_y_anthr__height_mean,
         ph_y_anthr__weight_mean) %>%
  filter_events_abcd(conditions = c("annual")) %>%
  filter(session_id %in% c("ses-00A", "ses-02A", "ses-04A", "ses-06A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-02A", "ses-04A", "ses-06A"),
                        labels = c("Baseline", "Year_2", "Year_4", "Year_6"))
  ) %>%
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    height = ph_y_anthr__height_mean,
    weight = ph_y_anthr__weight_mean
  ) %>%
  droplevels() %>%
  drop_na(height, weight)

### Reshape data from long to wide format for LCSM
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = c(height, weight),
    names_sep = "_"
  ) %>%
  drop_na(starts_with("height_"), starts_with("weight_"))
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table for both constructs
descriptives_table <- df_long %>%
  select(session_id, height, weight) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      height ~ "Height (in)",
      weight ~ "Weight (lbs)"
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

/stage4-artifacts/lcsm-bivariate/descriptives_table.html

# Statistical Analysis

## Step 1: Univariate LCSM for Height {.code}

```r
### Fit univariate LCSM for height to establish baseline dynamics
height_model <- '
  # Latent true scores for height
  h1 =~ 1*height_Baseline
  h2 =~ 1*height_Year_2
  h3 =~ 1*height_Year_4
  h4 =~ 1*height_Year_6

  # Latent change scores for height
  dh12 =~ 1*h2
  dh23 =~ 1*h3
  dh34 =~ 1*h4

  # Autoregressive paths
  h2 ~ 1*h1
  h3 ~ 1*h2
  h4 ~ 1*h3

  # Means
  h1 ~ 1
  dh12 ~ 1
  dh23 ~ 1
  dh34 ~ 1

  # Variances and covariances
  h1 ~~ h1
  dh12 ~~ dh*dh12
  dh23 ~~ dh*dh23
  dh34 ~~ dh*dh34
  h1 ~~ dh12
  h1 ~~ dh23
  h1 ~~ dh34
  dh12 ~~ dh23
  dh12 ~~ dh34
  dh23 ~~ dh34

  # Fix manifest intercepts to zero
  height_Baseline ~ 0*1
  height_Year_2 ~ 0*1
  height_Year_4 ~ 0*1
  height_Year_6 ~ 0*1

  # Residual variances constrained equal
  height_Baseline ~~ eh*height_Baseline
  height_Year_2 ~~ eh*height_Year_2
  height_Year_4 ~~ eh*height_Year_4
  height_Year_6 ~~ eh*height_Year_6

  # Fix intermediate true score residual variances to zero
  h2 ~~ 0*h2
  h3 ~~ 0*h3
  h4 ~~ 0*h4
'

fit_height <- sem(height_model, data = df_wide, missing = "ml")
```

## Step 2: Univariate LCSM for Weight {.code}

```r
### Fit univariate LCSM for weight
weight_model <- '
  # Latent true scores for weight
  w1 =~ 1*weight_Baseline
  w2 =~ 1*weight_Year_2
  w3 =~ 1*weight_Year_4
  w4 =~ 1*weight_Year_6

  # Latent change scores for weight
  dw12 =~ 1*w2
  dw23 =~ 1*w3
  dw34 =~ 1*w4

  # Autoregressive paths
  w2 ~ 1*w1
  w3 ~ 1*w2
  w4 ~ 1*w3

  # Means
  w1 ~ 1
  dw12 ~ 1
  dw23 ~ 1
  dw34 ~ 1

  # Variances and covariances
  w1 ~~ w1
  dw12 ~~ dw*dw12
  dw23 ~~ dw*dw23
  dw34 ~~ dw*dw34
  w1 ~~ dw12
  w1 ~~ dw23
  w1 ~~ dw34
  dw12 ~~ dw23
  dw12 ~~ dw34
  dw23 ~~ dw34

  # Fix manifest intercepts to zero
  weight_Baseline ~ 0*1
  weight_Year_2 ~ 0*1
  weight_Year_4 ~ 0*1
  weight_Year_6 ~ 0*1

  # Residual variances constrained equal
  weight_Baseline ~~ ew*weight_Baseline
  weight_Year_2 ~~ ew*weight_Year_2
  weight_Year_4 ~~ ew*weight_Year_4
  weight_Year_6 ~~ ew*weight_Year_6

  # Fix intermediate true score residual variances to zero
  w2 ~~ 0*w2
  w3 ~~ 0*w3
  w4 ~~ 0*w4
'

fit_weight <- sem(weight_model, data = df_wide, missing = "ml")
```

## Step 3: Bivariate LCSM with Cross-Domain Coupling {.code}

```r
### Bivariate LCSM: coupled height and weight change
bivariate_model <- '
  # --- Height Process ---
  h1 =~ 1*height_Baseline
  h2 =~ 1*height_Year_2
  h3 =~ 1*height_Year_4
  h4 =~ 1*height_Year_6

  dh12 =~ 1*h2
  dh23 =~ 1*h3
  dh34 =~ 1*h4

  h2 ~ 1*h1
  h3 ~ 1*h2
  h4 ~ 1*h3

  h1 ~ 1
  dh12 ~ 1
  dh23 ~ 1
  dh34 ~ 1

  h1 ~~ h1
  dh12 ~~ dh*dh12
  dh23 ~~ dh*dh23
  dh34 ~~ dh*dh34
  h1 ~~ dh12
  h1 ~~ dh23
  h1 ~~ dh34
  dh12 ~~ dh23
  dh12 ~~ dh34
  dh23 ~~ dh34

  height_Baseline ~ 0*1
  height_Year_2 ~ 0*1
  height_Year_4 ~ 0*1
  height_Year_6 ~ 0*1

  height_Baseline ~~ eh*height_Baseline
  height_Year_2 ~~ eh*height_Year_2
  height_Year_4 ~~ eh*height_Year_4
  height_Year_6 ~~ eh*height_Year_6

  h2 ~~ 0*h2
  h3 ~~ 0*h3
  h4 ~~ 0*h4

  # --- Weight Process ---
  w1 =~ 1*weight_Baseline
  w2 =~ 1*weight_Year_2
  w3 =~ 1*weight_Year_4
  w4 =~ 1*weight_Year_6

  dw12 =~ 1*w2
  dw23 =~ 1*w3
  dw34 =~ 1*w4

  w2 ~ 1*w1
  w3 ~ 1*w2
  w4 ~ 1*w3

  w1 ~ 1
  dw12 ~ 1
  dw23 ~ 1
  dw34 ~ 1

  w1 ~~ w1
  dw12 ~~ dw*dw12
  dw23 ~~ dw*dw23
  dw34 ~~ dw*dw34
  w1 ~~ dw12
  w1 ~~ dw23
  w1 ~~ dw34
  dw12 ~~ dw23
  dw12 ~~ dw34
  dw23 ~~ dw34

  weight_Baseline ~ 0*1
  weight_Year_2 ~ 0*1
  weight_Year_4 ~ 0*1
  weight_Year_6 ~ 0*1

  weight_Baseline ~~ ew*weight_Baseline
  weight_Year_2 ~~ ew*weight_Year_2
  weight_Year_4 ~~ ew*weight_Year_4
  weight_Year_6 ~~ ew*weight_Year_6

  w2 ~~ 0*w2
  w3 ~~ 0*w3
  w4 ~~ 0*w4

  # --- Cross-Domain Coupling ---
  # Does height level predict subsequent weight change?
  dw12 ~ gamma_hw*h1
  dw23 ~ gamma_hw*h2
  dw34 ~ gamma_hw*h3

  # Does weight level predict subsequent height change?
  dh12 ~ gamma_wh*w1
  dh23 ~ gamma_wh*w2
  dh34 ~ gamma_wh*w3

  # --- Cross-Domain Covariances ---
  h1 ~~ w1        # Initial status covariance
  dh12 ~~ dw12    # Change covariance period 1
  dh23 ~~ dw23    # Change covariance period 2
  dh34 ~~ dw34    # Change covariance period 3
'

fit_bivariate <- sem(bivariate_model, data = df_wide, missing = "ml")

summary(fit_bivariate, fit.measures = TRUE, standardized = TRUE)
```

## Step 4: Compare Uncoupled vs Coupled Models {.code}

```r
### Fit an uncoupled bivariate model (no cross-domain paths) for comparison
uncoupled_model <- '
  # --- Height Process ---
  h1 =~ 1*height_Baseline
  h2 =~ 1*height_Year_2
  h3 =~ 1*height_Year_4
  h4 =~ 1*height_Year_6

  dh12 =~ 1*h2
  dh23 =~ 1*h3
  dh34 =~ 1*h4

  h2 ~ 1*h1
  h3 ~ 1*h2
  h4 ~ 1*h3

  h1 ~ 1
  dh12 ~ 1
  dh23 ~ 1
  dh34 ~ 1

  h1 ~~ h1
  dh12 ~~ dh*dh12
  dh23 ~~ dh*dh23
  dh34 ~~ dh*dh34
  h1 ~~ dh12
  h1 ~~ dh23
  h1 ~~ dh34
  dh12 ~~ dh23
  dh12 ~~ dh34
  dh23 ~~ dh34

  height_Baseline ~ 0*1
  height_Year_2 ~ 0*1
  height_Year_4 ~ 0*1
  height_Year_6 ~ 0*1

  height_Baseline ~~ eh*height_Baseline
  height_Year_2 ~~ eh*height_Year_2
  height_Year_4 ~~ eh*height_Year_4
  height_Year_6 ~~ eh*height_Year_6

  h2 ~~ 0*h2
  h3 ~~ 0*h3
  h4 ~~ 0*h4

  # --- Weight Process ---
  w1 =~ 1*weight_Baseline
  w2 =~ 1*weight_Year_2
  w3 =~ 1*weight_Year_4
  w4 =~ 1*weight_Year_6

  dw12 =~ 1*w2
  dw23 =~ 1*w3
  dw34 =~ 1*w4

  w2 ~ 1*w1
  w3 ~ 1*w2
  w4 ~ 1*w3

  w1 ~ 1
  dw12 ~ 1
  dw23 ~ 1
  dw34 ~ 1

  w1 ~~ w1
  dw12 ~~ dw*dw12
  dw23 ~~ dw*dw23
  dw34 ~~ dw*dw34
  w1 ~~ dw12
  w1 ~~ dw23
  w1 ~~ dw34
  dw12 ~~ dw23
  dw12 ~~ dw34
  dw23 ~~ dw34

  weight_Baseline ~ 0*1
  weight_Year_2 ~ 0*1
  weight_Year_4 ~ 0*1
  weight_Year_6 ~ 0*1

  weight_Baseline ~~ ew*weight_Baseline
  weight_Year_2 ~~ ew*weight_Year_2
  weight_Year_4 ~~ ew*weight_Year_4
  weight_Year_6 ~~ ew*weight_Year_6

  w2 ~~ 0*w2
  w3 ~~ 0*w3
  w4 ~~ 0*w4

  # --- Cross-Domain Covariances (but no coupling) ---
  h1 ~~ w1
  dh12 ~~ dw12
  dh23 ~~ dw23
  dh34 ~~ dw34
'

fit_uncoupled <- sem(uncoupled_model, data = df_wide, missing = "ml")

### Format comparison table
comparison_table <- data.frame(
  Model = c("Uncoupled", "Coupled"),
  df = c(fitMeasures(fit_uncoupled, "df"),
         fitMeasures(fit_bivariate, "df")),
  AIC = c(AIC(fit_uncoupled), AIC(fit_bivariate)),
  BIC = c(BIC(fit_uncoupled), BIC(fit_bivariate)),
  CFI = c(fitMeasures(fit_uncoupled, "cfi"),
          fitMeasures(fit_bivariate, "cfi")),
  RMSEA = c(fitMeasures(fit_uncoupled, "rmsea"),
            fitMeasures(fit_bivariate, "rmsea"))
) %>%
  gt() %>%
  tab_header(title = "Model Comparison: Uncoupled vs. Coupled BLCSM") %>%
  fmt_number(columns = c(AIC, BIC, CFI, RMSEA), decimals = 3) %>%
  fmt_number(columns = df, decimals = 0)

gt::gtsave(comparison_table, filename = "model_comparison.html")
comparison_table
```

## Model Comparison Output {.output}

/stage4-artifacts/lcsm-bivariate/model_comparison.html

## Format Coupled Model Results {.code}

```r
### Extract and format key parameters from the coupled model
coupled_params <- broom::tidy(fit_bivariate) %>%
  filter(op %in% c("~1", "~~", "~")) %>%
  select(term, estimate, std.error, statistic, p.value) %>%
  gt() %>%
  tab_header(title = "Bivariate Latent Change Score Model: Coupled Results") %>%
  fmt_number(columns = c(estimate, std.error, statistic, p.value), decimals = 3)

gt::gtsave(coupled_params, filename = "model_summary.html")

### Extract fit indices
fit_indices <- fitMeasures(fit_bivariate,
  c("chisq", "df", "pvalue", "cfi", "tli", "rmsea", "srmr", "aic", "bic"))

fit_indices_table <- data.frame(
  Metric = names(fit_indices),
  Value = as.numeric(fit_indices)
) %>%
  gt() %>%
  tab_header(title = "Model Fit Indices: Coupled BLCSM") %>%
  fmt_number(columns = Value, decimals = 3) %>%
  cols_label(Metric = "Fit Measure", Value = "Value")

gt::gtsave(fit_indices_table, filename = "model_fit_indices.html")
```

## Model Summary Output {.output}

/stage4-artifacts/lcsm-bivariate/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lcsm-bivariate/model_fit_indices.html

## Interpretation {.note}

The coupled bivariate model (CFI = 0.972, df = 6) improved substantially over the uncoupled specification (AIC = 209,335 vs. 209,587), confirming that cross-domain coupling dynamics are present beyond what concurrent covariances capture. Mean height rose from 55.4 cm at Baseline through gains of 6.9, 7.4, and 5.2 cm across successive periods. Mean weight rose from 81.9 lbs through gains of 24.7, 30.4, and 14.0 lbs — both domains show decelerating growth in the final period.

The **coupling parameters** reveal asymmetric cross-domain dynamics. Height positively predicted subsequent weight change (gamma_hw = 0.001, p < .001): taller youth gained slightly more weight in the next period. Weight negatively predicted subsequent height change (gamma_wh = -0.025, p < .001): heavier youth grew less in height, consistent with the idea that greater body mass may constrain further linear growth. The asymmetry — height facilitating weight gain but weight constraining height growth — highlights distinct developmental mechanisms.

**Cross-domain covariances** were positive throughout: initial height and weight correlated strongly (55.6, p < .001), and within-period change covariances ranged from 60.3 to 72.9 (all p < .001), confirming coordinated physical development. Note that the within-domain equality constraints (equal change and residual variances) produced some boundary estimates for height, suggesting that future analyses might benefit from relaxing these constraints with additional time points.

## Visualization {.code}

```r
### Visualize coupled change across both constructs
df_wide <- df_wide %>%
  mutate(
    height_change_12 = height_Year_2 - height_Baseline,
    height_change_23 = height_Year_4 - height_Year_2,
    height_change_34 = height_Year_6 - height_Year_4,
    weight_change_12 = weight_Year_2 - weight_Baseline,
    weight_change_23 = weight_Year_4 - weight_Year_2,
    weight_change_34 = weight_Year_6 - weight_Year_4
  )

### Scatter plots of coupled changes across three periods
plot_data <- bind_rows(
  df_wide %>% select(h = height_change_12, w = weight_change_12) %>%
    mutate(Period = "Baseline to Year 2"),
  df_wide %>% select(h = height_change_23, w = weight_change_23) %>%
    mutate(Period = "Year 2 to Year 4"),
  df_wide %>% select(h = height_change_34, w = weight_change_34) %>%
    mutate(Period = "Year 4 to Year 6")
) %>%
  mutate(Period = factor(Period, levels = c("Baseline to Year 2",
                                             "Year 2 to Year 4",
                                             "Year 4 to Year 6")))

coupled_plot <- ggplot(plot_data, aes(x = h, y = w)) +
  geom_point(alpha = 0.10, size = 0.6, color = "#4A90D9") +
  geom_smooth(method = "lm", se = TRUE, color = "#E74C3C", linewidth = 1) +
  facet_wrap(~Period) +
  labs(
    title = "Coupled Change: Height vs. Weight Across Assessment Periods",
    subtitle = "Bivariate LCSM — Four Time Points",
    x = "Height Change (in)",
    y = "Weight Change (lbs)"
  ) +
  theme_minimal()

ggsave(
  filename = "visualization.png",
  plot = coupled_plot,
  width = 12, height = 5, dpi = 300
)
```

## Visualization {.output}

![Coupled Change Scatter Plot](stage4-artifacts/lcsm-bivariate/visualization.png)

## Visualization Notes {.note}

The scatter plots display the association between observed height change and weight change across three successive assessment periods. The regression lines summarize the direction and strength of the concurrent association within each period. A positive slope indicates that youth who grew more in height also tended to gain more weight, consistent with coordinated physical development. Comparing across panels reveals whether the height-weight coupling strengthens or weakens over developmental time. Note that these are observed associations — the BLCSM coupling parameters provide the latent (measurement-error-adjusted) estimates of cross-domain predictive relationships that operate with a temporal lag, which is a stronger test of directional influence than concurrent correlation.

# Discussion

The Bivariate Latent Change Score Model extends the univariate LCSM to address a fundamental question in developmental research: how do changes in one domain relate to changes in another? By estimating cross-domain coupling parameters, the BLCSM tests whether the level of one construct predicts subsequent change in the other, providing evidence for temporal precedence that goes beyond simple concurrent correlations.

The model comparison between uncoupled and coupled specifications provides a formal test of whether cross-domain dynamics improve model fit beyond what is captured by concurrent covariances alone. When coupling parameters are significant, they suggest that knowing a youth's height (or weight) at one time point improves prediction of their weight (or height) change in the next period. Asymmetric coupling — where one direction is significant but not the other — is particularly informative, as it identifies which developmental process leads and which follows.

Using four time points with equality constraints (equal change variances and residual variances within each domain) produces a properly identified bivariate model with testable fit indices. Important limitations include the assumption that coupling effects are constant across time periods, that within-domain change variance is homogeneous, and that the measurement model assumes no systematic method effects. Extensions include models with proportional effects (change depending on own prior level in addition to the other domain), time-varying coupling strengths, and three-process models that incorporate a third developmental domain.

# Additional Resources

### McArdle (2009): Latent Variable Modeling of Differences and Changes {.resource}

Foundational paper introducing the latent change score framework and its bivariate extension for modeling coupled developmental processes across multiple domains.

**Badge:** PAPER
**URL:** https://doi.org/10.1146/annurev.psych.60.110707.163612

### Grimm, Ram & Estabrook: Growth Modeling {.resource}

Textbook covering structural equation modeling approaches to longitudinal data, with detailed chapters on bivariate latent change score models including coupled dynamics and R code.

**Badge:** BOOK
**URL:** https://www.guilford.com/books/Growth-Modeling/Grimm-Ram-Estabrook/9781462526062

### Kievit et al. (2018): Mutualism in Cognitive Development {.resource}

Applied example of bivariate latent change score models demonstrating coupled cognitive development, illustrating how BLCSM can reveal mutualistic relationships between developing abilities.

**Badge:** PAPER
**URL:** https://doi.org/10.1177/0956797617710785

### Usami, Murayama & Hamaker (2019): Unified Framework for Longitudinal Models {.resource}

Comprehensive framework paper comparing latent change score models with cross-lagged panel models, clarifying when bivariate LCSM is preferred and how coupling parameters relate to other approaches for studying reciprocal developmental processes.

**Badge:** PAPER
**URL:** https://doi.org/10.1037/met0000210
