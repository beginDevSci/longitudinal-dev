---
title: 'LGCM: Multivariate'
slug: mlgcm
author: Biostatistics Working Group
date_iso: 2025-11-04
tags:
  - abcd-study
  - trajectory
  - parallel-process
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: lavaan
covariates: None
outcome_type: Continuous
description: Fit multivariate latent growth curve models to estimate parallel developmental processes and relate their intercepts and slopes using lavaan.
---

# Overview

## Summary {.summary}

Multivariate Latent Growth Curve Modeling (MLGCM) simultaneously models trajectories of multiple outcomes, revealing how different developmental processes unfold together over time. By estimating intercept and slope factors for each construct in a unified framework, MLGCM captures individual growth patterns while quantifying dynamic interrelationships between developmental trajectories. This tutorial examines co-development of externalizing and internalizing symptoms in ABCD youth across four annual assessments, estimating parallel growth parameters and cross-domain correlations to reveal comorbidity patterns and shared developmental processes.

## Features {.features}

- **When to Use:** Applicable when you want to model parallel growth processes (e.g., anxiety and suppression) simultaneously to see how trajectories are linked.
- **Key Advantage:** Captures covariance between multiple latent intercepts/slopes, revealing whether growth in one domain correlates with growth in another.
- **What You'll Learn:** How to specify multivariate LGCMs, interpret covariance among latent factors, and visualize coupled trajectories.

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
library(gtsummary)    # Creating publication-quality tables
library(lavaan)       # Structural Equation Modeling in R
library(broom)        # For tidying model outputs
library(gt)           # For creating formatted tables
library(patchwork)    # For combining ggplots

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
  "ab_g_dyn__design_site",
  "ab_g_stc__design_id__fam",
  "mh_p_cbcl__synd__ext_tscore",
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
### Clean and transform variables for analysis
df_long <- abcd_data %>%
  # Filter to baseline through Year 3
  filter(session_id %in% c("ses-00", "ses-00A", "ses-01A", "ses-02A", "ses-03A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    participant_id = factor(participant_id),
    session_id = factor(
      session_id,
      levels = c("ses-00", "ses-00A", "ses-01A", "ses-02A", "ses-03A"),
      labels = c("Baseline", "Baseline", "Year_1", "Year_2", "Year_3")
    ),
    family_id = factor(ab_g_stc__design_id__fam)
  ) %>%
  rename(
    site = ab_g_dyn__design_site,                        # site already a factor from NBDCtools
    externalizing = mh_p_cbcl__synd__ext_tscore,        # already numeric from NBDCtools
    internalizing = mh_p_cbcl__synd__int_tscore         # already numeric from NBDCtools
  ) %>%
  select(participant_id, session_id, site, family_id, externalizing, internalizing) %>%
  droplevels() %>%
  drop_na(externalizing, internalizing)                 # Remove rows with missing outcomes

# Reshape data from long to wide format
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = c(externalizing, internalizing),
    names_sep = "_"
  ) %>%
  drop_na(starts_with("externalizing_"), starts_with("internalizing_"))  # Require complete data
```

## Descriptive Statistics {.code}

```r
### [Descriptives.r]

### Create descriptive summary table
descriptives_table <- df_long %>%
  select(session_id, externalizing, internalizing) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      externalizing ~ "Externalizing",
      internalizing ~ "Internalizing"
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
  gt::gtsave(descriptives_table,
             filename = "descriptives_table.html")

### Print the table
descriptives_table

```

## Descriptive Statistics Output {.output}

/stage4-artifacts/mlgcm/descriptives_table.html

# Statistical Analysis

## Fit Multivariate Model {.code}

```r
#[Model.r]

