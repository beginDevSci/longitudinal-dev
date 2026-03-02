---
title: "LCSM: Basic"
slug: lcsm-basic
author: Biostatistics Working Group
date_iso: 2025-11-06
tags:
  - abcd-study
  - latent-change
  - structural-equation-modeling
family: LCSM
family_label: Latent Change Score Models (LCSM)
engine: lavaan
engines:
  - lavaan
covariates: None
outcome_type: Continuous
difficulty: intermediate
timepoints: 3_5
summary: Model true change between measurement occasions using a latent change score model with four time points, separating measurement error from systematic developmental change in internalizing problems across ABCD youth.
description: Model true change between measurement occasions using a latent change score model with four time points, separating measurement error from systematic developmental change in internalizing problems across ABCD youth.
---

# Overview

## Summary {.summary}

Latent Change Score Models (LCSM) provide a framework for modeling change between measurement occasions by treating change as a latent variable rather than a simple observed difference. Unlike raw difference scores, LCSM separates true change from measurement error, allowing researchers to estimate the reliability of change, model proportional effects (where change depends on prior status), and correlate change with other variables. This tutorial applies LCSM with four time points (Baseline, Year 2, Year 4, Year 6) to analyze changes in CBCL Internalizing T-scores in ABCD youth, demonstrating how to specify and interpret basic LCSM parameters including initial status, change scores, and their variances and covariances.

## Features {.features}

- **When to Use:** Apply when you want to model change as an explicit latent construct with four or more time points, especially when measurement error is a concern or when you need a properly identified model with testable fit indices.
- **Key Advantage:** LCSM separates true score variance from error variance in the change score, providing more reliable estimates of individual differences in change than simple difference scores.
- **What You'll Learn:** How to specify a basic LCSM in lavaan, interpret the mean and variance of latent change, assess whether initial status predicts subsequent change, and evaluate model fit.

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
    "mh_p_cbcl__synd__int_tscore"
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
# Uses 4 time points (Baseline, Year 2, Year 4, Year 6) for model identification
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site,
         ab_g_stc__design_id__fam, mh_p_cbcl__synd__int_tscore) %>%
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
    internalizing = mh_p_cbcl__synd__int_tscore
  ) %>%
  droplevels() %>%
  drop_na(internalizing)

### Reshape data from long to wide format for LCSM
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = internalizing,
    names_prefix = "Int_"
  ) %>%
  drop_na(starts_with("Int_"))  # Require complete data across time points
```

## Descriptive Statistics {.code}

```r
### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, internalizing) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      internalizing ~ "Internalizing (T-score)"
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

/stage4-artifacts/lcsm-basic/descriptives_table.html

# Statistical Analysis

## Define and Fit Basic LCSM {.code}

```r
# Define the Latent Change Score Model
# Parsimonious specification: equal change variances, equal residual variances,
# initial-to-first-change covariance only (all other latent covariances zero)
model <- '
  # Define latent true scores at each time point
  eta1 =~ 1*Int_Baseline
  eta2 =~ 1*Int_Year_2
  eta3 =~ 1*Int_Year_4
  eta4 =~ 1*Int_Year_6

  # Define latent change scores
  delta12 =~ 1*eta2
  delta23 =~ 1*eta3
  delta34 =~ 1*eta4

  # Autoregressive paths (carryover from prior true score)
  eta2 ~ 1*eta1
  eta3 ~ 1*eta2
  eta4 ~ 1*eta3

  # Means of latent variables
  eta1 ~ 1           # Mean of initial status
  delta12 ~ 1        # Mean change T1 to T2
  delta23 ~ 1        # Mean change T2 to T3
  delta34 ~ 1        # Mean change T3 to T4

  # Variances
  eta1 ~~ eta1               # Variance of initial status
  delta12 ~~ d*delta12       # Change variances constrained equal
  delta23 ~~ d*delta23
  delta34 ~~ d*delta34

  # Initial status to first change covariance only
  eta1 ~~ delta12

  # Fix all other latent covariances to zero
  eta1 ~~ 0*delta23
  eta1 ~~ 0*delta34
  delta12 ~~ 0*delta23
  delta12 ~~ 0*delta34
  delta23 ~~ 0*delta34

  # Residual variances constrained equal across time points
  Int_Baseline ~~ e*Int_Baseline
  Int_Year_2 ~~ e*Int_Year_2
  Int_Year_4 ~~ e*Int_Year_4
  Int_Year_6 ~~ e*Int_Year_6

  # Fix manifest intercepts to zero (latent means carry the mean structure)
  Int_Baseline ~ 0*1
  Int_Year_2 ~ 0*1
  Int_Year_4 ~ 0*1
  Int_Year_6 ~ 0*1

  # Fix intermediate true score residual variances to zero
  eta2 ~~ 0*eta2
  eta3 ~~ 0*eta3
  eta4 ~~ 0*eta4
'

# Fit the LCSM model
fit <- sem(model, data = df_wide, missing = "ml")

# Display model summary
summary(fit, fit.measures = TRUE, standardized = TRUE)
```

