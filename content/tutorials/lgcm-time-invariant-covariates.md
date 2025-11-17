---
title: "LGCM: Time-Invariant Covariates"
slug: lgcm-time-invariant-covariates
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - latent-growth-curve-model
  - time-invariant-covariate
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: lavaan
covariates: TIC
outcome_type: Continuous
description: Add time-invariant covariates to latent growth curve models to evaluate how stable characteristics shift baseline levels and slopes of ABCD emotional suppression.
---

# Overview

## Summary {.summary}

Latent Growth Curve Modeling with time-invariant covariates extends basic growth modeling by explaining why individuals differ in initial levels and rates of change. By incorporating predictors like demographics or socioeconomic factors as covariates of latent intercept and slope factors, this approach reveals how stable characteristics shape developmental trajectories. This tutorial examines emotional suppression in ABCD youth across four annual assessments, modeling how demographic and socioeconomic factors predict both baseline suppression levels and individual rates of change over time.

## Features {.features}

- **When to Use:** Use when baseline demographics or socioeconomic factors may predict both initial levels and individual growth trajectories.
- **Key Advantage:** Integrates covariates directly into the latent intercept and slope, revealing how each covariate shifts starting points and trajectories simultaneously.
- **What You'll Learn:** How to add covariates to the LGCM, interpret their effects on intercept/slope factors, and evaluate whether covariates improve overall model fit.

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
# Load required libraries
library(NBDCtools)    # ABCD data access helper
library(arrow)        # For reading Parquet files
library(tidyverse)    # Data manipulation
library(lavaan)       # Latent growth curve modeling
library(gtsummary)    # Publication-ready tables
library(gt)           # Table formatting
library(corrplot)     # Correlation visualization

# Set random seed for reproducible family member selection
set.seed(123)

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "ab_g_dyn__visit_age",
    "ab_g_stc__cohort_sex",
    "ab_g_stc__cohort_race__nih",
    "ab_g_dyn__cohort_edu__cgs",
    "ab_g_dyn__cohort_income__hhold__3lvl",
    "mh_y_erq__suppr_mean"
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
# Create longitudinal dataset
df_long <- abcd_data %>%
  # Filter to ERQ assessment waves (Years 3-6)
  filter(session_id %in% c("ses-03A", "ses-04A", "ses-05A", "ses-06A")) %>%
  arrange(participant_id, session_id)

# Clean and transform variables
df_long <- df_long %>%
  mutate(
    participant_id = factor(participant_id),
    session_id = factor(session_id,
                       levels = c("ses-03A", "ses-04A", "ses-05A", "ses-06A"),
                       labels = c("Year_3", "Year_4", "Year_5", "Year_6")),
    site = factor(ab_g_dyn__design_site),
    family_id = factor(ab_g_stc__design_id__fam),
    age = as.numeric(ab_g_dyn__visit_age),
    sex = factor(ab_g_stc__cohort_sex,
                 levels = c("1", "2"),
                 labels = c("Male", "Female")),
    race = factor(ab_g_stc__cohort_race__nih,
                  levels = c("2", "3", "4", "5", "6", "7", "8"),
                  labels = c("White", "Black", "Asian", "AI/AN", "NH/PI", "Multi-Race", "Other")),
    education = as.numeric(ab_g_dyn__cohort_edu__cgs),
    income = as.numeric(ab_g_dyn__cohort_income__hhold__3lvl),
    suppression = round(as.numeric(mh_y_erq__suppr_mean), 2)
  ) %>%
  # Select analysis variables
  select(participant_id, session_id, site, family_id, age, sex, race, education, income, suppression) %>%
  drop_na()

# Get baseline covariates (Year 3)
baseline_covariates <- df_long %>%
  filter(session_id == "Year_3") %>%
  select(participant_id, age, sex, education, income) %>%
  mutate(
    age_c = age - mean(age, na.rm = TRUE),
    female = ifelse(sex == "Female", 1, 0),
    education_c = education - mean(education, na.rm = TRUE),
    income_c = income - mean(income, na.rm = TRUE)
  ) %>%
  select(participant_id, age_c, female, education_c, income_c)
```

## Reshape to Wide Format {.code}

```r
# Reshape suppression to wide format
df_wide <- df_long %>%
  select(participant_id, session_id, suppression, site) %>%
  pivot_wider(
    names_from = session_id,
    values_from = suppression,
    names_prefix = "Suppression_"
  ) %>%
  # Merge with baseline predictors
  left_join(baseline_covariates, by = "participant_id") %>%
  drop_na()
```

## Descriptive Statistics {.code}

```r
# Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, suppression) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      suppression ~ "Suppression"
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

/stage4-artifacts/lgcm-time-invariant-covariates/descriptives_table.html

# Statistical Analysis

## Define and Fit LGCM with Covariates {.code}

```r
# Specify LGCM with time-invariant covariates
model <- "
  # Define growth factors
  i =~ 1*Suppression_Year_3 + 1*Suppression_Year_4 +
       1*Suppression_Year_5 + 1*Suppression_Year_6
  s =~ 0*Suppression_Year_3 + 1*Suppression_Year_4 +
       2*Suppression_Year_5 + 3*Suppression_Year_6

  # Estimate factor means (conditional on covariates)
  i ~ 1
  s ~ 1

  # Estimate factor (co)variances (residual after covariates)
  i ~~ i
  s ~~ s
  i ~~ s

  # Equal residual variances (can be relaxed if needed)
  Suppression_Year_3 ~~ res_var*Suppression_Year_3
  Suppression_Year_4 ~~ res_var*Suppression_Year_4
  Suppression_Year_5 ~~ res_var*Suppression_Year_5
  Suppression_Year_6 ~~ res_var*Suppression_Year_6

  # Covariate predictions of growth factors
  i ~ age_c + female + education_c + income_c
  s ~ age_c + female + education_c + income_c
"

# Fit model with cluster-robust standard errors
fit <- lavaan(
  model,
  data = df_wide,
  missing = "fiml",
  cluster = "site"
)

# Display model summary
summary(fit)
```