model <- "
  # Growth model for externalizing
  i_var1 =~ 1*externalizing_Baseline + 1*externalizing_Year_1 + 1*externalizing_Year_2 + 1*externalizing_Year_3
  s_var1 =~ 0*externalizing_Baseline + 1*externalizing_Year_1 + 2*externalizing_Year_2 + 3*externalizing_Year_3

  # Growth model for internalizing
  i_var2 =~ 1*internalizing_Baseline + 1*internalizing_Year_1 + 1*internalizing_Year_2 + 1*internalizing_Year_3
  s_var2 =~ 0*internalizing_Baseline + 1*internalizing_Year_1 + 2*internalizing_Year_2 + 3*internalizing_Year_3

  # ✅ Estimate latent means
  i_var1 ~ 1
  s_var1 ~ 1
  i_var2 ~ 1
  s_var2 ~ 1

  # ✅ Fix observed variable intercepts to 0 for identification
  externalizing_Baseline ~ 0*1
  externalizing_Year_1   ~ 0*1
  externalizing_Year_2   ~ 0*1
  externalizing_Year_3   ~ 0*1

  internalizing_Baseline ~ 0*1
  internalizing_Year_1   ~ 0*1
  internalizing_Year_2   ~ 0*1
  internalizing_Year_3   ~ 0*1
"

### Fit the model using ML for handling missing data
fit <- sem(model, data = df_wide, missing = "ml")

### Check the summary to identify potential issues
summary(fit, fit.measures = TRUE, standardized = TRUE, rsquare = TRUE)

model_summary <- summary(fit)

model_summary

### Convert output to a tidy dataframe and then to gt table
model_summary_table <- broom::tidy(fit) %>%
  gt() %>%
  tab_header(title = "Multivariate LGCM Results") %>%
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

/stage4-artifacts/mlgcm/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/mlgcm/model_fit_indices.html

## Interpretation {.note}

Externalizing symptoms averaged 45.4 at Baseline and declined by 0.416 points per year (both p < .001); internalizing symptoms started at 48.4 and fell by 0.274 points annually. Thus, both domains improved over the study window. Intercept variances of roughly 80 and slope variances of 3–4 confirmed large between-person differences, so the population mean hides substantial heterogeneity.

Model fit was mixed—SRMR = 0.030 was excellent, whereas CFI/TLI (0.921/0.899) and RMSEA (0.144) suggested that additional cross-domain links might further reduce misfit. Even so, the estimated covariances captured the core comorbidity pattern: youth who started high on externalizing also tended to start high on internalizing (covariance = 58.6, p < .001), and declines in one domain generally coincided with declines in the other (slope covariance = 4.32, p < .001). Negative intercept–slope covariances within each domain imply diminishing returns—youth with the highest baseline symptoms improved, but at a slower rate than peers with milder presentations.

## Visualization {.code}