## Format Model Summary Table {.code}

```r
# Extract model summary
model_summary <- summary(fit)

# Convert lavaan output to a tidy dataframe and then to gt table
model_summary_table <- broom::tidy(fit) %>%
  filter(op %in% c("~1", "~~", "~")) %>%  # Focus on key parameters
  select(term, estimate, std.error, statistic, p.value) %>%
  gt() %>%
  tab_header(title = "Latent Change Score Model Results") %>%
  fmt_number(columns = c(estimate, std.error, statistic, p.value), decimals = 3)

# Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)
```

## Format Model Fit Indices Table {.code}

```r
# Extract and save model fit indices
fit_indices <- fitMeasures(fit, c("chisq", "df", "pvalue", "cfi", "tli", "rmsea", "srmr", "aic", "bic"))

fit_indices_table <- data.frame(
  Metric = names(fit_indices),
  Value = as.numeric(fit_indices)
) %>%
  gt() %>%
  tab_header(title = "Model Fit Indices") %>%
  fmt_number(columns = Value, decimals = 3) %>%
  cols_label(
    Metric = "Fit Measure",
    Value = "Value"
  )

# Save fit indices table
gt::gtsave(
  data = fit_indices_table,
  filename = "model_fit_indices.html",
  inline_css = FALSE
)
```

## Model Summary Output {.output}

/stage4-artifacts/lcsm-basic/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lcsm-basic/model_fit_indices.html

## Interpretation {.note}

The parsimonious LCSM fit the data well (CFI = 0.985, TLI = 0.985, RMSEA = 0.065, SRMR = 0.052, df = 6), indicating that the equality constraints on change variances and residual variances are appropriate for CBCL Internalizing T-scores across biennial intervals. This is a substantial improvement over physical growth measures, which tend to have such high adjacent-wave stability that latent change score models struggle with identification.

Mean internalizing at baseline was 48.32 (SE = 0.155, p < .001), close to the normed T-score mean of 50. The mean latent change scores were -0.72 (Baseline to Year 2, p < .001), -0.09 (Year 2 to Year 4, p = .515), and -0.40 (Year 4 to Year 6, p = .003), indicating modest average decreases in internalizing problems over time, with the largest decline in the first period and a non-significant change in the middle period.

Initial status variance was 74.90 (p < .001), confirming substantial individual differences in baseline internalizing levels. The constrained change score variance was 15.81 (p < .001), indicating meaningful individual differences in biennial change — some youth increased while others decreased in internalizing symptoms within each period. The covariance between initial status and the first change score was negative (-12.17, p < .001), suggesting a compensatory pattern: youth with higher baseline internalizing levels tended to show greater decreases (or smaller increases) in the first period, consistent with regression toward the mean. Residual variance was 33.78 (constrained equal across waves), representing approximately 31% of observed variance — reasonable measurement error for a parent-reported broadband syndrome scale.

## Visualization {.code}

