---
title: "LGCM: Basic"
slug: lgcm-basic
author: Biostatistics Working Group
date_iso: 2025-11-04
tags:
  - abcd-study
  - trajectory
  - growth
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: lavaan
covariates: None
outcome_type: Continuous
description: Introduce latent growth curve modeling to estimate average emotional suppression trajectories, growth rates, and individual variability across repeated ABCD assessments.
---

# Overview

## Summary {.summary}

Latentzz Growth Curve Modeling (LGCM) analyzes longitudinal change by estimating growth trajectories as latent factors while distinguishing systematic development from measurement error. Using intercept and slope parameters, LGCM captures both population-average patterns and individual differences in developmental processes, providing more accurate estimates than traditional repeated measures approaches. This tutorial applies LGCM to examine emotional suppression in ABCD youth across four annual assessments, estimating the average trajectory and individual variation in initial levels and rates of change.

## Features {.features}

- **When to Use:** Ideal when you have repeated ABCD measures and want to model the average growth trajectory plus individual deviations.
- **Key Advantage:** LGCM provides latent intercept and slope factors, so you can quantify both initial status and change over time with measurement error accounted for.
- **What You'll Learn:** How to specify a basic LGCM in lavaan, interpret intercept/slope estimates, and assess overall model fit and residual structure.

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
### Create a long-form dataset with relevant columns
df_long <- abcd_data %>%
  select(participant_id, session_id, ab_g_dyn__design_site, ab_g_stc__design_id__fam, mh_y_erq__suppr_mean) %>%
  # Filter to Years 3-6 annual assessments using NBDCtools
  filter_events_abcd(conditions = c("annual", ">=3", "<=6")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    session_id = factor(session_id,
                        levels = c("ses-03A", "ses-04A", "ses-05A", "ses-06A"),
                        labels = c("Year_3", "Year_4", "Year_5", "Year_6"))  # Relabel sessions for clarity
  ) %>%
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    suppression = mh_y_erq__suppr_mean
  ) %>%
  droplevels() %>%                                     # Drop unused factor levels
  drop_na(suppression)                                 # Remove rows with missing outcome data

### Reshape data from long to wide format
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = suppression,
    names_prefix = "Suppression_"
  ) %>%
  drop_na(starts_with("Suppression_"))  # Require complete data across all time points
```

## Descriptive Statistics {.code}

```r
### Descriptives
### Create descriptive summary table
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

/stage4-artifacts/lgcm-basic/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r
# Define model specification
model <- " i =~ 1*Suppression_Year_3 + 1*Suppression_Year_4 + 1*Suppression_Year_5 + 1*Suppression_Year_6
           s =~ 0*Suppression_Year_3 + 1*Suppression_Year_4 + 2*Suppression_Year_5 + 3*Suppression_Year_6

           # Intercept and slope variances
           i ~~ i
           s ~~ s

           # Residual variances for each observed variable
           Suppression_Year_3 ~~ var_baseline*Suppression_Year_3
           Suppression_Year_4 ~~ var_year1*Suppression_Year_4
           Suppression_Year_5 ~~ var_year2*Suppression_Year_5
           Suppression_Year_6 ~~ var_year3*Suppression_Year_6
"

fit <- growth(model, data = df_wide, missing = "ml")
model_summary <- summary(fit)

model_summary

### Convert lavaan output to a tidy dataframe and then to gt table
model_summary_table <- broom::tidy(fit) %>%
  gt() %>%
  tab_header(title = "Latent Growth Curve Model Results") %>%
  fmt_number(columns = c(estimate, std.error, statistic, p.value), decimals = 3)

### Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)

### Extract and save model fit indices
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

### Save fit indices table
gt::gtsave(
  data = fit_indices_table,
  filename = "model_fit_indices.html",
  inline_css = FALSE
)

