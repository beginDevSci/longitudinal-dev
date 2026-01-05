---
title: "LGCM Reference"
slug: "lgcm-pilot-reference"
description: "Quick lookup for LGCM syntax, fit indices, parameters, and troubleshooting."
category: "growth-models"
tags: ["LGCM", "reference", "lavaan", "cheat-sheet"]
r_packages: ["lavaan"]
guide_type: "reference"
parent_method: "lgcm-pilot"
---

## LGCM Quick Reference

Fast lookup for syntax, fit indices, and troubleshooting. For step-by-step learning, see the [Tutorial](/guides/lgcm-pilot-tutorial).

**Jump to:** [Syntax](#lavaan-syntax) · [Estimation](#estimation--missing-data) · [Extract Output](#extract-output) · [Model Comparison](#model-comparison) · [Fit Indices](#fit-indices) · [Errors & Fixes](#common-errors--fixes) · [Extensions](#advanced-extensions) · [Resources](#resources)

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

### Correlated Adjacent Residuals

```r
y1 ~~ y2
y2 ~~ y3
y3 ~~ y4
y4 ~~ y5
```

### Multigroup Growth

```r
fit <- growth(model, data = data_wide, group = "group_var")
```

---

## Estimation & Missing Data

| Scenario | Estimator | Code |
|----------|-----------|------|
| Complete data | ML (default) | `growth(model, data)` |
| Missing data (MAR) | FIML | `missing = "fiml"` |
| Non-normal data | MLR | `estimator = "MLR"` |
| Ordinal outcomes | WLSMV | `estimator = "WLSMV"` |

**FIML for missing data:**
```r
fit <- growth(model, data = data_wide, missing = "fiml")
```

**Robust SEs (non-normality):**
```r
fit <- growth(model, data = data_wide, estimator = "MLR")
```

**Ordinal outcomes:**
```r
fit <- growth(model, data = data_wide, estimator = "WLSMV")
```

---

## Extract Output

### Fit Indices

```r
fitmeasures(fit, c("chisq", "df", "pvalue", "cfi", "rmsea", "srmr"))
```

### Growth Factor Means

```r
parameterEstimates(fit) %>%
  filter(op == "~1", lhs %in% c("intercept", "slope"))
```

### Growth Factor Variances/Covariances

```r
parameterEstimates(fit) %>%
  filter(op == "~~", lhs %in% c("intercept", "slope"),
         rhs %in% c("intercept", "slope"))
```

### Standardized Estimates

```r
parameterEstimates(fit, standardized = TRUE) %>%
  select(lhs, op, rhs, est, se, std.all)
```

### R² (Variance Explained)

```r
inspect(fit, "r2")
```

### Residual Correlations

```r
resid(fit, type = "cor")$cov %>% round(3)
```

### Modification Indices

```r
modindices(fit, sort = TRUE, minimum.value = 10)
```

---

## Model Comparison

### Nested Models (LRT)

```r
anova(fit_constrained, fit_full)
```

### Robust LRT (for MLR fits)

```r
anova(fit1, fit2, method = "satorra.bentler.2001")
```

### Information Criteria

```r
AIC(fit1); AIC(fit2)  # Lower = better
BIC(fit1); BIC(fit2)  # Lower = better
```

---

## Fit Indices

| Index | Good | Acceptable | Interpretation |
|-------|------|------------|----------------|
| χ² p-value | > .05 | > .01 | Non-significant = adequate fit |
| CFI | ≥ .95 | ≥ .90 | Comparative fit (vs. null model) |
| RMSEA | ≤ .06 | ≤ .08 | Population misfit estimate |
| SRMR | ≤ .08 | ≤ .10 | Avg. residual correlation |

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

---

## Time Coding

| Centering | Loadings | Intercept = |
|-----------|----------|-------------|
| Wave 1 (default) | 0, 1, 2, 3, 4 | Score at Wave 1 |
| Midpoint | -2, -1, 0, 1, 2 | Score at Wave 3 |
| Final wave | -4, -3, -2, -1, 0 | Score at Wave 5 |
| Actual months | 0, 3, 6, 12, 24 | Score at baseline |

Slope interpretation unchanged by centering; intercept moves to the zero-point.

---

## Common Errors & Fixes

| Issue | Symptom | Fix |
|-------|---------|-----|
| **Negative variance** | `slope variance = -2.4` | Constrain to 0 or simplify model |
| **Non-convergence** | "Model did not converge" | Check N, outliers; provide start values |
| **Huge SEs** | SE > estimate | Parameter poorly identified; simplify |
| **Poor fit** | CFI < .90, RMSEA > .10 | Check linearity; consider quadratic |
| **Variable name mismatch** | lavaan error about variables | Use `names(data)` to verify spelling |

---

## Troubleshooting

### Non-Positive Definite Matrix

- Very high correlations between time points
- Variance estimates near zero
- Try constraining residual variances equal

### Negative Variance (Heywood Case)

```r
slope ~~ 0*slope           # Constrain to zero
intercept ~~ 0*slope       # Also fix covariance
```

### Non-Convergence

- Increase iterations: `control = list(iter.max = 10000)`
- Check sample size relative to model complexity
- Simplify model (fewer free parameters)

### Flag Problematic Estimates

```r
parameterEstimates(fit) %>%
  filter(op == "~~", est < 0)
```

---

## Interpretation Pitfalls

| Mistake | Reality |
|---------|---------|
| "Everyone improved" (slope mean = 2) | Check slope variance—many may have slope ≤ 0 |
| "High starters declined" (negative I-S corr) | Negative correlation = slower growth, not decline |
| "Good fit = correct model" | Good fit ≠ true model; consider alternatives |
| Comparing intercepts across studies | Intercept meaning depends on time coding |

For detailed examples and fixes, see [LGCM Overview](/guides/lgcm-pilot).

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

**Degrees of freedom:**
```r
fitmeasures(fit, "df")
```

---

## Advanced Extensions

### Time-Invariant Predictors (TIC)

Baseline characteristics predicting growth:

```r
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  intercept ~ treatment + age
  slope     ~ treatment + age
'
```

| Coefficient | Interpretation |
|-------------|----------------|
| `intercept ~ treatment` | Group difference in starting level |
| `slope ~ treatment` | Group difference in rate of change |

### Time-Varying Covariates (TVC)

Concurrent predictors at each wave:

```r
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  y1 ~ x1
  y2 ~ x2
  y3 ~ x3
  y4 ~ x4
  y5 ~ x5
'
```

### Other Extensions

| Extension | Description | Notes |
|-----------|-------------|-------|
| Growth Mixture Models | Latent subgroups with different trajectories | Complex; requires specialized packages |
| Parallel Process | Two outcomes growing together | Relate intercepts/slopes across processes |
| Latent Change Score | Change between adjacent waves | Tests dynamic coupling |

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

## Resources

### Books

| Book | Focus |
|------|-------|
| Grimm, Ram, & Estabrook (2017). *Growth Modeling*. Guilford. | Comprehensive SEM/MLM coverage |
| Bollen & Curran (2006). *Latent Curve Models*. Wiley. | Classic SEM treatment |
| Singer & Willett (2003). *Applied Longitudinal Data Analysis*. Oxford. | MLM perspective, excellent pedagogy |
| Little (2013). *Longitudinal Structural Equation Modeling*. Guilford. | Measurement issues |

### Key Articles

| Article | Contribution |
|---------|--------------|
| Curran, Obeidat, & Losardo (2010). *J Cognition & Development* | 12 FAQs comparing LGCM and MLM |
| McNeish & Matta (2018). *Behavior Research Methods* | When approaches differ in practice |
| Preacher et al. (2008). *Latent growth curve modeling*. Sage. | Centering and time coding guidance |

### Online

- [lavaan tutorial](https://lavaan.ugent.be/tutorial/growth.html)
- [QuantDev tutorials](https://quantdev.ssri.psu.edu/)
- [Curran-Bauer Analytics](https://www.youtube.com/@curranbauer) (YouTube)

---

## Software

### R Packages

| Package | Use Case |
|---------|----------|
| **lavaan** | Primary choice; free, flexible, active development |
| **OpenMx** | Maximum flexibility; steeper learning curve |
| **lme4 / nlme** | MLM approach; equivalent for basic models |
| **semTools** | lavaan extensions (measurement invariance) |
| **lcmm** | Latent class / mixture growth models |

### Other Options

| Software | Notes |
|----------|-------|
| **Mplus** | Gold standard for complex/mixture models; licensed |
| **Stata (sem)** | Good documentation; integrates with Stata workflow |
| **semopy** (Python) | SEM in Python; active development |

**Recommendation:** Use lavaan for learning and most research. Use Mplus for mixture models or complex specifications.

---

## Links

- [Tutorial](/guides/lgcm-pilot-tutorial) — Step-by-step worked example
- [LGCM Overview](/guides/lgcm-pilot) — When to use, key concepts, mathematical notation
- [lavaan documentation](https://lavaan.ugent.be/tutorial/growth.html) — Official guide