```r
### Create visualization of change score distribution
df_wide <- df_wide %>%
  mutate(
    observed_change_12 = Int_Year_2 - Int_Baseline,
    observed_change_23 = Int_Year_4 - Int_Year_2,
    observed_change_34 = Int_Year_6 - Int_Year_4
  )

### Plot distribution of change scores across three periods
change_plot <- df_wide %>%
  select(observed_change_12, observed_change_23, observed_change_34) %>%
  pivot_longer(cols = everything(), names_to = "Period", values_to = "Change") %>%
  mutate(Period = factor(Period,
                         levels = c("observed_change_12", "observed_change_23",
                                    "observed_change_34"),
                         labels = c("Baseline to Year 2", "Year 2 to Year 4",
                                    "Year 4 to Year 6"))) %>%
  ggplot(aes(x = Change, fill = Period)) +
  geom_histogram(bins = 30, alpha = 0.7, position = "identity") +
  facet_wrap(~Period, scales = "free_y") +
  labs(
    title = "Distribution of Internalizing Change Across Assessment Periods",
    subtitle = "Basic LCSM — Four Time Points (Biennial Intervals)",
    x = "Internalizing Change (T-score)",
    y = "Count"
  ) +
  theme_minimal() +
  theme(legend.position = "none")

ggsave(
  filename = "visualization.png",
  plot = change_plot,
  width = 10, height = 5, dpi = 300
)
```

## Visualization {.output}

![Internalizing Change Distribution](stage4-artifacts/lcsm-basic/visualization.png)

## Visualization Notes {.note}

The histograms show the distribution of observed internalizing changes across the three biennial assessment periods. These are raw difference scores — the LCSM estimates are adjusted for measurement error and thus provide more reliable estimates of true change variance. The spread of each distribution reflects individual differences in change rates: a wider distribution indicates greater heterogeneity. Comparing across periods reveals whether change patterns are stable or shifting over time. Note that CBCL T-scores are normed with a mean of 50, so changes centered near zero indicate stable average levels, while systematic shifts suggest developmental trends in internalizing symptoms.

# Discussion

The Latent Change Score Model provides several advantages over simple difference scores for analyzing developmental change. By modeling change as a latent variable, LCSM separates true change from measurement error, yielding more reliable estimates of both average change and individual differences in change. This is particularly important for behavioral measures like the CBCL, where measurement error can attenuate difference score reliability.

This tutorial uses a parsimonious constraint strategy designed for stable estimation with four time points: change variances and residual variances are each constrained equal across periods, and the only within-domain covariance estimated is between initial status and the first change score. This provides adequate degrees of freedom for formal goodness-of-fit testing while avoiding the Heywood cases (negative variance estimates) that can arise with more complex covariance structures when inter-wave stability is high. The zero-covariance constraints on non-adjacent latent variables are substantively motivated — they assume that once initial status is accounted for, change scores in non-adjacent periods are conditionally independent.

The covariance between initial status and the first change score tests whether development follows a compensatory or cumulative pattern: a negative covariance suggests that youth with higher initial internalizing levels show less increase (or more decrease) in symptoms, while a positive covariance would indicate cumulative risk. This parameter is particularly informative for understanding whether regression toward the mean operates in internalizing symptom trajectories. Extensions include proportional effects (where change depends on current level), bivariate models examining coupled change across internalizing and externalizing domains (see the Bivariate LCSM tutorial), and piecewise specifications allowing change rates to differ across developmental periods.

# Additional Resources

### lavaan Tutorial on Latent Change Score Models {.resource}

Comprehensive guide to specifying LCSM and related longitudinal models in lavaan, including univariate and bivariate change score models with worked examples.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/growth.html

### McArdle (2009): Latent Variable Modeling of Differences and Changes {.resource}

Foundational paper by John McArdle introducing the latent change score framework, explaining the mathematical basis and advantages over traditional difference score approaches.

**Badge:** PAPER
**URL:** https://doi.org/10.1146/annurev.psych.60.110707.163612

### Grimm, Ram & Estabrook: Growth Modeling {.resource}

Textbook covering structural equation modeling approaches to longitudinal data, with detailed chapters on latent change score models, including R code examples.

**Badge:** BOOK
**URL:** https://www.guilford.com/books/Growth-Modeling/Grimm-Ram-Estabrook/9781462526062

### semPlot Package for Model Visualization {.resource}

R package for creating publication-quality path diagrams of structural equation models including LCSM, helping to visualize the model structure and parameter estimates.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=semPlot