```

## Model Summary Output {.output}

/stage4-artifacts/lgcm-basic/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lgcm-basic/model_fit_indices.html

## Interpretation {.note}

The LGCM fit was generally strong (CFI = 0.949, TLI = 0.938, SRMR = 0.045), with only the RMSEA (0.092) hinting at modest residual misfit. Average suppression at Year 3 was 3.109 (SE = 0.012, p < .001) and rose by 0.110 points per year (SE = 0.005, p < .001), indicating a small but reliable increase. Intercept and slope variances (0.322 and 0.046, both p < .001) confirmed that adolescents differed markedly in both starting levels and rates of change. The negative intercept–slope covariance (−0.039, p < .001) implies that youth who began with high suppression tended to grow more slowly, whereas those starting lower closed the gap. Residual variances declined from 0.439 at Year 3 to 0.292 by Year 6, suggesting that measurements became more stable across successive assessments. Overall, the model depicts a cohort-wide rise in suppression layered on top of substantial between-person heterogeneity.

## Visualization {.code}

```r
### Visualization
### Select a subset of participants
n_sample <- min(150, length(unique(df_long$participant_id)))
selected_ids <- sample(unique(df_long$participant_id), n_sample)
df_long_selected <- df_long %>% filter(participant_id %in% selected_ids)

### Plot Suppression Growth
visualization <- ggplot(df_long_selected, aes(x = session_id, y = suppression, group = participant_id)) +
    geom_line(alpha = 0.3, color = "gray") +
    geom_point(size = 1.5, color = "blue") +
    geom_smooth(aes(group = 1), method = "lm", color = "red", linewidth = 1.2, se = TRUE, fill = "lightpink") +
    labs(
        title = "Emotional Suppression Trajectories Over Time",
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

## Visualization {.output}

![Emotional Suppression Trajectory Plot](/stage4-artifacts/lgcm-basic/visualization.png)

## Visualization Notes {.note}

Each gray line shows a participant’s suppression trajectory across the four assessments, while blue points mark the observed scores and the red line traces the sample-wide mean. The upward tilt of the red line signals a cohort-level increase in suppression, yet the fan of gray lines makes it clear that individuals follow very different paths—some rise steeply, others flatten or dip. Because the observed points hug their respective lines, the plot also reassures us that the smoothing reflects the actual data rather than an artifact of model assumptions. In short, the figure simultaneously communicates the population trend and the heterogeneity that motivates a latent growth curve approach.

# Discussion

The analysis reveals heterogeneous suppression trajectories, with the overall trend indicating increasing suppression over time while individual trajectories varied substantially. Some participants exhibited slower or faster growth patterns, demonstrating the value of modeling random slope variability. The model captured significant individual differences in suppression trajectories by allowing for random slopes, improving fit compared to a model with only fixed effects.

The inclusion of both random intercepts and slopes provided a more flexible framework for understanding variability in initial suppression levels and growth rates across participants. The latent growth curve model (LGCM) enables a more detailed examination of longitudinal trends by modeling both baseline differences (intercepts) and individual variability in rates of change (slopes), offering deeper insights into developmental patterns over time.

# Additional Resources

### lavaan Growth Curve Tutorial {.resource}

Official lavaan documentation for latent growth curve modeling basics, covering model specification, parameter estimation, and result interpretation for unconditional growth models.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/growth.html

### Structural Equation Modeling in lavaan {.resource}

Comprehensive vignette covering growth models within the structural equation modeling framework, including detailed examples of latent growth curve specifications and output interpretation.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/lavaan/vignettes/intro.pdf

### Longitudinal Data Analysis by Singer & Willett {.resource}

Foundational textbook on growth curve modeling. Chapters 3-4 provide thorough coverage of unconditional growth models, including interpretation of intercepts, slopes, and random effects for longitudinal data. Note: access may require institutional or paid subscription.

**Badge:** BOOK
**URL:** https://oxford.universitypressscholarship.com/view/10.1093/acprof:oso/9780195152968.001.0001

### semPlot Package for Model Visualization {.resource}

R package for creating publication-quality path diagrams of structural equation models and latent growth curve models, with extensive customization options for visualization.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=semPlot
