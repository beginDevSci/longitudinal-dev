---
title: "GEE Walkthrough: Worked Example"
slug: "gee-walkthrough"
description: "Step-by-step GEE analysis from setup to interpretation — comparing correlation structures, robust SEs, and GEE vs GLMM results."
category: "mixed-models"
tags: ["GEE", "worked-example", "geepack", "R", "simulation"]
r_packages: ["geepack", "lme4", "tidyverse", "MASS"]
guide_type: "tutorial"
parent_method: "gee"
---

## GEE Tutorial

This tutorial walks through a complete GEE analysis with a binary longitudinal outcome. By the end, you will have:

1. Simulated correlated binary data with known parameters
2. Fit GEE with different working correlation structures
3. Compared robust vs. naive standard errors
4. Compared GEE (marginal) and GLMM (conditional) results

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
│    Correlated binary outcome    │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 3. Visualize                    │
│    Proportions over time        │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 4. Fit GEE Models               │
│    Compare correlation structs  │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 5. Robust vs. Naive SEs         │
│    Compare inference methods    │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 6. Compare GEE and GLMM         │
│    Marginal vs. conditional     │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 7. Interpret Results            │
│    ORs, predicted probabilities │
└─────────────────────────────────┘
```

---

## Step 1: Setup

```r
# Load packages
library(tidyverse)   # Data manipulation and plotting
library(geepack)     # GEE fitting
library(lme4)        # GLMM for comparison
library(MASS)        # mvrnorm for simulation

# Set seed for reproducibility
set.seed(2024)
```

---

## Step 2: Simulate Data

We simulate 200 participants measured at 5 waves with a binary outcome. The data-generating process uses a random intercept (creating exchangeable-like correlation) so we know the true marginal and conditional parameters.

### Known Population Parameters

| Parameter | Conditional (log-odds) | Marginal (approximate) |
|-----------|----------------------|----------------------|
| Intercept | -0.5 | ≈ -0.42 |
| Time | -0.3 per wave | ≈ -0.25 |
| Treatment | -0.8 | ≈ -0.67 |
| Random intercept SD | 1.0 | — |

Marginal coefficients are attenuated relative to conditional — this is what we'll verify.

```r
n <- 200
waves <- 5
N <- n * waves

person_df <- tibble(
  id = 1:n,
  treatment = rep(c(0, 1), each = n / 2),
  ri = rnorm(n, 0, 1.0)  # Random intercept
)

df <- person_df %>%
  crossing(time = 0:(waves - 1)) %>%
  arrange(id, time) %>%
  mutate(
    eta  = -0.5 + (-0.3 * time) + (-0.8 * treatment) + ri,
    prob = plogis(eta),
    y    = rbinom(N, 1, prob)
  )

head(df)
```

✅ **Checkpoint**: You should see columns `id`, `treatment`, `time`, and `y` (0/1).

---

## Step 3: Visualize

```r
df %>%
  group_by(time, treatment) %>%
  summarise(prop = mean(y), .groups = "drop") %>%
  mutate(group = factor(treatment, labels = c("Control", "Treatment"))) %>%
  ggplot(aes(x = time, y = prop, color = group)) +
  geom_line(linewidth = 1) +
  geom_point(size = 3) +
  scale_y_continuous(limits = c(0, 1), labels = scales::percent) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Proportion (y = 1)",
       title = "Observed Prevalence by Group Over Time",
       color = "Group") +
  theme_minimal()
```

**What to look for:**
- Declining prevalence over time (negative time effect)
- Lower prevalence in the treatment group
- Group trajectories — roughly parallel on this scale?

✅ **Checkpoint**: Treatment group should be consistently below the control group, both declining over waves.

---

## Step 4: Fit GEE Models

### Exchangeable Correlation

```r
fit_exch <- geeglm(y ~ time + treatment, id = id, data = df,
                    family = binomial, corstr = "exchangeable")
summary(fit_exch)
```

### AR(1) Correlation

```r
fit_ar1 <- geeglm(y ~ time + treatment, id = id, data = df,
                   family = binomial, corstr = "ar1")
summary(fit_ar1)
```

### Independence

```r
fit_indep <- geeglm(y ~ time + treatment, id = id, data = df,
                     family = binomial, corstr = "independence")
summary(fit_indep)
```

### Compare Estimates Across Structures

```r
bind_rows(
  tibble(corstr = "Exchangeable",
         term = names(coef(fit_exch)),
         est = coef(fit_exch),
         se = summary(fit_exch)$coefficients[, "Std.err"]),
  tibble(corstr = "AR(1)",
         term = names(coef(fit_ar1)),
         est = coef(fit_ar1),
         se = summary(fit_ar1)$coefficients[, "Std.err"]),
  tibble(corstr = "Independence",
         term = names(coef(fit_indep)),
         est = coef(fit_indep),
         se = summary(fit_indep)$coefficients[, "Std.err"])
) %>%
  mutate(across(c(est, se), \(x) round(x, 3))) %>%
  pivot_wider(names_from = corstr, values_from = c(est, se))
