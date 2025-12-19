---
title: "Latent Growth Curve Models"
slug: "lgcm-pilot"
description: "Model individual trajectories over time using SEM-based growth curves."
category: "growth-models"
tags: ["LGCM", "SEM", "longitudinal", "growth-curves"]
r_packages: ["lavaan", "tidyverse", "MASS"]
---

## Why Study Growth?

Longitudinal data capture something powerful: **change**. When you measure the same people over time, you can ask questions that matter:

- How do symptoms evolve after treatment begins?
- Do children's reading skills develop at the same rate?
- Does cognitive decline accelerate with age?

These are questions about **trajectories**—not just whether groups differ at a single moment, but how individuals change over time.

Traditional approaches like repeated measures ANOVA focus on group means: "Did the average score increase?" But that question often misses what we actually care about.

Consider a therapy study where the average patient improves by 8 points over 6 months. Success? Look at the individual trajectories:

![Individual Growth Trajectories](/images/guides/lgcm/fig01_spaghetti_all.png)

*Each line is one person. Some improved dramatically. Some stayed flat. A few got worse. The average hides enormous individual variation.*

The average tells you "the group improved"—but the real questions are:

- How much do people *differ* in their starting points?
- How much do they *differ* in their rates of change?
- Are these two things related? (Do high starters change faster or slower?)

**Latent Growth Curve Models (LGCM)** answer exactly these questions. Every person gets their own trajectory—a starting point (intercept) and rate of change (slope)—and the model quantifies how much people vary in these trajectories.

---

> [!tip] **Before You Continue**
>
> Look at the spaghetti plot above. Before reading further, consider:
>
> 1. **Intercept variance**: How spread out are the starting points at Wave 1?
> 2. **Slope variance**: Do the lines fan out over time, or stay roughly parallel?
> 3. **Intercept-slope relationship**: Do high starters tend to change faster, slower, or neither?
>
> Hold these observations in mind—we'll return to them when discussing the model parameters.

---

## What LGCM Provides

LGCM offers four capabilities that simpler methods lack:

### Individual Trajectories

Each person has their own growth curve. The model estimates not just the average trajectory, but the **variance** around it—how much people differ in starting points and rates of change.

### Flexible Time Structures

Waves don't need to be equally spaced. Measure at baseline, 3 months, 6 months, and 2 years? No problem—just code time accordingly. The slope becomes "change per unit time" in whatever metric you choose.

### Missing Data Handling

Modern estimation (Full Information Maximum Likelihood) uses all available data. Participants who miss a wave still contribute information—no listwise deletion required.

### Testable Model Fit

Unlike approaches that simply estimate parameters, LGCM produces fit indices. You can ask: Does a linear trajectory actually describe these data? Is a more complex model (quadratic, piecewise) warranted?

---

## When LGCM is Appropriate

LGCM works well when you have:

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ waves (4+ preferred for testable fit) |
| **Continuous outcome** | Or ordinal with 5+ categories |
| **Interest in individual differences** | Not just "did the mean change?" |
| **Adequate sample** | 100+ for simple models; more for complex |

### When to Consider Alternatives

> [!note]
> LGCM may not be the best choice when you have:
>
> - **Categorical outcomes with few categories** — Consider growth models for categorical data
> - **Interest only in group means** — Repeated measures ANOVA may suffice
> - **Intensive longitudinal data (100+ waves)** — Consider time-series approaches

---

## Key Components of Linear LGCM

Regardless of your statistical background, these are the building blocks of every LGCM.

### The Two Latent Factors

Every person in your study has two numbers you can't directly observe:

| Factor | What It Captures | Defined By |
|--------|------------------|------------|
| **Intercept (η₁)** | Each person's level at Time = 0 | Where you place "0" in time coding |
| **Slope (η₂)** | Each person's rate of change per time unit | Constant in linear model |

These aren't measured directly—LGCM infers them from the pattern of observed scores across time.

### Factor Loadings: Encoding Time

In LGCM, loadings are **fixed**, not estimated:

| Factor | Loadings | Why |
|--------|----------|-----|
| Intercept | 1, 1, 1, 1, 1 | Contributes equally at every time point |
| Slope | 0, 1, 2, 3, 4 | Encodes time; contribution grows linearly |

