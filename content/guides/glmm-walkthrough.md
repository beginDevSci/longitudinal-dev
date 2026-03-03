---
title: "GLMM Walkthrough: Worked Example"
slug: "glmm-walkthrough"
description: "Step-by-step GLMM analysis from setup to interpretation — binary and count outcomes using simulated data in R."
category: "mixed-models"
tags: ["GLMM", "worked-example", "glmmTMB", "R", "simulation"]
r_packages: ["glmmTMB", "lme4", "tidyverse", "MASS", "performance"]
guide_type: "tutorial"
parent_method: "glmm"
---

## GLMM Tutorial

This tutorial walks through two complete GLMM analyses — one binary, one count. By the end, you will have:

1. Simulated longitudinal data with known parameters for both outcome types
2. Visualized trajectories on both the observed and link scales
3. Fit and compared binary and count GLMMs
4. Interpreted results as odds ratios and incidence rate ratios

---

## Workflow Overview

```
┌─────────────────────────────────┐
│ 1. Setup                        │
│    Load packages, set seed      │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 2. Simulate Data                │
│    Binary + count outcomes      │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 3. Visualize                    │
│    Observed proportions/counts  │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 4. Fit Binary GLMM              │
│    Logistic random intercept    │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 5. Fit Count GLMM               │
│    Poisson → Negative Binomial  │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 6. Diagnostics                  │
│    Overdispersion, residuals    │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 7. Interpret Results            │
│    OR, IRR, predicted values    │
└─────────────────────────────────┘
```

---

## Step 1: Setup

```r
# Load packages
library(tidyverse)   # Data manipulation and plotting
library(lme4)        # glmer for binary GLMM
library(glmmTMB)     # Negative binomial GLMM
library(performance) # Model diagnostics

# Set seed for reproducibility
set.seed(2024)
```

---

## Step 2: Simulate Data

We create a dataset with 300 people measured at 5 waves, containing both a binary and a count outcome.

### Known Population Parameters

| Parameter | Binary (log-odds) | Count (log scale) |
|-----------|-------------------|-------------------|
| Intercept mean | -0.5 (P ≈ 0.38) | 1.5 (count ≈ 4.5) |
| Time effect | -0.2 per wave | 0.10 per wave |
| Treatment effect | -0.6 | -0.3 |
| Random intercept SD | 1.0 | 0.5 |

```r
n <- 300        # Participants
waves <- 5      # Time points
N <- n * waves  # Total observations

# Person-level data
person_df <- tibble(
  id = 1:n,
  treatment = rep(c(0, 1), each = n / 2),
  ri_binary = rnorm(n, 0, 1.0),   # Random intercept (binary)
  ri_count  = rnorm(n, 0, 0.5)    # Random intercept (count)
)

# Long-format with time
df <- person_df %>%
  crossing(time = 0:(waves - 1)) %>%
  arrange(id, time) %>%
  mutate(
    # Binary outcome (logistic)
    eta_bin = -0.5 + (-0.2 * time) + (-0.6 * treatment) + ri_binary,
    prob    = plogis(eta_bin),
    binary  = rbinom(N, size = 1, prob = prob),

    # Count outcome (Poisson with known parameters)
    eta_cnt  = 1.5 + (0.10 * time) + (-0.3 * treatment) + ri_count,
    lambda   = exp(eta_cnt),
    count    = rpois(N, lambda = lambda)
  )

head(df)
```

✅ **Checkpoint**: You should see a tibble with columns `id`, `treatment`, `time`, `binary` (0/1), and `count` (non-negative integers).

---

## Step 3: Visualize

**Always plot before modeling.** Visualization on the observed scale reveals patterns that summary statistics miss.

### Binary Outcome: Proportion Over Time

```r
df %>%
  group_by(time, treatment) %>%
  summarise(prop = mean(binary), .groups = "drop") %>%
  mutate(group = factor(treatment, labels = c("Control", "Treatment"))) %>%
  ggplot(aes(x = time, y = prop, color = group)) +
  geom_line(linewidth = 1) +
  geom_point(size = 3) +
  scale_y_continuous(limits = c(0, 1), labels = scales::percent) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Proportion (y = 1)",
       title = "Binary Outcome by Group Over Time",
       color = "Group") +
  theme_minimal()
```

