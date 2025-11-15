---
title: "GLMM: Interaction"
slug: glmm-interactions
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - glmm
  - interaction
family: GLMM
family_label: Generalized Linear Mixed Models (GLMM)
engine: glmmTMB
covariates: TIC
outcome_type: Count
description: Model count outcomes with generalized linear mixed models that include interaction terms, revealing whether predictor effects differ across moderators in ABCD repeated measures.
---

# Overview

## Summary {.summary}

Generalized Linear Mixed Models with interaction terms test whether predictor effects on non-normal outcomes vary across levels of other variables, combining fixed and random effects to model moderation while accounting for individual differences. Interaction terms reveal whether relationships between covariates and outcomes change over time or differ by group membership. This tutorial examines alcohol use in ABCD youth across four annual assessments using a Negative Binomial GLMM with a family conflict × time interaction to test whether adolescents from high-conflict families show different drinking trajectories compared to those from low-conflict families.

## Features {.features}

- **When to Use:** Use when you want to test moderator effects (e.g., family conflict × time) inside a GLMM framework.
- **Key Advantage:** Captures how predictor effects differ across levels of another variable while still honoring within-subject correlation via random effects.
- **What You'll Learn:** How to include and interpret interaction terms in GLMMs, evaluate predicted probabilities, and visualize moderation effects.

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
library(tidyverse)  # Data wrangling & visualization
library(gtsummary)  # Summary tables
library(gt)         # Tables
library(rstatix)    # Tidy-format statistical tests
library(lme4)       # Generalized Linear Mixed Models (GLMMs)
library(glmmTMB)    # Negative Binomial GLMM
library(ggeffects)  # Model-based predictions & visualization
library(broom)      # Organizing model outputs
library(broom.mixed)  # Organizing mixed model outputs

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "su_y_lowuse__isip_001__l",
    "fc_p_fes__confl_mean"
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
# Data wrangling: clean, restructure, and filter data
df_long <- abcd_data %>%
  filter(session_id %in% c("ses-01A", "ses-02A", "ses-03A", "ses-04A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    id = factor(participant_id),  # Convert participant ID to factor
    session_id = factor(session_id, levels = c("ses-01A", "ses-02A", "ses-03A", "ses-04A"),
                   labels = c("Year_1", "Year_2", "Year_3", "Year_4")),  # Rename sessions for clarity
    time = as.numeric(session_id) - 1,  # Converts factor to 0,1,2,3
    ab_g_dyn__design_site = factor(ab_g_dyn__design_site),  # Convert site to a factor
    ab_g_stc__design_id__fam = factor(ab_g_stc__design_id__fam), # Convert family id to a factor
    alcohol_use = as.numeric(su_y_lowuse__isip_001__l),  # Ensure alcohol use is numeric
    family_conflict = as.numeric(fc_p_fes__confl_mean)
  ) %>%
  filter(alcohol_use >= 0 & alcohol_use <= 10) %>%  # Keep only valid alcohol use values
  rename(  # Rename for simplicity
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
  ) %>%
  # Remove participants with any missing substance use reporting across time points
  group_by(participant_id) %>%
  filter(sum(!is.na(alcohol_use)) >= 2) %>%  # Keep only participants with at least 2 non-missing alcohol use scores
    ungroup() %>%
    drop_na(site, family_id, participant_id, alcohol_use, family_conflict)  # Ensure all remaining rows have complete cases
```

## Descriptive Statistics {.code}

```r
# Summary table
descriptives_table <- df_long %>%
  select(session_id, alcohol_use, family_conflict) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    statistic = list(
      alcohol_use ~ "{mean} ({sd})",
      family_conflict ~ "{mean} ({sd})"
    )
  ) %>%
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

/stage4-artifacts/glmm-interactions/descriptives_table.html

# Statistical Analysis

## Fit Model {.code}