```

✅ **Checkpoint**: Point estimates should be similar across structures (consistency property). Standard errors may differ — this reflects efficiency differences. The exchangeable structure should match the data-generating process best since we used a random intercept.

### Examine Working Correlations

```r
# Estimated correlation parameter
fit_exch$geese$alpha   # Exchangeable: single α
fit_ar1$geese$alpha    # AR(1): single α
```

---

## Step 5: Robust vs. Naive SEs

GEE reports robust (sandwich) SEs by default in `geepack`. Let's compare them to naive (model-based) SEs.

```r
# Extract both SE types
robust_se <- summary(fit_exch)$coefficients[, "Std.err"]

# Naive SEs from the model-based covariance
naive_cov <- fit_exch$geese$vbeta.naiv
naive_se  <- sqrt(diag(naive_cov))

comparison <- tibble(
  term    = names(coef(fit_exch)),
  robust  = round(robust_se, 4),
  naive   = round(naive_se, 4),
  ratio   = round(robust_se / naive_se, 3)
)
comparison
```

**Interpreting the ratio:**
- Ratio ≈ 1.0: Working correlation is approximately correct
- Ratio > 1.2: Working correlation is misspecified — robust SEs are wider (protecting you)
- Ratio < 0.8: Unusual — may indicate small-sample issues

✅ **Checkpoint**: Since the data were generated with exchangeable-like correlation and we fit exchangeable, the ratio should be close to 1.0.

---

## Step 6: Compare GEE and GLMM

This is the key comparison: marginal (GEE) vs. conditional (GLMM) estimates for the same data.

### Fit GLMM

```r
fit_glmm <- glmer(y ~ time + treatment + (1 | id),
                   data = df, family = binomial)
```

### Side-by-Side Comparison

```r
tibble(
  term      = names(coef(fit_exch)),
  GEE_est   = round(coef(fit_exch), 3),
  GEE_se    = round(summary(fit_exch)$coefficients[, "Std.err"], 3),
  GLMM_est  = round(fixef(fit_glmm), 3),
  GLMM_se   = round(summary(fit_glmm)$coefficients[, "Std. Error"], 3),
  ratio     = round(coef(fit_exch) / fixef(fit_glmm), 3)
)
```

**Expected pattern:**
- GEE coefficients should be **smaller in magnitude** than GLMM coefficients
- The attenuation ratio is approximately c* = 1/√(1 + c²σ²), where σ² is the random intercept variance and c ≈ 16√3/(15π)
- With σ² = 1.0: ratio ≈ 0.84 for logistic models

✅ **Checkpoint**: GEE estimates should be approximately 80–85% of GLMM estimates. This isn't bias — they target different quantities.

### Predicted Probabilities

```r
# GEE: marginal predictions
pred_grid <- expand.grid(time = 0:4, treatment = c(0, 1))
pred_grid$gee_prob <- predict(fit_exch, newdata = pred_grid, type = "response")

# GLMM: conditional predictions (at RE = 0)
pred_grid$glmm_prob <- predict(fit_glmm, newdata = pred_grid,
                                type = "response", re.form = NA)

pred_grid %>%
  mutate(group = factor(treatment, labels = c("Control", "Treatment"))) %>%
  pivot_longer(c(gee_prob, glmm_prob), names_to = "model", values_to = "prob") %>%
  mutate(model = ifelse(model == "gee_prob", "GEE (marginal)", "GLMM (conditional)")) %>%
  ggplot(aes(x = time, y = prob, color = group, linetype = model)) +
  geom_line(linewidth = 1) +
  geom_point(size = 2) +
  scale_y_continuous(limits = c(0, 1), labels = scales::percent) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Predicted Probability",
       title = "GEE vs. GLMM Predicted Probabilities",
       color = "Group", linetype = "Model") +
  theme_minimal()