**What to look for:**
- Declining proportions over time (negative time effect)
- Lower proportions in the treatment group
- Roughly parallel trends on this scale (or diverging — which the logit scale linearizes)

### Count Outcome: Mean Count Over Time

```r
df %>%
  group_by(time, treatment) %>%
  summarise(mean_count = mean(count), .groups = "drop") %>%
  mutate(group = factor(treatment, labels = c("Control", "Treatment"))) %>%
  ggplot(aes(x = time, y = mean_count, color = group)) +
  geom_line(linewidth = 1) +
  geom_point(size = 3) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Mean Count",
       title = "Count Outcome by Group Over Time",
       color = "Group") +
  theme_minimal()
```

**What to look for:**
- Increasing counts over time (positive time effect on log scale)
- Lower counts in the treatment group
- Whether the increase looks multiplicative (constant ratio) rather than additive (constant difference)

✅ **Checkpoint**: Both plots should show group separation consistent with the treatment effects above.

---

## Step 4: Fit Binary GLMM

### Random Intercept Model

```r
fit_bin <- glmer(binary ~ time + treatment + (1 | id),
                 data = df, family = binomial)

summary(fit_bin)
```

### Examine Fixed Effects

```r
# Log-odds coefficients
fixef(fit_bin)

# Odds ratios with 95% CI
cbind(
  OR = exp(fixef(fit_bin)),
  exp(confint(fit_bin, method = "Wald", parm = "beta_"))
)
```

✅ **Checkpoint**: The time coefficient should be near -0.2 (log-odds) and the treatment coefficient near -0.6. Odds ratios should be approximately exp(-0.2) ≈ 0.82 and exp(-0.6) ≈ 0.55.

### Compare with AGQ

For binary outcomes, Laplace approximation can be inaccurate with few observations per person. Compare:

```r
fit_bin_agq <- glmer(binary ~ time + treatment + (1 | id),
                     data = df, family = binomial, nAGQ = 7)

# Compare estimates
cbind(Laplace = fixef(fit_bin), AGQ = fixef(fit_bin_agq))
```

✅ **Checkpoint**: With 5 waves per person, estimates should be very similar. Differences > 5% would suggest Laplace is insufficient.

---

## Step 5: Fit Count GLMM

### Poisson Model (Baseline)

```r
fit_pois <- glmmTMB(count ~ time + treatment + (1 | id),
                    data = df, family = poisson)

summary(fit_pois)
```

### Check for Overdispersion

```r
# Overdispersion ratio (Pearson residuals)
overdisp <- sum(residuals(fit_pois, type = "pearson")^2) / df.residual(fit_pois)
cat("Overdispersion ratio:", round(overdisp, 2), "\n")
```

Values near 1.0 indicate Poisson variance is adequate. Values substantially above 1.0 suggest overdispersion — switch to negative binomial.

### Negative Binomial Model

```r
fit_nb <- glmmTMB(count ~ time + treatment + (1 | id),
                  data = df, family = nbinom2)

summary(fit_nb)
```

### Compare Poisson vs. Negative Binomial

```r
# AIC comparison
AIC(fit_pois, fit_nb)
```

