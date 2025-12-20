---
title: "LGCM Tutorial: Worked Example"
slug: "lgcm-pilot-tutorial"
description: "Step-by-step LGCM analysis from setup to interpretation using simulated data in R."
category: "growth-models"
tags: ["LGCM", "tutorial", "lavaan", "R"]
r_packages: ["lavaan", "tidyverse", "MASS"]
---

## LGCM Tutorial

This tutorial walks through a complete LGCM analysis. By the end, you will have:

1. Simulated longitudinal data with known parameters
2. Visualized individual trajectories
3. Fit and compared growth models
4. Interpreted the results

**Time**: ~20 minutes if running code along the way.

---

## Workflow Overview

```
Setup → Simulate Data → Visualize → Fit Models → Compare → Interpret
```

---

## Step 1: Setup

```r
# Load packages
library(tidyverse)  # Data manipulation and plotting
library(lavaan)     # SEM and growth models
library(MASS)       # For mvrnorm (simulation)

# Set seed for reproducibility
set.seed(2024)
```

---

## Step 2: Simulate Data

We'll create data with **known population parameters** so we can verify our model recovers them:

| Parameter | True Value |
|-----------|------------|
| Intercept mean | 50 |
| Slope mean | 2 |
| Intercept variance | 100 (SD = 10) |
| Slope variance | 1 (SD = 1) |
| I-S covariance | -2 (r ≈ -0.20) |
| Residual variance | 25 (SD = 5) |

```r
# Sample size and time points
n <- 400
time_points <- 0:4

# Latent factor covariance matrix
psi <- matrix(c(100, -2,
                -2,   1), nrow = 2)

# Generate individual intercepts and slopes
factors <- mvrnorm(n = n,
                   mu = c(50, 2),
                   Sigma = psi)

# Generate observed data
data_wide <- tibble(id = 1:n) %>%
  mutate(
    int_i = factors[, 1],
    slp_i = factors[, 2],
    y1 = int_i + slp_i * 0 + rnorm(n, 0, 5),
    y2 = int_i + slp_i * 1 + rnorm(n, 0, 5),
    y3 = int_i + slp_i * 2 + rnorm(n, 0, 5),
    y4 = int_i + slp_i * 3 + rnorm(n, 0, 5),
    y5 = int_i + slp_i * 4 + rnorm(n, 0, 5)
  ) %>%
  select(id, y1:y5)

# Check structure
head(data_wide)
```

✅ **Checkpoint**: You should see a tibble with 6 rows and columns `id`, `y1`–`y5`. Values will differ due to random generation, but should be in the 30–70 range.

---

## Step 3: Visualize

**Always plot before modeling.**

```r
# Reshape for plotting
data_long <- data_wide %>%
  pivot_longer(y1:y5, names_to = "wave", values_to = "y") %>%
  mutate(time = as.numeric(gsub("y", "", wave)) - 1)

# Spaghetti plot
ggplot(data_long, aes(x = time, y = y, group = id)) +
  geom_line(alpha = 0.15, color = "gray40") +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Score",
       title = "Individual Growth Trajectories",
       subtitle = "N = 400 participants, 5 waves") +
  theme_minimal()
```

**What to look for:**
- General trend (up, down, flat?)
- Spread at baseline (intercept variance)
- Fan pattern (slope variance)
- Nonlinearity (curves vs. straight lines)

```r
# Add mean trajectory
mean_traj <- data_long %>%
  group_by(time) %>%
  summarise(mean_y = mean(y))

ggplot(data_long, aes(x = time, y = y)) +
  geom_line(aes(group = id), alpha = 0.1, color = "gray40") +
  geom_line(data = mean_traj, aes(y = mean_y),
            color = "steelblue", linewidth = 1.5) +
  geom_point(data = mean_traj, aes(y = mean_y),
             color = "steelblue", size = 3) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Score",
       title = "Individual Trajectories with Mean Overlay") +
  theme_minimal()
```

![Individual Trajectories with Mean](/images/guides/lgcm/fig02_spaghetti_mean.png)

✅ **Checkpoint**: Your plot should show upward-trending lines with visible spread at baseline and a "fan" pattern over time.

---

## Step 4: Fit Models

### Intercept-Only Model (Baseline)

Fit a model with no growth—everyone has a stable mean:

```r
model_intercept <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
'

fit_intercept <- growth(model_intercept, data = data_wide,
                        missing = "fiml")
```

### Linear Growth Model

Add the slope factor:

```r
model_linear <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit_linear <- growth(model_linear, data = data_wide,
                     missing = "fiml")
```

*Note: We specify `missing = "fiml"` explicitly—good practice even when data are complete.*

### Linear Growth with Equal Residuals

Test whether residual variances can be constrained equal:

```r
model_linear_eq <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  # Constrain residual variances to equality
  y1 ~~ rv*y1
  y2 ~~ rv*y2
  y3 ~~ rv*y3
  y4 ~~ rv*y4
  y5 ~~ rv*y5
'

fit_linear_eq <- growth(model_linear_eq, data = data_wide,
                        missing = "fiml")
```

---

## Step 5: Compare Models

### Is there growth?

