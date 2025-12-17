---
title: "Linear Mixed Models"
slug: "lmm"
description: "Learn to analyze repeated measures and clustered data using linear mixed models in R with lme4."
category: "multilevel-models"
tags: ["LMM", "mixed-effects", "longitudinal", "lme4", "multilevel"]
r_packages: ["lme4", "lmerTest", "tidyverse"]
---

## Overview

### Why Mixed Models?

Real-world data rarely consists of independent observations. Students are nested within classrooms, patients are measured repeatedly over time, employees work within companies. When observations are **clustered**, standard regression violates the independence assumption and produces biased standard errors.

**Linear Mixed Models (LMM)** handle this by modeling both:
- **Fixed effects**: Population-average relationships
- **Random effects**: Cluster-specific deviations

> [!tip]
> LMM is ideal when you have repeated measures or hierarchical data and want to account for correlation within clusters while estimating population-level effects.

### What LMM Provides

Linear Mixed Models offer several advantages for longitudinal and clustered data:

- **Handles Non-Independence**: Explicitly models correlation within clusters
- **Flexible Data Structures**: Works with unbalanced designs and missing data
- **Individual Predictions**: Shrinkage estimators (BLUPs) for cluster-specific effects
- **Efficient Estimation**: Uses all available data via maximum likelihood
- **Generalizable**: Same framework handles growth curves, crossed designs, and complex nesting

> **Note:** Unlike repeated-measures ANOVA, LMM doesn't require complete cases or sphericity assumptions.

### When LMM is Appropriate

LMM is well-suited for:

| Requirement | Guideline |
|-------------|-----------|
| **Clustered data** | Repeated measures, nested structures, or both |
| **Continuous outcome** | Or bounded outcomes with careful interpretation |
| **Interest in both levels** | Population effects AND cluster-specific variation |
| **Sample size** | N ≥ 30 clusters for reliable variance estimates |

## Conceptual Foundations

### Two Levels of Variation

LMM partitions variation into:

**Within-Cluster Variation (Level 1)**
- How individuals vary around their cluster mean
- Modeled by residual variance

**Between-Cluster Variation (Level 2)**
- How cluster means/slopes vary around the population average
- Modeled by random effect variances

### The Basic LMM Equation

For a simple random intercept model, the outcome \(y_{ij}\) for observation \(i\) in cluster \(j\) is:

$$
y_{ij} = \beta_0 + \beta_1 x_{ij} + u_{0j} + \epsilon_{ij}
$$

Where:
- \(\beta_0\) = population intercept (fixed effect)
- \(\beta_1\) = population slope (fixed effect)
- \(u_{0j}\) = cluster \(j\)'s deviation from population intercept (random effect)
- \(\epsilon_{ij}\) = residual error

Random effects are assumed normally distributed:
$$
u_{0j} \sim N(0, \tau^2_{00})
$$

> [!warning]
> The random effects represent *deviations from* the fixed effects, not the cluster-specific values themselves. The cluster-specific intercept is \(\beta_0 + u_{0j}\).

### Adding Random Slopes

When the effect of a predictor varies across clusters:

$$
y_{ij} = \beta_0 + \beta_1 x_{ij} + u_{0j} + u_{1j} x_{ij} + \epsilon_{ij}
$$

Now both intercept (\(u_{0j}\)) and slope (\(u_{1j}\)) vary by cluster. The random effects have a covariance structure:

$$
\begin{pmatrix} u_{0j} \\ u_{1j} \end{pmatrix} \sim N\left(\begin{pmatrix} 0 \\ 0 \end{pmatrix}, \begin{pmatrix} \tau^2_{00} & \tau_{01} \\ \tau_{01} & \tau^2_{11} \end{pmatrix}\right)
$$

### Key Components

A typical LMM includes:

1. **Fixed effects**: Population-average intercept and slopes
2. **Random effects**: Cluster-specific deviations (intercepts and/or slopes)
3. **Variance components**: Between-cluster variance(s) and their covariance
4. **Residual variance**: Within-cluster variation not explained by the model

## Model Specification & Fit

