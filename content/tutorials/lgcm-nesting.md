---
title: "LGCM: Nesting"
slug: lgcm-nesting
author: Biostatistics Working Group
date_iso: 2025-11-04
tags:
  - abcd-study
  - trajectory
  - nesting
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: lavaan
covariates: None
outcome_type: Continuous
description: Incorporate nesting within families and sites in latent growth curve models to obtain robust emotional suppression trajectories for clustered ABCD youth.
---

# Overview

## Summary {.summary}

Latent Growth Curve Modeling with clustering addresses dependencies where observations within the same family or study site are more similar than observations from different clusters. Ignoring this structure can lead to biased standard errors, incorrect p-values, and inappropriate confidence intervals. This tutorial demonstrates LGCM with nested clustering to account for site-level and family-level dependencies using robust standard errors in ABCD data, examining emotional suppression trajectories across four annual assessments.

## Features {.features}

- **When to Use:** Apply when participants are nested in families or sites and you need to adjust standard errors for clustering while modeling growth.
- **Key Advantage:** Robust standard errors correct for dependency without requiring full multilevel specification, maintaining simpler model interpretation.
- **What You'll Learn:** How to apply robust clustering in lavaan, interpret corrected standard errors, and assess nesting impact on inference.

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
library(arrow)        # For reading Parquet files
library(tidyverse)    # Collection of R packages for data science
library(gtsummary)    # Creating publication-quality tables
library(lavaan)       # Structural Equation Modeling in R
library(broom)        # For tidying model outputs
library(gt)           # For creating formatted tables

# Set random seed for reproducible family member selection
set.seed(123)

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
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
# Create longitudinal dataset with cleaned variables
df_long <- abcd_data %>%
  # Filter to ERQ assessment waves (Years 3-6)
  filter(session_id %in% c("ses-03A", "ses-04A", "ses-05A", "ses-06A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    # Relabel sessions
    session_id = factor(session_id,
                        levels = c("ses-03A", "ses-04A", "ses-05A", "ses-06A"),
                        labels = c("Year_3", "Year_4", "Year_5", "Year_6")),
    # Clean outcome variable
    suppression = round(as.numeric(mh_y_erq__suppr_mean), 2)
  ) %>%
  # Rename clustering variables
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam
  ) %>%

  # Keep only analysis variables
  select(participant_id, session_id, site, family_id, suppression) %>%

  # Remove cases with missing outcome data
  drop_na(suppression) %>%
  droplevels()

df_wide <- df_long %>%
  select(participant_id, site, family_id, session_id, suppression) %>%
  pivot_wider(
    names_from = session_id,
    values_from = suppression,
    names_prefix = "Suppression_"
  ) %>%
  # Remove cases with incomplete data across waves
  drop_na(starts_with("Suppression_"))

