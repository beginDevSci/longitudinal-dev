---
title: "GEE Reference"
slug: "gee-reference"
description: "Quick lookup for GEE syntax, correlation structures, QIC, robust SEs, and troubleshooting."
category: "mixed-models"
tags: ["GEE", "quick-reference", "geepack", "syntax"]
r_packages: ["geepack"]
guide_type: "reference"
parent_method: "gee"
---

## GEE Quick Reference

Fast lookup for syntax, correlation structures, and troubleshooting. For step-by-step learning, see the [Walkthrough](/guides/gee-walkthrough). For conceptual background, see the [Overview](/guides/gee).

**Jump to:** [Syntax](#syntax) · [Correlation Structures](#correlation-structures) · [Extract Output](#extract-output) · [Model Comparison](#model-comparison-qic) · [Robust SEs](#robust-vs-naive-ses) · [Diagnostics](#diagnostics) · [Errors & Fixes](#common-errors--fixes) · [Extensions](#extensions) · [Resources](#resources)

---

## Syntax

### geepack::geeglm

```r
library(geepack)

# Binary — exchangeable correlation
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = binomial, corstr = "exchangeable")

# Binary — AR(1)
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = binomial, corstr = "ar1")

# Binary — independence
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = binomial, corstr = "independence")

# Binary — unstructured
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = binomial, corstr = "unstructured")

# Count — Poisson with exchangeable
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = poisson, corstr = "exchangeable")

# Continuous — Gaussian with AR(1)
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = gaussian, corstr = "ar1")
```

> [!note] **Data must be sorted by cluster**
>
> `geeglm` requires rows sorted by the `id` variable. Use `arrange(id, time)` before fitting. Unsorted data produce wrong results without warning.

### Interaction Terms

```r
# Time × treatment
fit <- geeglm(y ~ time * treatment, id = id, data = df,
              family = binomial, corstr = "exchangeable")

# Polynomial time
fit <- geeglm(y ~ time + I(time^2) + treatment, id = id, data = df,
              family = binomial, corstr = "exchangeable")
```

### Offset (Exposure)

```r
fit <- geeglm(events ~ time + treatment, id = id, data = df,
              family = poisson, corstr = "exchangeable",
              offset = log(exposure_time))
```

---

## Correlation Structures

### Structure Comparison

| Structure | corstr | Pattern | Parameters | Best for |
|-----------|--------|---------|------------|----------|
| Independence | `"independence"` | No correlation | 0 | Baseline; weak correlation |
| Exchangeable | `"exchangeable"` | All pairs equal | 1 | Cluster-like data; default |
| AR(1) | `"ar1"` | Decay with lag | 1 | Time series; temporal decay |
| Unstructured | `"unstructured"` | All pairs free | T(T−1)/2 | Few time points; many clusters |

### Correlation Matrices

```
Independence:       Exchangeable:       AR(1):              Unstructured:
[1 0 0 0]          [1 α α α]          [1  α  α² α³]       [1   r₁₂ r₁₃ r₁₄]
[0 1 0 0]          [α 1 α α]          [α  1  α  α²]       [r₁₂ 1   r₂₃ r₂₄]
[0 0 1 0]          [α α 1 α]          [α² α  1  α ]       [r₁₃ r₂₃ 1   r₃₄]
[0 0 0 1]          [α α α 1]          [α³ α² α  1 ]       [r₁₄ r₂₄ r₃₄ 1  ]
```

### Parameter Count by Time Points

| T (waves) | Independence | Exchangeable | AR(1) | Unstructured |
|-----------|-------------|-------------|-------|-------------|
| 3 | 0 | 1 | 1 | 3 |
| 5 | 0 | 1 | 1 | 10 |
| 8 | 0 | 1 | 1 | 28 |
| 10 | 0 | 1 | 1 | 45 |

### How to Choose

1. Start with **exchangeable** — safe default
2. If temporal decay is plausible, try **AR(1)**
3. Compare using **QIC** (see below)
4. Check robust vs naive SE agreement — large discrepancy = poor fit
5. Use **unstructured** only with ≤ 5 time points and ≥ 100 clusters

### Extract Estimated Correlation

```r
# Correlation parameter(s)
fit$geese$alpha

# Full estimated working correlation matrix
# (not directly available; reconstruct from alpha and corstr)
```

---

## Extract Output

### Coefficients

```r
coef(fit)
summary(fit)$coefficients
```

### Odds Ratios / Incidence Rate Ratios

```r
# ORs (binary) or IRRs (count)
exp(coef(fit))

# With confidence intervals
exp(confint(fit))

# Or manually from robust SEs
beta <- coef(fit)
se   <- summary(fit)$coefficients[, "Std.err"]
cbind(OR = exp(beta),
      lower = exp(beta - 1.96 * se),
      upper = exp(beta + 1.96 * se))
```

### Robust Standard Errors (Default)

```r
# Robust (sandwich) — reported by default in geepack
summary(fit)$coefficients[, "Std.err"]
```

### Naive Standard Errors

```r
# Model-based (naive) — only valid if corstr is correct
sqrt(diag(fit$geese$vbeta.naiv))
```

### Predicted Values

```r
# Marginal predictions
predict(fit, type = "response")

# For new data
predict(fit, newdata = nd, type = "response")

# Link-scale predictions
predict(fit, type = "link")
```

### Working Correlation Parameter

```r
fit$geese$alpha
```

### Dispersion (Scale) Parameter

```r
fit$geese$gamma
```

---

## Model Comparison (QIC)

QIC (Quasi-likelihood under the Independence model Criterion) is the GEE analogue of AIC. Lower = better.

### Computing QIC

```r
# geepack provides QIC
QIC(fit_exch)
QIC(fit_ar1)
QIC(fit_indep)

# Compare in a table
data.frame(
  corstr = c("Exchangeable", "AR(1)", "Independence"),
  QIC = c(QIC(fit_exch)[1], QIC(fit_ar1)[1], QIC(fit_indep)[1])
)
```

### What QIC Compares

| Comparison | Use QIC for |
|-----------|-------------|
| Correlation structures | Same mean model, different corstr |
| Mean model terms | Same corstr, different predictors |
| Across families | Not recommended (different quasi-likelihoods) |

> [!note]
> QIC is based on quasi-likelihood, not full likelihood. It's a useful guide for relative model comparison within GEE, but doesn't have the same theoretical guarantees as AIC/BIC.

---

## Robust vs. Naive SEs

### Quick Comparison

```r
robust <- summary(fit)$coefficients[, "Std.err"]
naive  <- sqrt(diag(fit$geese$vbeta.naiv))

data.frame(
  term  = names(coef(fit)),
  robust = round(robust, 4),
  naive  = round(naive, 4),
  ratio  = round(robust / naive, 3)
)
```

### Interpreting the Ratio

| Ratio | Meaning |
|-------|---------|
| 0.9–1.1 | Working correlation is approximately correct |
| 1.1–1.3 | Mild misspecification — robust SEs wider |
| > 1.3 | Substantial misspecification — consider different corstr |
| < 0.9 | Unusual — may indicate small-sample instability |

### When to Use Which

| SE Type | When |
|---------|------|
| **Robust (sandwich)** | Always — default choice |
| **Naive (model-based)** | Only if you're confident the working correlation is correct |
| **Bias-corrected robust** | < 40 clusters |

---

## Diagnostics

### Check Correlation Structure Choice

```r
# Compare robust vs naive SEs (large discrepancy = misspecification)
robust <- summary(fit)$coefficients[, "Std.err"]
naive  <- sqrt(diag(fit$geese$vbeta.naiv))
max(abs(robust / naive - 1))  # Should be < 0.2
```

### Residuals

```r
# Pearson residuals
res <- residuals(fit, type = "pearson")

# Plot residuals vs fitted
plot(fitted(fit), res, xlab = "Fitted", ylab = "Pearson Residuals")
abline(h = 0, lty = 2)
```

### Overdispersion (Count Models)

```r
# Dispersion parameter (should be near 1 for Poisson)
fit$geese$gamma
```

### Influence Diagnostics

```r
# Cook's distance analogue — identify influential clusters
# (Not built into geepack; use leave-one-cluster-out)
ids <- unique(df$id)
betas <- matrix(NA, length(ids), length(coef(fit)))
for (i in seq_along(ids)) {
  fit_i <- update(fit, data = df[df$id != ids[i], ])
  betas[i, ] <- coef(fit_i)
}
# Check for outlier clusters
apply(betas, 2, sd)
```

---

## Common Errors & Fixes

| Issue | Symptom | Fix |
|-------|---------|-----|
| **Unsorted data** | Wrong results, no warning | `arrange(id, time)` before fitting |
| **Non-convergence** | Warning or NaN coefficients | Simplify model; check for separation |
| **Singular correlation** | Error in correlation update | Reduce T; use simpler structure |
| **Too few clusters** | Unreliable sandwich SEs | Need 40+ clusters; use bias correction |
| **Separation (binary)** | Huge coefficients | Remove predictor or collapse categories |

---

## Troubleshooting

### Data Not Sorted

The most common GEE mistake. Always sort before fitting:

```r
df <- df %>% arrange(id, time)
```

### Unstructured Won't Converge

Too many time points relative to clusters:

```r
# Switch from unstructured to exchangeable or AR(1)
fit <- geeglm(y ~ time + x, id = id, data = df,
              family = binomial, corstr = "exchangeable")
```

### Very Small Clusters

With only 2–3 observations per person, AR(1) may be unstable. Use exchangeable or independence.

### Missing Data Handling

Standard GEE assumes MCAR. If you suspect MAR dropout:

```r
# Option 1: Use GLMM instead (handles MAR via FIML)
fit_glmm <- glmer(y ~ time + x + (1 | id), data = df, family = binomial)

# Option 2: Multiple imputation + GEE (requires additional packages)
```

### Small-Sample Bias Correction

With 20–40 clusters, use the Mancl-DeRouen correction. `geepack` does not natively support this; consider the `geesmv` package or manual calculation.

---

## Interpretation Pitfalls

> [!caution]
> These mistakes are common but avoidable:

| Mistake | Reality |
|---------|---------|
| "GEE OR = 1.5, so each person's odds increase 50%" | GEE gives *marginal* (population-averaged) ORs, not individual |
| "The exchangeable correlation is the true correlation" | Working correlation is a nuisance parameter, not a reliable estimate |
| "GEE can handle MAR dropout" | Standard GEE assumes MCAR; use weighted GEE or GLMM for MAR |
| "More time points = more power" | Power comes from clusters (people), not observations per cluster |
| "GEE and GLMM should give the same OR" | They target different quantities; marginal OR < conditional OR |

---

## Quick Formulas

**Marginal odds ratio:**
```r
OR <- exp(coef(fit)["treatment"])
```

**Marginal incidence rate ratio:**
```r
IRR <- exp(coef(fit)["time"])
```

**Confidence interval for OR:**
```r
beta <- coef(fit)["treatment"]
se   <- summary(fit)$coefficients["treatment", "Std.err"]
exp(beta + c(-1.96, 1.96) * se)
```

**Approximate marginal-to-conditional conversion (logistic):**
```r
# Given GLMM random intercept variance sigma2
c_star <- 1 / sqrt(1 + (16 * sqrt(3) / (15 * pi))^2 * sigma2)
marginal_beta <- conditional_beta * c_star
```

---

## Extensions

### Time-Varying Covariates

```r
fit <- geeglm(y ~ time + current_stress + treatment, id = id,
              data = df, family = binomial, corstr = "exchangeable")
```

### Interactions

```r
# Time × group interaction (different trends by group)
fit <- geeglm(y ~ time * treatment, id = id, data = df,
              family = binomial, corstr = "exchangeable")
```

### Count Outcomes

```r
fit <- geeglm(count ~ time + treatment, id = id, data = df,
              family = poisson, corstr = "exchangeable")
exp(coef(fit))  # IRRs
```

### Continuous Outcomes

```r
fit <- geeglm(score ~ time + treatment, id = id, data = df,
              family = gaussian, corstr = "ar1")
```

### Multinomial / Ordinal

GEE for ordinal outcomes is available via `multgee` or `ordgee` (in `geepack`):

```r
# Ordinal GEE
fit <- ordgee(ordered(y) ~ time + treatment, id = id, data = df,
              corstr = "exchangeable")
```

---

## Marginal vs. Conditional

| Aspect | GEE (Marginal) | GLMM (Conditional) |
|--------|-----------------|---------------------|
| **Target** | Population average | Subject-specific |
| **Coefficients** | Marginal effects | Conditional effects |
| **OR/IRR magnitude** | Smaller (attenuated) | Larger |
| **Random effects** | Not estimated | Estimated |
| **Missing data** | MCAR | MAR (FIML) |
| **Model comparison** | QIC | AIC, BIC, LRT |
| **Distributional assumptions** | Fewer | More |
| **For linear models** | Identical to LMM fixed effects | Identical to GEE |

---

## Parameters

| Parameter | Interpretation |
|-----------|----------------|
| β₀ (intercept) | Marginal log-odds (or log-count) at reference levels |
| β₁ (time) | Marginal change per time unit on link scale |
| βₖ (predictor) | Marginal effect of predictor on link scale |
| α (correlation) | Working correlation parameter(s) |
| φ (scale) | Dispersion parameter |

---

## Resources

### Books

| Book | Focus |
|------|-------|
| Diggle et al. (2002). *Analysis of Longitudinal Data*. 2nd ed. Oxford. | Foundational text; GEE and mixed models |
| Fitzmaurice et al. (2011). *Applied Longitudinal Analysis*. 2nd ed. Wiley. | Excellent applied coverage of GEE |
| Hardin & Hilbe (2013). *Generalized Estimating Equations*. 2nd ed. CRC Press. | Dedicated GEE treatment |

### Key Articles

| Article | Contribution |
|---------|--------------|
| Liang & Zeger (1986). *Biometrika*, 73(1), 13–22. | Original GEE paper |
| Zeger et al. (1988). *Biometrics*, 44(4), 1049–1060. | Marginal vs conditional models |
| Pan (2001). *Biometrics*, 57(1), 120–125. | QIC for model selection |
| Mancl & DeRouen (2001). *Biometrics*, 57(1), 126–134. | Small-sample SE correction |

### Online

- [geepack CRAN page](https://cran.r-project.org/package=geepack)
- [Fitzmaurice et al. textbook site](https://content.sph.harvard.edu/fitzmaur/ala2e/)

---

## Software

| Package | Use Case |
|---------|----------|
| **geepack** | Primary R package; `geeglm` interface |
| **gee** | Original R implementation; less flexible than geepack |
| **geeM** | Matrix-based GEE; handles larger problems |
| **geesmv** | Bias-corrected sandwich estimators for small samples |
| **multgee** | Multinomial/ordinal GEE |

---

## Related Tutorials

| Tutorial | Focus |
|----------|-------|
| [GEE Basic](/tutorials/gee) | Introduction to GEE with geepack |
| [GEE Count Outcomes](/tutorials/gee-count) | Poisson and NB marginal models |
| [GEE Time-Varying Covariates](/tutorials/gee-time-varying-covariate) | Within-person predictors |

---

## Related Guides

- [GEE Overview](/guides/gee) — When to use, conceptual foundations, marginal vs conditional
- [GEE Walkthrough](/guides/gee-walkthrough) — Step-by-step worked example
- [GLMM Overview](/guides/glmm) — Conditional (subject-specific) alternative to GEE
- [LMM Overview](/guides/lmm) — Linear mixed models for continuous outcomes
