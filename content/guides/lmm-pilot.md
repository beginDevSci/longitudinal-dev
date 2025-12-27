---
title: "Linear Mixed Models"
slug: "lmm-pilot"
description: "Model individual trajectories while accounting for the nested structure of longitudinal data."
category: "mixed-models"
tags: ["LMM", "guide", "lme4", "multilevel"]
guide_type: "hub"
---

## Why Mixed Models?

When you measure the same people repeatedly, your observations aren't independent. A person who scores high at Time 1 will likely score high at Time 2. Standard regression assumes each data point is independent—violating this assumption inflates your confidence and produces misleading p-values.

Consider a simple example: 200 participants measured at 5 time points each. You have 1,000 observations, but they're not 1,000 independent pieces of information. The 5 observations from Person 1 are more similar to each other than to observations from Person 47.

| Approach | What It Does | Problem |
|----------|--------------|---------|
| **Ignore nesting** | Treat all 1,000 observations as independent | SEs too small, p-values too optimistic |
| **Aggregate** | Compute person-means, analyze 200 means | Loses within-person information about change |
| **Repeated measures ANOVA** | Model time as fixed factor | Assumes compound symmetry; limited to balanced data |
| **Mixed models** | Model between- and within-person variation | Appropriate standard errors; individual trajectories |

**Linear Mixed Models (LMM)** solve this by explicitly modeling the dependency structure. The "mixed" refers to a mixture of:

- **Fixed effects**: Parameters that apply to everyone (average intercept, average slope)
- **Random effects**: Parameters that vary across individuals (person-specific deviations)

**Quick mental model**: Think of LMM as "fit one regression line per person, then summarize those lines." Each individual gets their own intercept and slope, but these estimates are informed by the whole sample—extreme values get pulled toward the group average, and people with sparse data borrow strength from others.

> [!info] **About This Guide**
>
> This page explains *what* LMM is and *why* it works—building conceptual understanding before implementation.
>
> - **Ready to see code?** → [Tutorial: Worked Example](/guides/lmm-pilot-tutorial)
> - **Need syntax or fit thresholds?** → [Quick Reference](/guides/lmm-pilot-reference)

---

## What LMM Provides

LMM offers several capabilities that make it well-suited for longitudinal analysis:

### Individual Trajectories Within a Unified Model

Each person gets their own intercept and slope, but these aren't estimated in isolation. The model "borrows strength" across individuals, leading to more stable estimates—especially for people with few observations or extreme values.

### Handles Unbalanced Data Gracefully

Real longitudinal data are messy: participants miss waves, measurement timing varies, some people drop out. LMM handles all of this naturally. Unlike repeated measures ANOVA, you don't need complete data from everyone.

### Flexible Time Structures

Time can be equally spaced (0, 1, 2, 3, 4), unequally spaced (0, 3, 6, 12, 24 months), or person-specific (actual dates of measurement). Just code time as a continuous variable with appropriate values.

### Separates Within- and Between-Person Variation

LMM explicitly partitions variance into:

- **Between-person**: How much do people differ from each other?
- **Within-person**: How much does each person vary around their own trajectory?

This distinction is fundamental to understanding longitudinal change.

### Natural Framework for Predictors

Adding predictors is straightforward:

- **Time-invariant predictors** (e.g., treatment group, gender): Predict individual differences in trajectories
- **Time-varying predictors** (e.g., daily stress, current medication): Predict occasion-specific deviations

---

## When LMM is Appropriate

LMM works well when you have:

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ observations per person (fewer limits what you can estimate) |
| **Continuous outcome** | Or ordinal with many categories; see GLMM for binary/count |
| **Nested structure** | Observations clearly belong to individuals |
| **Interest in change** | Not just "do groups differ?" but "how do individuals change?" |
| **Adequate sample** | 50+ individuals for simple models; 100+ recommended |

**Example scenarios where LMM shines:**

- **Therapy outcome study**: 150 clients complete depression inventories at 8 weekly sessions. Some miss sessions. You want to estimate average symptom reduction and identify who improves faster.
- **Cognitive aging study**: 300 older adults complete memory tests every 2 years over a decade. Timing varies slightly. You want to model decline and test whether education predicts slower decline.
- **Intensive longitudinal diary study**: 80 participants report daily mood for 30 days. You're interested in how daily stress relates to mood within persons, while capturing stable individual differences.