### Data Requirements

Your data should be in **long format** for lme4:

| id | time | score | group |
|----|------|-------|-------|
| 1 | 0 | 10 | A |
| 1 | 1 | 12 | A |
| 1 | 2 | 15 | A |
| 2 | 0 | 8 | B |
| 2 | 1 | 9 | B |
| ... | ... | ... | ... |

### Basic lme4 Syntax

```r
library(lme4)
library(lmerTest)  # Adds p-values to summary

# Random intercept model
fit1 <- lmer(score ~ time + (1 | id), data = mydata)

# Random intercept and slope
fit2 <- lmer(score ~ time + (1 + time | id), data = mydata)

# Uncorrelated random effects
fit3 <- lmer(score ~ time + (1 | id) + (0 + time | id), data = mydata)
```

### Formula Syntax Guide

| Formula | Meaning |
|---------|---------|
| `(1 | id)` | Random intercept per id |
| `(1 + time | id)` | Correlated random intercept and slope |
| `(0 + time | id)` | Random slope only (no intercept) |
| `(1 | id) + (0 + time | id)` | Uncorrelated random intercept and slope |
| `(1 | school/class)` | Nested random effects |

### Model Comparison

Compare nested models using likelihood ratio tests:

```r
# Fit models
fit_ri <- lmer(score ~ time + (1 | id), data = mydata, REML = FALSE)
fit_rs <- lmer(score ~ time + (1 + time | id), data = mydata, REML = FALSE)

# Compare
anova(fit_ri, fit_rs)
```

> [!important]
> Use `REML = FALSE` (maximum likelihood) when comparing models with different fixed effects. Use REML (default) for final parameter estimates once you've chosen your model.

## Interpretation

### Understanding Model Output

When you run `summary(fit)` in lme4, focus on these sections:

**Random Effects**

```
Random effects:
 Groups   Name        Variance Std.Dev. Corr
 id       (Intercept) 102.45   10.12
          time          4.12    2.03    0.25
 Residual              25.00    5.00
```

- **Variance components**: How much clusters vary in intercepts/slopes
- **Correlation**: Relationship between random intercept and slope
- **Residual**: Within-cluster variation not explained by predictors

**Fixed Effects**

```
Fixed effects:
            Estimate Std. Error       df t value Pr(>|t|)
(Intercept)   50.123      0.712   198.00   70.41   <2e-16 ***
time           2.987      0.142   198.00   21.04   <2e-16 ***
```

- **Estimates**: Population-average effects
- **Standard errors**: Account for clustering (unlike OLS)
- **df**: Degrees of freedom (Satterthwaite approximation from lmerTest)

### Intraclass Correlation Coefficient (ICC)

The ICC measures what proportion of total variance is between clusters:

$$
ICC = \frac{\tau^2_{00}}{\tau^2_{00} + \sigma^2}
$$

```r
# Calculate ICC from random intercept model
VarCorr(fit)
# ICC = between-cluster variance / total variance
```

- **ICC near 0**: Little clustering; OLS might be fine
- **ICC near 1**: Most variance is between clusters; LMM essential

### Extracting Predictions

```r
# Population-level predictions (fixed effects only)
predict(fit, re.form = NA)

# Cluster-specific predictions (including random effects)
predict(fit, re.form = NULL)

# Extract BLUPs (random effects)
ranef(fit)

# Coefficients per cluster (fixed + random)
coef(fit)
```

> [!tip]
> BLUPs (Best Linear Unbiased Predictors) are "shrunk" toward the population mean. Clusters with less data are shrunk more, borrowing strength from the full sample.

## Worked Example

This section provides a complete, runnable R workflow demonstrating LMM analysis.

### Setup

```r
# Load required packages
library(tidyverse)
library(lme4)
library(lmerTest)

# Set seed for reproducibility
set.seed(2024)
```

### Simulate Data

