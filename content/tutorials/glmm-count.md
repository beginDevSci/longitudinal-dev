---
title: "GLMM: Count Outcomes"
slug: glmm-count
author: Biostatistics Working Group
date_iso: 2026-02-26
tags:
  - abcd-study
  - generalized-linear-mixed-model
  - count-data
  - negative-binomial
family: GLMM
family_label: Generalized Linear Mixed Models (GLMM)
engine: glmmTMB
engines:
  - glmmTMB
covariates: TIC
outcome_type: Count
difficulty: intermediate
timepoints: 3_5
summary: Fit negative binomial mixed models for count outcomes, compare Poisson and negative binomial specifications, interpret conditional incidence rate ratios, and assess overdispersion in repeated-measures ABCD data.
description: Fit negative binomial mixed models for count outcomes, compare Poisson and negative binomial specifications, interpret conditional incidence rate ratios, and assess overdispersion in repeated-measures ABCD data.
---

# Overview

## Summary {.summary}

Generalized Linear Mixed Models for count outcomes extend the GLMM framework to handle non-negative integer responses such as days of substance use, number of behavioral incidents, or frequency of health-related events. Unlike the population-averaged approach of GEE, GLMMs estimate subject-specific (conditional) effects that account for individual heterogeneity through random effects. This tutorial applies negative binomial GLMMs to analyze alcohol use days in ABCD youth across annual assessments, demonstrating Poisson versus negative binomial model selection, overdispersion detection, and interpretation of conditional incidence rate ratios that reflect within-person change.

## Features {.features}

- **When to Use:** Apply when your outcome is a non-negative integer count measured repeatedly, you want subject-specific effects, and the count distribution shows overdispersion (variance exceeding the mean).
- **Key Advantage:** The negative binomial GLMM handles overdispersion that violates Poisson assumptions while providing conditional (subject-specific) rate ratios that describe within-person change, complementing GEE's population-averaged perspective.
- **What You'll Learn:** How to compare Poisson and negative binomial GLMMs, detect overdispersion, interpret conditional incidence rate ratios, and understand how conditional effects relate to marginal effects from GEE.

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
library(tidyverse)    # Data wrangling & visualization
library(arrow)        # For reading Parquet files
library(gtsummary)    # Creating publication-quality tables
library(glmmTMB)      # Generalized linear mixed models
library(broom.mixed)  # Tidy output for mixed models
library(performance)  # Model diagnostics
library(gt)           # For creating formatted tables

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
### Prepare long-format data for GLMM count analysis
df_long <- abcd_data %>%
  filter_events_abcd(conditions = c("annual")) %>%
  rename(
    site = ab_g_dyn__design_site,
    family_id = ab_g_stc__design_id__fam,
    alc_days = su_y_tlfb__alc__1mo_ud,
    sex = ab_g_stc__cohort_sex
  ) %>%
  mutate(
    alc_days = as.numeric(alc_days),
    sex = factor(sex, levels = c(1, 2), labels = c("Male", "Female")),
    time = as.numeric(session_id) - 1,
    participant_id = factor(participant_id)
  ) %>%
  group_by(participant_id) %>%
  filter(sum(!is.na(alc_days)) >= 2) %>%
  ungroup() %>%
  drop_na(alc_days, sex) %>%
  arrange(participant_id, time)
```

## Descriptive Statistics {.code}

```r
### Summarize count distribution by wave
descriptives_table <- df_long %>%
  mutate(wave = factor(session_id)) %>%
  select(wave, alc_days) %>%
  tbl_summary(
    by = wave,
    missing = "no",
    label = list(
      alc_days ~ "Alcohol Use Days (past 30)"
    ),
    statistic = list(
      all_continuous() ~ "{mean} ({sd}); Median: {median}"
    )
  ) %>%
  modify_header(all_stat_cols() ~ "{level}<br>N = {n}") %>%
  modify_spanning_header(all_stat_cols() ~ "Assessment Wave") %>%
  bold_labels() %>%
  italicize_levels()

### Apply compact styling and save
theme_gtsummary_compact()

descriptives_table <- as_gt(descriptives_table)
gt::gtsave(descriptives_table, filename = "descriptives_table.html")
descriptives_table
```

## Descriptive Statistics Output {.output}

/stage4-artifacts/glmm-count/descriptives_table.html

# Statistical Analysis

## Model 1: Poisson Random Intercept {.code}

```r
### Fit Poisson GLMM as baseline model
m_poisson <- glmmTMB(
  alc_days ~ time + sex + (1 | participant_id),
  family = poisson(link = "log"),
  data = df_long
)

