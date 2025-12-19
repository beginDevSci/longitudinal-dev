---
title: "Latent Growth Curve Models: A Distill-Style Guide"
slug: "lgcm-newtest-v1"
description: "An interactive exploration of how to model individual change trajectories using LGCM—designed for understanding, not just reference."
category: "growth-models"
tags: ["LGCM", "SEM", "longitudinal", "lavaan", "growth-curves", "interactive"]
r_packages: ["lavaan", "tidyverse", "MASS"]
---

## The Mystery of Individual Change

---

Here's a puzzle that haunts longitudinal researchers:

**You run a therapy study. The average patient improves by 8 points over 6 months. Success, right?**

But look closer at the individual trajectories:

![Individual Growth Trajectories](/images/guides/lgcm/fig01_spaghetti_all.png)

*Each line is one person. Some improved dramatically. Some stayed flat. A few got worse.*

The average hides enormous individual variation. Traditional methods tell you "the group improved"—but that's not what you actually want to know.

**The real questions are:**
- How much do people *differ* in their starting points?
- How much do they *differ* in their rates of change?
- Are these two things related? (Do high starters change faster or slower?)

This is what Latent Growth Curve Models reveal.

---

### Before We Continue: Make a Prediction

> [!tip] **Self-Check: What Do You Expect?**
>
> Look at the spaghetti plot above. Before reading further, answer these questions:
>
> 1. **Intercept variance**: How spread out are the starting points at Wave 1? (A lot / A little)
> 2. **Slope variance**: Do the lines fan out over time, or stay parallel? (Fan out / Parallel)
> 3. **Intercept-slope correlation**: Do high starters tend to change faster, slower, or neither?
>
> Write down your guesses. We'll check them against the model estimates later.

This kind of prediction—forming hypotheses before seeing results—dramatically improves learning. You'll remember the answer better when it either confirms or surprises you.

---

## The Core Insight: Two Hidden Variables

Every person in your study has two numbers you can't directly observe:

| Hidden Variable | What It Represents |
|-----------------|-------------------|
| **Intercept (η₁)** | Where they started |
| **Slope (η₂)** | How fast they changed |

The magic of LGCM is that it *infers* these hidden variables from the pattern of observed scores across time.

---

### Try It Yourself: Explore the Parameters

Adjust the sliders below to see how different population parameters produce different trajectory patterns:

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

3. **Set Intercept-Slope Correlation = -0.8**: Notice how high starters now tend to have flatter slopes. This is "regression to the mean."

4. **Set Intercept-Slope Correlation = +0.8**: Now high starters grow even faster. This is the "rich get richer" pattern.

> [!note] **Why This Matters**
>
> These patterns aren't just statistical curiosities—they have substantive meaning. A negative intercept-slope correlation in a therapy study might mean patients with severe symptoms improve more. A positive correlation might indicate a "Matthew effect" where advantages compound.

---

## Two Roads to the Same Model

Here's something that confuses many researchers: LGCM can be described in two completely different ways that are mathematically identical.

### Which Framing Clicks for You?

<table>
<tr>
<th width="50%">🔷 SEM Framing: "Factor Analysis of Time"</th>
<th width="50%">🔶 Multilevel Framing: "Regressions Within People"</th>
</tr>
<tr>
<td>

**Intuition**: Your repeated measures are like questionnaire items—they're *indicators* of underlying latent factors.

The intercept and slope are *latent variables* that explain the correlations among your time points.

**Think of it as**: The same way extraversion explains correlations among personality items, the growth factors explain correlations among waves.

</td>
<td>

**Intuition**: Each person gets their own regression line: `Score = Intercept + Slope × Time`.

Level 1 models within-person change; Level 2 models the distribution of intercepts and slopes across people.

**Think of it as**: Fit 400 separate regressions, then summarize the distribution of those 400 intercepts and 400 slopes.

</td>
</tr>
<tr>
<td>

**The equation**:

```
y₁ = 1×η₁ + 0×η₂ + ε₁
y₂ = 1×η₁ + 1×η₂ + ε₂
y₃ = 1×η₁ + 2×η₂ + ε₃
y₄ = 1×η₁ + 3×η₂ + ε₄
y₅ = 1×η₁ + 4×η₂ + ε₅
```

The numbers (1,1,1,1,1) and (0,1,2,3,4) are *fixed factor loadings* encoding time.

</td>
<td>

**The equation**:

```
Level 1: yᵢₜ = β₀ᵢ + β₁ᵢ×Time + εᵢₜ

Level 2: β₀ᵢ = γ₀₀ + u₀ᵢ
         β₁ᵢ = γ₁₀ + u₁ᵢ
```