> [!note] **When to consider alternatives**
>
> LMM may not be the best choice when you have: only 2 time points (simpler methods like change scores may suffice), categorical outcomes (use GLMM), interest in latent variables (consider SEM approaches), complex dependency structures (time series models), or interest in discrete trajectory classes (growth mixture models).

---

## Key Components

### The Two-Level Structure

Longitudinal data have a natural hierarchy:

```
Level 2: Persons (i = 1, 2, ..., N)
    │
    └── Level 1: Observations within persons (t = 1, 2, ..., Tᵢ)
```

**Level 1** describes what happens *within* each person over time. This is where change lives.

**Level 2** describes how people *differ from each other*. This is where individual differences live.

LMM simultaneously answers two questions: *How do people change on average?* (fixed effects) and *How do individuals differ in that change?* (random effects).

### Fixed Effects

**What they are**: Population-average parameters—the intercept and slope that describe the "typical" trajectory.

**What they tell you**: "On average, people start at X and change by Y per unit time."

**Notation**: γ₀₀ (average intercept), γ₁₀ (average slope)

### Random Effects

**What they are**: Individual-specific deviations from the fixed effects. Not estimated directly as parameters, but characterized by their variance.

**What they tell you**: "People vary around the average intercept with SD = X. People vary around the average slope with SD = Y."

**Notation**: u₀ᵢ, u₁ᵢ (the deviations); τ₀₀, τ₁₁ (their variances)

**Key insight**: We don't estimate a separate intercept for each person as a "parameter." Instead, we estimate the *average* intercept (fixed) and the *variance* of intercepts across people (random). Person-specific estimates are derived quantities (BLUPs), not directly estimated coefficients.

### The Combined Model

The Level 1 and Level 2 equations combine into:

```
yᵢₜ = γ₀₀ + γ₁₀(Time) + u₀ᵢ + u₁ᵢ(Time) + εᵢₜ
      \_____________/   \_______________/   \__/
       Fixed effects    Random effects     Residual
```

This is the "mixed" model: fixed effects (γ) plus random effects (u) plus residual (ε).

### When to Treat an Effect as Random

| Treat as Fixed | Treat as Random |
|----------------|-----------------|
| Small number of categories | Many units sampled from larger population |
| Categories are the focus of interest | Units are incidental to research question |
| Want to compare specific categories | Want to generalize beyond sample |
| Example: Treatment vs. Control | Example: Individual participants |

In longitudinal analysis, **person** is almost always random (you want to generalize beyond your specific participants). **Time** is typically fixed when modeled as a continuous covariate.

---

## Building Intuition: From Simple to Complex

Let's build intuition by starting simple and adding complexity.

### Model 0: No Random Effects (What NOT to Do)

Ignore that observations are nested—one intercept, one slope for everyone:

```
yᵢₜ = β₀ + β₁(Time) + εᵢₜ
```

**What this assumes**: Observations are independent. Everyone follows the exact same trajectory. All variation is random noise.

**Problem**: Standard errors are wrong because we're treating correlated observations as independent.

> [!warning] **Model 0 is for illustration only**
>
> Do not use a no-random-effects model for longitudinal data. It ignores nesting and produces misleading standard errors and p-values.

### Model 1: Random Intercept Only

Allow intercepts to vary, but everyone shares the same slope:

```
yᵢₜ = [γ₀₀ + u₀ᵢ] + γ₁₀(Time) + εᵢₜ
```

Each person has their own starting level, but lines are **parallel**—everyone changes at the same rate.

**When this is enough**: If you have few time points, or if slope variance is genuinely small.

### Model 2: Random Intercept and Slope

Allow both to vary:

```
yᵢₜ = [γ₀₀ + u₀ᵢ] + [γ₁₀ + u₁ᵢ](Time) + εᵢₜ
```

Each person has their own intercept AND slope. Lines can have different starting points and different angles—they're **non-parallel** and can cross.

**The correlation**: Random intercepts and slopes can be correlated. Negative correlation means high starters tend to have flatter slopes (or steeper declines).

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