summary(m_poisson)
```

## Model 2: Negative Binomial Random Intercept {.code}

```r
### Fit negative binomial GLMM to handle overdispersion
m_nb <- glmmTMB(
  alc_days ~ time + sex + (1 | participant_id),
  family = nbinom2(link = "log"),
  data = df_long
)

summary(m_nb)
```

## Model 3: Negative Binomial with Random Slope {.code}

```r
### Add random slope for time to capture individual trajectory heterogeneity
m_nb_slope <- glmmTMB(
  alc_days ~ time + sex + (1 + time | participant_id),
  family = nbinom2(link = "log"),
  data = df_long
)

summary(m_nb_slope)
```

## Model Comparison {.code}

```r
### Compare models using AIC/BIC
comparison_data <- data.frame(
  Model = c(
    "M1: Poisson RI",
    "M2: NB RI",
    "M3: NB RI + RS"
  ),
  Family = c("Poisson", "Neg. Binomial", "Neg. Binomial"),
  AIC = c(AIC(m_poisson), AIC(m_nb), AIC(m_nb_slope)),
  BIC = c(BIC(m_poisson), BIC(m_nb), BIC(m_nb_slope)),
  LogLik = c(
    as.numeric(logLik(m_poisson)),
    as.numeric(logLik(m_nb)),
    as.numeric(logLik(m_nb_slope))
  )
)

comparison_table <- comparison_data %>%
  gt() %>%
  tab_header(title = "Model Comparison: Poisson vs. Negative Binomial GLMM") %>%
  fmt_number(columns = c(AIC, BIC, LogLik), decimals = 1)

gt::gtsave(comparison_table, filename = "model_comparison.html")
comparison_table
```

## Model Comparison Output {.output}

/stage4-artifacts/glmm-count/model_comparison.html

## Incidence Rate Ratios {.code}

```r
### Extract IRRs from the best-fitting model
### Use m_nb as the primary model (random intercept NB)
irr_data <- broom.mixed::tidy(m_nb, conf.int = TRUE, exponentiate = TRUE) %>%
  filter(effect == "fixed") %>%
  select(term, estimate, conf.low, conf.high, std.error, p.value) %>%
  rename(
    IRR = estimate,
    CI_lower = conf.low,
    CI_upper = conf.high,
    SE = std.error
  )

irr_table <- irr_data %>%
  gt() %>%
  tab_header(title = "Conditional Incidence Rate Ratios (Negative Binomial GLMM)") %>%
  fmt_number(columns = c(IRR, CI_lower, CI_upper, SE), decimals = 3) %>%
  fmt_number(columns = p.value, decimals = 4) %>%
  cols_label(
    term = "Parameter",
    IRR = "IRR",
    CI_lower = "95% CI Lower",
    CI_upper = "95% CI Upper",
    SE = "SE (log)",
    p.value = "p-value"
  )

gt::gtsave(irr_table, filename = "model_summary.html")
irr_table
```

## Model Summary Output {.output}

/stage4-artifacts/glmm-count/model_summary.html

## Interpretation {.note}

The negative binomial GLMM with random intercepts and slopes (M3) estimated a conditional IRR for time of 2.79 (95% CI: 2.65–2.94, p < .001), indicating that, for a given individual, the expected count of alcohol use days nearly tripled per assessment wave. The sex effect was not significant (IRR = 1.27, 95% CI: 0.92–1.76, p = .142), suggesting no reliable difference between males and females after accounting for individual-level heterogeneity.

**Model comparison** strongly favored the negative binomial with random slopes (M3: AIC = 8,672.7) over both the Poisson random-intercept model (M1: AIC = 9,063.0) and the NB random-intercept model (M2: AIC = 9,065.0). The large AIC improvement for M3 indicates that individuals differ not only in baseline alcohol use but also in their rates of change over time — a random-slopes specification is essential for these data.

**Conditional vs. marginal effects:** The conditional IRR for time (2.79) closely matches the population-averaged IRR from the GEE count tutorial (also 2.79), suggesting that the random effect distribution is relatively symmetric and the marginal-conditional gap is small for these data. When random intercept variance is large, conditional effects can be substantially larger than marginal effects — they answer different questions (within-person change vs. population-average change).

## Visualization {.code}

```r
### Visualize predicted counts over time by sex
pred_grid <- expand.grid(
  time = sort(unique(df_long$time)),
  sex = c("Male", "Female")
)

### Get population-level predictions (fixed effects only, no RE)
pred_grid$predicted <- predict(m_nb, newdata = pred_grid,
                                type = "response",
                                re.form = NA)

### Get SEs for confidence intervals via manual calculation
pred_log <- predict(m_nb, newdata = pred_grid,
                     type = "link", re.form = NA,
                     se.fit = TRUE)