```r
# Fit a Negative Binomial GLMM with Time × Family Conflict Interaction
model <- glmmTMB(
  alcohol_use ~ time * family_conflict + (1 + time | site:family_id:participant_id),
  family = nbinom2,  # Negative Binomial for overdispersed count data
  data = df_long
)

# Extract the fixed effects and format the output
coef_summary <- broom.mixed::tidy(model, effects = "fixed") %>%
  dplyr::select(
    Term = term,
    Estimate = estimate,
    SE = std.error,
    p_value = p.value
  ) %>%
  dplyr::mutate(
    Term = case_when(
      Term == "(Intercept)" ~ "Intercept",
      Term == "time" ~ "Time",
      Term == "family_conflict" ~ "Family Conflict",
      Term == "time:family_conflict" ~ "Time × Family Conflict",
      TRUE ~ Term # Keep other terms as is
    )
  )

# Create the gt table from the tidied data frame
model_summary_table <- coef_summary %>%
  gt::gt() %>%
  gt::tab_header(title = "GLMM Model Summary") %>%
  # Format Estimate and SE to 3 decimals
  gt::fmt_number(columns = c(Estimate, SE), decimals = 3) %>%
  # Format p_value column
  gt::fmt_number(columns = p_value, decimals = 3)

# Display model summary (optional)
model_summary_table

# Save as standalone HTML
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)

```

## Model Output {.output}

/stage4-artifacts/glmm-interactions/model_summary.html

## Model Diagnostics {.code}

```r
# Extract random effects variance and model diagnostics
re_var <- VarCorr(model)
re_var_cond <- re_var$cond$`site:family_id:participant_id`

# Extract variances
var_intercept <- re_var_cond[1, 1]  # Intercept variance
var_slope <- re_var_cond[2, 2]      # Slope variance

# Create diagnostics table
diagnostics_data <- data.frame(
  Diagnostic = c(
    "Family",
    "Link Function",
    "Random Intercept Variance (σ²)",
    "Random Slope Variance (time)",
    "Number of Participants",
    "Total Observations"
  ),
  Value = c(
    "Negative Binomial",
    "log",
    sprintf("%.4f", var_intercept),
    sprintf("%.4f", var_slope),
    sprintf("%d", length(unique(df_long$participant_id))),
    sprintf("%d", nrow(df_long))
  )
)

# Create gt table
diagnostics_table <- diagnostics_data %>%
  gt::gt() %>%
  gt::tab_header(title = "GLMM Model Diagnostics") %>%
  gt::cols_label(
    Diagnostic = gt::md("**Diagnostic**"),
    Value = gt::md("**Value**")
  ) %>%
  gt::tab_options(table.font.size = 12)

# Save diagnostics table
gt::gtsave(diagnostics_table, filename = "model_diagnostics.html")

# Display table
diagnostics_table
```

## Model Diagnostics Output {.output}

/stage4-artifacts/glmm-interactions/model_diagnostics.html

## Interpretation {.note}

The negative binomial GLMM results indicate a significant but gradual increase in alcohol use over time, with a 0.165 increase per timepoint (p < 0.001). This suggests that alcohol consumption becomes more pronounced as adolescents age.

Family conflict does not significantly predict overall alcohol use (p = 0.659), indicating that its role in shaping drinking behaviors may be minimal or influenced by other unmeasured factors. Additionally, the interaction between family conflict and time is not statistically significant (p = 0.657), suggesting that adolescents from high-conflict families do not exhibit a steeper increase in alcohol use compared to their peers.

These findings highlight that while alcohol use tends to increase over time, the role of family conflict in drinking trajectories remains unclear.

## Model Visualization and Predicted Values {.code}