Suppose Person A has only 2 observations, both very high. If we estimate their trajectory independently (OLS), we'd get an extreme intercept based on just 2 points.

But we have information from 199 other people. Shouldn't that tell us something?

### What Shrinkage Does

Mixed models "shrink" extreme individual estimates toward the group mean. The amount of shrinkage depends on:

1. **How extreme the person's data are**: More extreme = more shrinkage
2. **How reliable the person's data are**: Fewer observations = more shrinkage
3. **How variable the population is**: Less population variability = more shrinkage

### Three Approaches to Individual Estimation

| Approach | What it does | Limitation |
|----------|--------------|------------|
| **Complete pooling** | Everyone gets the grand mean | Misses real heterogeneity |
| **No pooling (OLS)** | Separate regression per person | Extreme estimates with sparse data |
| **Partial pooling (LMM)** | Weighted average of individual data and group mean | Best of both worlds |

LMM's partial pooling gives more weight to an individual's data when they have many observations and the data are consistent, and more weight to the group mean when observations are sparse or variable.

<figure style="margin: 1.5rem 0;">
<img src="/images/guides/lmm/lmm_fig10_shrinkage.png" alt="Shrinkage Demonstration" style="border-radius: 8px; border: 1px solid rgba(255,255,255,0.1);" />
<figcaption style="font-style: italic; margin-top: 0.5rem; color: rgba(255,255,255,0.7);">Shrinkage of intercept and slope estimates. Points below the diagonal indicate BLUP estimates closer to the grand mean than OLS estimates. Arrows show direction of shrinkage.</figcaption>
</figure>

### Why This Matters

1. **Better individual predictions**: Shrunken estimates are typically more accurate than raw OLS estimates
2. **Handles sparse data**: People with few observations still get reasonable estimates
3. **Automatic regularization**: Protects against overfitting

---

<details>
<summary><strong>Mathematical Foundations</strong> (optional formal notation)</summary>

### The Two-Level Model

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

Where the random effects follow a bivariate normal distribution:

```
[u₀ᵢ]     [0]   [τ₀₀  τ₀₁]
[u₁ᵢ] ~ N([0], [τ₁₀  τ₁₁])
```

### Combined Model

Substituting Level 2 into Level 1:

```
yᵢₜ = γ₀₀ + γ₁₀(Time) + u₀ᵢ + u₁ᵢ(Time) + εᵢₜ
```

### Variance of Observations

The marginal variance of yᵢₜ:

```
Var(yᵢₜ) = τ₀₀ + 2(Time)τ₀₁ + (Time)²τ₁₁ + σ²
```

The covariance between two observations from the same person:

```
Cov(yᵢₜ, yᵢₜ') = τ₀₀ + (Time + Time')τ₀₁ + (Time × Time')τ₁₁
```

</details>

---

## Interactive Exploration

> [!tip] **Build intuition with the interactive tool**
>
> The visualization below lets you experiment with LMM parameters and see how they shape trajectories.

<iframe
  src="/images/guides/lmm/interactive/lmm_random_effects_explorer.html"
  width="100%"
  height="700"
  style="border: 1px solid rgba(6, 182, 212, 0.3); border-radius: 12px; margin: 1.5rem 0;"
  title="Interactive LMM Random Effects Explorer">
</iframe>

This tool lets you:

- Adjust fixed effects (average intercept and slope) and see the mean trajectory change
- Modify random effect variances and watch individual trajectories spread out or converge
- Change the intercept-slope correlation to see fan-in or fan-out patterns
- Observe how the ICC changes as you adjust between- and within-person variance

---

## Practical Considerations

### Data Format

LMM requires **long format**: one row per observation.

```
Wide format (won't work):        Long format (required):

id   y_t1  y_t2  y_t3             id   time   y
1    48    52    56               1    0      48
2    55    54    57               1    1      52
                                  1    2      56
                                  2    0      55
                                  2    1      54
                                  ...
```

### Time Coding

How you code time affects interpretation:

| Coding | Time values | Intercept means |
|--------|-------------|-----------------|
| Zero at start | 0, 1, 2, 3, 4 | Expected score at baseline |
| Zero at center | -2, -1, 0, 1, 2 | Expected score at middle wave |
| Actual time | 0, 6, 12, 18, 24 (months) | Expected score at month 0; slope = change per month |