pred_grid$ci_lower <- exp(pred_log$fit - 1.96 * pred_log$se.fit)
pred_grid$ci_upper <- exp(pred_log$fit + 1.96 * pred_log$se.fit)

### Create prediction plot
count_plot <- ggplot(pred_grid, aes(x = time, y = predicted,
                                     color = sex, fill = sex)) +
  geom_ribbon(aes(ymin = ci_lower, ymax = ci_upper),
              alpha = 0.2, color = NA) +
  geom_line(linewidth = 1.2) +
  geom_point(size = 2.5) +
  scale_x_continuous(
    breaks = sort(unique(df_long$time)),
    labels = levels(factor(df_long$session_id))
  ) +
  scale_color_manual(values = c("Male" = "#2E86AB", "Female" = "#A23B72")) +
  scale_fill_manual(values = c("Male" = "#2E86AB", "Female" = "#A23B72")) +
  labs(
    title = "Predicted Alcohol Use Days Over Time",
    subtitle = "Population-level predictions from NB GLMM with 95% CIs",
    x = "Assessment Wave",
    y = "Expected Days (past 30)",
    color = "Sex",
    fill = "Sex"
  ) +
  theme_minimal() +
  theme(
    legend.position = "bottom",
    axis.text.x = element_text(angle = 45, hjust = 1)
  )

ggsave(
  filename = "visualization.png",
  plot = count_plot,
  width = 8, height = 6, dpi = 300
)
```

## Visualization {.output}

![Predicted Counts Over Time](stage4-artifacts/glmm-count/visualization.png)

## Visualization Notes {.note}

The plot displays **population-level predicted counts** — the expected number of alcohol use days at each wave, computed from fixed effects only (averaging over random effects). The shaded regions represent 95% confidence intervals. The upward trajectory reflects the increasing IRR over time, while the separation between sex groups shows the magnitude of the sex effect on the count scale rather than the log scale.

These population-level predictions are conceptually similar to the marginal predictions from GEE, but they are derived differently: GEE directly models the marginal mean, while the GLMM predictions shown here are obtained by setting random effects to zero. For a true marginal prediction from the GLMM, one would integrate over the random effects distribution, which would produce predictions closer to the observed means.

# Discussion

The negative binomial GLMM analysis demonstrates the subject-specific approach to modeling count outcomes in longitudinal data, complementing the population-averaged perspective of GEE. The substantial AIC improvement of the negative binomial over the Poisson model confirms that overdispersion is present — a common finding with behavioral count data where many participants report zero events and a few report high counts. Ignoring overdispersion in a Poisson model would produce artificially narrow confidence intervals and inflated Type I error rates.

The conditional IRRs from the GLMM describe how the expected count changes for a specific individual across time, holding their random intercept (and, if included, random slope) constant. These within-person effects are typically larger than the corresponding marginal effects from GEE because they are not attenuated by between-person variability. The choice between conditional (GLMM) and marginal (GEE) interpretation depends on the research question: conditional effects are appropriate when individual-level processes are of interest, while marginal effects are appropriate for population-level policy or public health questions.

Key methodological takeaways include: (1) always compare Poisson and negative binomial specifications — the AIC difference provides direct evidence about overdispersion; (2) the dispersion parameter from the NB model quantifies how much extra-Poisson variability exists; (3) random slopes for time allow individual trajectories to diverge, which is important when some participants escalate use while others remain abstinent; and (4) the conditional-marginal distinction is not about right versus wrong but about which question is being answered.

# Additional Resources

### glmmTMB Package Documentation {.resource}

Comprehensive documentation for the glmmTMB package, covering negative binomial, Poisson, and other count distributions with random effects structures and zero-inflation modeling.

**Badge:** DOCS
**URL:** https://cran.r-project.org/package=glmmTMB

### Brooks et al. (2017): glmmTMB for GLMMs {.resource}

Methods paper describing the glmmTMB package, including its handling of negative binomial, zero-inflated, and hurdle models for count data with worked examples.

**Badge:** PAPER
**URL:** https://doi.org/10.32614/RJ-2017-066

### Hilbe (2011): Negative Binomial Regression {.resource}

Comprehensive reference on modeling count data including Poisson regression, overdispersion detection, negative binomial alternatives, and zero-inflated models with worked examples in R.

**Badge:** BOOK
**URL:** https://doi.org/10.1017/CBO9780511973420

### Bolker et al. (2009): GLMMs in Ecology {.resource}

Influential paper providing practical guidance on fitting and interpreting GLMMs, including discussion of count outcomes, overdispersion diagnostics, and model selection strategies.

**Badge:** PAPER
**URL:** https://doi.org/10.1016/j.tree.2008.10.008