## Format Model Summary Table {.code}

```r
# Extract model summary
model_summary <- summary(fit)

model_summary

# Convert lavaan output to a tidy dataframe and then to gt table
model_summary_table <- broom::tidy(fit) %>%
  gt() %>%
  tab_header(title = "Latent Growth Curve Model Results") %>%
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

/stage4-artifacts/lgcm-time-invariant-covariates/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lgcm-time-invariant-covariates/model_fit_indices.html

## Interpretation {.note}

Average suppression at Year 3 was 3.07 (SE = 0.019, p < .001) and climbed by 0.134 points per year (SE = 0.010, p < .001). Intercept and slope variances (0.311 and 0.043, both p < .001) confirmed sizable between-person differences, while the negative intercept–slope covariance (−0.034, p < .001) implied that adolescents starting high tended to rise more slowly. Fit indices (CFI = 0.927, TLI = 0.899, SRMR = 0.032, RMSEA = 0.063) indicate the model captures the main longitudinal signal with only minor approximation error.

Time-invariant covariates added interpretable structure. Older youth entered with higher suppression (β = 0.067, p < .001) yet showed flatter growth (β = −0.025, p = .001). Females increased more slowly than males (β = −0.046, p < .001). Lower household income related to slightly higher intercepts and marginally steeper slopes (β = −0.056 and 0.018, p = .032/.054), whereas higher parental education predicted lower starting suppression (β = −0.045, p = .003). Collectively, the results highlight heterogeneous developmental courses shaped by demographic context as well as idiosyncratic factors.

## Visualization {.code}

```r
# Plotting the Suppression data over time from the df_long dataframe
# Select a subset of participants
selected_ids <- sample(unique(df_long$participant_id), 150)
df_long_selected <- df_long %>% filter(participant_id %in% selected_ids)

# Plot Suppression Growth
visualization <- ggplot(df_long_selected, aes(x = session_id, y = suppression, group = participant_id)) +
    geom_line(alpha = 0.3, color = "gray") +
    geom_point(size = 1.5, color = "blue") +
    geom_smooth(aes(group = 1), method = "lm", color = "red", linewidth = 1.2, se = TRUE, fill = "lightpink") +
    labs(
        title = "Suppression Growth with Confidence Intervals",
        x = "Time (Years from Baseline)",
        y = "Suppression Score"
    ) +
    theme_minimal()

print(visualization)

# Save the plot
ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Overall Growth Trajectory](/stage4-artifacts/lgcm-time-invariant-covariates/visualization.png)

## Visualization Notes {.note}

This plot visualizes individual and overall trends in Suppression trajectories across four annual assessments. Each gray line represents the trajectory of a randomly selected subset of participants, illustrating individual variability in suppression changes over time.

The blue points denote observed suppression measurements at each timepoint, providing a direct visualization of data distribution. The red line represents the estimated mean trajectory across all participants, and the shaded confidence band conveys the range of uncertainty around this mean estimate.

The visualization highlights two key findings. The first is the General trend: On average, suppression increases over time, as indicated by the upward slope of the red line. The second is Individual differences: While many participants follow the overall trend, individual trajectories vary, with some showing stability or even decreases in suppression. This plot underscores the importance of modeling both within- and between-person variability.

# Discussion

Suppression generally increased across assessments, yet youth followed noticeably different paths. Significant variance in intercepts and slopes showed that some participants accelerated quickly while others barely changed, making covariate effects essential for interpretation.

Age, sex, and socioeconomic markers all contributed uniquely. Youth who were older at Year 3 entered the model with higher suppression but progressed more slowly thereafter, suggesting a ceiling effect. Males exhibited flatter slopes overall, and higher parental education predicted lower initial suppression, even after accounting for other demographics. These patterns highlight meaningful stratification in both starting points and growth rates.

Allowing random slopes was critical for capturing that heterogeneity. The final model, with both random intercepts and slopes plus time-invariant predictors, cleanly separated average trends from participant-specific deviations. This specification improved fit and yielded interpretable fixed effects, emphasizing that longitudinal analyses benefit from simultaneously modeling covariate influences and the unexplained variability that remains at the individual level.

# Additional Resources

### lavaan Regression Tutorial {.resource}

Official lavaan guide on including predictors of latent growth factors, demonstrating how time-invariant covariates can predict individual differences in intercepts and slopes.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/cov.html

### Growth Models with Covariates {.resource}

Detailed tutorial with practical examples of time-invariant covariate effects on growth parameters, including interpretation of covariate-by-intercept and covariate-by-slope associations.

**Badge:** VIGNETTE
**URL:** https://quantdev.ssri.psu.edu/sites/qdev/files/GMswCovariates.html

### Centering in Growth Models {.resource}

Best practices for centering time-invariant predictors in latent growth curve models. Discusses grand-mean centering versus group-mean centering and their interpretational implications (Enders & Tofighi, 2007). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/2007-10421-007

### broom.mixed for Tidy SEM Output {.resource}

R package for extracting and formatting lavaan results with predictors into clean, publication-ready tables. Particularly useful for models with multiple covariates.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=broom.mixed
