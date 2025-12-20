---
title: "LGCM Reference"
slug: "lgcm-pilot-reference"
description: "Quick lookup for LGCM syntax, fit indices, parameters, and troubleshooting."
category: "growth-models"
tags: ["LGCM", "reference", "lavaan", "cheat-sheet"]
r_packages: ["lavaan"]
---

## LGCM Quick Reference

Fast lookup for syntax, fit indices, and troubleshooting. For step-by-step learning, see the [Tutorial](/guides/lgcm-pilot-tutorial).

---

## lavaan Syntax

### Basic Linear Growth

```r
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'
fit <- growth(model, data = data_wide)
summary(fit, fit.measures = TRUE, standardized = TRUE)
```

### Equal Residual Variances

```r
y1 ~~ rv*y1
y2 ~~ rv*y2
y3 ~~ rv*y3
y4 ~~ rv*y4
y5 ~~ rv*y5
```

### Fixed Slope (No Variance)

```r
slope ~~ 0*slope
intercept ~~ 0*slope
```

### Predictor of Growth

```r
intercept ~ predictor
slope ~ predictor
```

### Quadratic Growth

```r
intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
quad      =~ 0*y1 + 1*y2 + 4*y3 + 9*y4 + 16*y5
```

### Piecewise Growth

```r
intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
slope1    =~ 0*y1 + 1*y2 + 2*y3 + 2*y4 + 2*y5
slope2    =~ 0*y1 + 0*y2 + 0*y3 + 1*y4 + 2*y5
```

### Latent Basis (Freed Loadings)

```r
intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
slope     =~ 0*y1 + y2 + y3 + y4 + 1*y5
```

---

## Fit Indices

| Index | Good | Acceptable | Interpretation |
|-------|------|------------|----------------|
| χ² p-value | > .05 | > .01 | Non-significant = adequate fit |
| CFI | ≥ .95 | ≥ .90 | Comparative fit (vs. null model) |
| RMSEA | ≤ .06 | ≤ .08 | Population misfit estimate |
| SRMR | ≤ .08 | ≤ .10 | Avg. residual correlation |

**Extract in lavaan:**
```r
fitmeasures(fit, c("chisq", "df", "pvalue", "cfi", "rmsea", "srmr"))
```

---

## Parameters

| Parameter | Symbol | Interpretation |
|-----------|--------|----------------|
| Intercept mean | μᵢ | Average score at Time = 0 |
| Slope mean | μₛ | Average change per time unit |
| Intercept variance | ψᵢᵢ | Individual differences in starting level |
| Slope variance | ψₛₛ | Individual differences in change rate |
| I-S covariance | ψᵢₛ | Start-change relationship |
| Residual variance | θ | Unexplained variance at each wave |

**Extract in lavaan:**
```r
parameterEstimates(fit) %>%
  filter(op %in% c("~1", "~~"))
```

---

## Time Coding

| Centering | Loadings | Intercept = |
|-----------|----------|-------------|
| Wave 1 (default) | 0, 1, 2, 3, 4 | Score at Wave 1 |
| Midpoint | -2, -1, 0, 1, 2 | Score at Wave 3 |
| Final wave | -4, -3, -2, -1, 0 | Score at Wave 5 |
| Actual months | 0, 3, 6, 12, 24 | Score at baseline |

**The slope is always "change per unit"**—centering only moves the intercept.

---

## Requirements

| Aspect | Minimum | Recommended |
|--------|---------|-------------|
| Time points | 3 | 4+ |
| Sample size (simple) | 100 | 150+ |
| Sample size (complex) | 200 | 300+ |
| Data format | Wide | Wide |

---

## Distributional Assumptions

ML estimation assumes multivariate normality. Check before fitting:

| Check | Acceptable | Action if Violated |
|-------|------------|-------------------|
| Skewness | \|skew\| < 2 | Use MLR estimator |
| Kurtosis | \|kurt\| < 7 | Use MLR estimator |
| Outliers | Few, not extreme | Investigate; consider robust estimation |
| Floor/ceiling effects | Minimal | May need transformed outcomes |

**Using robust estimation in lavaan:**
```r
fit <- growth(model, data = data_wide, estimator = "MLR")
```

MLR provides robust standard errors and Satorra-Bentler scaled χ² (use `estimator = "MLR"` or `se = "robust"`).