```r

### Calculate means for each time point for externalizing and internalizing
mean_externalizing <- df_wide %>%
    summarise(across(starts_with("externalizing"), mean, na.rm = TRUE))

mean_internalizing <- df_wide %>%
    summarise(across(starts_with("internalizing"), mean, na.rm = TRUE))

### Reshape the data for plotting
mean_externalizing_long <- pivot_longer(mean_externalizing, cols = everything(), names_to = "Time", values_to = "Mean_externalizing")
mean_internalizing_long <- pivot_longer(mean_internalizing, cols = everything(), names_to = "Time", values_to = "Mean_internalizing")

### Plot the mean trajectories for externalizing
externalizing_plot <- ggplot(mean_externalizing_long, aes(x = Time, y = Mean_externalizing, group = 1)) +
    geom_line(color = "blue", size = 1.2) +
    geom_point(color = "blue") +
    labs(title = "Mean Growth Trajectory for externalizing", y = "Mean externalizing", x = "Time Point") +
    theme_minimal() +
    theme(axis.text.x = element_text(angle = 45, hjust = 1))

### Plot the mean trajectories for internalizing
internalizing_plot <- ggplot(mean_internalizing_long, aes(x = Time, y = Mean_internalizing, group = 1)) +
    geom_line(color = "red", size = 1.2) +
    geom_point(color = "red") +
    labs(title = "Mean Growth Trajectory for internalizing", y = "Mean internalizing", x = "Time Point") +
    theme_minimal() +
    theme(axis.text.x = element_text(angle = 45, hjust = 1))

### Combine the plots side by side using patchwork
combined_plot <- externalizing_plot + internalizing_plot
print(combined_plot)

# Save as a high-resolution PNG file
ggsave(filename = "visualization.png",
       plot = combined_plot,
       width = 10,  # Specify width in inches (or units)
       height = 5, # Specify height in inches (or units)
       units = "in", # Specify units (e.g., "in", "cm", "mm")
       dpi = 300) # Specify resolution (e.g., 300 for good quality)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/mlgcm/visualization.png)

## Visualization Notes {.note}

This visualization displays the mean trajectories for externalizing (blue) and internalizing (red) behaviors across four annual assessments. Both plots show the average change patterns at the group level, highlighting the overall developmental trends.

The externalizing trajectory (left panel) shows a modest decline from baseline through Year 3, with symptoms decreasing approximately 0.4 points per year. The internalizing trajectory (right panel) exhibits a similar but slightly smaller downward trend, declining approximately 0.3 points annually. Both patterns demonstrate symptom improvement over time at the population level.

The parallel visualization format facilitates comparison of trajectory shapes and magnitude differences between the two behavioral domains. While both show declining trends, externalizing symptoms begin at lower baseline levels (around 45) compared to internalizing symptoms (around 48), and both converge toward similar levels by Year 3. This coordinated decline across domains aligns with the positive slope covariance identified in the model, suggesting shared developmental processes or common influences driving improvement in both behavioral domains.

# Discussion

The multivariate growth model captured parallel declines in both externalizing (slope = −0.416, SE = 0.028, p < .001) and internalizing behaviors (slope = −0.274, SE = 0.032, p < .001). Intercept and slope variances were significant for each domain, confirming that youth entered adolescence with very different symptom levels and moved along heterogeneous pathways even when the overall trend pointed downward.

Model diagnostics were mixed. The SRMR of 0.030 suggested strong absolute fit, whereas the CFI (0.921) and TLI (0.899) fell shy of the 0.95 benchmark and the RMSEA (0.144) exceeded common cutoffs, implying that additional cross-loadings or time-specific covariances might still be needed to fully capture the observed structure.

Cross-domain associations were pronounced. Positive intercept (58.643, p < .001) and slope (4.321, p < .001) covariances indicated that youth with elevated baseline externalizing symptoms also tended to start high on internalizing problems and that improvements occurred in tandem. Negative intercept–slope covariances within each domain suggested diminishing returns: adolescents starting with the greatest difficulties exhibited slower recovery. Together, these findings highlight meaningful comorbidity and underscore the value of parallel-process models for charting cascading behavioral change.

# Additional Resources

### lavaan Multivariate Growth Models {.resource}

Official lavaan documentation on parallel process models with multiple outcomes, demonstrating how to specify correlated intercepts and slopes across different constructs.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/growth.html#multivariate-growth-models

### Parallel Process LGCM Tutorial {.resource}

Step-by-step quantitative methods tutorial on modeling correlated growth trajectories between two or more outcomes, including interpretation of cross-domain associations.

**Badge:** VIGNETTE
**URL:** https://quantdev.ssri.psu.edu/tutorials/parallel-process-latent-growth-curve-models

### Multivariate Growth Models in SEM {.resource}

Foundational methodology paper on coupled change processes in multivariate longitudinal data, covering theoretical frameworks and practical applications (Bollen & Curran, 2004). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/2004-16692-003

### lavaan Model Syntax Examples {.resource}

Collection of lavaan syntax examples for complex structural equation models including multivariate growth curves, with code snippets and interpretation guidance.

**Badge:** TOOL
**URL:** https://lavaan.ugent.be/tutorial/syntax1.html