✅ **Checkpoint**: Since data were simulated from Poisson, the NB dispersion parameter should be large (indicating the extra parameter isn't needed) and AIC should be similar or slightly favor Poisson. With real data, you'd often see NB win.

### Examine Fixed Effects (Count Model)

```r
# Log-scale coefficients
fixef(fit_pois)$cond

# Incidence rate ratios
exp(fixef(fit_pois)$cond)
```

✅ **Checkpoint**: Time coefficient ≈ 0.10 (IRR ≈ 1.11), treatment coefficient ≈ -0.30 (IRR ≈ 0.74).

---

## Step 6: Diagnostics

### Binary Model Diagnostics

Standard residual plots are less informative for binary GLMMs. Focus on:

```r
# Random effects distribution — should be approximately normal
re_bin <- ranef(fit_bin)$id
qqnorm(re_bin[[1]], main = "Random Intercepts (Binary)")
qqline(re_bin[[1]])

# Variance of random intercept
VarCorr(fit_bin)
```

✅ **Checkpoint**: QQ-plot should be roughly linear. Random intercept SD should be near 1.0 (the true value).

### Count Model Diagnostics

```r
# Random effects normality
re_pois <- ranef(fit_pois)$cond$id
qqnorm(re_pois[[1]], main = "Random Intercepts (Count)")
qqline(re_pois[[1]])

# Model performance summary
check_model(fit_pois)
```

### Predicted vs. Observed (Count)

```r
df$pred_count <- predict(fit_pois, type = "response")

df %>%
  group_by(time, treatment) %>%
  summarise(
    observed  = mean(count),
    predicted = mean(pred_count),
    .groups = "drop"
  ) %>%
  pivot_longer(c(observed, predicted), names_to = "type", values_to = "value") %>%
  mutate(group = factor(treatment, labels = c("Control", "Treatment"))) %>%
  ggplot(aes(x = time, y = value, color = group, linetype = type)) +
  geom_line(linewidth = 1) +
  geom_point(size = 2) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Mean Count",
       title = "Predicted vs. Observed Counts") +
  theme_minimal()
```

✅ **Checkpoint**: Predicted and observed lines should overlap closely.

---

## Step 7: Interpret Results

### Binary Model Summary

```r
summary(fit_bin)
```

### Compare Estimates to True Values

| Parameter | True | Estimate | SE | OR |
|-----------|------|----------|-----|-----|
| Intercept | -0.50 | ~-0.50 | ~0.10 | ~0.61 |
| Time | -0.20 | ~-0.20 | ~0.04 | ~0.82 |
| Treatment | -0.60 | ~-0.60 | ~0.14 | ~0.55 |
| RI SD | 1.00 | ~1.00 | — | — |

*Values are illustrative; your numbers will differ with the random seed.*

**Interpretation** (conditional, subject-specific):

- **Intercept**: At Wave 1, a control-group participant with average random intercept has a 38% probability of the outcome
- **Time**: Each additional wave, the odds of the outcome decrease by ~18% (OR ≈ 0.82) for a given individual
- **Treatment**: At any given wave, treatment participants have ~45% lower odds than controls (OR ≈ 0.55)
- **RI SD**: Substantial between-person heterogeneity (SD = 1 on the log-odds scale)

### Count Model Summary

```r
summary(fit_pois)
```

| Parameter | True | Estimate | SE | IRR |
|-----------|------|----------|-----|-----|
| Intercept | 1.50 | ~1.50 | ~0.04 | ~4.48 |
| Time | 0.10 | ~0.10 | ~0.01 | ~1.11 |
| Treatment | -0.30 | ~-0.30 | ~0.06 | ~0.74 |
| RI SD | 0.50 | ~0.50 | — | — |

**Interpretation** (conditional, subject-specific):

- **Intercept**: At Wave 1, a control participant with average random intercept has an expected count of ~4.5
- **Time**: Each additional wave, the expected count increases by ~11% (IRR ≈ 1.11)
- **Treatment**: Treatment participants have ~26% lower expected counts than controls (IRR ≈ 0.74)
- **RI SD**: Moderate between-person variability (SD = 0.5 on log scale)

### Predicted Probabilities (Binary)

Plot predicted probabilities across time by group:

```r
pred_grid <- expand.grid(time = 0:4, treatment = c(0, 1))
pred_grid$prob <- predict(fit_bin, newdata = pred_grid,
                          type = "response", re.form = NA)
pred_grid$group <- factor(pred_grid$treatment,
                          labels = c("Control", "Treatment"))

ggplot(pred_grid, aes(x = time, y = prob, color = group)) +
  geom_line(linewidth = 1.2) +
  geom_point(size = 3) +
  scale_y_continuous(limits = c(0, 1), labels = scales::percent) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Predicted Probability",
       title = "Predicted Probabilities (at average random effect)",
       color = "Group") +
  theme_minimal()
```

> [!note] **re.form = NA**
>
> Setting `re.form = NA` gives predictions at random effects = 0 (the "average" person). These are conditional predictions at the random effect mean — not marginal predictions averaged over the random effect distribution.

---

## Written Summary

> We fit a logistic GLMM with random intercepts to a binary longitudinal outcome (N = 300, 5 waves). The odds of the outcome decreased over time (OR = 0.82, 95% CI [0.75, 0.89], p < .001) and were lower in the treatment group (OR = 0.55, 95% CI [0.41, 0.73], p < .001). Substantial between-person heterogeneity was captured by the random intercept (SD = 1.02 on the log-odds scale).
>
> For the count outcome, a Poisson GLMM with random intercepts showed increasing counts over time (IRR = 1.11, 95% CI [1.08, 1.13], p < .001) and lower counts in the treatment group (IRR = 0.74, 95% CI [0.66, 0.83], p < .001). The Poisson model was adequate (overdispersion ratio ≈ 1.0); a negative binomial comparison confirmed no meaningful overdispersion.

---

## Complete Script

```r
# ============================================
# GLMM Complete Worked Example
# ============================================

library(tidyverse)
library(lme4)
library(glmmTMB)
library(performance)
set.seed(2024)

# --- Simulate data ---
n <- 300; waves <- 5; N <- n * waves

person_df <- tibble(
  id = 1:n,
  treatment = rep(c(0, 1), each = n / 2),
  ri_binary = rnorm(n, 0, 1.0),
  ri_count  = rnorm(n, 0, 0.5)
)

df <- person_df %>%
  crossing(time = 0:(waves - 1)) %>%
  arrange(id, time) %>%
  mutate(
    eta_bin = -0.5 + (-0.2 * time) + (-0.6 * treatment) + ri_binary,
    prob    = plogis(eta_bin),
    binary  = rbinom(N, 1, prob),
    eta_cnt = 1.5 + (0.10 * time) + (-0.3 * treatment) + ri_count,
    lambda  = exp(eta_cnt),
    count   = rpois(N, lambda)
  )

# --- Binary GLMM ---
fit_bin <- glmer(binary ~ time + treatment + (1 | id),
                 data = df, family = binomial)
summary(fit_bin)
cbind(OR = exp(fixef(fit_bin)),
      exp(confint(fit_bin, method = "Wald", parm = "beta_")))

# --- Count GLMM ---
fit_pois <- glmmTMB(count ~ time + treatment + (1 | id),
                    data = df, family = poisson)
fit_nb   <- glmmTMB(count ~ time + treatment + (1 | id),
                    data = df, family = nbinom2)
AIC(fit_pois, fit_nb)
summary(fit_pois)
exp(fixef(fit_pois)$cond)

# --- Diagnostics ---
check_model(fit_pois)
VarCorr(fit_bin)
```

---

## Adapt to Your Data

To use this workflow with your own dataset:

### 1. Read and Prepare Data

```r
df <- read_csv("my_longitudinal_data.csv") %>%
  mutate(
    id = factor(id),
    time = wave - 1  # Center time at first wave
  )
```

### 2. Choose the Right Family

| Outcome | Family | Package |
|---------|--------|---------|
| Binary (0/1) | `binomial` | `lme4::glmer` or `glmmTMB` |
| Count (no overdispersion) | `poisson` | Either |
| Count (overdispersed) | `nbinom2` | `glmmTMB` |
| Count (zero-inflated) | `nbinom2` + `zi` | `glmmTMB` |

### 3. Start Simple, Build Up

```r
# Start with random intercept
fit1 <- glmer(outcome ~ time + group + (1 | id),
              data = df, family = binomial)

# Add random slope if warranted
fit2 <- glmer(outcome ~ time + group + (time | id),
              data = df, family = binomial)

# Compare
anova(fit1, fit2)
```

✅ **Checkpoint**: Once your basic model runs, layer on diagnostics, model comparisons, and additional predictors as shown above.

---

## Next Steps

- **Need syntax or thresholds?** → [Reference](/guides/glmm-reference)
- **Hit an error?** → [Reference: Troubleshooting](/guides/glmm-reference#troubleshooting)
- **Back to overview** → [GLMM Guide](/guides/glmm)
