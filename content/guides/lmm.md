---
title: "Linear Mixed Models"
slug: "lmm"
description: "Learn to fit and interpret linear mixed models (multilevel models) for longitudinal data using R with lme4."
category: "mixed-models"
tags: ["LMM", "mixed-models", "multilevel", "lme4", "longitudinal"]
r_packages: ["lme4", "lmerTest", "tidyverse", "MASS"]
---

## Overview

---

## Why Mixed Models for Longitudinal Data?

**Quick mental model**: Think of LMM as "fit one regression line per person, then summarize those lines." Each individual gets their own intercept and slope, but these estimates are informed by the whole sample—extreme values get pulled toward the group average, and people with sparse data borrow strength from others. The result is a population-average trend plus a measure of how much individuals vary around it.

When you measure the same people repeatedly, your observations aren't independent. A person who scores high at Time 1 will likely score high at Time 2. Standard regression assumes each data point is independent—violating this assumption inflates your confidence and produces misleading p-values.

**Linear Mixed Models (LMM)** solve this by explicitly modeling the dependency structure. The "mixed" in the name refers to a mixture of:

- **Fixed effects**: Parameters that apply to everyone (average intercept, average slope)
- **Random effects**: Parameters that vary across individuals (person-specific deviations)

This is exactly what you need for longitudinal data: a way to estimate population-average trends while accounting for individual differences.

### The Nesting Problem

Consider a simple example: 200 participants measured at 5 time points each. You have 1,000 observations, but they're not 1,000 independent pieces of information. The 5 observations from Person 1 are more similar to each other than to observations from Person 47.

> [!info]
> **Why nested data need mixed models**
> When you ignore nesting, you either overstate your evidence or throw away within-person information.
>
> | Approach                    | What It Does                                | Problem                                              |
> | --------------------------- | ------------------------------------------- | ---------------------------------------------------- |
> | **Ignore nesting**          | Treat all 1,000 observations as independent | SEs too small, p-values too optimistic               |
> | **Aggregate**               | Compute person-means, analyze 200 means     | Loses within-person information about change         |
> | **Repeated measures ANOVA** | Model time as fixed factor                  | Assumes compound symmetry; limited to balanced data  |
> | **Mixed models**            | Model between- and within-person variation  | Appropriate standard errors; individual trajectories |

### What Makes LMM "Mixed"?

The model combines two types of effects:

**Fixed effects** define the average trajectory:

```
Average score = γ₀₀ + γ₁₀(Time)
```

**Random effects** capture individual deviations from that average:

```
Person i's score = [γ₀₀ + u₀ᵢ] + [γ₁₀ + u₁ᵢ](Time) + εᵢₜ
```

Where:

- `γ₀₀` = Average intercept (fixed)
- `γ₁₀` = Average slope (fixed)
- `u₀ᵢ` = Person i's deviation in intercept (random)
- `u₁ᵢ` = Person i's deviation in slope (random)
- `εᵢₜ` = Residual noise at each occasion

The random effects are assumed to follow a normal distribution with mean 0. We estimate their variance—this tells us how much individuals differ.

---

## What LMM Provides

LMM offers several capabilities that make it well-suited for longitudinal analysis:

### Individual Trajectories Within a Unified Model

Each person gets their own intercept and slope, but these aren't estimated in isolation. The model "borrows strength" across individuals, leading to more stable estimates—especially for people with few observations or extreme values.

### Handles Unbalanced Data Gracefully

Real longitudinal data are messy:

- Participants miss waves
- Measurement timing varies
- Some people drop out

LMM handles all of this naturally. Unlike repeated measures ANOVA, you don't need complete data from everyone.

### Flexible Time Structures

Time can be:

- Equally spaced (0, 1, 2, 3, 4)
- Unequally spaced (0, 3, 6, 12, 24 months)
- Person-specific (actual dates of measurement)

Just code time as a continuous variable with appropriate values.

**Handling irregular timing**: LMM naturally handles person-specific measurement schedules. If participants were assessed on different dates, use the actual time values (e.g., months since baseline, or age at each assessment). The model uses each observation's actual time value—no need for everyone to be measured at identical intervals. This is one of LMM's major advantages over repeated measures ANOVA.

### Separates Within- and Between-Person Variation

LMM explicitly partitions variance into:

- **Between-person**: How much do people differ from each other?
- **Within-person**: How much does each person vary around their own trajectory?

This distinction is fundamental to understanding longitudinal change.

### Natural Framework for Predictors

Adding predictors is straightforward:

- **Time-invariant predictors** (e.g., treatment group, gender): Predict individual differences in trajectories
- **Time-varying predictors** (e.g., daily stress, current medication): Predict occasion-specific deviations

### What You'll Do in R

By the end of this tutorial, you will use `lmer()` from the **lme4** package to estimate growth curves, compare random intercept and random slope models, extract individual trajectories (BLUPs), and visualize how shrinkage pulls extreme estimates toward the group mean. The Worked Example section provides a complete, runnable script you can adapt for your own data.

---

## When LMM is Appropriate

LMM works well when you have:

| Requirement            | Guideline                                                                       |
| ---------------------- | ------------------------------------------------------------------------------- |
| **Repeated measures**  | 3+ observations per person (fewer is possible but limits what you can estimate) |
| **Continuous outcome** | Or ordinal with many categories; see GLMM for binary/count outcomes             |
| **Nested structure**   | Observations clearly belong to individuals                                      |
| **Interest in change** | Not just "do groups differ?" but "how do individuals change?"                   |
| **Adequate sample**    | 50+ individuals for simple models; 100+ recommended                             |

### Example Scenarios Where LMM Shines

- **Therapy outcome study**: 150 clients complete depression inventories at each of 8 weekly sessions. Some clients miss sessions. You want to estimate average symptom reduction and identify who improves faster.
- **Cognitive aging study**: 300 older adults complete memory tests every 2 years over a decade. Timing varies slightly across participants. You want to model decline trajectories and test whether education predicts slower decline.
- **Intensive longitudinal diary study**: 80 participants report daily mood for 30 days. You're interested in how daily stress relates to mood fluctuations within persons, while also capturing stable individual differences in average mood.

### When to Consider Alternatives

- **Only 2 time points**: Simpler methods (change scores, ANCOVA) may suffice
- **Categorical outcomes**: Generalized linear mixed models (GLMM)
- **Interest in latent variables**: Structural equation modeling (SEM) / LGCM
- **Complex dependency structures**: Time series models for intensive longitudinal data
- **Interest in discrete trajectory classes**: Growth mixture models

