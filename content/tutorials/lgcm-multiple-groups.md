---
title: "LGCM: Multiple Groups"
slug: lgcm-multiple-groups
author: Biostatistics Working Group
date_iso: 2025-11-05
tags:
  - abcd-study
  - latent-growth-curve-model
  - group-comparison
family: LGCM
family_label: Latent Growth Curve Models (LGCM)
engine: lavaan
covariates: None
outcome_type: Continuous
description: Compare latent growth trajectories between groups using multigroup LGCM, testing memory development differences while accounting for related ABCD participants.
---

# Overview

## Summary {.summary}

Multigroup Latent Growth Curve Modeling (MG-LGCM) tests whether growth patterns differ systematically across groups by estimating separate intercept and slope parameters for each group while allowing measurement and structural equivalence testing. This approach addresses key questions about developmental heterogeneity: Do groups show equivalent developmental patterns? Are baseline differences maintained over time? Do groups change at different rates? This tutorial examines sex differences in memory trajectories across four assessments in ABCD youth, testing increasingly constrained models to determine whether boys and girls exhibit equivalent or distinct developmental patterns.

## Features {.features}

- **When to Use:** When you need to compare growth trajectories between groups (e.g., sex, site) across ABCD waves.
- **Key Advantage:** Multiple-group LGCM estimates intercepts/slopes per group and tests equality constraints to detect group differences.
- **What You'll Learn:** How to specify multiple-group LGCMs in lavaan, compare nested models, and interpret group-specific growth factors.

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
library(tidyverse)    # Data manipulation and visualization
library(arrow)        # For reading Parquet files
library(gtsummary)    # Publication-ready descriptive tables
library(gt)           # Table formatting
library(lavaan)       # Structural equation modeling
library(semPlot)      # Path diagram visualization

# Set random seed for reproducible family member selection
set.seed(123)

### Load harmonized ABCD data required for this analysis
requested_vars <- c(
    "ab_g_dyn__design_site",
    "ab_g_stc__design_id__fam",
    "ab_g_stc__cohort_sex",
    "ab_g_dyn__visit_age",
    "nc_y_nihtb__picsq__agecor_score"
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
  # Select memory assessment waves
  filter(session_id %in% c("ses-00A", "ses-02A", "ses-04A", "ses-06A")) %>%
  arrange(participant_id, session_id) %>%
  mutate(
    # Relabel session IDs for clarity
    session_id = factor(session_id,
                        levels = c("ses-00A", "ses-02A", "ses-04A", "ses-06A"),
                        labels = c("Year_0", "Year_2", "Year_4", "Year_6")),
    # Create sex grouping variable with meaningful labels
    sex = factor(case_when(
      ab_g_stc__cohort_sex == 1 ~ "Male",
      ab_g_stc__cohort_sex == 2 ~ "Female",
      TRUE ~ NA_character_
    ))
  ) %>%
  # Rename for clarity
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    age = ab_g_dyn__visit_age,
    memory = nc_y_nihtb__picsq__agecor_score
  ) %>%
  # Remove invalid memory scores and missing sex data
  filter(memory > 1, !is.na(sex)) %>%
  select(participant_id, session_id, site, family_id, age, sex, memory) %>%
  droplevels()
```

## Reshape to Wide Format {.code}

```r
# Reshape data from long to wide format
df_wide <- df_long %>%
  pivot_wider(
    names_from = session_id,
    values_from = c(memory, age, sex, family_id),
    names_sep = "_"
  ) %>%
  # Clean column names by removing prefix
  rename_with(~ str_replace_all(., "Year_", ""), everything()) %>%
  # Use sex from baseline for grouping
  mutate(sex_group = sex_0) %>%
  # Remove cases with missing memory data
  drop_na(starts_with("memory_"))