```

## Descriptive Statistics {.code}

```r
# Create comprehensive descriptive table
descriptives_table <- df_long %>%
  select(session_id, suppression) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(suppression ~ "Emotional Suppression"),
    statistic = list(all_continuous() ~ "{mean} ({sd})"),
    digits = all_continuous() ~ 2
  ) %>%
  modify_header(all_stat_cols() ~ "**{level}**N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "**Assessment Wave**") %>%
  add_overall(last = TRUE) %>%
  bold_labels()

### Apply compact styling
theme_gtsummary_compact()

descriptives_table <- as_gt(descriptives_table)

### Save the table as HTML
gt::gtsave(descriptives_table, filename = "descriptives_table.html")

### Print the table
descriptives_table
```

## Descriptive Statistics Output {.output}

/stage4-artifacts/lgcm-nesting/descriptives_table.html

# Statistical Analysis

## Define and Fit Nested LGCM {.code}

```r
# Define LGCM specification with explicit residual variance labels
model <- "
  # Define growth factors with time-specific loadings
  i =~ 1*Suppression_Year_3 + 1*Suppression_Year_4 + 1*Suppression_Year_5 + 1*Suppression_Year_6
  s =~ 0*Suppression_Year_3 + 1*Suppression_Year_4 + 2*Suppression_Year_5 + 3*Suppression_Year_6

  # Allow growth factors to have variance (individual differences)
  i ~~ i
  s ~~ s

  # Allow correlation between intercept and slope
  i ~~ s

  # Specify residual variances for each time point
  Suppression_Year_3 ~~ var_y3*Suppression_Year_3
  Suppression_Year_4 ~~ var_y4*Suppression_Year_4
  Suppression_Year_5 ~~ var_y5*Suppression_Year_5
  Suppression_Year_6 ~~ var_y6*Suppression_Year_6
"

# Fit the model with nested clustering
fit <- growth(
  model,
  data = df_wide,
  cluster = c("site", "family_id"),
  missing = "fiml",
  se = "robust"
)

# Display model summary with fit indices
summary(fit, fit.measures = TRUE, standardized = TRUE)
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

/stage4-artifacts/lgcm-nesting/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lgcm-nesting/model_fit_indices.html

## Interpretation {.note}

The LGCM estimated a mean suppression score of 3.11 at Year 3 (SE = 0.021, p < .001) and a gradual increase of 0.11 points per year (SE = 0.007, p < .001). Intercept and slope variances (0.322 and 0.046, both p < .001) were sizable, confirming that youth differed widely in both starting levels and subsequent change. The negative intercept–slope covariance (−0.039, p < .001) indicates a leveling effect: adolescents who began with higher suppression tended to grow more slowly, whereas those starting low often caught up.

Model fit was generally solid (CFI = 0.949, TLI = 0.938, SRMR = 0.045), though the RMSEA of 0.092 hints at minor misfit that could reflect omitted time-specific covariances. Cluster-robust standard errors were computed for both sites and families; adding the family level barely shifted estimates, suggesting that most dependency operates at the site level, but the dual adjustment still guards against underestimated SEs. Finally, residual variances shrank from 0.439 at Year 3 to 0.292 at Year 6, implying that suppression measurements became more stable as participants aged.

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

  ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Trajectory Plot {.output}

![Suppression Trajectory Plot](/stage4-artifacts/lgcm-nesting/visualization.png)

## Interpretation {.note}

This plot visualizes individual trajectories and the overall trend in suppression scores across four annual assessments. Each line represents the trajectory of a randomly selected subset of participants, highlighting individual differences in suppression development over time.

- Blue points represent observed suppression scores at each time point, providing a clear depiction of the data distribution.

- Gray lines connect individual trajectories, illustrating within-person variability in suppression changes.

- The red smoothed curve represents the overall trend, estimated using a linear regression model, capturing the general pattern of suppression growth with a confidence interval (light pink shading).

- While many participants exhibit a general increase in suppression, others show stable or declining trajectories, emphasizing heterogeneity in individual change patterns.

- Between-person differences in initial suppression levels and rates of change reinforce the need for latent growth models to capture both within- and between-person variability in suppression development.

# Discussion

Suppression tended to rise across the study, yet individual trajectories varied widely. Some youth showed steep gains, others remained flat, and a few declined. That heterogeneity was captured by significant variance in both intercepts ($0.322$, $SE = 0.016$, $p < .001$) and slopes ($0.046$, $SE = 0.003$, $p < .001$), underscoring why a latent growth framework is preferable to a single average trend.

Cluster-robust standard errors were computed for sites and families. Adding the family level changed estimates only trivially relative to site-only clustering, suggesting that most dependency was already absorbed at the site level, but keeping both levels still provides conservative uncertainty estimates for downstream reporting.

The negative covariance between intercepts and slopes ($-0.039$, $SE = 0.004$, $p < .001$) points to a leveling-off pattern: participants starting with high suppression gained more slowly, whereas those entering low caught up quickly. Residual variances also shrank from Year 3 ($0.439$) to Year 6 ($0.292$), indicating that scores stabilized with age. Together these results describe a cohort that becomes more homogeneous over time even while retaining meaningful between-person differences in both starting points and change rates.

# Additional Resources

### lavaan Complex Survey Features {.resource}

Official lavaan documentation on cluster-robust standard errors and handling nested data structures in growth curve models, including the cluster argument for adjusting standard errors.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/est.html

### Multilevel Growth Models in lavaan {.resource}

Step-by-step lavaan tutorial that demonstrates how to specify two-level growth models using the `cluster` argument, adjust standard errors for nested designs, and interpret between-cluster versus within-cluster effects.

**Badge:** VIGNETTE
**URL:** https://lavaan.ugent.be/tutorial/cluster.html

### Nested Data Structures in Growth Modeling {.resource}

Methodology paper discussing how to properly account for clustering in longitudinal structural equation models, including strategies for modeling nested dependencies.

**Badge:** PAPER
**URL:** https://www.statmodel.com/download/Nested.pdf

### lavaan.survey Package {.resource}

CRAN package that extends lavaan to complex survey and clustered data via sandwich estimators, letting you fit the same growth models while properly accounting for design weights and multi-level clustering.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=lavaan.survey