```r
# Simulation parameters
n_subjects <- 50    # Number of individuals
n_times <- 5        # Measurements per person

# Population parameters (fixed effects)
beta_0 <- 50        # Population intercept
beta_1 <- 3         # Population slope (growth per time unit)

# Random effect variances
tau_00 <- 100       # Between-person variance in intercepts
tau_11 <- 4         # Between-person variance in slopes
tau_01 <- 5         # Covariance between intercept and slope
sigma_sq <- 25      # Residual variance

# Generate random effects
Sigma <- matrix(c(tau_00, tau_01, tau_01, tau_11), 2, 2)
re <- MASS::mvrnorm(n_subjects, c(0, 0), Sigma)

# Generate data in long format
data <- expand.grid(
  id = 1:n_subjects,
  time = 0:(n_times - 1)
) %>%
  mutate(
    u0 = re[id, 1],
    u1 = re[id, 2],
    score = beta_0 + u0 + (beta_1 + u1) * time + rnorm(n(), 0, sqrt(sigma_sq)),
    id = factor(id)
  )

head(data)
```

### Visualize Individual Trajectories

```r
# Spaghetti plot with individual trajectories
ggplot(data, aes(x = time, y = score, group = id)) +
  geom_line(alpha = 0.3) +
  geom_smooth(aes(group = NULL), method = "lm",
              color = "red", linewidth = 1.5) +
  labs(title = "Individual Growth Trajectories",
       x = "Time", y = "Score") +
  theme_minimal()
```

### Fit the LMM

```r
# Random intercept and slope model
fit <- lmer(score ~ time + (1 + time | id), data = data)

# View results
summary(fit)
```

### Model Diagnostics

```r
# Residual plots
plot(fit)

# Q-Q plot of residuals
qqnorm(resid(fit))
qqline(resid(fit))

# Q-Q plot of random effects
lattice::qqmath(ranef(fit))
```

### Check Parameter Recovery

Compare estimated parameters to true simulation values:

| Parameter | True Value | Interpretation |
|-----------|------------|----------------|
| Fixed intercept | 50 | Population average at time 0 |
| Fixed slope | 3 | Population average change per time |
| Random intercept SD | 10 | √100 = between-person SD in intercepts |
| Random slope SD | 2 | √4 = between-person SD in slopes |
| Residual SD | 5 | √25 = within-person SD |

## Reference & Resources

### Cheat Sheet

**Quick Syntax Reference**

```r
# Random intercept
lmer(y ~ x + (1 | group), data)

# Random intercept and slope (correlated)
lmer(y ~ x + (1 + x | group), data)

# Random intercept and slope (uncorrelated)
lmer(y ~ x + (1 | group) + (0 + x | group), data)

# Nested random effects
lmer(y ~ x + (1 | school/class), data)

# Crossed random effects
lmer(y ~ x + (1 | subject) + (1 | item), data)

# Model comparison
anova(fit1, fit2)

# Extract components
fixef(fit)    # Fixed effects
ranef(fit)    # Random effects (BLUPs)
coef(fit)     # Cluster coefficients
VarCorr(fit)  # Variance components
```

**Common Issues and Solutions**

| Problem | Solution |
|---------|----------|
| Singular fit | Simplify random structure or check data |
| Convergence warning | Scale predictors, increase iterations |
| No p-values | Load lmerTest or use bootstrap |
| Large residuals | Check for outliers, add predictors |

### Common Pitfalls

> [!caution]
> These mistakes are common but avoidable:

1. **Ignoring clustering**: Standard errors will be wrong; use LMM
2. **Over-complex random structure**: Start simple, add complexity if justified
3. **Wrong df for p-values**: Use lmerTest or bootstrap methods
4. **Misinterpreting random effects**: BLUPs are deviations, not absolute values
5. **Forgetting to center predictors**: Centering aids interpretation and convergence

### Recommended Resources

- **Gelman & Hill (2007)**: *Data Analysis Using Regression and Multilevel Models* - Classic introduction
- **Bates et al. (2015)**: *Fitting Linear Mixed-Effects Models Using lme4* - Package vignette
- **Barr et al. (2013)**: Random effects structure for confirmatory hypothesis testing
- **lme4 documentation**: https://cran.r-project.org/package=lme4