**For categorical outcomes** (ordinal with few categories):
```r
fit <- growth(model, data = data_wide, estimator = "WLSMV")
```

---

## Model Comparison

```r
# Nested models (likelihood ratio test)
anova(fit_constrained, fit_full)

# With robust SEs
anova(fit1, fit2, method = "satorra.bentler.2001")

# Any models (information criteria)
AIC(fit1); AIC(fit2)  # Lower = better
BIC(fit1); BIC(fit2)  # Lower = better
```

---

## Common Pitfalls

| Issue | Symptom | Fix |
|-------|---------|-----|
| **Negative variance** | `slope variance = -2.4` | Constrain to 0 or simplify model |
| **Non-convergence** | "Model did not converge" | Check N, outliers; provide start values |
| **Huge SEs** | SE > estimate | Parameter poorly identified; simplify |
| **Poor fit** | CFI < .90, RMSEA > .10 | Check linearity; consider quadratic |
| **Variable name mismatch** | lavaan error about variables | Use `names(data)` to verify spelling |

---

## Troubleshooting

### "Covariance matrix not positive definite"

- Check for very high correlations between time points
- Verify variance estimates aren't near zero
- Consider constraining residual variances equal

### Negative Variance Estimate

```r
# Option 1: Constrain to zero
slope ~~ 0*slope

# Option 2: Constrain to small positive
slope ~~ 0.001*slope
```

### Non-Convergence

```r
# Provide starting values
fit <- growth(model, data = data_wide,
              start = list(intercept ~ 50, slope ~ 2))
```

### Checking for Problems

```r
# Modification indices (large = localized misfit)
modindices(fit, sort = TRUE, minimum.value = 10)

# Residual correlations (should be near 0)
resid(fit, type = "cor")$cov %>% round(3)

# Flag negative variances
parameterEstimates(fit) %>%
  filter(op == "~~", est < 0)
```

---

## Diagnostics Checklist

**Before running:**
- [ ] Data in wide format
- [ ] Variable names match syntax
- [ ] Time coding reflects actual spacing
- [ ] Trajectories visualized

**Before reporting:**
- [ ] Model fit indices reported
- [ ] All parameters with SEs
- [ ] Variances interpreted (not just means)
- [ ] Time coding stated

---

## Interpretation Pitfalls

| Mistake | Reality |
|---------|---------|
| "Everyone improved" (slope mean = 2) | Check slope variance—many may have slope ≤ 0 |
| "High starters declined" (negative I-S corr) | Negative correlation = slower growth, not decline |
| "Good fit = correct model" | Good fit ≠ true model; consider alternatives |
| Comparing intercepts across studies | Intercept meaning depends on time coding |

---

## Quick Formulas

**Intercept-slope correlation:**
```r
r_is <- cov_is / sqrt(var_i * var_s)
```

**Proportion with positive slopes:**
```r
pnorm(0, mean = slope_mean, sd = sqrt(slope_var), lower.tail = FALSE)
```

**Degrees of freedom (T waves, free residuals):**
```
df = T(T+1)/2 - 5
```

---

## Path Diagram

```
              ┌─────────────┐          ┌─────────────┐
              │  Intercept  │          │    Slope    │
              │     (I)     │◄────────►│     (S)     │
              └──────┬──────┘          └──────┬──────┘
                     │                        │
     ┌───────┬───────┼───────┬───────┐       │
     │1      │1      │1      │1      │1      │
     ▼       ▼       ▼       ▼       ▼       │
   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐    │
   │y1 │   │y2 │   │y3 │   │y4 │   │y5 │    │
   └─┬─┘   └─┬─┘   └─┬─┘   └─┬─┘   └─┬─┘    │
     ▲       ▲       ▲       ▲       ▲       │
     │0      │1      │2      │3      │4      │
     └───────┴───────┴───────┴───────┴───────┘

Intercept loadings: all = 1
Slope loadings: 0, 1, 2, 3, 4 (encode time)
```

---

## Links

- [Tutorial](/guides/lgcm-pilot-tutorial) — Step-by-step worked example
- [LGCM Overview](/guides/lgcm-pilot) — When to use, key concepts
- [lavaan documentation](https://lavaan.ugent.be/tutorial/growth.html) — Official guide