**Recommendation**: Start with zero at baseline (0, 1, 2, ...). Adjust if your research question focuses on a different time point.

### Missing Data

LMM handles missing observations gracefully:

- Uses all available data (no listwise deletion)
- Assumes data are **Missing at Random (MAR)**: missingness can depend on observed variables but not on the missing values themselves
- No special syntax needed—just include all observations you have

**Caution**: If dropout is related to the outcome trajectory itself (MNAR), estimates may be biased. Consider sensitivity analyses.

### Intraclass Correlation (ICC)

The ICC tells you what proportion of total variance is between-person (stable individual differences) versus within-person (change + noise).

<figure style="margin: 1.5rem 0;">
<img src="/images/guides/lmm/lmm_fig11_icc.png" alt="ICC Visualization" style="border-radius: 8px; border: 1px solid rgba(255,255,255,0.1);" />
<figcaption style="font-style: italic; margin-top: 0.5rem; color: rgba(255,255,255,0.7);">Understanding the ICC. Top: Variance partitioning. Bottom: What high vs. low ICC data look like—high ICC means people differ but each person is consistent; low ICC means people are similar but there's lots of occasion-specific noise.</figcaption>
</figure>

An ICC of 0.65 means 65% of variance is between persons—observations within a person are more similar to each other than to other people's observations. This justifies using mixed models: there's meaningful clustering to account for.

---

## Common Pitfalls

Before applying LMM, be aware of these frequent mistakes.

**Using REML for fixed effects comparison** — Comparing models with different fixed effects using REML. *Reality:* REML likelihoods aren't comparable when fixed effects differ. Use ML (`REML = FALSE`) for model comparison.

**Ignoring singular fit warnings** — Dismissing "boundary (singular) fit" messages. *Reality:* A variance at zero often means the model is over-specified. Check `VarCorr()` and consider simplifying.

**Over-specifying random effects** — Including random quadratic with only 4 time points. *Reality:* You need enough within-person observations to estimate complex random effects. Start simple.

**Treating BLUPs as data** — Using extracted random effects in secondary analyses as if they were observed data. *Reality:* BLUPs are estimates with uncertainty. Include predictors in the model, not in post-hoc analyses of BLUPs.

**Time coded as factor** — Treating time as categorical when you want a continuous slope. *Reality:* Factor time gives dummy variables, not a growth trajectory. Keep time numeric for growth models.

**Misinterpreting ICC** — "ICC = 0.65 means the model explains 65%." *Reality:* ICC is variance *partitioning*, not variance *explained*. It tells you how much is between- vs. within-person.

**Ignoring residual assumptions** — Not checking whether residuals are approximately normal and homoscedastic. *Reality:* Severe violations can bias standard errors. Always plot residuals vs. fitted.

**Confusing marginal and conditional R²** — Reporting only marginal R² when random effects matter. *Reality:* Marginal = fixed effects only; Conditional = fixed + random. Report both.

**Centering confusion** — Not being clear about what the intercept represents. *Reality:* The intercept's meaning depends on where time = 0. Be explicit about your centering choice.

**Conflating statistical and practical significance** — "Slope variance is significant, so individual differences matter." *Reality:* With large samples, even tiny variances are significant. Interpret effect sizes substantively.

---

## Summary

You now have the conceptual foundation for understanding LMM:

- **Nested data require mixed models**—ignoring the dependency structure produces misleading standard errors
- **Fixed effects** capture population-average trajectories; **random effects** capture individual variation around those averages
- **Shrinkage** improves individual estimates by borrowing strength across people—especially valuable with sparse data
- **Time coding** determines what the intercept represents and must match your research question
- **ICC** tells you how much variance is between- vs. within-person, justifying the mixed model approach

This is enough to understand what LMM does and why. To actually *fit* one, continue to the worked example.

---

## Next Steps

<div style="display: flex; gap: 1rem; flex-wrap: wrap; margin-top: 1rem;">

**[Tutorial: Worked Example →](/guides/lmm-pilot-tutorial)**
Step-by-step R code to simulate data, fit models, and interpret results.

**[Quick Reference →](/guides/lmm-pilot-reference)**
Syntax patterns, fit evaluation, and troubleshooting.

</div>