```r
anova(fit_intercept, fit_linear)
```

✅ **Confirm**: Δχ² should be significant (p < .001). This means linear growth improves fit over a flat trajectory.

### Are equal residual variances justified?

```r
anova(fit_linear_eq, fit_linear)
```

✅ **Confirm**: If p > .05, the simpler model (equal residuals) is justified.

### Information criteria

```r
data.frame(
  Model = c("Intercept-only", "Linear", "Linear (equal resid)"),
  AIC = c(AIC(fit_intercept), AIC(fit_linear), AIC(fit_linear_eq)),
  BIC = c(BIC(fit_intercept), BIC(fit_linear), BIC(fit_linear_eq))
) %>%
  mutate(across(where(is.numeric), \(x) round(x, 1)))
```

✅ **Confirm**: Lower AIC/BIC = better fit. Linear models should beat intercept-only.

---

## Step 6: Check Diagnostics

```r
# Fit indices
fitmeasures(fit_linear, c("chisq", "df", "pvalue", "cfi", "rmsea", "srmr"))
```

✅ **Check fit indices**:
- CFI > .95 ✓
- RMSEA < .08 ✓
- SRMR < .08 ✓

See [Reference](/guides/lgcm-pilot-reference#fit-indices) for detailed interpretation.

```r
# Check for problematic estimates (negative variances)
parameterEstimates(fit_linear) %>%
  filter(op == "~~") %>%
  select(lhs, rhs, est, se) %>%
  mutate(flag = ifelse(est < 0, "NEGATIVE", ""))
```

✅ **Confirm**: All variance estimates should be positive. If any are negative, see [Troubleshooting](/guides/lgcm-pilot-reference#troubleshooting).

---

## Step 7: Interpret Results

```r
summary(fit_linear, fit.measures = TRUE, standardized = TRUE)
```

### Compare Estimates to True Values

| Parameter | Estimate | SE | True Value |
|-----------|----------|-----|------------|
| Intercept mean | 49.82 | 0.51 | 50 |
| Slope mean | 2.04 | 0.06 | 2 |
| Intercept variance | 98.34 | 8.12 | 100 |
| Slope variance | 0.97 | 0.12 | 1 |
| I-S covariance | -1.89 | 0.62 | -2 |
| Residual variance | 24.56 | 1.23 | 25 |

✅ **Success**: Estimates closely recover the true population parameters. This confirms the model is working correctly.

### Interpret Each Parameter

- **Intercept mean ≈ 50**: Average score at Wave 1
- **Slope mean ≈ 2**: Average increase per wave
- **Intercept variance ≈ 100** (SD ≈ 10): People differed in starting levels
- **Slope variance ≈ 1** (SD ≈ 1): People differed in growth rates
- **I-S covariance ≈ -2** (r ≈ -0.19): Higher starters grew slightly slower

### Variance Explained

```r
inspect(fit_linear, "r2")
```

✅ **Check**: R² values of 0.80–0.89 indicate the trajectory explains most variance at each wave.

---

## Written Summary

> We estimated a linear latent growth model for 400 participants across 5 waves. The model fit well (χ²(10) = 12.35, p = .26; CFI = 0.998; RMSEA = 0.024; SRMR = 0.025).
>
> On average, participants started at 49.82 (SE = 0.51) and increased by 2.04 units per wave (SE = 0.06), both p < .001. Significant individual differences emerged in starting levels (variance = 98.34, SD ≈ 10) and growth rates (variance = 0.97, SD ≈ 1). The negative intercept-slope covariance (-1.89, p = .002, r ≈ -0.19) indicates that participants who started higher grew slightly slower.

---

## Complete Script

```r
# ============================================
# LGCM Complete Worked Example
# ============================================

library(tidyverse)
library(lavaan)
library(MASS)
set.seed(2024)

# Simulate data
n <- 400
psi <- matrix(c(100, -2, -2, 1), nrow = 2)
factors <- mvrnorm(n, mu = c(50, 2), Sigma = psi)

data_wide <- tibble(id = 1:n) %>%
  mutate(
    int = factors[,1], slp = factors[,2],
    y1 = int + slp*0 + rnorm(n, 0, 5),
    y2 = int + slp*1 + rnorm(n, 0, 5),
    y3 = int + slp*2 + rnorm(n, 0, 5),
    y4 = int + slp*3 + rnorm(n, 0, 5),
    y5 = int + slp*4 + rnorm(n, 0, 5)
  ) %>% select(id, y1:y5)

# Fit models
model_int <- 'intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5'
model_lin <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit_int <- growth(model_int, data = data_wide, missing = "fiml")
fit_lin <- growth(model_lin, data = data_wide, missing = "fiml")

# Compare and summarize
anova(fit_int, fit_lin)
summary(fit_lin, fit.measures = TRUE, standardized = TRUE)
```

---

## Next Steps

- **Need syntax or thresholds?** → [Reference](/guides/lgcm-pilot-reference)
- **Hit an error?** → [Reference: Troubleshooting](/guides/lgcm-pilot-reference#common-pitfalls)
- **Back to overview** → [LGCM Guide](/guides/lgcm-pilot)