> **Note on LGCM**: A companion tutorial covers **Latent Growth Curve Models (LGCM)**, an SEM-based approach to longitudinal analysis. For basic growth models (random intercept and slope), LMM and LGCM produce mathematically equivalent estimates—the choice often comes down to software preference and whether you need SEM features like latent variables or fit indices. See [LMM vs. LGCM Comparison](#lmm-vs-lgcm-comparison) for details.

---

## What You'll Learn

> [!tip]
> **Quick start path if you're short on time**
> - Read the *Quick mental model* paragraph at the start of [Why Mixed Models for Longitudinal Data?](#why-mixed-models-for-longitudinal-data).
> - Run the *Quick Start* code block in [Quick Start (Impatient Reader Version)](#quick-start-impatient-reader-version).
> - Skim the [Cheat Sheet](#cheat-sheet) for syntax reference.
>
> This gives you enough to fit a basic model and interpret the output. Come back to the full tutorial when you need deeper understanding.

By the end of this tutorial, you will be able to:

1. **Understand** the logic of mixed models for longitudinal data
2. **Recognize** when LMM is appropriate for your research question
3. **Specify** random intercept and random slope models in R
4. **Estimate** models using lme4 and interpret the output
5. **Compare** nested models using likelihood ratio tests
6. **Interpret** fixed effects, variance components, and R²
7. **Extract** and visualize individual trajectories
8. **Avoid** common pitfalls

### How This Tutorial is Organized

| Section                       | Purpose                | When to Use            |
| ----------------------------- | ---------------------- | ---------------------- |
| **Overview** (you are here)   | Orientation            | Read first             |
| **Conceptual Foundations**    | Build intuition        | Understand the model   |
| **Model Specification & Fit** | Technical details      | Set up your analysis   |
| **Worked Example**            | Complete analysis in R | Follow along with code |
| **Reference**                 | Lookup materials       | Consult as needed      |

You can read straight through or jump to specific sections. The Reference section is designed for lookup—consult it when you need syntax help, troubleshooting, or want to explore extensions.

---

## Conceptual Foundations

This section builds your intuition for how LMM works. We'll start simple and add complexity gradually.

---

## The Two-Level Structure

LMM simultaneously answers two questions: *How do people change on average?* (the fixed effects) and *How do individuals differ in that change?* (the random effects). The two-level structure makes this possible.

Longitudinal data have a natural hierarchy:

```
Level 2: Persons (i = 1, 2, ..., N)
    │
    └── Level 1: Observations within persons (t = 1, 2, ..., Tᵢ)
```

**Level 1** describes what happens _within_ each person over time. This is where change lives.

**Level 2** describes how people _differ from each other_. This is where individual differences live.

### A Concrete Example

Imagine tracking depression symptoms over 5 therapy sessions for 100 clients.

**Level 1 question**: "Does this person's depression decrease over sessions?"

**Level 2 question**: "Do some people improve faster than others? Does treatment type predict rate of improvement?"

LMM addresses both simultaneously.

### The Level-1 Model

For person _i_ at time _t_:

```
yᵢₜ = β₀ᵢ + β₁ᵢ(Timeₜ) + εᵢₜ
```

Each person has their own intercept (β₀ᵢ) and slope (β₁ᵢ). The residual εᵢₜ captures occasion-specific deviation.

### The Level-2 Model

But where do those person-specific parameters come from? Level 2:

```
β₀ᵢ = γ₀₀ + u₀ᵢ    (intercept)
β₁ᵢ = γ₁₀ + u₁ᵢ    (slope)
```

Each person's intercept is the average intercept (γ₀₀) plus a person-specific deviation (u₀ᵢ). Same for slope.

### The Combined Model

Substituting Level 2 into Level 1:

```
yᵢₜ = γ₀₀ + γ₁₀(Time) + u₀ᵢ + u₁ᵢ(Time) + εᵢₜ
      \_____________/   \_______________/   \__/
       Fixed effects    Random effects     Residual
```

This is the "mixed" model: fixed effects (γ) plus random effects (u) plus residual (ε).

---

## Fixed vs. Random Effects

This distinction is central to mixed models—and a common source of confusion.

### Fixed Effects

**What they are**: Population-average parameters. The intercept and slope that describe the "typical" trajectory.

**What they tell you**: "On average, people start at X and change by Y per unit time."

**Notation**: γ₀₀, γ₁₀ (sometimes β₀, β₁ in simpler notation)

**In lme4 output**: Listed under "Fixed effects"

### Random Effects

**What they are**: Individual-specific deviations from the fixed effects. Not estimated directly as parameters, but characterized by their variance.

**What they tell you**: "People vary around the average intercept with SD = X. People vary around the average slope with SD = Y."

**Notation**: u₀ᵢ, u₁ᵢ (the effects themselves); τ₀₀, τ₁₁ (their variances)

**In lme4 output**: Listed under "Random effects" as variance components

### The Key Insight

We don't estimate a separate intercept for each person as a "parameter." Instead, we estimate:

1. The **average** intercept (fixed effect)
2. The **variance** of intercepts across people (random effect variance)

From these, we can derive person-specific estimates (BLUPs—more on this later).

> [!didactic]
> **What's estimated vs. what's derived**
> Mixed models estimate the *variance* of random effects, not each person's intercept and slope as separate free parameters.
> The person-specific estimates you extract (BLUPs) are derived quantities with their own uncertainty, not directly estimated coefficients.
> This matters because BLUPs should not be treated as error-free datapoints in secondary analyses.

### When to Treat an Effect as Random

| Treat as Fixed                       | Treat as Random                           |
| ------------------------------------ | ----------------------------------------- |
| Small number of categories           | Many units sampled from larger population |
| Categories are the focus of interest | Units are incidental to research question |
| Want to compare specific categories  | Want to generalize beyond sample          |
| Example: Treatment vs. Control       | Example: Individual participants          |

In longitudinal analysis, **person** is almost always random (you want to generalize beyond your specific 200 participants). **Time** is typically fixed when modeled as a continuous covariate.

---

## Building Up: From Intercept to Slope

Let's build intuition by starting simple and adding complexity.

### Model 0: Complete Pooling (No Random Effects)

Ignore that observations are nested:

```
yᵢₜ = β₀ + β₁(Time) + εᵢₜ
```

One intercept, one slope for everyone. The residual εᵢₜ captures _all_ deviation from the line.

**What this model assumes**:
- Observations are independent (clearly wrong for longitudinal data)
- Everyone follows the exact same trajectory
- All variation is random noise

**Problem**: Standard errors are wrong because we're treating correlated observations as independent.

> [!pitfall]
> **Model 0 is for illustration only.**
> Model 0 is shown to illustrate what *not* to do. It ignores the nested structure of the data and produces misleading standard errors and p-values.
> **Do not use a no-random-effects model for longitudinal data in practice.**

### Model 1: Random Intercept Only

Allow intercepts to vary:

```
yᵢₜ = [γ₀₀ + u₀ᵢ] + γ₁₀(Time) + εᵢₜ
```

Each person has their own intercept, but everyone shares the same slope. Lines are parallel.

**What this model assumes**:
- People differ in their starting level
- Everyone changes at the *same rate* over time
- Random intercepts are normally distributed
- Residuals are independent and identically distributed

**What this captures**: People differ in their baseline level, but change at the same rate.

**When this is enough**: If you have few time points, or if slope variance is genuinely small.

**Key takeaways**

- Random intercept models allow people to start at different levels but assume a common rate of change.
- When slopes truly differ, forcing them to be equal pushes that variability into the residual and can hide important heterogeneity.

### Model 2: Random Intercept and Slope

Allow both to vary:

```
yᵢₜ = [γ₀₀ + u₀ᵢ] + [γ₁₀ + u₁ᵢ](Time) + εᵢₜ
```

Each person has their own intercept _and_ slope. Lines can have different starting points and different angles.

**What this model assumes**:
- People differ in both their starting level and rate of change
- Random intercepts and slopes follow a bivariate normal distribution
- The intercept-slope correlation is the same across the population
- Residuals are independent and identically distributed

**What this captures**: People differ in where they start AND how fast they change.

**The correlation**: Random intercepts and slopes can be correlated. Negative correlation means high starters tend to have flatter slopes (or steeper declines).

**Key takeaways**

- Random intercept + slope models let both starting point and rate of change vary across people.
- The intercept–slope correlation tells you whether higher starters tend to change faster, slower, or at the same rate.

### Visualizing the Difference

```
Random Intercept Only          Random Intercept + Slope
(parallel lines)               (non-parallel lines)

    │    ╱                         │      ╱
    │   ╱                          │   ╱╱
    │  ╱                           │  ╱ ╱
    │ ╱                            │ ╱  ╱
    │╱                             │╱  ╱
    └──────────                    └─────────
       Time                           Time

All same slope,                  Different people,
different intercepts             different slopes
```

---

## Shrinkage and Partial Pooling

This concept is unique to mixed models and represents one of their key advantages.

### The Problem with Extreme Estimates

Suppose Person A has only 2 observations, both very high. If we estimate their trajectory independently, we'd get an extreme intercept based on just 2 points.

But we have information from 199 other people. Shouldn't that tell us something?

### What Shrinkage Does

Mixed models "shrink" extreme individual estimates toward the group mean. The amount of shrinkage depends on:

1. **How extreme the person's data are**: More extreme = more shrinkage
2. **How reliable the person's data are**: Fewer observations = more shrinkage
3. **How variable the population is**: Less population variability = more shrinkage

### Intuition: "Regression to the Mean, Done Right"

If Person A has an extreme estimated intercept, it's probably partly real and partly noise. Shrinkage gives us a better guess by pulling toward the average—but only as much as warranted by the data.

### Visualizing Shrinkage

```
                    ○ Person A (raw estimate: very high, n=2)
                   ╱
                  ╱  shrinkage pulls toward mean
                 ╱
        ────────●───────────  Group mean
                 ╲
                  ╲  shrinkage pulls toward mean
                   ╲
                    ○ Person B (raw estimate: very low, n=2)
```

Persons with more observations experience less shrinkage—their estimates are more reliable.

### Why This Matters

1. **Better individual predictions**: Shrunken estimates are typically more accurate than raw OLS estimates for each person
2. **Handles sparse data**: People with few observations still get reasonable estimates
3. **Automatic regularization**: Protects against overfitting

### Three Approaches to Individual Estimation

| Approach | What it does | When it struggles |
|----------|-------------|-------------------|
| **Complete pooling** | Everyone gets the grand mean—ignores individual differences entirely | Misses real heterogeneity |
| **No pooling (OLS)** | Fit separate regression for each person—ignores others' data | Extreme estimates with sparse data; overfits |
| **Partial pooling (LMM)** | Each person's estimate is a weighted average of their data and the group mean | Best of both worlds; handles sparse data gracefully |

For example, suppose the group mean intercept is 60. Two people have noisy separate OLS estimates based on very little data:

| Person | OLS intercept (no pooling) | BLUP intercept (partial pooling) | Group mean |
|--------|----------------------------|----------------------------------|------------|
| A      | 80                         | 70                               | 60         |
| B      | 40                         | 50                               | 60         |

Person A's very high OLS intercept is pulled down toward 60; Person B's very low intercept is pulled up. The less data a person has, the stronger this pull toward the group mean.

LMM's partial pooling gives more weight to an individual's data when they have many observations and the data are consistent, and more weight to the group mean when observations are sparse or variable. This adaptive weighting is why mixed models typically outperform both extremes.

### Shrinkage in Practice

The figure below demonstrates shrinkage by comparing OLS estimates (computed separately for each person, ignoring others) with BLUP estimates (from the mixed model, borrowing strength across people).

![Shrinkage Demonstration](/images/guides/lmm/lmm_fig10_shrinkage.png)

_Figure 10: Shrinkage of intercept and slope estimates. Points below the diagonal line indicate that the BLUP estimate is closer to the grand mean than the OLS estimate. Arrows show the direction of shrinkage._

Key observations:

- Points that lie far from the grand mean (red dotted lines) in the OLS estimates are pulled inward in the BLUP estimates
- The degree of shrinkage depends on how extreme the estimate is and how reliable the person's data are
- This "borrowing of strength" across individuals is a core advantage of mixed models

**Key takeaways**

- Mixed models automatically shrink noisy individual estimates toward the group mean, especially when data per person are sparse.
- Partial pooling typically yields more accurate individual predictions than either complete pooling or separate OLS per person.

---

## Choosing Your Random Effects Structure

A key modeling decision: which effects should be random?

### The Principle

Include random effects for:

- Effects that **can** vary across individuals
- Effects whose variance you **want to estimate**
- Effects where you have **enough data** to estimate variance

### Common Structures for Longitudinal Data

| Structure                               | lme4 formula | What it assumes |
| --------------------------------------- | ------------ | --------------- | ----------------------------------------- | ---------------------------- |
| Random intercept only                   | `(1          | id)`            | Same slope for everyone                   |
| Random intercept + slope                | `(1 + time   | id)`            | Intercepts and slopes vary, can correlate |
| Random intercept + slope (uncorrelated) | `(1          | id) + (0 + time | id)`                                      | Both vary, but independently |

### Guidance

**Start simple**: Begin with random intercept only. Add random slope if theoretically motivated or if model comparison supports it.

**Don't over-specify**: A random slope with only 3 time points per person is hard to estimate. You need enough within-person observations.

> **Key constraint**: The number of *observations per person*—not total sample size—limits what random effects you can estimate. Having 1,000 people with 3 time points each doesn't let you estimate a random quadratic; you need more repeated measures per person for that.

**Watch for convergence**: Complex random effects structures may not converge. This often signals that the data don't support that complexity.

**Let theory guide you**: If individual differences in change rate are central to your question, you probably need a random slope.

### The "Keep It Maximal" Debate

Some argue you should include the maximal random effects structure justified by design (Barr et al., 2013). Others argue for parsimony (Matuschek et al., 2017).

For longitudinal data with a continuous time variable, a random slope is often theoretically motivated and worth including if the model converges.

---

## Interactive Exploration

> [!info]
> **Optional: interactive visualization**
> The interactive tool below is useful for building intuition about how random effects and variance components shape trajectories, but it's not required to understand or use the models in this tutorial.

To build intuition for how LMM parameters shape individual trajectories, try the interactive visualization:

**[Random Effects Explorer](/images/guides/lmm/interactive/lmm_random_effects_explorer.html)**

This tool lets you:

- Adjust fixed effects (average intercept and slope) and see the mean trajectory change
- Modify random effect variances and watch individual trajectories spread out or converge
- Change the intercept-slope correlation to see fan-in or fan-out patterns
- Observe how the ICC changes as you adjust between- and within-person variance
- Toggle shrinkage visualization to compare OLS vs. BLUP estimates

Experiment with extreme values to develop intuition for what each parameter controls.

---

## Model Specification & Fit

This section covers the technical details: preparing data, specifying models in R, and evaluating fit.

---

## Data Requirements

### Data Format: Long vs. Wide

LMM requires **long format**: one row per observation.

**Wide format** (each row is a person):

```
id    y_time1  y_time2  y_time3  y_time4  y_time5
1     48       52       54       56       59
2     55       54       57       55       58
```

**Long format** (each row is an observation):

```
id    time    y
1     0       48
1     1       52
1     2       54
1     3       56
1     4       59
2     0       55
2     1       54
...
```

### Converting Wide to Long

```r
library(tidyverse)

data_long <- data_wide %>%
  pivot_longer(
    cols = starts_with("y_time"),
    names_to = "wave",
    names_prefix = "y_time",
    values_to = "y"
  ) %>%
  mutate(time = as.numeric(wave) - 1)  # Code time as 0, 1, 2, 3, 4
```

### Time Coding

> [!pitfall]
> **Time must be numeric for growth models.**
> If your time variable is a factor, `lmer()` will estimate dummy variables for each time point rather than a single slope.
> This gives you wave-specific means, not a growth trajectory. Before fitting, check:
>
> ```r
> class(data$time)
> ```
>
> If it says `"factor"` or `"character"`, convert or recode so that time is numeric (for example, `0, 1, 2, 3, 4`).

How you code time affects interpretation:

| Coding         | Time values               | Intercept means                                     |
| -------------- | ------------------------- | --------------------------------------------------- |
| Zero at start  | 0, 1, 2, 3, 4             | Expected score at baseline                          |
| Zero at center | -2, -1, 0, 1, 2           | Expected score at middle wave                       |
| Actual time    | 0, 6, 12, 18, 24 (months) | Expected score at month 0; slope = change per month |

**Recommendation**: Start with zero at baseline (0, 1, 2, ...). Adjust if your research question focuses on a different time point.

### Sample Size Considerations

| What you're estimating                | Rough minimum              |
| ------------------------------------- | -------------------------- |
| Fixed effects only (random intercept) | 30+ individuals            |
| Random slopes                         | 50+ individuals            |
| Complex random structures             | 100+ individuals           |
| Cross-level interactions              | 100+ individuals per group |

More observations per person help estimate within-person effects and random slope variance. More individuals help estimate between-person effects and variance components.

### Missing Data

LMM handles missing observations gracefully:

- Uses all available data (no listwise deletion)
- Assumes data are **Missing at Random (MAR)**: missingness can depend on observed variables but not on the missing values themselves
- No special syntax needed—just include all observations you have

**Caution**: If dropout is related to the outcome trajectory itself (MNAR), estimates may be biased. Consider sensitivity analyses.

---

## Specifying Models in lme4

The `lme4` package uses a formula syntax that separates fixed and random effects.

### Basic Syntax

```r
library(lme4)

model <- lmer(y ~ fixed_effects + (random_effects | grouping), data = data)
```

### Fixed Effects

Specified just like in `lm()`:

```r
y ~ 1                    # Intercept only
y ~ time                 # Intercept + time slope
y ~ time + treatment     # + treatment effect
y ~ time * treatment     # + time × treatment interaction
```

### Random Effects

Specified in parentheses with `|` indicating the grouping variable:

```r
(1 | id)                 # Random intercept for each id
(1 + time | id)          # Random intercept and slope, allowed to correlate
(0 + time | id)          # Random slope only (no random intercept)
(1 | id) + (0 + time | id)  # Random intercept and slope, uncorrelated
```

### Common Models for Longitudinal Data

**Random intercept only**:

```r
mod_ri <- lmer(y ~ time + (1 | id), data = data_long)
```

**Random intercept and slope**:

```r
mod_rs <- lmer(y ~ time + (1 + time | id), data = data_long)
```

**With a between-person predictor**:

```r
mod_pred <- lmer(y ~ time + treatment + (1 + time | id), data = data_long)
```

**With a cross-level interaction** (does treatment affect slope?):

```r
mod_int <- lmer(y ~ time * treatment + (1 + time | id), data = data_long)
```

---

## Estimation: ML vs. REML

In practice, use ML when comparing models with different fixed effects, then refit your final chosen model with REML for reporting variance components.

lme4 offers two estimation methods:

### Maximum Likelihood (ML)

- Estimates fixed effects and variance components simultaneously
- Variance estimates are slightly **biased downward** (especially with few groups)
- **Required for comparing models with different fixed effects**

```r
mod_ml <- lmer(y ~ time + (1 | id), data = data, REML = FALSE)
```

### Restricted Maximum Likelihood (REML)

- First removes fixed effects, then estimates variance components
- Variance estimates are **less biased**
- **Default in lme4**
- Cannot compare models with different fixed effects

```r
mod_reml <- lmer(y ~ time + (1 | id), data = data, REML = TRUE)  # default
```

### When to Use Which

| Situation                                                              | Use        |
| ---------------------------------------------------------------------- | ---------- |
| Final model, reporting variance components                             | REML       |
| Comparing models with **same** fixed effects, different random effects | REML or ML |
| Comparing models with **different** fixed effects                      | ML         |

**Workflow**: Use ML for model comparison, then refit final model with REML for reporting.

> **Quick recipe**:
> 1. **Model comparison**: Fit competing models with `REML = FALSE`
> 2. **Select best model**: Use `anova()` or AIC/BIC
> 3. **Final reporting**: Refit the chosen model with `REML = TRUE` (the default) for unbiased variance estimates

---

## Model Comparison

### Likelihood Ratio Test (Nested Models)

For nested models (one is a constrained version of the other):

```r
mod_ri <- lmer(y ~ time + (1 | id), data = data, REML = FALSE)
mod_rs <- lmer(y ~ time + (1 + time | id), data = data, REML = FALSE)

anova(mod_ri, mod_rs)
```

The test compares model fit via the change in log-likelihood.

**Note**: Testing whether variance = 0 is a boundary test. The reported p-value is conservative (the true p-value is approximately half).

### Information Criteria

**AIC** and **BIC** allow comparison of non-nested models:

```r
AIC(mod_ri, mod_rs)
BIC(mod_ri, mod_rs)
```

- Lower = better
- AIC favors complexity slightly more than BIC
- BIC is more conservative (penalizes parameters more)

### Comparing Fixed Effects

To compare models with different fixed effects (e.g., does adding treatment improve fit?):

```r
mod1 <- lmer(y ~ time + (1 + time | id), data = data, REML = FALSE)
mod2 <- lmer(y ~ time + treatment + (1 + time | id), data = data, REML = FALSE)

anova(mod1, mod2)
```

**Important**: Use `REML = FALSE` when comparing models with different fixed effects.

---

## Evaluating Model Fit

Unlike SEM, LMM doesn't produce absolute fit indices like CFI or RMSEA. Instead, we use:

### Variance Explained: R²

The `performance` package provides pseudo-R² measures:

```r
library(performance)

r2(mod_rs)
```

This returns:

- **Marginal R²**: Variance explained by fixed effects alone
- **Conditional R²**: Variance explained by fixed + random effects

### Residual Diagnostics

Check model assumptions visually:

```r
# Residuals vs. fitted
plot(mod_rs)

# Q-Q plot for residuals
qqnorm(resid(mod_rs))
qqline(resid(mod_rs))

# Q-Q plot for random effects (BLUPs)
qqnorm(ranef(mod_rs)$id[,1])  # Random intercepts
qqline(ranef(mod_rs)$id[,1])
```

**Note on Q-Q plots for random effects**: This plot checks whether the BLUPs (person-specific deviations) look approximately normally distributed—a rough diagnostic for the assumption that random effects follow a normal distribution. Mild departures are usually tolerable, especially with large samples. Severe non-normality might indicate model misspecification or suggest exploring alternative distributions.

### What to Look For

| Diagnostic                | Good                    | Problematic                 |
| ------------------------- | ----------------------- | --------------------------- |
| Residuals vs. fitted      | Random scatter around 0 | Funnel shape, curves        |
| Q-Q plot (residuals)      | Points on line          | Heavy tails, skewness       |
| Q-Q plot (random effects) | Points on line          | Heavy tails (less critical) |

### Convergence Warnings

lme4 will warn you if:

- Model fails to converge
- Singular fit (variance estimate at boundary, usually 0)

**Singular fit** often means:

- Random effect variance is very small
- Model is over-specified for the data
- Consider simplifying random effects structure

---

## Inference for Fixed Effects

### The p-Value Problem

Unlike `lm()`, `lmer()` does **not** provide p-values by default. Why?

The issue is degrees of freedom. In simple regression, df = n - p. In mixed models, the effective sample size is ambiguous—somewhere between the number of observations and the number of groups.

> [!didactic]
> **How to obtain p-values for mixed models**
> - **Option 1 (recommended)**: Use `lmerTest` with Satterthwaite degrees of freedom for routine analyses.
> - **Option 2**: Use Kenward–Roger via `lmerTest` for more accurate small-sample inference.
> - **Option 3**: Use parametric bootstrap for high-precision tests (slower, but robust).
> - **Option 4**: Use confidence intervals (profile or bootstrap) when you prefer interval estimates over p-values.

### Solutions

**1. lmerTest package** (recommended for most cases):

```r
library(lmerTest)

mod <- lmer(y ~ time + (1 + time | id), data = data)
summary(mod)  # Now includes p-values via Satterthwaite approximation
```

**2. Kenward-Roger approximation** (more accurate, slower):

```r
library(lmerTest)

mod <- lmer(y ~ time + (1 + time | id), data = data)
anova(mod, ddf = "Kenward-Roger")
```

**3. Parametric bootstrap** (most accurate, slowest):

```r
library(pbkrtest)

# Compare models
PBmodcomp(mod_full, mod_reduced, nsim = 1000)
```

**4. Confidence intervals** (sidestep p-values):

```r
confint(mod, method = "profile")  # Profile likelihood
confint(mod, method = "boot")     # Bootstrap
```

### Practical Recommendation

Use `lmerTest` with Satterthwaite for routine analyses. For small samples or when precision matters, use Kenward-Roger or bootstrap.

**Note on method discrepancies**: Satterthwaite, Kenward-Roger, and bootstrap can yield slightly different p-values for the same effect, especially with small samples or complex models. With large samples and simple models, differences are usually trivial. If you see meaningful disagreement between methods, treat it as a flag that your conclusions may be sensitive to analytical choices—consider reporting multiple methods or relying on confidence intervals and effect sizes rather than precise p-values.

---

## Worked Example

This section walks through a complete analysis. Run the code yourself to see LMM in action.

### Quick Start (Impatient Reader Version)

> [!tip]
> **Quick Start: minimal working example**
> If you're following the quick start path, run this first, then skim the workflow diagram below.
>
> ```r
> library(lme4)
> library(lmerTest)
> # Assuming data_long has columns: id, time, y
> mod <- lmer(y ~ time + (1 + time | id), data = data_long)
> summary(mod)
> ```

This fits a random intercept and slope model. Keep reading for the full story.

---

## Practical Workflow Overview

```
┌─────────────────────────────────┐
│ 1. Setup                        │
│    Load packages                │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 2. Prepare Data                 │
│    Long format, check structure │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 3. Visualize                    │
│    Spaghetti plot, look first   │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 4. Fit Random Intercept Model   │
│    Baseline comparison          │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 5. Fit Random Slope Model       │
│    Main model of interest       │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 6. Compare Models               │
│    LRT, AIC/BIC                 │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 7. Interpret Results            │
│    Fixed effects, variances, R² │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 8. Extract Random Effects       │
│    Individual trajectories      │
└─────────────────────────────────┘
```

---

## Setup

```r
# Load packages
library(tidyverse)   # Data manipulation and plotting
library(lme4)        # Mixed models
library(lmerTest)    # p-values for lmer
library(performance) # R² and diagnostics

# Set seed for reproducibility
set.seed(2024)
```

---

## Simulate Data

> [!note]
> **Already have your own data?**
> If you already have a longitudinal dataset in long format with `id`, `time`, and `y`, you can skim this section and jump ahead to [Visualize](#visualize) and [Fit Random Intercept Model](#fit-random-intercept-model). Simulation here is mainly for learning and checking that the model recovers known parameters.

We'll simulate data with known parameters so we can verify our estimates.

**Why simulate?** Generating data with known "true" parameters lets you confirm that your modeling approach recovers those parameters. This is an excellent learning tool: if your estimates are close to the true values, you know your code is correct. It also builds intuition for how sample size, variance components, and other factors affect precision.

**Population parameters**:

| Parameter                        | True Value     |
| -------------------------------- | -------------- |
| Average intercept (γ₀₀)          | 50             |
| Average slope (γ₁₀)              | 2              |
| Intercept variance (τ₀₀)         | 100 (SD = 10)  |
| Slope variance (τ₁₁)             | 1 (SD = 1)     |
| Intercept-slope covariance (τ₀₁) | -2 (r ≈ -0.20) |
| Residual variance (σ²)           | 25 (SD = 5)    |

```r
# Parameters
n_persons <- 200
n_time <- 5
time_points <- 0:4

# Fixed effects
gamma_00 <- 50  # Average intercept
gamma_10 <- 2   # Average slope

# Random effects covariance matrix
tau <- matrix(c(100, -2,
                -2,   1), nrow = 2)

# Residual SD
sigma <- 5

# Generate random effects for each person
library(MASS)
random_effects <- mvrnorm(n = n_persons, mu = c(0, 0), Sigma = tau)
u0 <- random_effects[, 1]  # Random intercepts
u1 <- random_effects[, 2]  # Random slopes

# Generate data in long format
data_long <- expand_grid(
  id = 1:n_persons,
  time = time_points
) %>%
  mutate(
    u0_i = u0[id],
    u1_i = u1[id],
    y = gamma_00 + u0_i + (gamma_10 + u1_i) * time + rnorm(n(), 0, sigma)
  ) %>%
  select(id, time, y)

# Check structure
head(data_long, 10)
```

_Note: Your output will differ slightly due to random sampling, but the structure and general patterns should match._

```
# A tibble: 10 × 3
      id  time     y
   <int> <int> <dbl>
 1     1     0  61.2
 2     1     1  62.8
 3     1     2  65.1
 4     1     3  66.4
 5     1     4  69.2
 6     2     0  44.3
 7     2     1  47.8
 8     2     2  50.1
 9     2     3  53.2
10     2     4  54.9
```

---

## Visualize

**Always plot before modeling.**

```r
# Spaghetti plot: all individual trajectories
ggplot(data_long, aes(x = time, y = y, group = id)) +
  geom_line(alpha = 0.2, color = "gray40") +
  scale_x_continuous(breaks = 0:4, labels = paste("Time", 0:4)) +
  labs(x = "Time", y = "Score",
       title = "Individual Growth Trajectories",
       subtitle = paste("N =", n_persons, "participants")) +
  theme_minimal()
```

**What to look for**:

- General trend (up, down, flat?)
- Spread at baseline (intercept variance)
- Fan pattern (slope variance)—do lines diverge or stay parallel?
- Nonlinearity (curves vs. straight lines)
- Outliers

```r
# Add mean trajectory
mean_trajectory <- data_long %>%
  group_by(time) %>%
  summarise(mean_y = mean(y), .groups = "drop")

ggplot(data_long, aes(x = time, y = y)) +
  geom_line(aes(group = id), alpha = 0.15, color = "gray40") +
  geom_line(data = mean_trajectory, aes(y = mean_y),
            color = "steelblue", linewidth = 1.5) +
  geom_point(data = mean_trajectory, aes(y = mean_y),
             color = "steelblue", size = 3) +
  scale_x_continuous(breaks = 0:4) +
  labs(x = "Time", y = "Score",
       title = "Individual Trajectories with Mean Overlay") +
  theme_minimal()
```

---

## Fit Random Intercept Model

Start with a baseline model: random intercepts only (slopes fixed across people).

```r
mod_ri <- lmer(y ~ time + (1 | id), data = data_long)

summary(mod_ri)
```

**Output interpretation**:

```
Random effects:
 Groups   Name        Variance Std.Dev.
 id       (Intercept) 85.2     9.23
 Residual             45.3     6.73
Number of obs: 1000, groups:  id, 200

Fixed effects:
            Estimate Std. Error      df t value Pr(>|t|)
(Intercept)  49.856      0.712  198.00   70.02   <2e-16 ***
time          2.014      0.067  799.00   30.06   <2e-16 ***
```

**Note**: This model forces everyone to have the same slope. The "missing" slope variance gets absorbed into the residual.

---

## Fit Random Slope Model

Now allow slopes to vary across individuals.

**Fit the random intercept and slope model**

```r
mod_rs <- lmer(y ~ time + (1 + time | id), data = data_long)
summary(mod_rs)
```

**Expected output (truncated)**

```
Random effects:
 Groups   Name        Variance Std.Dev. Corr
 id       (Intercept) 98.54    9.93
          time         0.89    0.94   -0.18
 Residual             24.89    4.99
Number of obs: 1000, groups:  id, 200

Fixed effects:
            Estimate Std. Error       df t value Pr(>|t|)
(Intercept)  49.923      0.732  199.000   68.21   <2e-16 ***
time          2.021      0.085  199.000   23.78   <2e-16 ***
...
```

**Output interpretation**:

**Comparing to true values**:

| Parameter          | True Value | Estimate |
| ------------------ | ---------- | -------- |
| Intercept mean     | 50         | 49.92    |
| Slope mean         | 2          | 2.02     |
| Intercept variance | 100        | 98.54    |
| Slope variance     | 1          | 0.89     |
| I-S correlation    | -0.20      | -0.18    |
| Residual variance  | 25         | 24.89    |

The estimates closely recover the true population parameters.

---

## Compare Models

Is the random slope model significantly better than random intercept only?

```r
# Use ML for model comparison
mod_ri_ml <- lmer(y ~ time + (1 | id), data = data_long, REML = FALSE)
mod_rs_ml <- lmer(y ~ time + (1 + time | id), data = data_long, REML = FALSE)

anova(mod_ri_ml, mod_rs_ml)
```

**Output**:

```
Data: data_long
Models:
mod_ri_ml: y ~ time + (1 | id)
mod_rs_ml: y ~ time + (1 + time | id)
          npar    AIC    BIC  logLik deviance  Chisq Df Pr(>Chisq)
mod_ri_ml    4 6234.5 6254.1 -3113.2   6226.5
mod_rs_ml    6 6142.8 6172.2 -3065.4   6130.8 95.68  2  < 2.2e-16 ***
```

The random slope model fits significantly better (p < .001).

**Information criteria**:

```r
# AIC/BIC comparison
data.frame(
  Model = c("Random Intercept", "Random Slope"),
  AIC = c(AIC(mod_ri_ml), AIC(mod_rs_ml)),
  BIC = c(BIC(mod_ri_ml), BIC(mod_rs_ml))
)
```

Both AIC and BIC favor the random slope model.

---

## Interpret Results

### Fixed Effects

```r
# Fixed effects with confidence intervals
fixef(mod_rs)
confint(mod_rs, parm = "beta_", method = "Wald")
```

**Interpretation**:

| Parameter | Estimate | 95% CI         | Interpretation               |
| --------- | -------- | -------------- | ---------------------------- |
| Intercept | 49.92    | [48.49, 51.36] | Average score at Time 0      |
| Slope     | 2.02     | [1.85, 2.19]   | Average change per time unit |

"On average, participants started at 49.92 and increased by 2.02 units per time point."

### Variance Components

```r
VarCorr(mod_rs)
```

**Interpretation**:

| Component   | Variance | SD   | Interpretation                            |
| ----------- | -------- | ---- | ----------------------------------------- |
| Intercept   | 98.54    | 9.93 | People differ in starting level (SD ≈ 10) |
| Slope       | 0.89     | 0.94 | People differ in rate of change (SD ≈ 1)  |
| Correlation | -0.18    | —    | Higher starters grow slightly slower      |
| Residual    | 24.89    | 4.99 | Occasion-specific noise (SD ≈ 5)          |

### Intraclass Correlation (ICC)

From the random intercept model:

```r
# ICC = between-person variance / total variance
icc(mod_ri)
```

ICC ≈ 0.65 means 65% of the total variance is between persons (stable individual differences), and 35% is within persons (change over time + noise).

**Substantive interpretation**: An ICC of 0.65 tells you that people differ substantially from one another—their scores at any given time point are more similar to their own other time points than to other people's scores. This high ICC justifies using mixed models: there's meaningful clustering to account for. If ICC were near 0, you might question whether random intercepts add much.

![ICC Visualization](/images/guides/lmm/lmm_fig11_icc.png)

_Figure 11: Understanding the ICC. Top: Variance partitioning shown as pie chart and stacked bar. Bottom: What high vs. low ICC data look like—high ICC means people differ but each person is consistent; low ICC means people are similar but there's lots of occasion-specific noise._

The ICC helps you understand:

- **How clustered your data are**: High ICC means observations within a person are very similar
- **Whether random intercepts matter**: If ICC ≈ 0, there's little between-person variation to model
- **What proportion of variance is "stable"**: Higher ICC means more of the total variance reflects enduring individual differences

### Variance Explained (R²)

```r
r2(mod_rs)
```

- **Marginal R²**: Variance explained by time alone (fixed effect)
- **Conditional R²**: Variance explained by time + individual differences (fixed + random)

---

## Extract and Plot Random Effects

### Best Linear Unbiased Predictors (BLUPs)

*For conceptual background on BLUPs and shrinkage, see [Shrinkage and Partial Pooling](#shrinkage-and-partial-pooling).*

The random effects for each person:

```r
# Extract random effects
re <- ranef(mod_rs)$id
head(re)
```

```
   (Intercept)        time
1    11.234      0.512
2    -5.891     -0.234
3     2.156      0.089
...
```

These are person-specific **deviations** from the fixed effects.

### Individual Trajectories

```r
# Person-specific intercepts and slopes
person_effects <- data.frame(
  id = 1:n_persons,
  intercept = fixef(mod_rs)["(Intercept)"] + re$`(Intercept)`,
  slope = fixef(mod_rs)["time"] + re$time
)

head(person_effects)
```

### Plotting Individual Predictions

```r
# Get fitted values
data_long$fitted <- fitted(mod_rs)

# Plot a subset of individuals
sample_ids <- sample(unique(data_long$id), 30)

data_long %>%
  filter(id %in% sample_ids) %>%
  ggplot(aes(x = time, y = fitted, group = id)) +
  geom_line(alpha = 0.6, color = "steelblue") +
  geom_point(aes(y = y), alpha = 0.4, size = 1) +
  labs(x = "Time", y = "Score",
       title = "Model-Implied Trajectories for 30 Participants",
       subtitle = "Lines = fitted trajectories; Points = observed data") +
  theme_minimal()
```

### Shrinkage Visualization

> **Optional exploration**: The shrinkage visualization below is useful for building intuition about how LMM borrows strength across individuals, but it's not required for reporting standard results. Skip this section if you just need the basics.

Compare the shrunken (BLUP) estimates to OLS estimates for each person:

```r
# OLS estimates for each person (no pooling)
ols_estimates <- data_long %>%
  group_by(id) %>%
  summarise(
    ols_intercept = coef(lm(y ~ time))[1],
    ols_slope = coef(lm(y ~ time))[2],
    n_obs = n()
  )

# Combine with BLUPs
comparison <- person_effects %>%
  left_join(ols_estimates, by = "id")

# Plot shrinkage for intercepts
ggplot(comparison, aes(x = ols_intercept, y = intercept)) +
  geom_point(alpha = 0.5) +
  geom_abline(slope = 1, intercept = 0, linetype = "dashed", color = "red") +
  geom_vline(xintercept = fixef(mod_rs)["(Intercept)"], linetype = "dotted") +
  geom_hline(yintercept = fixef(mod_rs)["(Intercept)"], linetype = "dotted") +
  labs(x = "OLS Estimate (No Pooling)",
       y = "BLUP Estimate (Partial Pooling)",
       title = "Shrinkage of Individual Intercepts",
       subtitle = "Points below diagonal = shrinkage toward mean") +
  theme_minimal()
```

---

## Full Script

Here's everything in one self-contained block:

```r
# ==============================================================================
# LMM Complete Worked Example
# ==============================================================================

# Setup
library(tidyverse)
library(lme4)
library(lmerTest)
library(performance)
library(MASS)
set.seed(2024)

# Simulate data
n_persons <- 200
n_time <- 5
gamma_00 <- 50; gamma_10 <- 2
tau <- matrix(c(100, -2, -2, 1), nrow = 2)
sigma <- 5

random_effects <- mvrnorm(n_persons, c(0, 0), tau)
u0 <- random_effects[, 1]; u1 <- random_effects[, 2]

data_long <- expand_grid(id = 1:n_persons, time = 0:4) %>%
  mutate(
    y = gamma_00 + u0[id] + (gamma_10 + u1[id]) * time + rnorm(n(), 0, sigma)
  )

# Visualize
ggplot(data_long, aes(x = time, y = y, group = id)) +
  geom_line(alpha = 0.2) +
  theme_minimal() +
  labs(title = "Individual Trajectories")

# Fit models
mod_ri <- lmer(y ~ time + (1 | id), data = data_long)
mod_rs <- lmer(y ~ time + (1 + time | id), data = data_long)

# Compare models (use ML)
mod_ri_ml <- lmer(y ~ time + (1 | id), data = data_long, REML = FALSE)
mod_rs_ml <- lmer(y ~ time + (1 + time | id), data = data_long, REML = FALSE)
anova(mod_ri_ml, mod_rs_ml)

# Final results
summary(mod_rs)
VarCorr(mod_rs)
r2(mod_rs)

# Extract random effects
ranef(mod_rs)$id %>% head()
```

---

## Reference & Resources

This section contains lookup materials—consult as needed rather than reading straight through.

---

## Mathematical Notes

### The Two-Level Model (Formal Notation)

**Level 1** (within-person):

```
yᵢₜ = β₀ᵢ + β₁ᵢ(Timeₜ) + εᵢₜ
```

Where εᵢₜ ~ N(0, σ²)

**Level 2** (between-person):

```
β₀ᵢ = γ₀₀ + u₀ᵢ
β₁ᵢ = γ₁₀ + u₁ᵢ
```

Where:

```
[u₀ᵢ]     [0]   [τ₀₀  τ₀₁]
[u₁ᵢ] ~ N([0], [τ₁₀  τ₁₁])
```

### Combined Model

Substituting Level 2 into Level 1:

```
yᵢₜ = γ₀₀ + γ₁₀(Time) + u₀ᵢ + u₁ᵢ(Time) + εᵢₜ
```

### Matrix Form

For all observations from person i:

```
yᵢ = Xᵢγ + Zᵢuᵢ + εᵢ
```

Where:

- yᵢ = vector of outcomes for person i
- Xᵢ = design matrix for fixed effects
- γ = fixed effects vector
- Zᵢ = design matrix for random effects
- uᵢ = random effects vector for person i
- εᵢ = residual vector

### Variance of Observations

The marginal variance of yᵢₜ:

```
Var(yᵢₜ) = τ₀₀ + 2(Time)τ₀₁ + (Time)²τ₁₁ + σ²
```

The covariance between two observations from the same person:

```
Cov(yᵢₜ, yᵢₜ') = τ₀₀ + (Time + Time')τ₀₁ + (Time × Time')τ₁₁
```

---

## LMM vs. LGCM Comparison

Linear Mixed Models and Latent Growth Curve Models are mathematically equivalent for basic growth models. Here's how they relate:

### Parameter Correspondence

| Concept                    | LMM Notation | LGCM Notation                   |
| -------------------------- | ------------ | ------------------------------- |
| Average intercept          | γ₀₀          | μᵢ (intercept factor mean)      |
| Average slope              | γ₁₀          | μₛ (slope factor mean)          |
| Intercept variance         | τ₀₀          | ψᵢᵢ (intercept factor variance) |
| Slope variance             | τ₁₁          | ψₛₛ (slope factor variance)     |
| Intercept-slope covariance | τ₀₁          | ψᵢₛ (factor covariance)         |
| Residual variance          | σ²           | θ (residual variance)           |

### Key Differences

| Aspect                       | LMM                            | LGCM                        |
| ---------------------------- | ------------------------------ | --------------------------- |
| **Data format**              | Long (one row per observation) | Wide (one row per person)   |
| **Software**                 | lme4, nlme                     | lavaan, Mplus               |
| **Fit indices**              | No absolute fit indices        | CFI, RMSEA, SRMR            |
| **Degrees of freedom**       | Ambiguous for fixed effects    | Clear (model vs. saturated) |
| **Latent variables**         | Not directly modeled           | Natural extension           |
| **Time-varying covariates**  | Easy to add                    | More complex                |
| **Autocorrelated residuals** | Supported (nlme)               | Less common                 |

### When They Give Different Results

For the basic random intercept + slope model, LMM and LGCM produce identical estimates. Differences emerge with:

1. **Residual variance constraints**: LGCM can freely estimate residual variance at each wave; LMM typically constrains to equality
2. **Measurement models**: LGCM naturally incorporates latent variables with multiple indicators
3. **Missing data handling**: Both use ML/FIML, but implementations may differ slightly

### Choosing Between Them

| Choose LMM when...                      | Choose LGCM when...                |
| --------------------------------------- | ---------------------------------- |
| Time-varying covariates are central     | Latent variables are needed        |
| Unequal observation times per person    | Testing specific factor structures |
| You want individual predictions (BLUPs) | You want fit indices               |
| Familiar with lme4/mixed models         | Familiar with SEM/lavaan           |

For a brief non-technical summary of when to choose LMM vs. LGCM, see the [Note on LGCM](#when-lmm-is-appropriate) in the Overview section.

---

## FAQ & Common Pitfalls

**Pitfalls covered in this section**

- [Pitfall 1: Using REML for Fixed Effects Comparison](#pitfall-1-using-reml-for-fixed-effects-comparison)
- [Pitfall 2: Ignoring Singular Fit Warnings](#pitfall-2-ignoring-singular-fit-warnings)
- [Pitfall 3: Over-Specifying Random Effects](#pitfall-3-over-specifying-random-effects)
- [Pitfall 4: Treating BLUPs as Data](#pitfall-4-treating-blups-as-data)
- [Pitfall 5: Forgetting That Time Must Be Numeric](#pitfall-5-forgetting-that-time-must-be-numeric)
- [Pitfall 6: Misinterpreting ICC](#pitfall-6-misinterpreting-icc)
- [Pitfall 7: Ignoring Residual Assumptions](#pitfall-7-ignoring-residual-assumptions)
- [Pitfall 8: Comparing Marginal and Conditional R²](#pitfall-8-comparing-marginal-and-conditional-r)
- [Pitfall 9: Centering Confusion](#pitfall-9-centering-confusion)
- [Pitfall 10: Conflating Statistical and Practical Significance](#pitfall-10-conflating-statistical-and-practical-significance)

### Pitfall 1: Using REML for Fixed Effects Comparison

> [!pitfall]
> **Pitfall: comparing fixed-effects models with REML**
> REML likelihoods are not comparable across models with different fixed effects, so REML-based likelihood ratio tests for fixed-effects changes are invalid.

**The mistake**: Comparing models with different fixed effects using REML.

```r
# WRONG
mod1 <- lmer(y ~ time + (1 | id), data = data)  # REML default
mod2 <- lmer(y ~ time + treatment + (1 | id), data = data)
anova(mod1, mod2)  # Invalid comparison
```

**The problem**: REML likelihoods aren't comparable when fixed effects differ.

**The fix**: Use ML for comparison:

```r
mod1 <- lmer(y ~ time + (1 | id), data = data, REML = FALSE)
mod2 <- lmer(y ~ time + treatment + (1 | id), data = data, REML = FALSE)
anova(mod1, mod2)  # Valid
```

---

### Pitfall 2: Ignoring Singular Fit Warnings

> [!pitfall]
> **Pitfall: ignoring singular fit warnings**
> A singular fit means a variance component is estimated at or near zero. This often indicates an over-specified model or insufficient data, and results may be unreliable.

**The warning**: `boundary (singular) fit: see help('isSingular')`

**What it means**: A variance component is estimated at or near zero.

**Common causes**:

- Random effect variance is genuinely very small
- Model is over-specified for the data
- Too few observations per group

**What to do**:

- Check which variance is near zero: `VarCorr(model)`
- Consider simplifying the random effects structure
- Don't ignore it—results may be unreliable

---

### Pitfall 3: Over-Specifying Random Effects

> [!pitfall]
> **Pitfall: over-specifying random effects**
> Including complex random effects (e.g., correlated slopes with few time points) without sufficient data leads to convergence failures or singular fits.

**The mistake**: Including complex random effects without sufficient data.

```r
# Probably won't converge with 3 time points per person
mod <- lmer(y ~ time + I(time^2) + (1 + time + I(time^2) | id), data = data)
```

**The fix**: Start simple. Add complexity only if justified by theory and supported by data.

---

### Pitfall 4: Treating BLUPs as Data

> [!pitfall]
> **Pitfall: treating BLUPs as error-free data**
> BLUPs are shrunken estimates with uncertainty. Using them as observed data in secondary analyses (e.g., correlating BLUPs with outcomes) underestimates standard errors.

**The mistake**: Using extracted random effects as if they were observed data.

```r
# Problematic
re <- ranef(mod)$id
cor.test(re$`(Intercept)`, some_other_variable)  # SEs are wrong
```

**The problem**: BLUPs are estimates with uncertainty. Treating them as fixed values underestimates uncertainty and inflates Type I error.

**The fix**: Include predictors in the model, not in post-hoc analyses of BLUPs.

```r
# Correct approach: include the predictor in the model itself
mod <- lmer(y ~ time + some_other_variable + (1 + time | id), data = data)
```

This way, the model properly accounts for uncertainty in both the random effects and the predictor relationship.

---

### Pitfall 5: Forgetting That Time Must Be Numeric

> [!pitfall]
> **Pitfall: time coded as factor**
> If time is a factor, lmer() estimates dummy variables for each time point instead of a single slope—giving wave-specific means, not a growth trajectory.

**The mistake**: Treating time as a factor when you want a continuous slope.

```r
# This gives 4 dummy variables, not a slope
data$time <- factor(data$time)
mod <- lmer(y ~ time + (1 | id), data = data)
```

**The fix**: Keep time numeric for growth models:

```r
data$time <- as.numeric(data$time)
```

---

### Pitfall 6: Misinterpreting ICC

> [!pitfall]
> **Pitfall: misinterpreting ICC as variance explained**
> ICC is the proportion of variance that is *between-person*, not the proportion explained by the model. An ICC of 0.65 means 65% of variance is between people, not that 65% is explained.

**The mistake**: "ICC = 0.65 means the model explains 65% of variance."

**Reality**: ICC is the proportion of variance that is _between-person_ (not explained, just partitioned).

**Correct interpretation**: "65% of total variance is stable between-person differences; 35% is within-person variation."

---

### Pitfall 7: Ignoring Residual Assumptions

> [!pitfall]
> **Pitfall: ignoring residual diagnostics**
> LMM assumes residuals are approximately normal and homoscedastic. Severe violations can bias standard errors and p-values. Always plot residuals.

**The mistake**: Not checking whether residuals are approximately normal and homoscedastic.

**What to check**:

```r
plot(mod)          # Residuals vs. fitted
qqnorm(resid(mod)) # Normality of residuals
```

**What to do**: If assumptions are violated, consider transformations or robust methods.

---

### Pitfall 8: Comparing Marginal and Conditional R²

> [!pitfall]
> **Pitfall: confusing marginal and conditional R²**
> Marginal R² captures variance explained by fixed effects alone; conditional R² includes random effects. Report both, but don't compare them as if they measure the same thing.

**The mistake**: Reporting only marginal R² when random effects explain substantial variance.

**The difference**:

- Marginal R²: Fixed effects only
- Conditional R²: Fixed + random effects

**Best practice**: Report both, especially if they differ substantially.

---

### Pitfall 9: Centering Confusion

> [!pitfall]
> **Pitfall: unclear intercept interpretation due to centering**
> The intercept's meaning depends on how time is coded. Without explicit centering, the intercept may refer to a meaningless time point (e.g., time = 0 before the study began).

**The mistake**: Not being clear about what the intercept represents.

| Time coding     | Intercept means                        |
| --------------- | -------------------------------------- |
| 0, 1, 2, 3, 4   | Score at Time 0                        |
| -2, -1, 0, 1, 2 | Score at middle time point             |
| 1, 2, 3, 4, 5   | Score at Time 1 (if that's meaningful) |

**The fix**: Be explicit about centering and what the intercept represents.

---

### Pitfall 10: Conflating Statistical and Practical Significance

> [!pitfall]
> **Pitfall: equating statistical significance with practical importance**
> With large samples, even tiny variances are statistically significant. Always interpret effect sizes in the original scale and consider whether the magnitude matters substantively.

**The mistake**: "The random slope variance is significant, so individual differences in change are important."

**Reality**: With large samples, even tiny variances are statistically significant. A slope SD of 0.1 on a 100-point scale may not matter practically.

**Best practice**: Report effect sizes and interpret substantively.

---

### Pre-Flight Checklist

Before finalizing results:

- [ ] Is time coded as numeric?
- [ ] Did I use ML for model comparison and REML for final estimates?
- [ ] Did I check for singular fit warnings?
- [ ] Did I examine residual diagnostics?
- [ ] Am I clear about what the intercept represents?
- [ ] Did I report variance components, not just fixed effects?
- [ ] Did I report both marginal and conditional R²?
- [ ] Did I interpret random effects as variances (population-level), not as individual estimates?

---

## Cheat Sheet

### lme4 Formula Syntax

```r
# Random intercept only
lmer(y ~ time + (1 | id), data = data)

# Random intercept and slope (correlated)
lmer(y ~ time + (1 + time | id), data = data)

# Random intercept and slope (uncorrelated)
lmer(y ~ time + (1 | id) + (0 + time | id), data = data)

# With between-person predictor
lmer(y ~ time + treatment + (1 + time | id), data = data)

# With cross-level interaction
lmer(y ~ time * treatment + (1 + time | id), data = data)
```

---

### Key Functions

| Task                       | Code                       |
| -------------------------- | -------------------------- | ----------- |
| Fit model                  | `lmer(y ~ time + (1 + time | id), data)` |
| Summary                    | `summary(mod)`             |
| Fixed effects              | `fixef(mod)`               |
| Random effects (variances) | `VarCorr(mod)`             |
| Random effects (BLUPs)     | `ranef(mod)`               |
| Confidence intervals       | `confint(mod)`             |
| Model comparison           | `anova(mod1, mod2)`        |
| R²                         | `performance::r2(mod)`     |
| ICC                        | `performance::icc(mod)`    |
| Fitted values              | `fitted(mod)`              |
| Residuals                  | `resid(mod)`               |

---

### Parameter Quick Reference

| Parameter          | Symbol | lme4 Location                  |
| ------------------ | ------ | ------------------------------ |
| Average intercept  | γ₀₀    | Fixed effects: (Intercept)     |
| Average slope      | γ₁₀    | Fixed effects: time            |
| Intercept variance | τ₀₀    | Random effects: id (Intercept) |
| Slope variance     | τ₁₁    | Random effects: id time        |
| I-S correlation    | ρ      | Random effects: Corr           |
| Residual variance  | σ²     | Residual                       |

---

### ML vs. REML

| Use ML when...                                   | Use REML when...                     |
| ------------------------------------------------ | ------------------------------------ |
| Comparing models with different fixed effects    | Estimating final variance components |
| Calculating AIC/BIC for fixed effects comparison | Reporting variance estimates         |

```r
# ML
lmer(y ~ time + (1 | id), data = data, REML = FALSE)

# REML (default)
lmer(y ~ time + (1 | id), data = data, REML = TRUE)
```

---

### Checklist: Before Running

- [ ] Data in long format
- [ ] Time coded as numeric
- [ ] Grouping variable is a factor or character
- [ ] No missing values in predictors (or handle appropriately)

### Checklist: Before Reporting

- [ ] Model converged without warnings
- [ ] Residual diagnostics checked
- [ ] Fixed effects reported with SEs and CIs
- [ ] Variance components reported
- [ ] R² (marginal and conditional) reported
- [ ] Model comparison results included
- [ ] Time coding stated

---

## Advanced Extensions

> **Optional material**: This section covers extensions beyond basic growth modeling. Feel free to skip if you're just learning the fundamentals—you can always return later when your research requires these techniques.

### Time-Varying Covariates

Predictors that change across observations:

```r
# Effect of daily stress on daily mood
mod <- lmer(mood ~ time + stress + (1 + time | id), data = data)
```

**Interpretation**: The coefficient for `stress` represents the within-person effect—when a person's stress is higher than usual, how does their mood change?

**Centering matters**: Consider person-mean centering time-varying predictors to separate within- and between-person effects.

```r
data <- data %>%
  group_by(id) %>%
  mutate(
    stress_between = mean(stress),           # Person's average stress
    stress_within = stress - mean(stress)    # Deviation from person's average
  )

mod <- lmer(mood ~ time + stress_within + stress_between + (1 + time | id), data = data)
```

**Why center?** Without centering, the coefficient for a time-varying predictor confounds two questions: (1) Do people with higher average stress have different mood? (between-person effect) and (2) When a person's stress is above their own average, how does their mood change? (within-person effect). Person-mean centering separates these into distinct coefficients. The within-person effect (`stress_within`) answers the causal-ish question: "When this person's stress goes up, what happens to their mood?" The between-person effect (`stress_between`) describes stable individual differences.

---

### Cross-Level Interactions

Does a between-person variable moderate the slope?

```r
# Does treatment condition affect rate of change?
mod <- lmer(y ~ time * treatment + (1 + time | id), data = data)
```

**Interpretation**: The `time:treatment` interaction tells you whether the slope differs by treatment group.

---

### Non-Linear Growth

For quadratic growth:

```r
mod <- lmer(y ~ time + I(time^2) + (1 + time | id), data = data)
```

**Before specifying curvature**: Always plot observed trajectories first. A spaghetti plot will reveal whether curvature is pervasive or limited to a few individuals, and what functional form (quadratic, logarithmic, piecewise) might be appropriate. Don't add polynomial terms just because you can—let the data guide the functional form.

**Note**: Including a random quadratic term requires many observations per person and often causes convergence issues.

---

### Autocorrelated Residuals

If residuals are correlated across time (common in closely-spaced longitudinal data), use `nlme`:

```r
library(nlme)

mod <- lme(y ~ time,
           random = ~ 1 + time | id,
           correlation = corAR1(form = ~ time | id),
           data = data)
```

This models first-order autoregressive residuals.

---

### Generalized Linear Mixed Models (GLMM)

For non-normal outcomes:

```r
# Binary outcome
mod <- glmer(binary_y ~ time + (1 | id), data = data, family = binomial)

# Count outcome
mod <- glmer(count_y ~ time + (1 | id), data = data, family = poisson)
```

---

### Bayesian Mixed Models with brms

For fully Bayesian inference with flexible priors:

```r
library(brms)

mod <- brm(y ~ time + (1 + time | id),
           data = data,
           family = gaussian(),
           prior = c(prior(normal(0, 10), class = "b")),
           chains = 4, cores = 4)
```

**Advantages**:

- Full posterior distributions
- Handles complex models more gracefully
- Natural handling of uncertainty

---

## Recommended Resources

### Books

| Book                                                                                                     | Focus                               |
| -------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| Singer & Willett (2003). _Applied Longitudinal Data Analysis_. Oxford.                                   | Excellent pedagogy; MLM perspective |
| Snijders & Bosker (2012). _Multilevel Analysis_. Sage.                                                   | Comprehensive MLM treatment         |
| Raudenbush & Bryk (2002). _Hierarchical Linear Models_. Sage.                                            | Classic reference                   |
| Gelman & Hill (2007). _Data Analysis Using Regression and Multilevel/Hierarchical Models_. Cambridge.    | Practical, Bayesian-friendly        |
| West, Welch, & Galecki (2014). _Linear Mixed Models: A Practical Guide Using Statistical Software_. CRC. | Software-focused                    |

### Key Articles

| Article                                                                                                                               | Contribution                  |
| ------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------- |
| Bates, Mächler, Bolker, & Walker (2015). Fitting linear mixed-effects models using lme4. _Journal of Statistical Software_.           | Definitive lme4 reference     |
| Barr, Levy, Scheepers, & Tily (2013). Random effects structure for confirmatory hypothesis testing. _Journal of Memory and Language_. | "Keep it maximal" argument    |
| Matuschek et al. (2017). Balancing Type I error and power in linear mixed models. _Journal of Memory and Language_.                   | Counter to "maximal" approach |
| Luke (2017). Evaluating significance in linear mixed-effects models in R. _Behavior Research Methods_.                                | p-value methods comparison    |

### Online Tutorials

- **lme4 vignette**: `vignette("lmer", package = "lme4")`
- **UCLA IDRE**: https://stats.oarc.ucla.edu/r/seminars/
- **Bodo Winter's tutorials**: https://bodowinter.com/tutorials.html
- **Michael Clark's guide**: https://m-clark.github.io/mixed-models-with-R/

---

## Software Options

### R Packages

| Package         | Strengths                                      | Notes                    |
| --------------- | ---------------------------------------------- | ------------------------ |
| **lme4**        | Fast, reliable, standard                       | No p-values by default   |
| **lmerTest**    | Adds p-values to lme4                          | Use with lme4            |
| **nlme**        | Correlation structures, more flexible variance | Slower; different syntax |
| **brms**        | Bayesian, very flexible                        | Requires Stan            |
| **glmmTMB**     | Handles complex GLMMs, zero-inflation          | Alternative to lme4      |
| **performance** | Model diagnostics, R²                          | Companion package        |

### Other Software

| Software             | Strengths                               | Notes                            |
| -------------------- | --------------------------------------- | -------------------------------- |
| **Stata (mixed)**    | Integrated workflow, good documentation | Licensed                         |
| **SAS (PROC MIXED)** | Established, powerful                   | Licensed; steeper learning curve |
| **SPSS (MIXED)**     | GUI available                           | Less flexible                    |
| **HLM**              | Designed for multilevel models          | Separate software                |
| **MLwiN**            | Specialized for multilevel              | Can interface with R             |

### Choosing Software

For learning and most research purposes, **lme4 + lmerTest** is recommended:

- Free and open source
- Active development and community
- Extensive documentation and tutorials
- Handles most standard models efficiently

Use **nlme** when you need:

- Correlation structures (autoregressive residuals)
- Heterogeneous variances across groups

Use **brms** when you want:

- Bayesian inference with full posteriors
- Complex models that cause frequentist convergence issues
- Flexible priors

---

_End of Reference section._

---

_End of Document_
