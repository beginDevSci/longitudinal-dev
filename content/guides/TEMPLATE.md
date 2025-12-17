---
title: "Method Name (Replace Me)"
slug: "method-slug"
description: "A one-sentence summary of what the method does and when it is used."
category: "growth-models"  # Options: growth-models | mixed-models | survival | latent-variable
tags: ["longitudinal", "SEM", "multilevel"]
r_packages: ["tidyverse", "lme4", "lavaan"]
---

## Overview

This section answers high-level questions:

- What problem does this method solve?
- When is it appropriate?
- What kind of data structure is required?
- What are its biggest strengths & limitations?

Keep this section beginner-friendly and emphasize intuition.

### Why This Method?

[1-2 paragraphs explaining the problem this method solves and why it matters for longitudinal research]

> [!tip]
> Use callouts sparingly and only for emphasized insights or "rules of thumb."

### What This Method Provides

[Method Name] offers several capabilities:

- **Capability 1**: Description
- **Capability 2**: Description
- **Capability 3**: Description

> **Note:** [Important contextual note about the method]

### When This Method is Appropriate

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ time points recommended |
| **Outcome type** | Continuous (or ordinal with many categories) |
| **Sample size** | N ≥ 100 for simple models |

---

## Conceptual Foundations

Explain the theoretical or conceptual model underlying the method.

### Core Concept

[Explanation of the underlying statistical/mathematical concept]

Recommended inclusions:

- Model components and how they relate to longitudinal structure
- Why this method exists (context within SEM, MLM, or other frameworks)
- Interpretation of parameters
- Visual aids if applicable (figures, diagrams, conceptual paths)

### The Basic Equation

For a [model type], the observed score \(y_{it}\) for person \(i\) at time \(t\) is:

$$
y_{it} = \eta_{0i} + \eta_{1i} \cdot \lambda_t + \epsilon_{it}
$$

Where:
- \(\eta_{0i}\) = person \(i\)'s intercept (latent)
- \(\eta_{1i}\) = person \(i\)'s slope (latent)
- \(\lambda_t\) = time score at occasion \(t\) (fixed)
- \(\epsilon_{it}\) = residual error

> [!warning]
> [Important caveat or common misunderstanding about this equation]

### Key Components

1. **Component 1**: Description
2. **Component 2**: Description
3. **Component 3**: Description

---

## Model Specification & Fit

Cover:

- Data structure requirements
- Assumptions
- Recommended software (R packages, functions)
- Model syntax
- Fit indices and what they mean
- Pitfalls in specification

### Data Requirements

Your data should be in **wide format** for lavaan (or **long format** for lme4):

| id | y1 | y2 | y3 | y4 |
|----|----|----|----|----|
| 1 | 10 | 12 | 15 | 18 |
| 2 | 8 | 9 | 11 | 12 |
| ... | ... | ... | ... | ... |

### Basic Syntax

```r
library(package_name)

# Define the model
model <- '
  # Model specification here
  i =~ 1*y1 + 1*y2 + 1*y3 + 1*y4
  s =~ 0*y1 + 1*y2 + 2*y3 + 3*y4
'

# Fit the model
fit <- function_name(model, data = mydata)

# View results
summary(fit, fit.measures = TRUE)
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
> Always report multiple fit indices. A model can have good CFI but poor RMSEA, or vice versa.

---

## Interpretation

Explain how to read model output.

### Understanding Output

When you run `summary(fit)`, focus on:

**Fixed Effects / Factor Loadings**

```
Latent Variables:
                   Estimate  Std.Err  z-value  P(>|z|)
  i =~
    y1                1.000
    y2                1.000
```

- **Parameter 1**: Interpretation
- **Parameter 2**: Interpretation

**Random Effects / Variance Components**

```
Variances:
                   Estimate  Std.Err  z-value  P(>|z|)
   .y1                0.250    0.050    5.000    0.000
```

- **Intercept variance**: Individual differences in starting points
- **Slope variance**: Individual differences in change rates

### Effect Sizes

The slope factor \(\eta_{1i}\) represents systematic individual differences in change over time. A slope mean of 0.5 indicates [interpretation in context].

> [!tip]
> Convert to standardized units for cross-study comparison.

---

## Worked Example

This section provides a complete, runnable R workflow.

> **Note:** This section is automatically collapsed in the rendered guide.

### Setup

```r
# Load required packages
library(tidyverse)
library(package_name)

# Set seed for reproducibility
set.seed(2024)
```

### Simulate Data

```r
# Simulation parameters
n <- 500
time_points <- 4

# Generate data with individual trajectories
df <- data.frame(
  id = rep(1:n, each = time_points),
  time = rep(0:(time_points-1), n),
  y = 5 + 2*time + rnorm(n * time_points, sd = 0.5)
)

head(df)
```

### Fit the Model

```r
# Fit model
fit <- model_function(y ~ time + (time | id), data = df)

# View results
summary(fit)
```

> [!caution]
> Do not forget to check convergence warnings. Complex random effects structures often require careful tuning.

### Check Results

| Parameter | True Value | Estimated | Interpretation |
|-----------|------------|-----------|----------------|
| Intercept | 5.0 | ~5.0 | Average starting point |
| Slope | 2.0 | ~2.0 | Average rate of change |

---

## Reference & Resources

> **Note:** This section is automatically collapsed in the rendered guide.

### Cheat Sheet

**Quick Syntax Reference**

```r
# Basic patterns
model1 <- growth(model_spec, data = df)
model2 <- lmer(y ~ time + (time | id), data = df)
```

**Common Options**

| Option | Purpose |
|--------|---------|
| `fit.measures = TRUE` | Request fit indices |
| `standardized = TRUE` | Report standardized estimates |

### Common Pitfalls

> [!caution]
> These mistakes are common but avoidable:

| Issue | Why it matters | Fix |
|-------|----------------|-----|
| Misaligned time variable | Affects slope interpretation | Re-center time at meaningful point |
| Overly complex RE structure | Causes non-convergence | Start simple, build up |
| Ignoring missingness pattern | Biases estimates | Use FIML or appropriate methods |

### Key References

- **Singer & Willett (2003)**: *Applied Longitudinal Data Analysis* - Foundational text
- **Bollen & Curran (2006)**: *Latent Curve Models* - SEM perspective
- **Package documentation**: https://lavaan.org

### Related Guides

- [Compare with LMM Guide](/guides/lmm) - Mixed-effects approach
- [Multivariate Extension](/guides/mlgcm) - Multiple outcomes

---

<!--
AUTHORING NOTES (remove before publishing):

## Math Syntax
- Inline math with subscripts: Use \(...\) delimiters
  ✅ \(y_{it}\) for person \(i\)
  ❌ $y_{it}$ (underscores conflict with markdown)

- Display math: Use $$ on separate lines
  $$
  y_{it} = \eta_{0i} + \epsilon_{it}
  $$

## Callout Types
- > [!tip] - Green, for best practices
- > [!warning] - Amber, for cautions
- > [!note] - Blue, for additional info
- > [!caution] - Red, for pitfalls
- > [!important] - Amber, for key points

## Collapsible Sections
The "Worked Example" and "Reference & Resources" H2 sections are
automatically wrapped in collapsible <details> elements by the
markdown pipeline. No manual HTML needed.

## Tables
Tables are automatically wrapped for responsive horizontal scrolling.
-->