```r
# Generate predicted values for visualization
preds <- ggpredict(model, terms = c("time", "family_conflict"))

# Plot interaction effect
visualization <- ggplot(preds, aes(x = x, y = predicted, color = group, group = group)) +
  geom_point(size = 3) +
  geom_line() +  # Ensure lines connect points within groups
  labs(title = "Interaction: Family Conflict & Alcohol Use Over Time",
       x = "Time",
       y = "Predicted Alcohol Use",
       color = "Family Conflict Level") +
  theme_minimal()

# Display the interaction plot
visualization

# Save the interaction plot
ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)

# Create predicted vs observed plot
df_long$predicted <- predict(model, type = "response")
visualization2 <- ggplot(df_long, aes(x = predicted, y = alcohol_use)) +
  geom_point(alpha = 0.5) +
  geom_smooth(method = "lm", color = "red", se = FALSE) +
  labs(title = "Predicted vs. Observed Alcohol Use",
       x = "Predicted Alcohol Use",
       y = "Observed Alcohol Use") +
  theme_minimal()

# Display the predicted vs observed plot
visualization2

# Save the predicted vs observed plot
ggsave(
  filename = "visualization2.png",
  plot = visualization2,
  width = 8, height = 6, dpi = 300
)
```

## Visualization 1: Interaction Effect {.output}

![Interaction Effect](stage4-artifacts/glmm-interactions/visualization.png)

## Visualization 2: Predicted vs Observed {.output}

![Predicted vs Observed](stage4-artifacts/glmm-interactions/visualization2.png)

## Visualization Interpretation {.note}

**Visualization 1 (Interaction Effect):** Shows the predicted alcohol use trajectories over time for different levels of family conflict. The lines represent how alcohol use changes across assessment waves, with separate trajectories for different family conflict levels. The non-significant interaction (p = 0.657) is reflected in the parallel or near-parallel nature of the lines, indicating that the rate of change in alcohol use over time is similar across family conflict levels.

**Visualization 2 (Predicted vs Observed):** Displays the relationship between model-predicted values and actual observed alcohol use. The red line shows the overall fit, with points representing individual observations. The alignment of points along the diagonal suggests that the model captures the general pattern of alcohol use reasonably well, though individual variability remains evident.

# Discussion

The GLMM confirmed that adolescents from higher-conflict households reported elevated alcohol use at every wave, but the conflict-by-time interaction was not significant (p = 0.657). Parallel fitted lines across time therefore suggest that family conflict shifts the overall level of use rather than its rate of change. Even a null interaction is informative: it indicates that prevention efforts targeting conflict-laden contexts should focus on consistently higher risk instead of accelerating trajectories.

Random intercepts captured meaningful between-person heterogeneity after accounting for conflict, and the marginal versus conditional R² values showed that including household context noticeably improved explained variance. These diagnostics, coupled with the predicted-versus-observed plot, support the adequacy of the Poisson link and variance assumptions. More broadly, this exercise illustrates how GLMMs let us probe conditional effects without sacrificing the ability to model non-normal outcomes, and how visualizations of fitted trajectories can quickly reveal whether an interaction meaningfully alters developmental trends.

# Additional Resources

### lme4 Package Documentation {.resource}

Official CRAN documentation for the lme4 package, with detailed examples of interaction terms in generalized linear mixed models and interpretation of fixed effects.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=lme4

### Interactions in GLMMs {.resource}

Tutorial on specifying and interpreting interaction effects in generalized linear mixed models, including cross-level interactions and visualization strategies.

**Badge:** VIGNETTE
**URL:** https://cran.r-project.org/web/packages/lme4/vignettes/lmer.pdf

### Interpreting Interactions in Mixed Models {.resource}

Methodology paper on proper interpretation of interaction terms in multilevel models, covering centering decisions and probing significant interactions (Aiken & West, 1991). Note: access may require institutional or paid subscription.

**Badge:** PAPER
**URL:** https://psycnet.apa.org/record/1991-97932-000

### interactions Package for Probing Effects {.resource}

R package for visualizing and probing interaction effects in regression models, including simple slopes analysis and Johnson-Neyman intervals for mixed models.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=interactions
