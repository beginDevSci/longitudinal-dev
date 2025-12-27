---
title: "LMM Reference"
slug: "lmm-pilot-reference"
description: "Quick lookup for LMM syntax, diagnostics, parameters, and troubleshooting."
category: "mixed-models"
tags: ["LMM", "reference", "lme4", "cheat-sheet"]
r_packages: ["lme4", "lmerTest", "performance"]
guide_type: "reference"
parent_method: "lmm-pilot"
---

## LMM Quick Reference

Fast lookup for syntax, diagnostics, and troubleshooting. For step-by-step learning, see the [Tutorial](/guides/lmm-pilot-tutorial).

**Jump to:** [Syntax](#lme4-syntax) · [ML vs. REML](#estimation-ml-vs-reml) · [Extract Output](#extract-output) · [Model Comparison](#model-comparison) · [Diagnostics](#diagnostics) · [Parameters](#parameters) · [Time Coding](#time-coding) · [Errors & Fixes](#common-errors--fixes) · [Troubleshooting](#troubleshooting) · [Pitfalls](#interpretation-pitfalls) · [Formulas](#quick-formulas) · [Checklists](#checklists) · [Extensions](#advanced-extensions) · [Resources](#resources)

---

## lme4 Syntax

### Random Intercept Only

```r
mod <- lmer(y ~ time + (1 | id), data = data_long)
```

### Random Intercept and Slope (Correlated)

```r
mod <- lmer(y ~ time + (1 + time | id), data = data_long)
```

### Random Intercept and Slope (Uncorrelated)

```r
mod <- lmer(y ~ time + (1 | id) + (0 + time | id), data = data_long)
```

### With Between-Person Predictor

```r
mod <- lmer(y ~ time + treatment + (1 + time | id), data = data_long)
```

### Cross-Level Interaction

```r
mod <- lmer(y ~ time * treatment + (1 + time | id), data = data_long)
```

### Quadratic Time

```r
mod <- lmer(y ~ time + I(time^2) + (1 + time | id), data = data_long)
```

### Random Slope Only (No Random Intercept)

```r
mod <- lmer(y ~ time + (0 + time | id), data = data_long)
```

### Nested Random Effects

```r
# Students nested in schools
mod <- lmer(y ~ time + (1 + time | school/id), data = data_long)

# Equivalent explicit syntax
mod <- lmer(y ~ time + (1 + time | school) + (1 + time | school:id), data = data_long)
```

---

## Estimation: ML vs. REML

| Method | Use When | Code |
|--------|----------|------|
| **REML** (default) | Final model, reporting variances | `REML = TRUE` |
| **ML** | Comparing models with different fixed effects | `REML = FALSE` |

### REML (Default)

```r
mod <- lmer(y ~ time + (1 + time | id), data = data_long)
# or explicitly:
mod <- lmer(y ~ time + (1 + time | id), data = data_long, REML = TRUE)
```

### ML (For Model Comparison)

```r
mod <- lmer(y ~ time + (1 + time | id), data = data_long, REML = FALSE)
```

### Workflow

1. **Compare models** with `REML = FALSE`
2. **Select best model** via `anova()` or AIC/BIC
3. **Refit final model** with `REML = TRUE` for reporting

---

## Extract Output

### Summary

```r
summary(mod)
```

### Fixed Effects

```r
fixef(mod)
```

### Fixed Effects with Confidence Intervals

```r
confint(mod, parm = "beta_", method = "Wald")
# or profile likelihood (slower, more accurate):
confint(mod, parm = "beta_", method = "profile")
```

### Variance Components

```r
VarCorr(mod)
```

### Random Effects (BLUPs)

```r
ranef(mod)$id
```

### Fitted Values

```r
fitted(mod)
```

### Residuals

```r
resid(mod)
```

### R² (Marginal and Conditional)

```r
library(performance)
r2(mod)
```

- **Marginal R²**: Variance explained by fixed effects
- **Conditional R²**: Variance explained by fixed + random effects

### Intraclass Correlation (ICC)

```r
library(performance)
icc(mod)
```

### Coefficients Table with p-values

```r
library(lmerTest)
mod <- lmer(y ~ time + (1 + time | id), data = data_long)
summary(mod)  # Now includes p-values
```

---

## Model Comparison

### Likelihood Ratio Test (Nested Models)

```r
# Must use ML for comparing different fixed effects
mod1 <- lmer(y ~ time + (1 + time | id), data = data_long, REML = FALSE)
mod2 <- lmer(y ~ time + treatment + (1 + time | id), data = data_long, REML = FALSE)

anova(mod1, mod2)
```

### Comparing Random Effects Structures

```r
# Can use REML when fixed effects are identical
mod_ri <- lmer(y ~ time + (1 | id), data = data_long)
mod_rs <- lmer(y ~ time + (1 + time | id), data = data_long)

anova(mod_ri, mod_rs)
```

### Information Criteria

```r
AIC(mod1, mod2)  # Lower = better
BIC(mod1, mod2)  # Lower = better; more conservative
```

### Boundary Test Note

When testing whether a variance = 0 (e.g., random slope variance), the LRT is a boundary test. The reported p-value is conservative—the true p-value is approximately half.

---

## Diagnostics

### Residuals vs. Fitted

```r
plot(mod)
```

Look for: Random scatter around 0. Problematic: Funnel shape, curves.

### Q-Q Plot for Residuals

```r
qqnorm(resid(mod))
qqline(resid(mod))
```

Look for: Points on line. Problematic: Heavy tails, skewness.

### Q-Q Plot for Random Effects

```r
# Random intercepts
qqnorm(ranef(mod)$id[, "(Intercept)"])
qqline(ranef(mod)$id[, "(Intercept)"])

# Random slopes
qqnorm(ranef(mod)$id[, "time"])
qqline(ranef(mod)$id[, "time"])
```

### Check for Singular Fit

```r
isSingular(mod)  # TRUE = problem
```

### Inspect Variance Components

```r
VarCorr(mod)
# Check for variances at or near zero
```

### Model Performance Summary

```r
library(performance)
check_model(mod)  # Comprehensive diagnostic plots
```

---

## Parameters

| Parameter | Symbol | lme4 Location |
|-----------|--------|---------------|
| Average intercept | γ₀₀ | Fixed effects: `(Intercept)` |
| Average slope | γ₁₀ | Fixed effects: `time` |
| Intercept variance | τ₀₀ | Random effects: `id (Intercept)` |
| Slope variance | τ₁₁ | Random effects: `id time` |
| I-S correlation | ρ | Random effects: `Corr` |
| Residual variance | σ² | `Residual` |

### Reading lme4 Output

```
Random effects:
 Groups   Name        Variance Std.Dev. Corr
 id       (Intercept) 98.54    9.93          <- τ₀₀
          time         0.89    0.94     -0.18 <- τ₁₁, ρ
 Residual             24.89    4.99          <- σ²

Fixed effects:
            Estimate Std. Error ...
(Intercept)  49.923   0.732     <- γ₀₀
time          2.021   0.085     <- γ₁₀
```

---

## Time Coding

| Centering | Time Values | Intercept = |
|-----------|-------------|-------------|
| Wave 1 (default) | 0, 1, 2, 3, 4 | Score at Wave 1 |
| Midpoint | -2, -1, 0, 1, 2 | Score at Wave 3 |
| Final wave | -4, -3, -2, -1, 0 | Score at Wave 5 |
| Actual months | 0, 6, 12, 18, 24 | Score at baseline; slope = per month |

Slope interpretation unchanged by centering; intercept moves to the zero-point.

### Recoding Time

```r
# Center at midpoint
data_long$time_c <- data_long$time - 2

# Use actual months
data_long$months <- c(0, 6, 12, 18, 24)[data_long$wave]
```

---

## Common Errors & Fixes

| Issue | Symptom | Fix |
|-------|---------|-----|
| **Singular fit** | `boundary (singular) fit` | Simplify random effects; check `VarCorr()` |
| **Time as factor** | 4 dummy coefficients, not 1 slope | `time <- as.numeric(time)` |
| **REML comparison** | Invalid LRT for fixed effects | Use `REML = FALSE` |
| **Convergence failure** | "Model failed to converge" | Simplify model; scale predictors; increase iterations |
| **Missing p-values** | No p-values in summary | Load `lmerTest` before fitting |
| **Data not long format** | Error about missing variables | Reshape with `pivot_longer()` |

---

## Troubleshooting

### Singular Fit

**Warning**: `boundary (singular) fit: see help('isSingular')`

**What it means**: A variance component is estimated at or very near zero.

**Diagnose**:

```r
VarCorr(mod)  # Check which variance is near zero
```

**Solutions**:

1. Remove the problematic random effect
2. Use uncorrelated random effects: `(1 | id) + (0 + time | id)`
3. Accept if theoretically justified (variance genuinely near zero)

### Convergence Failure

**Warning**: "Model failed to converge" or "convergence code: -1"

**Solutions**:

```r
# Increase iterations
mod <- lmer(y ~ time + (1 + time | id), data = data_long,
            control = lmerControl(optCtrl = list(maxfun = 20000)))

# Try different optimizer
mod <- lmer(y ~ time + (1 + time | id), data = data_long,
            control = lmerControl(optimizer = "bobyqa"))

# Scale predictors
data_long$time_scaled <- scale(data_long$time)
```

### BLUPs in Secondary Analyses

**Problem**: Using `ranef()` output as data in follow-up analyses.

**Why it's wrong**: BLUPs have uncertainty; treating them as fixed underestimates SEs.

**Solution**: Include predictors in the original model:

```r
# Wrong
re <- ranef(mod)$id
cor.test(re$`(Intercept)`, external_variable)

# Right
mod <- lmer(y ~ time + external_variable + (1 + time | id), data = data_long)
```

### No p-values in Output

**Solution**: Load `lmerTest` before fitting:

```r
library(lmerTest)
mod <- lmer(y ~ time + (1 + time | id), data = data_long)
summary(mod)  # Now includes p-values
```

### Negative Variance Estimate

Unlike lavaan, lme4 constrains variances to be non-negative. If you see a variance at exactly zero, it's hitting the boundary—see Singular Fit above.

---

## Interpretation Pitfalls

| Mistake | Reality |
|---------|---------|
| "ICC = 0.65 means 65% explained" | ICC is variance *partitioning*, not variance *explained* |
| "Slope variance is significant, so it matters" | With large N, tiny variances are significant; interpret effect size |
| "Everyone improved" (slope mean = 2) | Check slope SD—some individuals may have negative slopes |
| "BLUPs are observed data" | BLUPs are shrunken estimates with uncertainty |
| "Marginal R² is low, model is bad" | Conditional R² captures individual differences |
| "Random effects are normally distributed" | This is an assumption—check Q-Q plots |
| "Intercept = baseline" | Only if time is coded 0 at baseline |
| "Use REML for all comparisons" | Use ML when fixed effects differ |

---

## Quick Formulas

### ICC (from random intercept model)

```r
# Manual calculation
vc <- as.data.frame(VarCorr(mod_ri))
tau00 <- vc$vcov[1]  # Intercept variance
sigma2 <- vc$vcov[2]  # Residual variance
icc <- tau00 / (tau00 + sigma2)

# Or use performance package
performance::icc(mod_ri)
```

### Intercept-Slope Correlation

```r
# From VarCorr
vc <- VarCorr(mod)
attr(vc$id, "correlation")[1, 2]

# Manual from covariance
tau01 <- attr(vc$id, "cov")[1, 2]
tau00 <- vc$id[1, 1]
tau11 <- vc$id[2, 2]
rho <- tau01 / sqrt(tau00 * tau11)
```

### Proportion with Positive Slopes

```r
gamma10 <- fixef(mod)["time"]
tau11 <- VarCorr(mod)$id["time", "time"]

pnorm(0, mean = gamma10, sd = sqrt(tau11), lower.tail = FALSE)
```

### Person-Specific Trajectory

```r
re <- ranef(mod)$id
person_i <- 5

intercept_i <- fixef(mod)["(Intercept)"] + re[person_i, "(Intercept)"]
slope_i <- fixef(mod)["time"] + re[person_i, "time"]
```

### Predicted Value for Person i at Time t

```r
y_hat <- intercept_i + slope_i * t
```

---

## Checklists

### Before Running

- [ ] Data in long format (one row per observation)
- [ ] Time coded as numeric (not factor)
- [ ] ID variable is factor or character
- [ ] Checked for missing data patterns
- [ ] Visualized individual trajectories

### Before Reporting

- [ ] Model converged without warnings
- [ ] Checked for singular fit
- [ ] Residual diagnostics examined
- [ ] Used ML for model comparison, REML for final estimates
- [ ] Fixed effects reported with SEs and CIs
- [ ] Variance components reported (not just fixed effects)
- [ ] R² (marginal and conditional) reported
- [ ] Time coding explicitly stated
- [ ] Interpreted effect sizes, not just p-values

---

## Advanced Extensions

### Time-Varying Covariates

```r
mod <- lmer(mood ~ time + stress + (1 + time | id), data = data_long)
```

**Centering for within/between separation**:

```r
data_long <- data_long %>%
  group_by(id) %>%
  mutate(
    stress_between = mean(stress, na.rm = TRUE),
    stress_within = stress - stress_between
  )

mod <- lmer(mood ~ time + stress_within + stress_between + (1 + time | id),
            data = data_long)
```

### Cross-Level Interactions

```r
# Does treatment moderate the slope?
mod <- lmer(y ~ time * treatment + (1 + time | id), data = data_long)
```

### Quadratic Growth

```r
mod <- lmer(y ~ time + I(time^2) + (1 + time | id), data = data_long)
```

Note: Random quadratic requires many observations per person.

### Autocorrelated Residuals (nlme)

```r
library(nlme)

mod <- lme(y ~ time,
           random = ~ 1 + time | id,
           correlation = corAR1(form = ~ time | id),
           data = data_long)
```

### Generalized Linear Mixed Models (GLMM)

```r
# Binary outcome
mod <- glmer(binary_y ~ time + (1 | id), data = data_long, family = binomial)

# Count outcome
mod <- glmer(count_y ~ time + (1 | id), data = data_long, family = poisson)
```

### Bayesian Mixed Models (brms)

```r
library(brms)

mod <- brm(y ~ time + (1 + time | id),
           data = data_long,
           family = gaussian(),
           chains = 4, cores = 4)
```

---

## LMM vs. LGCM

| Aspect | LMM | LGCM |
|--------|-----|------|
| Data format | Long | Wide |
| Software | lme4, nlme | lavaan, Mplus |
| Fit indices | R², ICC | CFI, RMSEA, SRMR |
| Individual estimates | BLUPs | Factor scores |
| Time-varying covariates | Easy | More complex |
| Latent variables | Not directly | Natural extension |

For basic growth models, LMM and LGCM produce **identical estimates**.

---

## Resources

### Books

| Book | Focus |
|------|-------|
| Singer & Willett (2003). *Applied Longitudinal Data Analysis*. Oxford. | Excellent pedagogy |
| Snijders & Bosker (2012). *Multilevel Analysis*. Sage. | Comprehensive MLM |
| Raudenbush & Bryk (2002). *Hierarchical Linear Models*. Sage. | Classic reference |
| Gelman & Hill (2007). *Data Analysis Using Regression and Multilevel Models*. Cambridge. | Practical, Bayesian-friendly |
| West, Welch, & Galecki (2014). *Linear Mixed Models*. CRC. | Software-focused |

### Key Articles

| Article | Contribution |
|---------|--------------|
| Bates et al. (2015). Fitting linear mixed-effects models using lme4. *JSS*. | Definitive lme4 reference |
| Barr et al. (2013). Random effects structure for confirmatory testing. *JML*. | "Keep it maximal" argument |
| Matuschek et al. (2017). Balancing Type I error and power. *JML*. | Counter to maximal approach |
| Luke (2017). Evaluating significance in LMMs in R. *BRM*. | p-value methods comparison |

### Online

- **lme4 vignette**: `vignette("lmer", package = "lme4")`
- **UCLA IDRE**: https://stats.oarc.ucla.edu/r/seminars/
- **Bodo Winter's tutorials**: https://bodowinter.com/tutorials.html
- **Michael Clark's guide**: https://m-clark.github.io/mixed-models-with-R/

---

## Software

### R Packages

| Package | Use Case | Notes |
|---------|----------|-------|
| **lme4** | Primary choice | Fast, reliable, standard |
| **lmerTest** | p-values for lme4 | Load before fitting |
| **nlme** | Correlation structures | Different syntax |
| **performance** | R², ICC, diagnostics | Companion to lme4 |
| **brms** | Bayesian inference | Requires Stan |
| **glmmTMB** | Complex GLMMs | Zero-inflation support |

### Other Software

| Software | Notes |
|----------|-------|
| **Stata (mixed)** | Good documentation |
| **SAS (PROC MIXED)** | Established, powerful |
| **SPSS (MIXED)** | GUI available |
| **HLM** | Specialized for MLM |

**Recommendation**: Use lme4 + lmerTest for most analyses. Use nlme for autocorrelated residuals. Use brms for Bayesian inference.

---

## Links

- [Tutorial](/guides/lmm-pilot-tutorial) — Step-by-step worked example
- [LMM Overview](/guides/lmm-pilot) — When to use, key concepts
- [lme4 documentation](https://cran.r-project.org/web/packages/lme4/vignettes/lmer.pdf) — Official vignette