Person *i*'s intercept and slope are draws from a population distribution with means γ₀₀, γ₁₀ and variances Var(u₀), Var(u₁).

</td>
</tr>
</table>

**The punchline**: These produce *identical* estimates. Choose based on your background:

- Comfortable with CFA/SEM? → SEM framing
- Comfortable with HLM/mixed models? → Multilevel framing
- New to both? → Multilevel is often more intuitive

---

### The Path Diagram: A Visual Grammar

![LGCM Path Diagram](/images/guides/lgcm/lgcm_path_diagram.svg)

*The intercept factor (η₁) has loadings fixed to 1 at every wave—it contributes equally throughout. The slope factor (η₂) has loadings 0, 1, 2, 3, 4—its contribution grows with time.*

> [!important] **The Key Constraint**
>
> Unlike exploratory factor analysis, LGCM *fixes* the factor loadings. You don't estimate them—you set them based on your time coding. This constraint is what makes the intercept and slope *interpretable* as starting level and rate of change.

---

## Checking Your Predictions

Remember your predictions from earlier? Let's see what the model actually estimates from our simulated data (N=400, 5 waves):

```r
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit <- growth(model, data = data_wide)
summary(fit)
```

### The Reveal: Growth Factor Estimates

| Parameter | Estimate | SE | What It Means |
|-----------|----------|-----|---------------|
| **Intercept Mean** | 50.2 | 0.51 | Average starting score ≈ 50 |
| **Slope Mean** | 1.98 | 0.05 | Average increase ≈ 2 points per wave |
| **Intercept Variance** | 99.8 | 7.3 | SD ≈ 10 → People start roughly ±10 points from mean |
| **Slope Variance** | 0.97 | 0.09 | SD ≈ 1 → People change roughly ±1 point/wave from mean |
| **I-S Covariance** | -1.89 | 0.61 | r ≈ -0.19 → High starters grow slightly slower |

**How did your predictions compare?**

1. ✓ **Intercept variance is substantial** (SD = 10 on a ~50-point scale = meaningful spread)
2. ✓ **Lines fan out** (slope SD = 1 means some people grow at +3/wave, others at +1/wave)
3. ✓ **Slight negative correlation** (high starters grow a bit slower—regression to mean)

---

## The Critical Decision: How to Code Time

The numbers you put as slope loadings determine *everything* about interpretation:

### Experiment: Same Data, Different Stories

<table>
<tr>
<th width="33%">Time at Wave 1</th>
<th width="33%">Time at Midpoint</th>
<th width="33%">Time at End</th>
</tr>
<tr>
<td>

**Loadings**: 0, 1, 2, 3, 4

**Intercept** = expected score at **Wave 1**

*"Where do people start?"*

</td>
<td>

**Loadings**: -2, -1, 0, 1, 2

**Intercept** = expected score at **Wave 3**

*"Where are people at the midpoint?"*

</td>
<td>

**Loadings**: -4, -3, -2, -1, 0

**Intercept** = expected score at **Wave 5**

*"Where do people end up?"*

</td>
</tr>
</table>

> [!warning] **The slope never changes**
>
> Recentering moves *where* the intercept is measured but doesn't change the rate of change. The slope means the same thing in all three versions—only the intercept's reference point shifts.

**When does centering matter?** When you add predictors! If you ask "Does baseline depression predict growth?", the intercept-predictor relationship depends on where you defined the intercept.

---

## When Things Go Wrong

### The Negative Variance Problem

You run your model and lavaan reports: `slope variance = -2.4`

Variances can't be negative. What happened?

