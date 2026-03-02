---
title: "GLMM Reference"
slug: "glmm-reference"
description: "Quick lookup for GLMM syntax, distributions, link functions, diagnostics, and troubleshooting."
category: "mixed-models"
tags: ["GLMM", "reference", "glmmTMB", "lme4", "cheat-sheet"]
r_packages: ["glmmTMB", "lme4"]
guide_type: "reference"
parent_method: "glmm"
---

## GLMM Quick Reference

Fast lookup for syntax, distributions, and troubleshooting. For step-by-step learning, see the [Walkthrough](/guides/glmm-walkthrough). For conceptual background, see the [Overview](/guides/glmm).

**Jump to:** [Syntax](#syntax) · [Distributions](#distributions--link-functions) · [Estimation](#estimation) · [Extract Output](#extract-output) · [Model Comparison](#model-comparison) · [Diagnostics](#diagnostics-checklist) · [Errors & Fixes](#common-errors--fixes) · [Extensions](#extensions) · [Resources](#resources)

---

## Syntax

### lme4::glmer

```r
library(lme4)

# Binary — random intercept
fit <- glmer(y ~ time + x + (1 | id), data = df, family = binomial)

# Binary — random intercept + slope
fit <- glmer(y ~ time + x + (time | id), data = df, family = binomial)

# Binary — uncorrelated random effects
fit <- glmer(y ~ time + x + (1 | id) + (0 + time | id),
             data = df, family = binomial)

# Poisson count
fit <- glmer(y ~ time + x + (1 | id), data = df, family = poisson)

# AGQ estimation (binary, single random effect only)
fit <- glmer(y ~ time + x + (1 | id), data = df,
             family = binomial, nAGQ = 7)
```

### glmmTMB

```r
library(glmmTMB)

# Binary
fit <- glmmTMB(y ~ time + x + (1 | id), data = df, family = binomial)

# Poisson
fit <- glmmTMB(y ~ time + x + (1 | id), data = df, family = poisson)

# Negative binomial (quadratic parameterization)
fit <- glmmTMB(y ~ time + x + (1 | id), data = df, family = nbinom2)

# Negative binomial (linear parameterization)
fit <- glmmTMB(y ~ time + x + (1 | id), data = df, family = nbinom1)

# Zero-inflated negative binomial
fit <- glmmTMB(y ~ time + x + (1 | id),
               zi = ~ 1,
               data = df, family = nbinom2)

# Zero-inflated with predictors of zero-inflation
fit <- glmmTMB(y ~ time + x + (1 | id),
               zi = ~ time + x,
               data = df, family = nbinom2)
```

### Random Effects Specification

| Syntax | Meaning |
|--------|---------|
| `(1 \| id)` | Random intercept |
| `(time \| id)` | Random intercept + slope (correlated) |
| `(1 \| id) + (0 + time \| id)` | Random intercept + slope (uncorrelated) |
| `(1 \| id/site)` | Nested random effects (site within id) |
| `(1 \| id) + (1 \| site)` | Crossed random effects |

---

## Distributions & Link Functions

### Canonical Links

| Outcome Type | Distribution | Link | g(μ) | R family |
|-------------|-------------|------|------|----------|
| Binary (0/1) | Bernoulli | Logit | log(μ/(1−μ)) | `binomial` |
| Count | Poisson | Log | log(μ) | `poisson` |
| Count (overdispersed) | Neg. Binomial | Log | log(μ) | `nbinom2` |
| Proportions | Binomial | Logit | log(μ/(1−μ)) | `binomial` (with weights) |
| Positive continuous | Gamma | Log | log(μ) | `Gamma(link="log")` |

### Variance Functions

| Distribution | Var(Y\|b) | Overdispersion? |
|-------------|----------|-----------------|
| Bernoulli | μ(1−μ) | Not applicable |
| Poisson | μ | No — if variance > mean, switch to NB |
| NB (nbinom2) | μ + μ²/θ | Yes — θ controls extra variance |
| NB (nbinom1) | μ(1 + α) | Yes — linear in mean |
| Gamma | μ²/ν | Yes — ν is shape parameter |

### Negative Binomial Variants

| glmmTMB family | Variance | When to use |
|---------------|----------|-------------|
| `nbinom2` | μ + μ²/θ | Default choice; quadratic mean-variance |
| `nbinom1` | μ(1 + α) | When variance is proportional to mean |

---

## Estimation

| Method | Package | Code | Best for |
|--------|---------|------|----------|
| Laplace | `glmer` | Default | General use; fast |
| Laplace | `glmmTMB` | Default | General use; more families |
| AGQ | `glmer` | `nAGQ = 7` | Binary with small clusters |
| MCMC | `brms` | `brm(...)` | Complex models; Bayesian |

**When to use AGQ over Laplace:**
- Binary outcomes with < 5 observations per cluster
- Estimates differ > 5% between Laplace and AGQ
- Only works with a single random effect (scalar)

---

## Extract Output

### Fixed Effects

```r
# lme4
fixef(fit)
summary(fit)$coefficients

# glmmTMB
fixef(fit)$cond          # Conditional model
fixef(fit)$zi            # Zero-inflation model (if present)
```

### Odds Ratios / Incidence Rate Ratios

```r
# lme4
exp(fixef(fit))
exp(confint(fit, method = "Wald", parm = "beta_"))

# glmmTMB
exp(fixef(fit)$cond)
exp(confint(fit, parm = "beta_"))
```

### Random Effects

```r
# Variance components
VarCorr(fit)

# Individual random effects (BLUPs / conditional modes)
ranef(fit)

# lme4: random effect SDs
as.data.frame(VarCorr(fit))

# glmmTMB: random effect SDs
sigma(fit)              # Dispersion
VarCorr(fit)$cond$id    # Random effect covariance matrix
```

### Predicted Values

```r
# Conditional predictions (includes random effects)
predict(fit, type = "response")

# Population-level predictions (random effects = 0)
predict(fit, newdata = nd, type = "response", re.form = NA)

# Link-scale predictions
predict(fit, type = "link", re.form = NA)
```

### Confidence Intervals

```r
# lme4 — Wald CIs
confint(fit, method = "Wald")

# lme4 — Profile CIs (slower, more accurate)
confint(fit, method = "profile")

# glmmTMB — Wald CIs
confint(fit)
```

---

## Model Comparison

### Nested Models (LRT)

```r
# lme4
anova(fit_simple, fit_complex)

# glmmTMB
anova(fit_simple, fit_complex)
```

### Information Criteria

```r
AIC(fit1, fit2)
BIC(fit1, fit2)
```

> [!note]
> AIC/BIC comparisons are valid only when models are fit to the **same data** using the **same likelihood**. Don't compare across packages or families without care.

### Random Effects Testing

```r
# Compare random intercept vs. random intercept + slope
fit1 <- glmer(y ~ time + (1 | id), data = df, family = binomial)
fit2 <- glmer(y ~ time + (time | id), data = df, family = binomial)
anova(fit1, fit2)
```

---

## Diagnostics Checklist

### Overdispersion (Count Models)

```r
# Pearson residual ratio
overdisp <- sum(residuals(fit, type = "pearson")^2) / df.residual(fit)

# performance package
check_overdispersion(fit)
```

| Ratio | Interpretation |
|-------|----------------|
| ≈ 1.0 | Adequate — Poisson is fine |
| 1.5–2.0 | Mild overdispersion — consider NB |
| > 2.0 | Overdispersed — use NB or quasi-likelihood |

### Zero-Inflation (Count Models)

```r
# Compare observed vs expected zeros
observed_zeros <- mean(df$y == 0)
predicted_zeros <- mean(predict(fit, type = "response") < 0.5)  # Rough

# Formal test (performance)
check_zeroinflation(fit)
```

### Random Effects Normality

```r
# QQ-plot of random intercepts
re <- ranef(fit)$id[[1]]   # lme4
qqnorm(re); qqline(re)

# Shapiro-Wilk test (small samples only)
shapiro.test(re)
```

### Residuals

```r
# Pearson residuals vs fitted
plot(fitted(fit), residuals(fit, type = "pearson"),
     xlab = "Fitted", ylab = "Pearson Residuals")
abline(h = 0, lty = 2)
```

### Convergence

```r
# lme4: check for warnings
fit@optinfo$conv$lme4

# glmmTMB: check convergence
fit$sdr$pdHess  # Should be TRUE
```

---

## Common Errors & Fixes

| Issue | Symptom | Fix |
|-------|---------|-----|
| **Non-convergence** | Warning: "Model failed to converge" | Simplify random effects; rescale predictors; try different optimizer |
| **Singular fit** | Warning: "boundary (singular) fit" | Random effect variance ≈ 0; simplify RE structure |
| **Complete separation** | Huge coefficients, huge SEs | A predictor perfectly predicts outcome; use penalized methods or remove predictor |
| **Negative variance** | Variance component < 0 | Model misspecified; simplify |
| **NaN in gradient** | glmmTMB: NaN function evaluation | Rescale predictors; change optimizer; simplify model |

---

## Troubleshooting

### Non-Convergence

```r
# lme4: try different optimizers
fit <- glmer(y ~ time + (1 | id), data = df, family = binomial,
             control = glmerControl(optimizer = "bobyqa",
                                    optCtrl = list(maxfun = 100000)))

# glmmTMB: try different optimizer
fit <- glmmTMB(y ~ time + (1 | id), data = df, family = binomial,
               control = glmmTMBControl(optimizer = optim,
                                        optArgs = list(method = "BFGS")))
```

### Singular Fit

- Random effect variance is at or near zero
- Often means the data don't support that random effect
- Solution: drop the problematic random effect or use a simpler structure

```r
# Check which component is singular
VarCorr(fit)
# If slope variance ≈ 0, drop it:
fit_simple <- glmer(y ~ time + (1 | id), data = df, family = binomial)
```

### Complete Separation (Binary)

- Occurs when a predictor perfectly predicts the outcome in a subgroup
- Symptoms: coefficient > 10, SE > 100
- Solutions: remove the predictor, collapse categories, or use penalized estimation

### Slow Fitting

- Reduce random effects complexity
- Center and scale continuous predictors
- For `glmmTMB`: ensure `parallel = FALSE` unless explicitly needed
- For `glmer` with AGQ: use `nAGQ = 1` (Laplace) for initial exploration

---

## Interpretation Pitfalls

| Mistake | Reality |
|---------|---------|
| "OR = 2.5 means 2.5× more likely" | OR is odds ratio, not probability ratio |
| Using GLMM ORs for population claims | GLMM gives conditional effects; marginal ORs are smaller |
| Comparing ORs across models with different REs | Adding random effects changes the scale of fixed effects |
| "Non-significant overdispersion test means Poisson is fine" | The test may be underpowered; always examine the ratio |
| Interpreting NB dispersion θ as a variance | θ is inverse dispersion; larger θ = less overdispersion |

---

## Quick Formulas

**Logit to probability:**
```r
p <- plogis(log_odds)       # = 1 / (1 + exp(-log_odds))
```

**Probability to logit:**
```r
log_odds <- qlogis(p)       # = log(p / (1 - p))
```

**Log to count:**
```r
count <- exp(log_count)
```

**Odds ratio from log-odds coefficient:**
```r
OR <- exp(beta)
```

**Incidence rate ratio from log coefficient:**
```r
IRR <- exp(beta)
```

**Proportion with positive slopes (random slope model):**
```r
pnorm(0, mean = fixef(fit)["time"],
      sd = sqrt(VarCorr(fit)$id["time", "time"]),
      lower.tail = FALSE)
```

---

## Extensions

### Time-Invariant Predictors

```r
fit <- glmer(y ~ time + treatment + sex + (1 | id),
             data = df, family = binomial)
```

### Time-Varying Covariates

```r
fit <- glmer(y ~ time + stress_current + (1 | id),
             data = df, family = binomial)
```

### Interactions

```r
# Time × treatment interaction
fit <- glmer(y ~ time * treatment + (1 | id),
             data = df, family = binomial)
```

### Offset (Exposure Time)

```r
# Count model with varying exposure
fit <- glmmTMB(events ~ time + treatment + (1 | id),
               offset = log(exposure_time),
               data = df, family = poisson)
```

### Ordinal Outcomes

```r
library(ordinal)
fit <- clmm(ordered_y ~ time + treatment + (1 | id), data = df)
```

### Three-Level Models

```r
# Observations nested in people nested in sites
fit <- glmer(y ~ time + (1 | site/id), data = df, family = binomial)
```

---

## Parameters

| Parameter | Symbol | Interpretation |
|-----------|--------|----------------|
| Fixed intercept | β₀ | Baseline level on link scale (at RE = 0) |
| Fixed slope | β₁ | Change per time unit on link scale |
| Random intercept SD | σ₀ | Between-person variability in baseline |
| Random slope SD | σ₁ | Between-person variability in change rate |
| RE correlation | ρ₀₁ | Relationship between baseline and change |
| Dispersion (NB) | θ | Inverse overdispersion; larger = less dispersion |
| ZI probability | π | Probability of structural zero |

---

## Resources

### Books

| Book | Focus |
|------|-------|
| Agresti (2013). *Categorical Data Analysis*. 3rd ed. Wiley. | Comprehensive GLM/GLMM theory |
| Bolker et al. (2009). *Trends in Ecology & Evolution*. | Practical GLMM strategies |
| Stroup (2013). *Generalized Linear Mixed Models*. CRC Press. | Applied GLMM with SAS/R |
| Zuur et al. (2009). *Mixed Effects Models and Extensions in Ecology with R*. Springer. | Ecological applications, overdispersion |

### Key Articles

| Article | Contribution |
|---------|--------------|
| Bolker et al. (2009). *TREE*, 24(3), 127–135. | "GLMMs in action" — practical guide |
| Brooks et al. (2017). *R Journal*, 9(2), 378–400. | glmmTMB package paper |
| Bates et al. (2015). *J Stat Software*, 67(1). | lme4 package paper |

### Online

- [glmmTMB vignettes](https://cran.r-project.org/package=glmmTMB)
- [lme4 documentation](https://cran.r-project.org/package=lme4)
- [Ben Bolker's GLMM FAQ](https://bbolker.github.io/mixedmodels-misc/glmmFAQ.html)

---

## Software

| Package | Use Case |
|---------|----------|
| **lme4::glmer** | Binary/Poisson GLMM; AGQ; mature ecosystem |
| **glmmTMB** | NB, zero-inflation, flexible families |
| **brms** | Bayesian GLMM; complex models; Stan backend |
| **ordinal::clmm** | Ordinal outcomes (cumulative link mixed model) |
| **MASS::glmer.nb** | NB via lme4 (limited; prefer glmmTMB) |
| **performance** | Model diagnostics (overdispersion, zero-inflation) |

---

## Related Tutorials

| Tutorial | Focus | Difficulty |
|----------|-------|------------|
| [GLMM Basic](/tutorials/glmm) | Introduction to GLMM with glmmTMB | Intro |
| [GLMM Binary Outcomes](/tutorials/glmm-binary) | Logistic random effects model | Intermediate |
| [GLMM Count Outcomes](/tutorials/glmm-count) | Negative binomial mixed model | Intermediate |
| [GLMM Interactions](/tutorials/glmm-interactions) | Cross-level and time interactions | Intermediate |

---

## Related Guides

- [GLMM Overview](/guides/glmm) — When to use, key concepts, conditional vs marginal
- [GLMM Walkthrough](/guides/glmm-walkthrough) — Step-by-step worked example
- [GEE Overview](/guides/gee) — Population-averaged alternative to GLMM
- [LMM Overview](/guides/lmm) — Linear mixed models for continuous outcomes