```

✅ **Checkpoint**: GEE and GLMM predictions at RE = 0 will be similar but not identical. The GEE curves represent population averages; the GLMM curves represent the "average individual."

---

## Step 7: Interpret Results

### GEE Results Summary

```r
summary(fit_exch)
```

| Parameter | Estimate | Robust SE | OR | Interpretation |
|-----------|----------|-----------|-----|----------------|
| Intercept | ~-0.42 | ~0.14 | ~0.66 | Population odds at Wave 1 (control) |
| Time | ~-0.25 | ~0.04 | ~0.78 | Per-wave change in population odds |
| Treatment | ~-0.67 | ~0.19 | ~0.51 | Treatment vs control population odds |

*Values are illustrative; yours will differ with the random seed.*

**Key interpretations** (marginal, population-averaged):

- **Time OR ≈ 0.78**: Across the population, the odds of the outcome decrease by ~22% per wave
- **Treatment OR ≈ 0.51**: The treatment group has ~49% lower odds than controls, on average across the population
- These describe **population trends**, not individual trajectories

### Written Summary

> We estimated a GEE model with exchangeable working correlation for a binary longitudinal outcome (N = 200, 5 waves). Using robust standard errors, the population-averaged odds of the outcome decreased over time (OR = 0.78, 95% CI [0.72, 0.85], p < .001) and were lower in the treatment group (OR = 0.51, 95% CI [0.35, 0.74], p < .001). The estimated within-person correlation was α = 0.22. Robust and naive SEs were in close agreement, supporting the exchangeable working correlation specification. Marginal estimates were attenuated relative to conditional GLMM estimates by a factor of ~0.84, consistent with the random intercept variance (σ² ≈ 1.0).

---

## Complete Script

```r
# ============================================
# GEE Complete Worked Example
# ============================================

library(tidyverse)
library(geepack)
library(lme4)
library(MASS)
set.seed(2024)

# --- Simulate data ---
n <- 200; waves <- 5; N <- n * waves

person_df <- tibble(
  id = 1:n,
  treatment = rep(c(0, 1), each = n / 2),
  ri = rnorm(n, 0, 1.0)
)

df <- person_df %>%
  crossing(time = 0:(waves - 1)) %>%
  arrange(id, time) %>%
  mutate(
    eta  = -0.5 + (-0.3 * time) + (-0.8 * treatment) + ri,
    prob = plogis(eta),
    y    = rbinom(N, 1, prob)
  )

# --- Fit GEE with different structures ---
fit_exch  <- geeglm(y ~ time + treatment, id = id, data = df,
                     family = binomial, corstr = "exchangeable")
fit_ar1   <- geeglm(y ~ time + treatment, id = id, data = df,
                     family = binomial, corstr = "ar1")
fit_indep <- geeglm(y ~ time + treatment, id = id, data = df,
                     family = binomial, corstr = "independence")

# --- Compare structures ---
summary(fit_exch)
summary(fit_ar1)

# --- Robust vs naive SEs ---
robust_se <- summary(fit_exch)$coefficients[, "Std.err"]
naive_se  <- sqrt(diag(fit_exch$geese$vbeta.naiv))
data.frame(term = names(coef(fit_exch)),
           robust = round(robust_se, 4),
           naive = round(naive_se, 4),
           ratio = round(robust_se / naive_se, 3))

# --- Compare GEE vs GLMM ---
fit_glmm <- glmer(y ~ time + treatment + (1 | id),
                   data = df, family = binomial)

data.frame(term = names(coef(fit_exch)),
           GEE = round(coef(fit_exch), 3),
           GLMM = round(fixef(fit_glmm), 3),
           ratio = round(coef(fit_exch) / fixef(fit_glmm), 3))

# --- ORs ---
exp(coef(fit_exch))
exp(confint(fit_exch))
```

---

## Adapt to Your Data

### 1. Read and Prepare

```r
df <- read_csv("my_data.csv") %>%
  mutate(id = factor(id), time = wave - 1) %>%
  arrange(id, time)  # GEE requires data sorted by cluster
```

> [!note] **Data must be sorted by cluster (id)**
>
> `geeglm` requires rows to be grouped by the clustering variable. If data aren't sorted, results will be wrong without warning.

### 2. Choose Family and Correlation

| Outcome | Family | Typical Correlation |
|---------|--------|-------------------|
| Binary | `binomial` | Exchangeable or AR(1) |
| Count | `poisson` | Exchangeable or AR(1) |
| Continuous | `gaussian` | AR(1) or unstructured |

### 3. Fit and Examine

```r
fit <- geeglm(outcome ~ time + group, id = id, data = df,
              family = binomial, corstr = "exchangeable")
summary(fit)
exp(coef(fit))        # ORs
exp(confint(fit))     # OR CIs
```

✅ **Checkpoint**: Always compare robust and naive SEs. Always sort data by cluster before fitting.

---

## Next Steps

- **Need syntax or thresholds?** → [Reference](/guides/gee-reference)
- **Back to overview** → [GEE Guide](/guides/gee)
- **Want individual-level effects?** → [GLMM Guide](/guides/glmm)