**Diagnosis**: This "Heywood case" usually means:
- True slope variance is near zero (everyone changes at nearly the same rate)
- Sample is too small for the model complexity
- Model is misspecified (maybe growth isn't linear)

**Solutions**:

```r
# Option 1: Constrain to zero (if theoretically justified)
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
  slope ~~ 0*slope  # Fix variance to zero
'

# Option 2: Constrain to small positive value
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
  slope ~~ 0.001*slope  # Near-zero but positive
'
```

> [!pitfall] **Don't ignore negative variances**
>
> A negative variance estimate is your data screaming that something is wrong. Investigate—don't just report the p-value and move on.

---

## From Model to Meaning: What to Report

### A Template for Your Methods Section

> We modeled change in [outcome] across [N time points] using a latent growth curve model in lavaan (Rosseel, 2012). Time was coded as [0, 1, 2...] such that the intercept represents [interpretation] and the slope represents [interpretation per time unit]. We used maximum likelihood estimation with robust standard errors (MLR) to account for [non-normality/missing data]. Model fit was evaluated using CFI, RMSEA, and SRMR.

### A Template for Your Results Section

> The linear growth model fit the data well (χ² = [X], df = [X], p = [X]; CFI = [X]; RMSEA = [X], 90% CI [X, X]; SRMR = [X]). On average, participants [increased/decreased] by [slope mean] points per [time unit] (SE = [X], p < [X]). There was significant variability in both starting levels (intercept variance = [X], p < [X]) and rates of change (slope variance = [X], p < [X]). The correlation between intercept and slope was [r] (p = [X]), indicating that [interpretation].

---

## Quick Reference: The Essential lavaan Syntax

```r
# Basic linear growth model
model <- '
  # Intercept: loadings all = 1
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5

  # Slope: loadings = time
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

# Fit the model
fit <- growth(model, data = data_wide)

# View results
summary(fit, fit.measures = TRUE, standardized = TRUE)

# Extract specific fit measures
fitmeasures(fit, c("cfi", "rmsea", "srmr"))

# Compare models
anova(fit_simple, fit_complex)
```

---

## Summary: The LGCM Mental Model

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│   Observed Data           Hidden Structure                  │
│   ─────────────           ────────────────                  │
│                                                             │
│   y₁  y₂  y₃  y₄  y₅  →   Intercept (η₁): Where you start  │
│   ↓   ↓   ↓   ↓   ↓                                         │
│   Scores over time        Slope (η₂): How fast you change   │
│                                                             │
│   ─────────────────────────────────────────────────────     │
│                                                             │
│   Questions LGCM Answers:                                   │
│                                                             │
│   • What's the average trajectory?  → Factor means          │
│   • How much do people vary?        → Factor variances      │
│   • Are start and change related?   → Factor covariance     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Going Deeper

This guide covered **linear growth with continuous outcomes**. LGCM can be extended to:

- **Quadratic/nonlinear growth**: Add a quadratic factor with loadings 0, 1, 4, 9, 16
- **Piecewise growth**: Different slopes for different phases
- **Predictors of growth**: Why do some people change more than others?
- **Parallel processes**: Two outcomes growing together
- **Latent class growth**: Finding subgroups with different trajectory shapes

**Comparison with alternatives**: See the [Linear Mixed Models Guide](/guides/lmm) for when multilevel modeling might be preferred (many time points, time-varying predictors, complex nesting).

---

## References & Resources

**Essential reading**:
- Bollen, K. A., & Curran, P. J. (2006). *Latent Curve Models: A Structural Equation Perspective*. Wiley.
- Preacher, K. J., et al. (2008). Latent growth curve modeling. SAGE.

**Software documentation**:
- [lavaan tutorial: Growth models](https://lavaan.ugent.be/tutorial/growth.html)
- [lavaan reference manual](https://cran.r-project.org/web/packages/lavaan/lavaan.pdf)

**When to use LGCM vs. alternatives**:
- Duncan, T. E., et al. (2006). *An Introduction to Latent Variable Growth Curve Modeling*. 2nd ed.

---

<details>
<summary><strong>Appendix: Simulating LGCM Data for Practice</strong></summary>

Use this code to generate data with known parameters—helpful for understanding what the model recovers.

```r
library(MASS)
library(tidyverse)

# Set parameters you want to recover
n <- 400
int_mean <- 50      # Average starting point
slp_mean <- 2       # Average growth per wave
int_var <- 100      # Variance in starting points (SD = 10)
slp_var <- 1        # Variance in slopes (SD = 1)
is_cov <- -2        # Intercept-slope covariance (negative = high starters grow slower)
res_var <- 25       # Residual variance (SD = 5)

# Generate latent factors
psi <- matrix(c(int_var, is_cov,
                is_cov, slp_var), nrow = 2)

set.seed(2024)
factors <- mvrnorm(n, mu = c(int_mean, slp_mean), Sigma = psi)

# Generate observed data
data_wide <- tibble(id = 1:n) %>%
  mutate(
    eta_int = factors[, 1],
    eta_slp = factors[, 2],
    y1 = eta_int + eta_slp * 0 + rnorm(n, 0, sqrt(res_var)),
    y2 = eta_int + eta_slp * 1 + rnorm(n, 0, sqrt(res_var)),
    y3 = eta_int + eta_slp * 2 + rnorm(n, 0, sqrt(res_var)),
    y4 = eta_int + eta_slp * 3 + rnorm(n, 0, sqrt(res_var)),
    y5 = eta_int + eta_slp * 4 + rnorm(n, 0, sqrt(res_var))
  ) %>%
  select(id, y1:y5)

# Now fit the model and compare estimates to true values!
```

</details>