```

## Descriptive Statistics by Group {.code}

```r
descriptives_table <- df_long %>%
  select(session_id, memory) %>%
  tbl_summary(
    by = session_id,
    missing = "no",
    label = list(
      memory ~ "Memory"
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

## Descriptive Statistics {.output}

/stage4-artifacts/lgcm-multiple-groups/descriptives_table.html

# Statistical Analysis

## Measurement Invariance Testing {.note}

This tutorial employs a multigroup latent growth curve modeling approach to test measurement invariance across sex groups. Measurement invariance testing evaluates whether the same construct is being measured equivalently across groups - a prerequisite for valid group comparisons. We fit a series of increasingly constrained models (M1-M4) that test different levels of equivalence: M1 tests full equality across groups, M2 relaxes latent mean constraints, M3 further relaxes variance/covariance constraints, and M4 (the least constrained) only constrains factor loadings to be equal across groups. By comparing model fit across these nested models, we can determine the appropriate level of invariance and whether observed group differences in trajectories reflect genuine developmental differences rather than measurement artifacts.

## Define and Fit Multigroup Models {.code}

```r
# Define Latent Growth Model (LGCM)
lgcm_model <- "
  # Latent Growth Model
  i =~ 1*memory_0 + 1*memory_2 + 1*memory_4 + 1*memory_6
  s =~ 0*memory_0 + 1*memory_2 + 2*memory_4 + 3*memory_6
"

# Define function to fit SEM model with different constraints
fit_lgcm_model <- function(model, group_constraints = NULL) {
  sem(
    model,
    data = df_wide,
    estimator = "ML",
    cluster = "family_id_0",
    se = "robust",
    missing = "fiml",
    group = "sex_0",  # Separating groups by sex
    group.equal = group_constraints
  )
}

# Fit models with increasing constraints
model_constraints <- list(
  M1 = c("loadings", "means", "lv.variances", "lv.covariances", "residuals"),
  M2 = c("loadings", "lv.variances", "lv.covariances", "residuals"),
  M3 = c("loadings", "residuals"),
  M4 = c("loadings")  # Least constrained model
)

# Fit models and store results in a list
fit <- lapply(model_constraints, function(constraints) fit_lgcm_model(lgcm_model, constraints))

# Assign meaningful names to models
names(fit) <- names(model_constraints)

# Compare models using ANOVA
anova_results <- anova(fit$M2, fit$M3, fit$M4)
print(anova_results)

# Print summary of the most constrained model (M4)
summary(fit$M4, fit.measures = TRUE)
```

## Format Model Summary Table {.code}

```r
# Extract model summary for M4
model_summary <- summary(fit$M4, fit.measures = TRUE)

model_summary

# Convert lavaan output to a tidy dataframe and then to gt table
model_summary_table <- broom::tidy(fit$M4) %>%
  gt() %>%
  tab_header(title = "Latent Growth Curve Model Results") %>%
  fmt_number(
    columns = c(estimate, std.error, statistic, p.value),
    decimals = 3
  )

# Save the gt table
gt::gtsave(
  data = model_summary_table,
  filename = "model_summary.html",
  inline_css = FALSE
)
```

## Format Model Fit Indices Table {.code}

```r
# Extract and save model fit indices for M4 (most constrained model)
fit_indices <- fitMeasures(fit$M4, c("chisq", "df", "pvalue", "cfi", "tli", "rmsea", "srmr", "aic", "bic"))

fit_indices_table <- data.frame(
  Metric = names(fit_indices),
  Value = as.numeric(fit_indices)
) %>%
  gt() %>%
  tab_header(title = "Model Fit Indices (M4)") %>%
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

## Model Summary Output-1 {.output}

/stage4-artifacts/lgcm-multiple-groups/model_summary.html

## Model Fit Indices Output {.output}

/stage4-artifacts/lgcm-multiple-groups/model_fit_indices.html

## Model Comparison Strategy {.note}

The analysis compares four nested models with progressively relaxed constraints to test measurement invariance:

- **M1 (Full Equality):** All parameters constrained equal across sex groups - tests whether males and females show identical growth patterns in all respects
- **M2 (Equal Variances/Covariances):** Relaxes latent mean constraints, allowing groups to differ in average intercepts and slopes while maintaining equal variances and covariances
- **M3 (Equal Loadings/Residuals):** Further relaxes variance/covariance constraints, permitting group differences in individual variability around mean trajectories
- **M4 (Metric Invariance):** The least constrained model, requiring only equal factor loadings across groups - establishes that time points are measured on the same scale for both groups

Model comparison uses chi-square difference tests to evaluate whether relaxing constraints significantly improves fit. Non-significant differences suggest the more constrained model is adequate, supporting equivalent measurement and structural parameters across groups.

```r
# Visualizing the latent growth curve model
sem_diagram <- semPaths(
  fit$M1,  # Your fitted LGCM model
  what = "path",   # Display path diagram
  whatLabels = "par",  # Show parameter estimates
  style = "lisrel",  # A clean layout for SEM models
  nCharNodes = 0,  # Avoid truncating variable names
  layout = "tree",  # Hierarchical layout (better for growth models)
  residuals = FALSE,  # Hide residuals for clarity
  curvePivot = TRUE,  # Adjust curvature of paths
  intercepts = FALSE,  # Hide intercepts for clarity
  edge.label.cex = 0.8,  # Adjust label size
  sizeMan = 7,  # Increase node size
  sizeLat = 10,  # Increase latent variable size
  group.label = TRUE
)

png(
  filename = "sem_diagram.png",
  width = 1200,
  height = 900,
  res = 150
)

```

## Interpretation {.note}

The multigroup LGCM found meaningful between-person variability in both baseline memory and change. Intercept variances were 117.0 for females and 100.5 for males, while slope variances were 7.0 and 5.1 respectively, indicating that members of each group start from different levels and improve at different rates. Covariance patterns hinted at subtle sex differences: females showed virtually no association between initial status and change (−0.14, p = .96), whereas males displayed a positive but only marginally significant covariance (4.85, p = .085), suggesting that higher-baseline males may recover slightly faster. Despite these nuances, nested model comparisons (M2→M3→M4) yielded Δχ² = 2.69, p = .443, so loosening equality constraints did not materially improve fit. The take-away is that memory growth curves are largely comparable across sex, with variance components doing most of the work to capture heterogeneity rather than group-specific fixed effects.

## Visualization {.code}

```r
# Select a subset of participants
set.seed(123)  # For reproducibility
selected_ids <- sample(unique(df_long$participant_id), 250)
df_long_selected <- df_long %>% filter(participant_id %in% selected_ids)

# Ensure 'sex' is a factor with meaningful labels
df_long_selected <- df_long_selected %>%
  mutate(sex = factor(sex, labels = c("Male", "Female")))

# Plot memory growth by sex group
visualization <- ggplot(df_long_selected, aes(x = session_id, y = memory, group = participant_id, color = sex)) +
    geom_line(alpha = 0.3, aes(color = sex)) +  # Individual trajectories
    geom_point(size = 1.5) +  # Observed data points
    geom_smooth(aes(group = sex, fill = sex), method = "lm", linewidth = 1.2, se = TRUE, alpha = 0.3) +
    facet_wrap(~sex) +  # Separate panels for each group
    labs(
        title = "Memory Growth Trajectories by Sex",
        x = "Time (Years from Baseline)",
        y = "Memory Score",
        color = "Sex Group",
        fill = "Sex Group"
    ) +
    theme_minimal() +
    theme(legend.position = "bottom")

      ggsave(
  filename = "visualization.png",
  plot = visualization,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Visualization](/stage4-artifacts/lgcm-multiple-groups/visualization.png)

## Visualization Notes {.note}

The faceted plot shows individual memory trajectories (faint lines) and group-level trends (bold smoothers) for males and females side by side. Both panels slope upward, matching the model’s conclusion that memory improves over time, yet the spreads differ slightly: female trajectories fan out more, echoing the larger intercept and slope variances estimated for that group. Points at each assessment anchor the smooth lines to the observed data, so readers can judge how well the fitted curves represent reality. Because the means rise at similar rates in both panels, the figure visually supports the invariance tests showing no compelling sex differences in the fixed effects, while still highlighting the heterogeneity that motivates a multigroup LGCM.

# Discussion

This analysis characterized substantial between-person variability in the estimated growth trajectory of memory. The observed trajectories ranged from rapid improvement to stability or decline—which demonstrated that a simple average or fixed-effects approach would likely be insufficient. This heterogeneity highlighted the need to capture individual deviations from the group trend for accurate inference.

Incorporating random slopes allowed for the estimation of individual-specific rates of change over time, rather than constraining all subjects to a common growth rate. By modeling both random intercepts (to capture baseline heterogeneity) and random slopes, the model effectively accounted for individual differences in both starting memory level and subsequent growth. This significantly enhanced model fit relative to simpler alternatives, supporting the need for this additional structure.

# Additional Resources

### lavaan Multi-Group Analysis {.resource}

Official lavaan documentation on multi-group growth curve modeling, including how to specify group constraints and test measurement invariance across groups.

**Badge:** DOCS
**URL:** https://lavaan.ugent.be/tutorial/groups.html

### Measurement Invariance in Growth Models {.resource}

Comprehensive methodology paper on testing invariance across groups in longitudinal structural equation models, covering configural, metric, and scalar invariance (Widaman et al., 2010).

**Badge:** PAPER
**URL:** https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3930030/

### Growth Curve Modeling by Preacher et al. {.resource}

Textbook chapter on multi-group growth models with practical examples, demonstrating how to compare trajectories across demographic groups and test equality constraints.

**Badge:** BOOK
**URL:** https://www.guilford.com/books/Growth-Curve-Modeling/Preacher-Wichman-MacCallum-Briggs/9781462523047

### semTools for Invariance Testing {.resource}

R package that automates measurement invariance tests in lavaan, providing convenient functions for sequential constraint testing and model comparison.

**Badge:** TOOL
**URL:** https://cran.r-project.org/package=semTools
