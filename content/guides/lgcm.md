---
title: "Latent Growth Curve Models: A Practical Tutorial"
slug: "lgcm"
description: "Learn to model individual trajectories over time using SEM-based growth curves in R with lavaan."
category: "growth-models"
tags: ["LGCM", "SEM", "longitudinal", "lavaan", "growth-curves"]
r_packages: ["lavaan", "tidyverse", "MASS"]
---

# Latent Growth Curve Models: A Practical Tutorial

## Overview

### Why Study Growth?

Longitudinal data capture something cross-sectional data cannot: **change**. When you measure the same people over time, you can ask questions that matter:

- How do symptoms evolve after treatment begins?
- Do children's reading skills develop at the same rate?
- Does cognitive decline accelerate with age?

Standard regression tells you about group averages, but it misses the individual story. **Latent Growth Curve Models (LGCM)** let you model each person's trajectory while borrowing strength from the full sample.

> [!tip]
> LGCM is ideal when you have 3+ repeated measures and want to understand both the average trajectory AND individual differences in change.

### What LGCM Provides

LGCM offers several capabilities that make it well-suited for longitudinal analysis:

- **Individual Trajectories**: Each person gets their own intercept (starting point) and slope (rate of change)
- **Flexible Time Coding**: Unequal spacing between measurements is handled naturally
- **Missing Data Handling**: Full-information maximum likelihood uses all available data
- **Model Fit Testing**: Global fit indices tell you whether your growth model matches the data
- **Path to Complexity**: Easy extension to predictors, multiple groups, nonlinear change

> **Note:** Unlike mixed-effects models, LGCM provides global fit indices (CFI, RMSEA, SRMR) that tell you whether your hypothesized growth structure matches the data.

### When LGCM is Appropriate

LGCM works well when you have:

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ time points (more is better for nonlinear models) |
| **Continuous outcome** | Or ordinal with many categories |
| **Interest in change** | Not just "do groups differ?" but "how do individuals change?" |
| **Sample size** | N ≥ 100 for simple models; more for complex models |

## Conceptual Foundations

### Two Ways to Understand LGCM

You can think about LGCM from two perspectives:

**The SEM (Factor Model) Perspective**

LGCM treats your repeated measures as indicators of two latent factors:
- **Intercept factor**: Captures where each person starts
- **Slope factor**: Captures how fast each person changes

The factor loadings are fixed (not estimated) to define the time metric.

**The Multilevel Perspective**

LGCM is mathematically equivalent to a two-level model:
- **Level 1**: Within-person change over time
- **Level 2**: Between-person differences in trajectories

### The Basic LGCM Equation

For a linear growth model, the observed score $y_{it}$ for person $i$ at time $t$ is:

$$
y_{it} = \eta_{0i} + \eta_{1i} \cdot \lambda_t + \epsilon_{it}
$$

Where:
- $\eta_{0i}$ = person $i$'s intercept (latent)
- $\eta_{1i}$ = person $i$'s slope (latent)
- $\lambda_t$ = time score at occasion $t$ (fixed, e.g., 0, 1, 2, 3)
- $\epsilon_{it}$ = residual error

> [!warning]
> The time scores ($\lambda_t$) must be chosen carefully. They define what "one unit of change" means in your model.

### Key Components of Linear LGCM

A basic linear LGCM has these components:

1. **Intercept factor (i)**: Loadings fixed at 1 for all time points
2. **Slope factor (s)**: Loadings fixed to reflect time (e.g., 0, 1, 2, 3)
3. **Factor means**: Average starting point and average rate of change
4. **Factor variances**: Individual differences in starting points and change rates
5. **Factor covariance**: Relationship between where people start and how fast they change
6. **Residual variances**: Occasion-specific measurement error

## Model Specification & Fit

### Data Requirements

Your data should be in **wide format** for lavaan:

| id | y1 | y2 | y3 | y4 |
|----|----|----|----|----|
| 1 | 10 | 12 | 15 | 18 |
| 2 | 8 | 9 | 11 | 12 |
| ... | ... | ... | ... | ... |

### Basic lavaan Syntax

```r
# Define the model
model <- '
  # Intercept factor: loadings fixed at 1
  i =~ 1*y1 + 1*y2 + 1*y3 + 1*y4

  # Slope factor: loadings define time metric
  s =~ 0*y1 + 1*y2 + 2*y3 + 3*y4
'

# Fit the model
fit <- growth(model, data = mydata)
```

### Fit Indices

Evaluate model fit using multiple indices:

| Index | Good Fit | Acceptable |
|-------|----------|------------|
| χ² p-value | > .05 | > .01 |
| RMSEA | < .05 | < .08 |
| CFI | > .95 | > .90 |
| SRMR | < .05 | < .08 |

> [!important]
> Always report multiple fit indices. A model can have good CFI but poor RMSEA, or vice versa. The pattern of indices tells the full story.

## Worked Example

This section provides a complete, runnable R workflow demonstrating LGCM analysis.

### Setup

```r
# Load required packages
library(tidyverse)
library(lavaan)
library(MASS)

# Set seed for reproducibility
set.seed(2024)
```

### Simulate Data

```r
# Simulation parameters
n <- 200  # Sample size
times <- 4  # Number of time points

# Population parameters
mu_i <- 50    # Mean intercept
mu_s <- 3     # Mean slope (growth per time unit)
var_i <- 100  # Variance of intercepts
var_s <- 4    # Variance of slopes
cov_is <- 5   # Covariance between intercept and slope
var_e <- 25   # Residual variance

# Generate latent factors
Sigma <- matrix(c(var_i, cov_is, cov_is, var_s), 2, 2)
factors <- mvrnorm(n, c(mu_i, mu_s), Sigma)
intercepts <- factors[, 1]
slopes <- factors[, 2]

# Generate observed data
data <- data.frame(id = 1:n)
for (t in 0:(times - 1)) {
  y <- intercepts + slopes * t + rnorm(n, 0, sqrt(var_e))
  data[[paste0("y", t + 1)]] <- y
}

head(data)
```

### Visualize Individual Trajectories

```r
# Reshape to long format for plotting
data_long <- data %>%
  pivot_longer(cols = starts_with("y"),
               names_to = "time",
               values_to = "score") %>%
  mutate(time = as.numeric(gsub("y", "", time)) - 1)

# Spaghetti plot
ggplot(data_long, aes(x = time, y = score, group = id)) +
  geom_line(alpha = 0.2) +
  geom_smooth(aes(group = NULL), method = "lm", color = "red", linewidth = 1.5) +
  labs(title = "Individual Growth Trajectories",
       x = "Time", y = "Score") +
  theme_minimal()
```

### Fit the LGCM

```r
# Define linear growth model
lgcm_model <- '
  # Latent factors
  i =~ 1*y1 + 1*y2 + 1*y3 + 1*y4
  s =~ 0*y1 + 1*y2 + 2*y3 + 3*y4
'

# Fit model
fit <- growth(lgcm_model, data = data)

# View results
summary(fit, fit.measures = TRUE, standardized = TRUE)
```

### Interpret Results

Key output to examine:

1. **Latent Means**: Compare estimated intercept and slope means to true values (50, 3)
2. **Latent Variances**: Check variance estimates against true values (100, 4)
3. **Covariance**: Compare to true value (5)
4. **Fit Indices**: Ensure model fits the data well

## Reference & Resources

### Cheat Sheet

**Quick Syntax Reference**

```r
# Basic linear growth
i =~ 1*y1 + 1*y2 + 1*y3 + 1*y4
s =~ 0*y1 + 1*y2 + 2*y3 + 3*y4

# Quadratic growth (add)
q =~ 0*y1 + 1*y2 + 4*y3 + 9*y4

# Free time scores (estimate optimal spacing)
s =~ 0*y1 + 1*y2 + s3*y3 + s4*y4

# Add predictor of growth
i ~ predictor
s ~ predictor
```

**Fit Index Guidelines**

| Index | Threshold | Interpretation |
|-------|-----------|----------------|
| χ² | p > .05 | Non-significant = good fit |
| RMSEA | < .06 | Close fit |
| CFI | > .95 | Good fit |
| SRMR | < .08 | Good fit |

### Common Pitfalls

> [!caution]
> These mistakes are common but avoidable:

1. **Too few time points**: Need 3+ for linear, 4+ for quadratic
2. **Wrong time coding**: Ensure loadings match actual time metric
3. **Ignoring missing data**: Use FIML, don't listwise delete
4. **Overfitting**: Don't free parameters without theoretical justification
5. **Poor fit ignored**: Address model misspecification before interpreting

### Recommended Resources

- **Bollen & Curran (2006)**: *Latent Curve Models* - Comprehensive textbook
- **Little (2013)**: *Longitudinal Structural Equation Modeling* - Practical guide
- **lavaan tutorials**: https://lavaan.ugent.be/tutorial/growth.html