The loadings 0, 1, 2, 3, 4 assume equally spaced waves with Time 0 at Wave 1. See [Time Coding](#time-coding) for centering options and unequal spacing.

### Factor Means: The Average Trajectory

| Parameter | Symbol | Interpretation |
|-----------|--------|----------------|
| Intercept mean | μ₁ | Average starting level across people |
| Slope mean | μ₂ | Average rate of change across people |

Together they define the average trajectory:

```
Average score at time t = μ₁ + μ₂ × t
```

*Example*: If μ₁ = 50 and μ₂ = 2, the average person starts at 50 and increases by 2 per wave.

### Factor Variances: Individual Differences

This is where LGCM shines—quantifying how much people differ.

| Parameter | Symbol | Interpretation |
|-----------|--------|----------------|
| Intercept variance | ψ₁₁ | How much people differ in starting levels |
| Slope variance | ψ₂₂ | How much people differ in change rates |

![Slope Variance Comparison](/images/guides/lgcm/slope_variance_comparison.svg)

*Left: Low slope variance—everyone changes at similar rates, so lines stay roughly parallel. Right: High slope variance—some people improve rapidly, others stay flat, some decline. The dark line represents the mean trajectory in both cases.*

**Interpreting scale**: Variances are in squared units. Take the square root for the standard deviation:

- If ψ₁₁ = 100 → SD = 10 → ~68% of people have intercepts within ±10 of the mean
- If ψ₂₂ = 1 → SD = 1 → ~68% of people have slopes within ±1 of the mean

![Distribution of Growth Parameters](/images/guides/lgcm/fig05_distributions.png)

*Histograms showing the distribution of individual intercepts (left) and slopes (right) across 400 participants.*

### Factor Covariance: The Intercept-Slope Relationship

One of LGCM's most informative parameters is the relationship between where people start and how fast they change:

| Sign | Interpretation |
|------|----------------|
| **Positive** | High starters grow faster (or decline slower). "Rich get richer." |
| **Negative** | High starters grow slower (or decline faster). Regression to mean. |
| **Zero** | Starting level doesn't predict change rate. |

![Intercept-Slope Relationship](/images/guides/lgcm/fig06_intercept_slope.png)

*Scatterplot of individual intercepts vs. slopes. The negative correlation (r ≈ -0.20) indicates that participants who started higher tended to grow slightly slower.*

These patterns aren't just statistical curiosities—they have substantive meaning:

- A **negative** intercept-slope correlation in a therapy study might mean patients with severe symptoms improve more
- A **positive** correlation in educational research might indicate a "Matthew effect" where initial advantages compound over time

Convert covariance to correlation for easier interpretation:

```
r = ψ₁₂ / √(ψ₁₁ × ψ₂₂)
```

A correlation of -0.20 means a modest negative relationship: those who start higher grow slightly slower.

### Residuals: What the Trajectory Doesn't Explain

```
Observed = Predicted + Residual
yᵢₜ = (η₁ᵢ + η₂ᵢ × t) + εᵢₜ
```

**Residual variance (θ)** captures the noise around individual trajectories—measurement error, occasion-specific factors, or model misspecification.

**Key assumptions**:
- Residuals uncorrelated across time (can be relaxed in advanced models)
- Can be equal or freely estimated across waves

---

## The Path Diagram

The path diagram provides a visual grammar for LGCM:

![LGCM Path Diagram](/images/guides/lgcm/lgcm_path_diagram.svg)

*The intercept factor has all loadings fixed to 1. The slope factor has loadings encoding time (0, 1, 2, 3, 4). The curved arrow represents the covariance between factors. Residuals (ε) capture occasion-specific variation.*

<details>
<summary>Text description (for accessibility)</summary>

```
              ┌─────────────┐          ┌─────────────┐
              │  Intercept  │          │    Slope    │
              │    (η₁)     │◄────────►│    (η₂)     │
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

Intercept loadings (top): all 1s
Slope loadings (bottom): 0, 1, 2, 3, 4
Double-headed arrow: factor covariance
Residual variances omitted for clarity
```

</details>

**Key insight**: Unlike exploratory factor analysis, LGCM *fixes* the factor loadings. You don't estimate them—you set them based on your time coding. This constraint is what makes the intercept and slope *interpretable* as starting level and rate of change.

---

## Visualizing Individual Trajectories

The path diagram shows the model structure; spaghetti plots show what it describes:

![Variability in Trajectories](/images/guides/lgcm/fig04_highlighted.png)

*Extreme cases highlighted to illustrate different combinations of intercept and slope: high starters with fast growth (blue), high starters with slow growth (purple), low starters with fast growth (orange), low starters with slow growth (red).*

This figure illustrates the four "corners" of the intercept-slope distribution:

| Quadrant | Intercept | Slope | Pattern |
|----------|-----------|-------|---------|
| Blue | High | High | Started high, grew fast |
| Purple | High | Low | Started high, grew slowly |
| Orange | Low | High | Started low, grew fast |
| Red | Low | Low | Started low, grew slowly |

The intercept-slope covariance determines how concentrated trajectories are in certain quadrants. A strong negative covariance would mean most people fall in the purple (high-low) and orange (low-high) quadrants.

---

## Interactive Exploration

To build deeper intuition for how LGCM parameters affect trajectories, use the interactive explorer below. Adjust the sliders to see how changing the intercept mean, slope variance, and other parameters affects the spaghetti plot in real-time.

<iframe
  src="/images/guides/lgcm/interactive/trajectory_explorer.html"
  width="100%"
  height="700"
  style="border: 1px solid rgba(6, 182, 212, 0.3); border-radius: 12px; margin: 1.5rem 0;"
  title="Interactive LGCM Trajectory Explorer">
</iframe>

**Experiments to try:**

1. **Set Slope SD = 0**: Watch all lines become parallel. No slope variance = everyone changes at exactly the same rate.

2. **Set Slope SD = 2**: Watch the "fan pattern" emerge. High slope variance = some people change rapidly, others slowly.

3. **Set Intercept-Slope Correlation = -0.8**: Notice how high starters now tend to have flatter slopes. This is regression to the mean.

4. **Set Intercept-Slope Correlation = +0.8**: Now high starters grow even faster. This is the "rich get richer" pattern.

5. **Increase Residual SD**: Watch individual trajectories become noisier while the underlying pattern remains.

---

## Data Requirements

Before fitting an LGCM, verify that your data meet these requirements.

### Minimum Time Points

| Time Points | What You Can Estimate |
|-------------|----------------------|
| 2 | LGCM offers little advantage; use change scores, ANCOVA, or regression instead |
| 3 | Linear growth with minimal testable fit (df = 1 with free residuals) |
| 4 | Linear growth with good testable fit (df = 5) |
| 4 | Quadratic growth (just-identified) |
| 5+ | Linear or quadratic with robust testable fit |

With only two time points, you can estimate mean change and variance of change, but a latent growth curve model does not offer meaningful advantages over simpler approaches.

**Recommendation**: Aim for at least 4 time points. With 3 time points you have only 1 degree of freedom for testing fit (assuming free residual variances), which provides limited power to detect model misspecification.

### Missing Data

Real longitudinal data almost always have missing observations. LGCM handles this well—but only under certain conditions.

**Full Information Maximum Likelihood (FIML)**: Modern SEM software uses FIML by default. It uses all available data from each participant without imputation or deletion.

**The MAR Assumption**: FIML assumes data are Missing At Random—missingness can depend on observed variables but not on the missing values themselves.

- *MAR example*: Participants with lower baseline scores drop out more often. (Baseline was observed.)
- *MNAR example*: Participants drop out because their current unobserved score is extreme. (Estimates may be biased.)

**Problematic patterns**:
- Sporadic missingness (random waves missing): Generally fine
- Monotone dropout: Fine under MAR, though you're extrapolating
- Systematic dropout related to trajectory: Potentially problematic

**Auxiliary variables**: Including variables that predict missingness (but aren't part of your main model) can make MAR more plausible and improve estimation. For example, if participants with lower education drop out more, including education as an auxiliary variable helps FIML "borrow" information appropriately.

### Distributional Considerations

Maximum likelihood estimation assumes multivariate normality. Violations can affect standard errors (underestimated) and chi-square statistics (inflated).

**What to check**: Skewness, kurtosis, outliers, floor/ceiling effects at each wave.

**Guidelines**:
- Skewness: |skew| < 2 is generally acceptable
- Kurtosis: |kurt| < 7 is generally acceptable
- For violations, use robust standard errors (MLR estimator)

**What to do**:
- Mild violations: Use robust standard errors
- Severe violations: Consider transformations or robust estimation
- Categorical outcomes: Use appropriate estimators (WLSMV)

### Pre-Flight Checklist

- [ ] 3+ time points (4+ preferred)
- [ ] Missingness examined and plausibly MAR
- [ ] Distributions checked
- [ ] Time structure known (spacing between waves)

---

## Time Coding

Slope factor loadings define how time enters the model. This determines:

- What the intercept represents
- How to interpret the slope
- Whether unequal spacing is handled correctly

### Standard Coding (0, 1, 2, 3, 4)

| Wave | Loading | Meaning |
|------|---------|----------|
| 1 | 0 | Intercept = expected score here |
| 2 | 1 | 1 unit of time elapsed |
| 3 | 2 | 2 units elapsed |
| 4 | 3 | 3 units elapsed |
| 5 | 4 | 4 units elapsed |

With this coding:
- **Intercept** = expected score at Time 0 (Wave 1)
- **Slope** = expected change per 1-unit increase in time

### Centering at Different Time Points

Shift the zero point to change what the intercept represents:

| Centering | Loadings | Intercept Meaning |
|-----------|----------|-------------------|
| Wave 1 (standard) | 0, 1, 2, 3, 4 | Expected score at Wave 1 |
| Midpoint (Wave 3) | -2, -1, 0, 1, 2 | Expected score at Wave 3 |
| Final (Wave 5) | -4, -3, -2, -1, 0 | Expected score at Wave 5 |

![Time Coding and Centering](/images/guides/lgcm/time_coding_centering.svg)

*How time coding determines the intercept's location. All three panels show the same trajectory, but the intercept (marked "I") refers to different time points depending on where you place the zero in your slope loadings. The slope (rate of change) remains identical across all three.*

<details>
<summary>Text description (if image doesn't render)</summary>

Three panels showing the same linear trajectory with different intercept locations:
- **Left (Wave 1 centering)**: Loadings 0,1,2,3,4 — intercept "I" marked at Wave 1
- **Center (Midpoint centering)**: Loadings -2,-1,0,1,2 — intercept "I" marked at Wave 3
- **Right (Final wave centering)**: Loadings -4,-3,-2,-1,0 — intercept "I" marked at Wave 5

The slope (line angle) is identical in all three; only the intercept reference point changes.

</details>

> [!note] **The slope never changes**
>
> Recentering moves *where* the intercept is measured but doesn't change the rate of change. The slope means the same thing in all versions—only the intercept's reference point shifts.

**When does centering matter?** When you add predictors. If you ask "Does baseline depression predict growth?", the intercept-predictor relationship depends on where you defined the intercept.

### Non-Equidistant Time Points

If waves aren't equally spaced, use actual time values:

| Assessment | Time | Loading |
|------------|------|---------|
| Baseline | Month 0 | 0 |
| Follow-up 1 | Month 3 | 3 |
| Follow-up 2 | Month 6 | 6 |
| Follow-up 3 | Month 12 | 12 |
| Follow-up 4 | Month 24 | 24 |

Now slope = change per **month**. Rescale for interpretability if needed (e.g., divide by 12 for slope = change per year).

---

## Model Specification

### Residual Variances: Equal vs. Free

- **Free (default)**: Each time point has its own residual variance. More flexible, uses more parameters.
- **Equal (constrained)**: All time points share the same residual variance. More parsimonious.

**Recommendation**: Start with free residuals, then test whether equality constraint worsens fit.

### Parameter Count

For a linear LGCM with 5 waves:

| Parameter | Free Residuals | Equal Residuals |
|-----------|---------------|------------------|
| Intercept mean | 1 | 1 |
| Slope mean | 1 | 1 |
| Intercept variance | 1 | 1 |
| Slope variance | 1 | 1 |
| I-S covariance | 1 | 1 |
| Residual variances | 5 | 1 |
| **Total** | **10** | **6** |

The data provide 5 means + 15 unique covariances = 20 pieces of information.

| Model | Parameters | df |
|-------|------------|-----|
| Free residuals | 10 | 10 |
| Equal residuals | 6 | 14 |

Positive degrees of freedom means the model is testable.

---

## Estimation & Fit Indices

### Maximum Likelihood Estimation

ML estimation:
- Uses observed means and covariances
- Assumes multivariate normality
- Handles missing data via FIML
- Produces standard errors for inference

If concerned about non-normality, use robust estimation (MLR) which provides robust standard errors and scaled test statistics (Satorra-Bentler correction).

### Fit Indices

#### Chi-Square (χ²)

Tests whether model-implied covariances match observed covariances.

- Non-significant p (> .05): Model fits adequately
- Significant p (< .05): Model doesn't fit perfectly

**Problem**: Large samples make trivial misfits significant. Don't rely on χ² alone.

#### RMSEA (Root Mean Square Error of Approximation)

Estimates population misfit, adjusted for parsimony.

| Value | Interpretation |
|-------|----------------|
| < 0.05 | Close fit |
| 0.05–0.08 | Reasonable fit |
| > 0.10 | Poor fit |

RMSEA rewards parsimony and provides a 90% confidence interval.

#### CFI (Comparative Fit Index)

Compares your model to a null model (all variables uncorrelated).

| Value | Interpretation |
|-------|----------------|
| > 0.95 | Good fit |
| 0.90–0.95 | Acceptable |
| < 0.90 | Poor fit |

#### SRMR (Standardized Root Mean Square Residual)

Average discrepancy between observed and implied correlations.

| Value | Interpretation |
|-------|----------------|
| < 0.08 | Good fit |
| > 0.10 | Poor fit |

### Interpreting Multiple Indices

No single index is definitive. Look for convergence:

- All indices suggest good fit → Confident
- Indices disagree → Investigate further
- Most indices suggest poor fit → Revise model

**Reasonable targets** (not hard cutoffs): CFI ≥ 0.95, RMSEA ≤ 0.06, SRMR ≤ 0.08

---

## Model Comparison

### Why Compare Models?

1. **Test hypotheses**: Is there growth? Do people differ?
2. **Select best model**: Balance fit and parsimony

### Nested vs. Non-Nested Models

**Nested**: One model is a constrained version of another.
- Intercept-only nested within linear growth
- Equal residuals nested within free residuals

**Non-nested**: Neither is a special case of the other.

### Likelihood Ratio Test (Nested Models)

If Model A is nested within Model B:

```
Δχ² = χ²(constrained) - χ²(unconstrained)
Δdf = df(constrained) - df(unconstrained)
```

Significant Δχ²: The constraints worsen fit; reject the simpler model.

### Information Criteria (Any Models)

**AIC**: -2×LL + 2×(parameters). Lower = better. Moderate parsimony penalty.

**BIC**: -2×LL + log(N)×(parameters). Lower = better. Stronger parsimony penalty.

If AIC and BIC agree, you're confident. If they disagree, acknowledge ambiguity.

### Common Comparisons

#### Test 1: Is There Growth?

The most fundamental comparison: does adding a slope factor significantly improve fit?

**[Open Interactive Model Comparison](/images/guides/lgcm/interactive/model_comparison.html)** *(opens in browser)*

<details>
<summary>Embedded interactive version</summary>

<iframe
  src="/images/guides/lgcm/interactive/model_comparison.html"
  width="100%"
  height="680"
  style="border: 1px solid rgba(6, 182, 212, 0.3); border-radius: 12px;"
  title="Interactive Model Comparison">
</iframe>

</details>

*Click the buttons to animate between an intercept-only model (flat trajectories) and a linear growth model (sloped trajectories). Notice how adding the slope factor allows the model to capture systematic change.*

Compare an intercept-only model to a linear growth model. Significant Δχ² means there's systematic change.

#### Test 2: Do People Differ in Growth?

Compare a model with fixed slope (variance = 0) to a model with random slope (variance estimated). Significant Δχ² means individuals differ in change rates.

**Note**: Testing variance = 0 is a boundary test. The p-value is conservative (true p ≈ half the reported value).

#### Test 3: Are Residual Variances Equal?

Compare free residual variances to constrained (equal) residual variances. Non-significant Δχ² means equal residuals are justified—use the simpler model.

### Decision Framework

1. Fit intercept-only model
2. Fit linear growth model
3. Compare (LRT): Is growth significant?
4. Test equal residual variances
5. Select final model based on LRT and AIC/BIC

### Reporting Example

> We compared an intercept-only model to a linear growth model. Linear growth fit significantly better, Δχ²(3) = 226.10, p < .001. Constraining residual variances to equality did not significantly worsen fit, Δχ²(4) = 5.23, p = .26. The final model includes linear growth with equal residual variances (CFI = 0.996, RMSEA = 0.025, SRMR = 0.030).

| Model | χ² | df | CFI | RMSEA | AIC | BIC |
|-------|-----|-----|------|-------|-----|-----|
| Intercept only | 238.45 | 13 | 0.82 | 0.21 | 10456 | 10484 |
| Linear growth | 12.35 | 10 | 0.998 | 0.024 | 10234 | 10274 |
| Linear (equal resid) | 17.58 | 14 | 0.996 | 0.025 | 10228 | 10260 |
