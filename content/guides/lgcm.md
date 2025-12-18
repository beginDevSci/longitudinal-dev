---
title: "Latent Growth Curve Models"
slug: "lgcm"
description: "Learn to model individual trajectories over time using SEM-based growth curves in R with lavaan."
category: "growth-models"
tags: ["LGCM", "SEM", "longitudinal", "lavaan", "growth-curves"]
r_packages: ["lavaan", "tidyverse", "MASS"]
---

## Overview

---

### Why Study Growth?

Longitudinal data capture something cross-sectional data cannot: **change**. When you measure the same people over time, you can ask questions that matter:

- How do symptoms evolve after treatment begins?
- Do children's reading skills develop at the same rate?
- Does cognitive decline accelerate with age?

These are questions about **trajectories**—not just whether groups differ, but how individuals change.

#### The Limits of Simpler Methods

Traditional approaches fall short:

| Method | Limitation |
|--------|------------|
| **Repeated measures ANOVA** | Focuses on group means, ignores individual trajectories, assumes equal correlations across time |
| **Difference scores** | Works only for 2 time points, unreliable, ignores trajectory shape |
| **Paired t-tests** | No information about variability in change, can't handle multiple waves |

These methods answer "Did the group change?" but not "How do individuals differ in their change?"

#### What Growth Modeling Adds

Latent Growth Curve Models (LGCM) shift the question from group averages to individual trajectories. Every person gets their own:

- **Intercept** — where they start
- **Slope** — how fast they change

The model then asks: How much do people vary in these trajectories? And what predicts that variation?

---

### What LGCM Provides

LGCM offers four capabilities that simpler methods lack:

#### Individual Trajectories
Each person has their own growth curve. The model estimates not just the average trajectory, but the **variance** around it—how much people differ in starting points and rates of change.

#### Flexible Time Structures
Waves don't need to be equally spaced. Measure at baseline, 3 months, 6 months, and 2 years? No problem—just code time accordingly. The slope becomes "change per unit time" in whatever metric you choose.

#### Missing Data Handling
Modern estimation (Full Information Maximum Likelihood) uses all available data. Participants who miss a wave still contribute information—no listwise deletion required.

#### Testable Model Fit
Unlike approaches that simply estimate parameters, LGCM produces fit indices. You can ask: Does a linear trajectory actually describe these data? Is a more complex model (quadratic, piecewise) warranted?

---

### When LGCM is Appropriate

LGCM works well when you have:

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ waves (4+ preferred for testable fit) |
| **Continuous outcome** | Or ordinal with 5+ categories (use `estimator = "WLSMV"`) |
| **Interest in individual differences** | Not just "did the mean change?" |
| **Adequate sample** | 100+ for simple models; more for complex |

#### When to Consider Alternatives

> [!note]
> LGCM may not be the best choice when you have: only 2 time points (use change scores, ANCOVA, or autoregressive models instead), categorical outcomes with few categories (consider growth models for categorical data), interest only in group means (repeated measures ANOVA may suffice), or intensive longitudinal data with 100+ waves (consider time-series approaches).

---

### What You'll Learn

By the end of this tutorial, you will be able to:

1. **Understand LGCM conceptually** from both SEM and multilevel perspectives
2. **Evaluate** whether your data meet the requirements
3. **Specify** a linear growth model with appropriate time coding
4. **Estimate** the model in R using lavaan
5. **Evaluate** model fit and compare alternatives
6. **Interpret** parameters substantively
7. **Avoid** common pitfalls

#### How This Tutorial is Organized

| Section | Purpose | When to Use |
|---------|---------|-------------|
| **Overview** (you are here) | Orientation | Read first |
| **Conceptual Foundations** | Build intuition | Understand the model |
| **Model Specification & Fit** | Technical details | Set up your analysis |
| **Worked Example** | Complete analysis in R | Follow along with code |
| **Reference** | Lookup materials | Consult as needed |

You can read straight through or jump to specific sections. The Reference section is designed for lookup—consult it when you need a formula, troubleshooting help, or want to explore extensions.

---

## Conceptual Foundations

This section builds your intuition for what LGCM actually models. Two perspectives exist—choose whichever clicks with your background.

---

### Two Ways to Understand LGCM

LGCM can be understood through two equivalent lenses: as a factor model (SEM perspective) or as a hierarchical model (multilevel perspective). **They describe the same model**—just different intuitions.

---

#### Side-by-Side Comparison

<table>
<tr>
<th width="50%">SEM Perspective: Growth as Factor Analysis</th>
<th width="50%">Multilevel Perspective: Trajectories Within People</th>
</tr>
<tr>
<td>

**Core idea**: Repeated measures are indicators of two latent factors—intercept and slope.

Just like questionnaire items tap into a latent construct, your time points tap into a person's underlying trajectory.

</td>
<td>

**Core idea**: Observations are nested within people. Each person has their own regression line.

Level 1 models within-person change; Level 2 models between-person differences in trajectories.

</td>
</tr>
<tr>
<td>

**The Intercept Factor**

Captures each person's baseline level. All loadings fixed to 1:

```
Intercept → y1: 1
Intercept → y2: 1
Intercept → y3: 1
Intercept → y4: 1
Intercept → y5: 1
```

The intercept contributes equally to every time point.

</td>
<td>

**Level 1: Within-Person Model**

```
yᵢₜ = β₀ᵢ + β₁ᵢ(Time) + εᵢₜ
```

Person *i*'s score at time *t* = their intercept + their slope × time + residual.

Each person gets their own line.

</td>
</tr>
<tr>
<td>

**The Slope Factor**

Captures each person's rate of change. Loadings encode time:

```
Slope → y1: 0
Slope → y2: 1
Slope → y3: 2
Slope → y4: 3
Slope → y5: 4
```

At Time 0, slope contributes nothing. At Time 4, slope contributes 4 × (person's slope).

</td>
<td>

**Level 2: Between-Person Model**

```
β₀ᵢ = γ₀₀ + u₀ᵢ  (intercept)
β₁ᵢ = γ₁₀ + u₁ᵢ  (slope)
```

Individual intercepts and slopes are draws from a population distribution.

- γ₀₀ = average intercept
- γ₁₀ = average slope
- u₀ᵢ, u₁ᵢ = person-specific deviations

</td>
</tr>
<tr>
<td>

**What the model estimates**:
- Factor means (average intercept, average slope)
- Factor variances (individual differences)
- Factor covariance (intercept-slope relationship)
- Residual variances

</td>
<td>

**What the model estimates**:
- Fixed effects (γ₀₀, γ₁₀)
- Random effect variances (Var(u₀), Var(u₁))
- Random effect covariance (Cov(u₀, u₁))
- Residual variance

</td>
</tr>
</table>

---

#### Parameter Correspondence

The two framings use different notation for the same quantities:

| Concept | SEM Notation | Multilevel Notation |
|---------|--------------|---------------------|
| Average starting level | Intercept factor mean (μ₁) | Fixed intercept (γ₀₀) |
| Average rate of change | Slope factor mean (μ₂) | Fixed slope (γ₁₀) |
| Variability in starting levels | Intercept variance (ψ₁₁) | Var(u₀ᵢ) |
| Variability in change rates | Slope variance (ψ₂₂) | Var(u₁ᵢ) |
| Start-change relationship | I-S covariance (ψ₁₂) | Cov(u₀ᵢ, u₁ᵢ) |
| Occasion-specific noise | Residual variance (θ) | Var(εᵢₜ) |

---

#### Path Diagram (SEM Representation)

![LGCM Path Diagram](/images/guides/lgcm/lgcm_path_diagram.svg)

*Figure: Path diagram of a linear LGCM. The intercept factor (blue) has all loadings fixed to 1. The slope factor (orange) has loadings encoding time (0, 1, 2, 3, 4). The curved arrow represents the covariance between factors. Residuals (ε) capture occasion-specific variation.*

<details>
<summary>ASCII version (for text-only environments)</summary>

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

---

#### Spaghetti Plot (Multilevel Representation)

The multilevel perspective naturally leads to visualizing individual trajectories:

![Individual Growth Trajectories](/images/guides/lgcm/fig01_spaghetti_all.png)

*Figure: Each line represents one participant's observed scores across 5 waves. The spread at Wave 1 shows intercept variance; the "fan" pattern shows slope variance.*

![Variability in Trajectories](/images/guides/lgcm/fig04_highlighted.png)

*Figure: Extreme cases highlighted to illustrate different combinations of intercept and slope. High starters with fast growth (blue), high starters with slow growth (purple), low starters with fast growth (orange), and low starters with slow growth (red).*

---

### Key Components of Linear LGCM

Regardless of which framing you prefer, these are the building blocks:

#### The Two Latent Factors

| Factor | What It Captures | Defined By |
|--------|------------------|------------|
| **Intercept (η₁)** | Each person's level at Time = 0 | Where you place "0" in time coding |
| **Slope (η₂)** | Each person's rate of change per time unit | Constant in linear model |

These aren't observed directly—we infer them from the pattern of scores over time.

#### Factor Loadings: Encoding Time

In LGCM, loadings are **fixed**, not estimated:

| Factor | Loadings | Why |
|--------|----------|-----|
| Intercept | 1, 1, 1, 1, 1 | Contributes equally at every time point |
| Slope | 0, 1, 2, 3, 4 | Encodes time; contribution grows linearly |

The loadings 0, 1, 2, 3, 4 assume equally spaced waves with Time 0 at Wave 1. Adjust if your design differs (e.g., for waves at months 0, 3, 6, 12, 24, use loadings 0, 3, 6, 12, 24 so slope = change per month).

#### Factor Means: The Average Trajectory

| Parameter | Symbol | Interpretation |
|-----------|--------|----------------|
| Intercept mean | μ₁ | Average starting level across people |
| Slope mean | μ₂ | Average rate of change across people |

Together they define the average trajectory:

```
Average score at time t = μ₁ + μ₂ × t
```

Example: If μ₁ = 50 and μ₂ = 2, the average person starts at 50 and increases by 2 per wave.

#### Factor Variances: Individual Differences

| Parameter | Symbol | Interpretation |
|-----------|--------|----------------|
| Intercept variance | ψ₁₁ | How much people differ in starting levels |
| Slope variance | ψ₂₂ | How much people differ in change rates |

![Slope Variance Comparison](/images/guides/lgcm/slope_variance_comparison.svg)

*Figure: Visualizing what slope variance means. Left panel shows low slope variance—everyone changes at similar rates, so lines stay roughly parallel. Right panel shows high slope variance—some people improve rapidly, others stay flat, and some decline. The dark line represents the mean trajectory in both cases.*

**Interpreting scale**: Variances are squared units. Take the square root for SD:
- If ψ₁₁ = 100 → SD = 10 → ~68% of people have intercepts within ±10 of the mean
- If ψ₂₂ = 1 → SD = 1 → ~68% of people have slopes within ±1 of the mean

![Distribution of Growth Parameters](/images/guides/lgcm/fig05_distributions.png)

*Figure: Histograms showing the distribution of individual intercepts (left) and slopes (right) across 400 participants.*

#### Factor Covariance: The Intercept-Slope Relationship

| Sign | Interpretation |
|------|----------------|
| **Positive** | High starters grow faster (or decline slower). "Rich get richer." |
| **Negative** | High starters grow slower (or decline faster). Regression to mean. |
| **Zero** | Starting level doesn't predict change rate. |

![Intercept-Slope Relationship](/images/guides/lgcm/fig06_intercept_slope.png)

*Figure: Scatterplot of individual intercepts vs. slopes. The negative correlation (r ≈ -0.20) indicates that participants who started higher tended to grow slightly slower.*

Convert to correlation for easier interpretation:

```
r = ψ₁₂ / √(ψ₁₁ × ψ₂₂)
```

A correlation of -0.20 means a modest negative relationship: those who start higher grow slightly slower.

#### Residuals: What the Trajectory Doesn't Explain

```
Observed = Predicted + Residual
yᵢₜ = (η₁ᵢ + η₂ᵢ × t) + εᵢₜ
```

**Residual variance (θ)**: Noise around individual trajectories—measurement error, occasion-specific factors, or model misspecification.

**Key assumptions**:
- Residuals uncorrelated across time (can be relaxed in advanced models)
- Can be equal or freely estimated across waves

---

### Choosing Your Framing

#### It Doesn't Matter Much

The model is identical—only the intuition differs. Both yield the same parameter estimates.

#### Practical Considerations

| If you're comfortable with... | Use this framing |
|------------------------------|------------------|
| CFA, SEM, path analysis | SEM perspective |
| Mixed models, HLM, lme4 | Multilevel perspective |
| Neither (new to both) | Multilevel often more intuitive |

#### Software Implications

| Framing | Natural Software | Syntax Style |
|---------|------------------|---------------|
| SEM | lavaan, Mplus, LISREL | Factor loadings, latent variables |
| Multilevel | lme4, nlme, HLM | Random effects, nesting |

**This tutorial uses SEM notation and lavaan syntax**, but references the multilevel interpretation when it clarifies a concept.

#### Why Both Perspectives Matter

- **SEM framing** connects LGCM to the broader world of structural equation modeling (mediation, moderation, latent variables)
- **Multilevel framing** connects to intensive longitudinal data, ecological momentary assessment, and time-series approaches
- **Knowing both** lets you read literature from different traditions and communicate with diverse collaborators

---

#### What This Tutorial Does (and Does Not) Cover

In this tutorial, we treat each repeated measure (y1–y5) as an observed outcome and focus on a **manifest-variable LGCM**: a growth model for observed scores over time. We are **not** fitting a full longitudinal measurement model (e.g., a factor structure at each wave with multiple indicators) or a longitudinal CFA.

Those more complex models—sometimes called "curve-of-factors" or "second-order growth models"—are important when you want to:
- Separate measurement error from true change
- Model growth in a latent construct measured by multiple items
- Test measurement invariance across time

These extensions are beyond the scope of this introductory tutorial. If your data include multiple indicators per construct at each wave, consider resources on longitudinal measurement models (e.g., Little, 2013; Grimm et al., 2017).

---

#### Key Takeaways

1. **Two factors** (intercept and slope) with **fixed loadings** that encode time

2. **Factor means** = average trajectory; **factor variances** = individual differences

3. **Factor covariance** = relationship between starting level and rate of change

4. **SEM and multilevel framings are equivalent**—choose based on your background

5. The model is **parsimonious**: 6–10 parameters describe trajectories for all individuals

---

#### Interactive Exploration

To build deeper intuition for how LGCM parameters affect trajectories, try the interactive explorer. Adjust the sliders to see how changing the intercept mean, slope variance, and other parameters affects the spaghetti plot in real-time.

**[Open Interactive Trajectory Explorer](/images/guides/lgcm/interactive/trajectory_explorer.html)** *(opens in browser)*

<details>
<summary>Embedded interactive version (may not render in all contexts)</summary>

<iframe
  src="/images/guides/lgcm/interactive/trajectory_explorer.html"
  width="100%"
  height="700"
  style="border: 1px solid #e2e8f0; border-radius: 8px;"
  title="Interactive LGCM Trajectory Explorer">
</iframe>

</details>

*Interactive Figure: Adjust LGCM parameters to see their effect on individual trajectories. Try setting Slope SD = 0 to see parallel lines, or increase it to watch the "fan pattern" emerge.*

> **Note**: If viewing on GitHub or a static site, click the link above to open the interactive version in your browser. The HTML file must be served locally or hosted—it won't work as a raw file link.

---

## Model Specification & Fit

This section covers the technical details: what your data need, how to specify the model, and how to evaluate whether it fits.

---

### Data Requirements

Before fitting an LGCM, verify that your data meet these requirements.

#### Minimum Time Points

| Time Points | What You Can Estimate |
|-------------|----------------------|
| 2 | LGCM offers little advantage; use change scores, ANCOVA, or regression instead |
| 3 | Linear growth with minimal testable fit (df = 1 with free residuals) |
| 4 | Linear growth with good testable fit (df = 5) |
| 4 | Quadratic growth (just-identified) |
| 5+ | Linear or quadratic with robust testable fit |

With only two time points, you can estimate mean change and variance of change, but a latent growth curve model does not offer meaningful advantages over simpler approaches like paired t-tests or ANCOVA.

**Recommendation**: Aim for at least 4 time points. With 3 time points you have only 1 degree of freedom for testing fit (assuming free residual variances), which provides limited power to detect model misspecification.

#### Sample Size

Small samples lead to unstable estimates, convergence problems, and improper solutions (e.g., negative variance estimates).

| Model Complexity | Suggested Minimum N |
|-----------------|---------------------|
| Simple linear LGCM | 100 |
| Linear with covariates | 150–200 |
| Quadratic growth | 200+ |
| Multiple-group models | 100+ per group |

These guidelines draw on simulation studies (e.g., Hertzog et al., 2006; Curran et al., 2010). Key findings:

- **Power for slope mean** is typically adequate with N = 100 if effect sizes are moderate
- **Power for slope variance** requires larger samples (N = 200+) because variance components are harder to estimate precisely
- **More time points help**: With 5+ waves, you can detect smaller effects with fewer participants

For formal power analysis, consider simulation-based approaches using the `simsem` R package or Mplus Monte Carlo facilities.

**Reference**: Hertzog, C., Lindenberger, U., Ghisletta, P., & von Oertzen, T. (2006). On the power of multivariate latent growth curve models to detect correlated change. *Psychological Methods, 11*(3), 244–252.

#### Missing Data

Real longitudinal data almost always have missing observations. LGCM handles this well—but only under certain conditions.

**Full Information Maximum Likelihood (FIML)**: Modern SEM software (including lavaan) uses FIML by default. It uses all available data from each participant without imputation or deletion.

**The MAR Assumption**: FIML assumes data are Missing At Random—missingness can depend on observed variables but not on the missing values themselves.

- *MAR example*: Participants with lower baseline scores drop out more often. (Baseline was observed.)
- *MNAR example*: Participants drop out because their current unobserved score is extreme. (Estimates may be biased.)

**Problematic patterns**:
- Sporadic missingness (random waves missing): Generally fine
- Monotone dropout: Fine under MAR, though you're extrapolating
- Systematic dropout related to trajectory: Potentially problematic

**Auxiliary variables**: Including variables that predict missingness (but aren't part of your main model) can make MAR more plausible and improve estimation. For example, if participants with lower education drop out more, including education as an auxiliary variable helps FIML "borrow" information appropriately.

```r
# Method 1: Include auxiliary variables that predict missingness
# Add correlations between auxiliaries and model variables
model_with_aux <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  # Correlate auxiliaries with latent factors
  intercept ~~ education + baseline_severity
  slope ~~ education + baseline_severity
'

fit <- growth(model_with_aux, data = data_wide, missing = "fiml")

# Method 2: Use auxiliary argument in lavaan (simpler)
fit <- growth(model, data = data_wide,
              missing = "fiml",
              auxiliary = c("education", "baseline_severity"))
```

#### Data Format

LGCM in lavaan requires **wide format**: each row is a participant, each column is a time point.

```
id    y1    y2    y3    y4    y5
1     48    52    54    56    59
2     55    54    57    55    58
3     42    45    47    50    52
```

If your data are in long format:

```r
library(tidyverse)

data_wide <- data_long %>%
  select(id, wave, y) %>%
  pivot_wider(names_from = wave, values_from = y, names_prefix = "y")
```

#### Distributional Considerations

ML estimation assumes multivariate normality. Violations can affect standard errors (underestimated) and chi-square statistics (inflated).

**What to check**: Skewness, kurtosis, outliers, floor/ceiling effects at each wave.

```r
# Visual check: histograms for each wave
data_wide %>%
  select(y1:y5) %>%
  pivot_longer(everything(), names_to = "wave", values_to = "score") %>%
  ggplot(aes(x = score)) +
  geom_histogram(bins = 30, fill = "steelblue", alpha = 0.7) +
  facet_wrap(~wave, scales = "free_y") +
  theme_minimal() +
  labs(title = "Distribution at Each Wave")

# Numerical check: skewness and kurtosis
data_wide %>%
  select(y1:y5) %>%
  summarise(across(everything(), list(
    mean = ~mean(., na.rm = TRUE),
    sd = ~sd(., na.rm = TRUE),
    skew = ~moments::skewness(., na.rm = TRUE),
    kurt = ~moments::kurtosis(., na.rm = TRUE)
  ))) %>%
  pivot_longer(everything()) %>%
  separate(name, into = c("wave", "stat"), sep = "_") %>%
  pivot_wider(names_from = stat, values_from = value)

# Multivariate normality test (optional, requires MVN package)
# install.packages("MVN")
# MVN::mvn(data_wide[, c("y1", "y2", "y3", "y4", "y5")])$multivariateNormality
```

**Guidelines**:
- Skewness: |skew| < 2 is generally acceptable
- Kurtosis: |kurt| < 7 is generally acceptable
- For violations, use `estimator = "MLR"` for robust standard errors

**What to do**:
- Mild violations: Use robust SEs (`estimator = "MLR"` in lavaan)
- Severe violations: Consider transformations or robust estimation
- Categorical outcomes: Use appropriate estimators (WLSMV)

#### Pre-Flight Checklist

- [ ] 3+ time points (4+ preferred)
- [ ] Adequate sample size (100+ for simple models)
- [ ] Missingness examined and plausibly MAR
- [ ] Data in wide format
- [ ] Distributions checked
- [ ] Time structure known (spacing between waves)

---

### Specifying a Linear LGCM

#### Time Coding: The Most Important Decision

Slope factor loadings define how time enters the model. This determines:
- What the intercept represents
- How to interpret the slope
- Whether unequal spacing is handled correctly

**Standard coding (0, 1, 2, 3, 4)**:

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

#### Centering at Different Time Points

Shift the zero point to change what the intercept represents:

| Centering | Loadings | Intercept Meaning |
|-----------|----------|-------------------|
| Wave 1 (standard) | 0, 1, 2, 3, 4 | Expected score at Wave 1 |
| Midpoint (Wave 3) | -2, -1, 0, 1, 2 | Expected score at Wave 3 |
| Final (Wave 5) | -4, -3, -2, -1, 0 | Expected score at Wave 5 |

![Time Coding and Centering](/images/guides/lgcm/time_coding_centering.svg)

*Figure: How time coding determines the intercept's location. All three panels show the same trajectory, but the intercept (marked "I") refers to different time points depending on where you place the zero in your slope loadings. The slope (rate of change) remains identical across all three.*

<details>
<summary>Text description (if image doesn't render)</summary>

Three panels showing the same linear trajectory with different intercept locations:
- **Left (Wave 1 centering)**: Loadings 0,1,2,3,4 — intercept "I" marked at Wave 1
- **Center (Midpoint centering)**: Loadings -2,-1,0,1,2 — intercept "I" marked at Wave 3
- **Right (Final wave centering)**: Loadings -4,-3,-2,-1,0 — intercept "I" marked at Wave 5

The slope (line angle) is identical in all three; only the intercept reference point changes.

</details>

The slope doesn't change—only the intercept shifts. Choose centering that matches your research question.

#### Non-Equidistant Time Points

If waves aren't equally spaced, use actual time values:

```
Baseline (month 0)  → Loading: 0
3 months            → Loading: 3
6 months            → Loading: 6
12 months           → Loading: 12
24 months           → Loading: 24
```

Now slope = change per **month**. Rescale for interpretability if needed (e.g., 0, 0.25, 0.5, 1, 2 for years).

#### Basic lavaan Syntax

```r
library(lavaan)

model <- '
  # Intercept factor: all loadings = 1
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5

  # Slope factor: loadings = time
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit <- growth(model, data = data_wide)
```

The `growth()` function automatically estimates:
- Factor means (intercept and slope)
- Factor variances and covariance
- Residual variances

#### Constraining Residual Variances

To force equal residual variances, use labels:

```r
model_equal_resid <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  # Constrain to equality
  y1 ~~ rv*y1
  y2 ~~ rv*y2
  y3 ~~ rv*y3
  y4 ~~ rv*y4
  y5 ~~ rv*y5
'
```

The label `rv` forces all five residual variances to be estimated as one parameter.

#### Residual Variances: Equal vs. Free

- **Free (default)**: Each time point has its own residual variance. More flexible, uses more parameters.
- **Equal (constrained)**: All time points share the same residual variance. More parsimonious.

**Recommendation**: Start with free residuals, then test whether equality constraint worsens fit (see Section 3.4).

#### Parameter Count

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

Positive df means the model is testable.

#### Identification Requirements

A model is identified when there's a unique best-fitting solution. LGCM achieves this through fixed loadings:

- **Linear growth**: 3+ time points (just-identified at 3, testable at 4+)
- **Quadratic growth**: 4+ time points (just-identified at 4, testable at 5+)

---

### Estimation & Fit Indices

#### Maximum Likelihood Estimation

lavaan uses ML by default: find parameter values that make your observed data most probable.

ML estimation:
- Uses observed means and covariances
- Assumes multivariate normality
- Handles missing data via FIML
- Produces standard errors for inference

**Robust estimation**: If concerned about non-normality:

```r
fit <- growth(model, data = data_wide, estimator = "MLR")
```

MLR provides robust standard errors and scaled test statistics (Satorra-Bentler correction).

#### Running and Viewing Results

```r
# Fit model
fit <- growth(model, data = data_wide)

# View results
summary(fit, fit.measures = TRUE, standardized = TRUE)

# Extract specific fit measures
fitmeasures(fit, c("chisq", "df", "pvalue", "cfi", "rmsea", "srmr"))
```

#### Fit Indices

##### Chi-Square (χ²)

Tests whether model-implied covariances match observed covariances.

- Non-significant p (> .05): Model fits adequately
- Significant p (< .05): Model doesn't fit perfectly

**Problem**: Large samples make trivial misfits significant. Don't rely on χ² alone.

##### RMSEA (Root Mean Square Error of Approximation)

Estimates population misfit, adjusted for parsimony.

| Value | Interpretation |
|-------|----------------|
| < 0.05 | Close fit |
| 0.05–0.08 | Reasonable fit |
| > 0.10 | Poor fit |

RMSEA rewards parsimony and provides a 90% confidence interval.

##### CFI (Comparative Fit Index)

Compares your model to a null model (all variables uncorrelated).

| Value | Interpretation |
|-------|----------------|
| > 0.95 | Good fit |
| 0.90–0.95 | Acceptable |
| < 0.90 | Poor fit |

##### SRMR (Standardized Root Mean Square Residual)

Average discrepancy between observed and implied correlations.

| Value | Interpretation |
|-------|----------------|
| < 0.08 | Good fit |
| > 0.10 | Poor fit |

#### Interpreting Multiple Indices

No single index is definitive. Look for convergence:

- All indices suggest good fit → Confident
- Indices disagree → Investigate further
- Most indices suggest poor fit → Revise model

**Reasonable targets** (not hard cutoffs): CFI ≥ 0.95, RMSEA ≤ 0.06, SRMR ≤ 0.08

#### Troubleshooting Common lavaan Issues

When fitting LGCMs, you may encounter errors or warnings. Here are the most common issues and how to resolve them:

**"Model did not converge":**
- Check sample size—is N too small for your model complexity?
- Inspect for missing data patterns or extreme outliers
- Try a simpler model first (e.g., intercept-only) to verify the data are working
- Consider providing starting values for difficult parameters:

```r
fit <- growth(model, data = data_wide,
              start = list(intercept ~ 50, slope ~ 2))
```

**"Covariance matrix of latent variables is not positive definite":**
- Check for very high correlations between time points (multicollinearity)
- Verify that variance estimates are not near zero or negative
- Consider simplifying the model (e.g., constraining residual variances to be equal)
- Check whether you have enough variability in your data

**Negative variance estimate (Heywood case):**
- This may indicate true near-zero variance in the population
- Could signal model misspecification (wrong functional form, missing covariates)
- Consider constraining the problematic variance to zero:

```r
model <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
  slope ~~ 0*slope  # Constrain slope variance to 0
'
```

**Huge standard errors:**
- Parameter poorly identified—model may be too complex for data
- Check modification indices for hints about model misspecification
- Consider whether you have enough variance in your outcome

**lavaan errors about variable names:**
- Confirm variable names in the syntax exactly match column names in your data
- Use `names(data_wide)` to verify spelling and capitalization
- Watch for spaces or special characters in variable names

For conceptual pitfalls related to interpretation (rather than estimation), see the [FAQ & Common Pitfalls](#faq--common-pitfalls) section.

---

### Model Comparison

#### Why Compare Models?

1. **Test hypotheses**: Is there growth? Do people differ?
2. **Select best model**: Balance fit and parsimony

#### Nested vs. Non-Nested Models

**Nested**: One model is a constrained version of another.
- Intercept-only nested within linear growth
- Equal residuals nested within free residuals

**Non-nested**: Neither is a special case of the other.

#### Likelihood Ratio Test (Nested Models)

If Model A is nested within Model B:

```
Δχ² = χ²(constrained) - χ²(unconstrained)
Δdf = df(constrained) - df(unconstrained)
```

Significant Δχ²: The constraints worsen fit; reject the simpler model.

**In lavaan**:

```r
anova(fit_constrained, fit_unconstrained)
```

**With MLR estimation**:

```r
anova(fit_constrained, fit_unconstrained, method = "satorra.bentler.2001")
```

#### Information Criteria (Any Models)

**AIC**: -2×LL + 2×(parameters). Lower = better. Moderate parsimony penalty.

**BIC**: -2×LL + log(N)×(parameters). Lower = better. Stronger parsimony penalty.

```r
fitmeasures(fit, c("aic", "bic"))
```

If AIC and BIC agree, you're confident. If they disagree, acknowledge ambiguity.

#### Common Comparisons

##### Test 1: Is There Growth?

The most fundamental comparison: does adding a slope factor significantly improve fit?

**[Open Interactive Model Comparison](/images/guides/lgcm/interactive/model_comparison.html)** *(opens in browser)*

<details>
<summary>Embedded interactive version (may not render in all contexts)</summary>

<iframe
  src="/images/guides/lgcm/interactive/model_comparison.html"
  width="100%"
  height="680"
  style="border: 1px solid #e2e8f0; border-radius: 8px;"
  title="Interactive Model Comparison">
</iframe>

</details>

*Interactive Figure: Click the buttons to animate between an intercept-only model (flat trajectories) and a linear growth model (sloped trajectories). Notice how adding the slope factor allows the model to capture systematic change.*

```r
model_intercept <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
'

model_linear <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit_int <- growth(model_intercept, data = data_wide)
fit_lin <- growth(model_linear, data = data_wide)
anova(fit_int, fit_lin)
```

Significant: Yes, there's systematic change.

##### Test 2: Do People Differ in Growth?

```r
model_fixed <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
  slope ~~ 0*slope          # Fix variance to 0
  intercept ~~ 0*slope      # Fix covariance to 0
'

model_random <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit_fixed <- growth(model_fixed, data = data_wide)
fit_random <- growth(model_random, data = data_wide)
anova(fit_fixed, fit_random)
```

Significant: Yes, individuals differ in change rates.

**Note**: Testing variance = 0 is a boundary test. The p-value is conservative (true p ≈ half the reported value).

##### Test 3: Are Residual Variances Equal?

```r
model_free <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

model_equal <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
  y1 ~~ rv*y1
  y2 ~~ rv*y2
  y3 ~~ rv*y3
  y4 ~~ rv*y4
  y5 ~~ rv*y5
'

fit_free <- growth(model_free, data = data_wide)
fit_equal <- growth(model_equal, data = data_wide)
anova(fit_equal, fit_free)
```

Non-significant: Equal residuals justified; use simpler model.

#### Decision Framework

1. Fit intercept-only model
2. Fit linear growth model
3. Compare (LRT): Is growth significant?
4. Test equal residual variances
5. Select final model based on LRT and AIC/BIC

#### Reporting

Example:

> We compared an intercept-only model to a linear growth model. Linear growth fit significantly better, Δχ²(3) = 226.10, p < .001. Constraining residual variances to equality did not significantly worsen fit, Δχ²(4) = 5.23, p = .26. The final model includes linear growth with equal residual variances (CFI = 0.996, RMSEA = 0.025, SRMR = 0.030).

| Model | χ² | df | CFI | RMSEA | AIC | BIC |
|-------|-----|-----|------|-------|-----|-----|
| Intercept only | 238.45 | 13 | 0.82 | 0.21 | 10456 | 10484 |
| Linear growth | 12.35 | 10 | 0.998 | 0.024 | 10234 | 10274 |
| Linear (equal resid) | 17.58 | 14 | 0.996 | 0.025 | 10228 | 10260 |

---

## Worked Example

This section walks through a complete analysis using simulated data. Run the code yourself to see LGCM in action.

---

### Practical Workflow Overview

```
┌─────────────────────────────────┐
│ 1. Setup                        │
│    Load packages, set seed      │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 2. Simulate/Load Data           │
│    Wide format, check structure │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 3. Visualize                    │
│    Spaghetti plot, look first   │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 4. Fit Baseline (Intercept)     │
│    No-growth comparison model   │
└────────────────┬────────────────┘
                 ▼
┌─────────────────────────────────┐
│ 5. Fit Linear Growth            │
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
│    Parameters, effect sizes     │
└─────────────────────────────────┘
```

---

### Setup

```r
# Load packages
library(tidyverse)  # Data manipulation and plotting
library(lavaan)     # SEM and growth models
library(MASS)       # For mvrnorm (simulation)

# Set seed for reproducibility
set.seed(2024)
```

---

### Simulate/Load Data

We'll simulate data with **known population parameters** so we can verify our estimates:

| Parameter | True Value |
|-----------|------------|
| Intercept mean | 50 |
| Slope mean | 2 |
| Intercept variance | 100 (SD = 10) |
| Slope variance | 1 (SD = 1) |
| Intercept-slope covariance | -2 (r ≈ -0.20) |
| Residual variance | 25 (SD = 5) |

```r
# Parameters
n <- 400
time_points <- 0:4

# Latent factor distribution
psi <- matrix(c(100, -2,
                -2,   1), nrow = 2)

factors <- mvrnorm(n = n,
                   mu = c(50, 2),      # intercept mean, slope mean
                   Sigma = psi)         # variance-covariance matrix

# Generate observed data
data_wide <- tibble(id = 1:n) %>%
  mutate(
    int_i = factors[, 1],
    slp_i = factors[, 2],
    y1 = int_i + slp_i * 0 + rnorm(n, 0, 5),
    y2 = int_i + slp_i * 1 + rnorm(n, 0, 5),
    y3 = int_i + slp_i * 2 + rnorm(n, 0, 5),
    y4 = int_i + slp_i * 3 + rnorm(n, 0, 5),
    y5 = int_i + slp_i * 4 + rnorm(n, 0, 5)
  ) %>%
  select(id, y1:y5)  # Keep only what lavaan needs

# Check structure
head(data_wide)
```

*Note: Your output will differ slightly due to random sampling, but the structure and general patterns should match.*

```
# A tibble: 6 × 6
     id    y1    y2    y3    y4    y5
  <int> <dbl> <dbl> <dbl> <dbl> <dbl>
1     1  52.3  55.1  58.2  60.4  63.1
2     2  48.7  49.2  51.8  52.9  55.0
3     3  61.2  62.5  64.1  67.3  68.9
...
```

```r
# Summary statistics
data_wide %>%
  select(y1:y5) %>%
  summary()

# Correlations (should show simplex pattern)
data_wide %>%
  select(y1:y5) %>%
  cor() %>%
  round(2)
```

---

### Visualize

**Always plot before modeling.**

```r
# Reshape for plotting
data_long <- data_wide %>%
  pivot_longer(y1:y5, names_to = "wave", values_to = "y") %>%
  mutate(time = as.numeric(gsub("y", "", wave)) - 1)

# Spaghetti plot
ggplot(data_long, aes(x = time, y = y, group = id)) +
  geom_line(alpha = 0.15, color = "gray40") +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Score",
       title = "Individual Growth Trajectories",
       subtitle = "N = 400 participants, 5 waves") +
  theme_minimal()
```

**What to look for:**
- General trend (up, down, flat?)
- Spread at baseline (intercept variance)
- Fan pattern (slope variance)—do lines diverge over time?
- Nonlinearity (curves vs. straight lines)

```r
# Add mean trajectory
mean_traj <- data_long %>%
  group_by(time) %>%
  summarise(mean_y = mean(y))

ggplot(data_long, aes(x = time, y = y)) +
  geom_line(aes(group = id), alpha = 0.1, color = "gray40") +
  geom_line(data = mean_traj, aes(y = mean_y),
            color = "steelblue", linewidth = 1.5) +
  geom_point(data = mean_traj, aes(y = mean_y),
             color = "steelblue", size = 3) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  labs(x = "Time", y = "Score",
       title = "Individual Trajectories with Mean Overlay") +
  theme_minimal()
```

![Individual Trajectories with Mean](/images/guides/lgcm/fig02_spaghetti_mean.png)

*Figure: Individual trajectories (gray) with mean trajectory overlay (blue). The mean line shows the average growth pattern across all 400 participants.*

---

### Fit Baseline LGCM

Start with an **intercept-only model** (no growth)—this is our comparison baseline.

```r
model_intercept <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
'

fit_intercept <- growth(model_intercept, data = data_wide)
summary(fit_intercept, fit.measures = TRUE)
```

This model assumes everyone has a stable mean across time (no systematic change). It will fit poorly if growth exists.

---

### Fit Alternative Model

Now fit the **linear growth model**:

```r
model_linear <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit_linear <- growth(model_linear, data = data_wide)
summary(fit_linear, fit.measures = TRUE, standardized = TRUE)
```

Also test **equal residual variances**:

```r
model_linear_eq <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  # Constrain residual variances to equality
  y1 ~~ rv*y1
  y2 ~~ rv*y2
  y3 ~~ rv*y3
  y4 ~~ rv*y4
  y5 ~~ rv*y5
'

fit_linear_eq <- growth(model_linear_eq, data = data_wide)
```

---

### Compare Models

#### Is there growth?

```r
anova(fit_intercept, fit_linear)
```

Expected output (illustrative—your exact values will differ due to random sampling):
```
Chi-Squared Difference Test

               Df   AIC   BIC  Chisq Chisq diff Df diff Pr(>Chisq)
fit_linear     10 11234 11274  12.35
fit_intercept  13 11856 11884 638.21     625.86       3  < 2.2e-16 ***
```

Δχ² is huge and p < .001 → **Linear growth significantly improves fit.**

#### Are equal residual variances justified?

```r
anova(fit_linear_eq, fit_linear)
```

If non-significant, equal residuals are fine—use the simpler model.

#### Information criteria

```r
data.frame(
  Model = c("Intercept-only", "Linear", "Linear (equal resid)"),
  AIC = c(AIC(fit_intercept), AIC(fit_linear), AIC(fit_linear_eq)),
  BIC = c(BIC(fit_intercept), BIC(fit_linear), BIC(fit_linear_eq))
) %>%
  mutate(across(where(is.numeric), \(x) round(x, 1)))
```

---

### Model Diagnostics

Before interpreting results, check for localized misfit and problematic residual patterns.

#### Modification indices

Modification indices suggest model improvements. Large values (>10) indicate where freeing a parameter would substantially improve fit.

```r
# Check for localized misfit
modindices(fit_linear, sort = TRUE, minimum.value = 10)
```

**What to look for**:
- Residual covariances (e.g., `y2 ~~ y3`): May indicate autocorrelation or unmeasured occasion-specific factors
- Cross-loadings: Usually shouldn't be freed in LGCM (would change interpretation)

**If modification indices suggest residual covariances**: Consider whether it's substantively meaningful (e.g., adjacent waves sharing method variance) or a sign of model misspecification.

#### Residual correlations

Examine whether the model adequately reproduces the observed correlations.

```r
# Residual correlation matrix (observed - implied)
resid(fit_linear, type = "cor")$cov %>%
  round(3)
```

**Interpretation**: Values should be close to zero (typically |r| < 0.10). Large residual correlations indicate the model doesn't fully capture the relationship between those variables.

#### Check for problematic estimates

```r
# Look for Heywood cases or boundary estimates
parameterEstimates(fit_linear) %>%
  filter(op == "~~") %>%
  select(lhs, rhs, est, se) %>%
  mutate(flag = ifelse(est < 0, "NEGATIVE", ""))
```

**Red flags**:
- Negative variances (Heywood cases): Model may be misspecified or sample too small
- Very large standard errors: Parameter poorly identified
- Variances estimated at exactly zero: May need to constrain or simplify

---

### Interpret Results

#### Extract fit indices

```r
fitmeasures(fit_linear, c("chisq", "df", "pvalue", "cfi", "rmsea", "srmr"))
```

Expected (approximately):
```
 chisq     df pvalue    cfi  rmsea   srmr
12.35  10.00   0.26  0.998  0.024  0.025
```

All indices indicate good fit.

#### Extract parameter estimates

```r
parameterEstimates(fit_linear) %>%
  filter(op %in% c("~1", "~~")) %>%
  select(lhs, op, rhs, est, se, pvalue) %>%
  mutate(across(c(est, se), round, 3),
         pvalue = ifelse(pvalue < .001, "<.001", round(pvalue, 3)))
```

#### Results table

| Parameter | Estimate | SE | p | True Value |
|-----------|----------|-----|------|------------|
| Intercept mean | 49.82 | 0.51 | <.001 | 50 |
| Slope mean | 2.04 | 0.06 | <.001 | 2 |
| Intercept variance | 98.34 | 8.12 | <.001 | 100 |
| Slope variance | 0.97 | 0.12 | <.001 | 1 |
| I-S covariance | -1.89 | 0.62 | .002 | -2 |
| Residual variance | 24.56 | 1.23 | <.001 | 25 |

**The estimates closely recover the true population parameters.**

#### Interpreting each parameter

| Parameter | What it means |
|-----------|---------------|
| **Intercept mean = 49.82** | Average score at Wave 1 was ~50 |
| **Slope mean = 2.04** | On average, scores increased by ~2 units per wave |
| **Intercept variance = 98.34** | People differed substantially in starting levels (SD ≈ 10) |
| **Slope variance = 0.97** | People differed in growth rates (SD ≈ 1); some grew by 3+/wave, others by <1 |
| **I-S covariance = -1.89** | Higher starters grew slightly slower (r ≈ -0.19); see [Pitfall 7](#pitfall-7) for interpretation guidance |
| **Residual variance = 24.56** | After accounting for trajectories, SD ≈ 5 of unexplained variation |

#### Converting to correlation

```r
# Intercept-slope correlation
cov_is <- -1.89
var_i <- 98.34
var_s <- 0.97

r_is <- cov_is / sqrt(var_i * var_s)
r_is  # ≈ -0.19
```

#### Proportion with positive slopes

```r
# What proportion of people are actually improving?
slope_mean <- 2.04
slope_sd <- sqrt(0.97)  # ≈ 0.98

# P(slope > 0) = P(Z > -2.04/0.98) = P(Z > -2.08)
pnorm(0, mean = slope_mean, sd = slope_sd, lower.tail = FALSE)
# ≈ 0.98 -- almost everyone is improving
```

#### Effect sizes and variance explained

Beyond raw parameter estimates, effect sizes help communicate the magnitude and practical significance of your findings.

```r
# R² for each observed variable (proportion explained by trajectory)
inspect(fit_linear, "r2")
```

Expected output:
```
   y1    y2    y3    y4    y5
0.800 0.825 0.856 0.878 0.893
```

**Interpretation**: The trajectory (intercept + slope) explains 80–89% of variance at each wave. The remaining 10–20% is residual variance (measurement error, occasion-specific factors). R² values of 0.70–0.90 are typical for well-fitting LGCMs.

```r
# Standardized solution (useful for comparing parameters)
standardizedSolution(fit_linear) %>%
  filter(op %in% c("~1", "~~")) %>%
  select(lhs, op, rhs, est.std, se, pvalue)
```

**Key standardized metrics**:

| Metric | How to compute | Interpretation |
|--------|---------------|----------------|
| **Standardized slope mean** | slope_mean / slope_SD | Effect size for average change (like Cohen's d) |
| **I-S correlation** | cov / √(var_i × var_s) | Standardized relationship (-1 to +1) |
| **R²** | `inspect(fit, "r2")` | Variance explained at each wave |

For this example:
- Standardized slope ≈ 2.04 / 0.98 ≈ 2.08 (very large effect—average growth is ~2 SDs of individual differences in growth)
- I-S correlation ≈ -0.19 (small negative relationship)

#### Written summary

> We estimated a linear latent growth model for 400 participants across 5 waves. The model fit well (χ²(10) = 12.35, p = .26; CFI = 0.998; RMSEA = 0.024; SRMR = 0.025).
>
> On average, participants started at 49.82 (SE = 0.51) and increased by 2.04 units per wave (SE = 0.06), both p < .001. Significant individual differences emerged in starting levels (variance = 98.34, SD ≈ 10) and growth rates (variance = 0.97, SD ≈ 1). The negative intercept-slope covariance (-1.89, p = .002, r ≈ -0.19) indicates that participants who started higher grew slightly slower.

*For common mistakes when interpreting these parameters, see [FAQ & Common Pitfalls](#faq--common-pitfalls).*

---

### Full Script

Here's everything in one self-contained block:

```r
# ============================================
# LGCM Complete Worked Example
# ============================================

# Setup
library(tidyverse)
library(lavaan)
library(MASS)
set.seed(2024)

# Simulate data (N=400, 5 waves)
n <- 400
psi <- matrix(c(100, -2, -2, 1), nrow = 2)
factors <- mvrnorm(n, mu = c(50, 2), Sigma = psi)

data_wide <- tibble(id = 1:n) %>%
  mutate(
    int = factors[,1], slp = factors[,2],
    y1 = int + slp*0 + rnorm(n, 0, 5),
    y2 = int + slp*1 + rnorm(n, 0, 5),
    y3 = int + slp*2 + rnorm(n, 0, 5),
    y4 = int + slp*3 + rnorm(n, 0, 5),
    y5 = int + slp*4 + rnorm(n, 0, 5)
  ) %>% select(id, y1:y5)

# Visualize
data_long <- data_wide %>%
  pivot_longer(y1:y5, names_to = "wave", values_to = "y") %>%
  mutate(time = as.numeric(gsub("y", "", wave)) - 1)

ggplot(data_long, aes(x = time, y = y, group = id)) +
  geom_line(alpha = 0.15) +
  theme_minimal() +
  labs(title = "Individual Trajectories")

# Fit models
model_int <- 'intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5'
model_lin <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit_int <- growth(model_int, data = data_wide)
fit_lin <- growth(model_lin, data = data_wide)

# Compare models
anova(fit_int, fit_lin)

# Final results
summary(fit_lin, fit.measures = TRUE, standardized = TRUE)
fitmeasures(fit_lin, c("chisq","df","pvalue","cfi","rmsea","srmr"))

# Save results for later use
saveRDS(fit_lin, "lgcm_linear_fit.rds")
# To reload: fit_lin <- readRDS("lgcm_linear_fit.rds")
```

---

### Using This Template With Your Own Data

To adapt this workflow to your own dataset, you mainly need to:

1. Read your data into R
2. Put it in wide format (one row per person, one column per time point)
3. Update the variable names in the lavaan syntax

```r
# 1. Read your own data (example)
# Suppose your outcome is 'depress' measured at 5 waves
# as depress_t1, depress_t2, ..., depress_t5
library(readr)
library(dplyr)

data_wide <- read_csv("my_longitudinal_data.csv") %>%
  select(id, depress_t1, depress_t2, depress_t3, depress_t4, depress_t5)

# 2. Specify the LGCM with your variable names
model <- '
  intercept =~ 1*depress_t1 + 1*depress_t2 + 1*depress_t3 + 1*depress_t4 + 1*depress_t5
  slope     =~ 0*depress_t1 + 1*depress_t2 + 2*depress_t3 + 3*depress_t4 + 4*depress_t5
'

# 3. Fit the model
fit <- growth(model, data = data_wide, missing = "fiml")
summary(fit, fit.measures = TRUE, standardized = TRUE)
```

**Adjusting time coding for your design:**

If your waves are not equally spaced, adjust the slope loadings to reflect actual time. For example, if you measured at baseline, 1 month, 3 months, 6 months, and 12 months:

```r
model <- '
  intercept =~ 1*y_baseline + 1*y_1mo + 1*y_3mo + 1*y_6mo + 1*y_12mo
  slope     =~ 0*y_baseline + 1*y_1mo + 3*y_3mo + 6*y_6mo + 12*y_12mo
'
# Now slope = change per month
```

Once this basic model works, you can layer on model comparisons, predictors, and diagnostics exactly as shown in the simulated example above.

---

## Reference & Resources

This section contains lookup materials—consult as needed rather than reading straight through.

---

### Mathematical Notes

This section provides optional formal notation for readers who want it. The main tutorial is fully self-contained without this material.

#### The LGCM Equation

For person *i* at time *t*, the observed score is:

```
yᵢₜ = ηᵢ₁ + ηᵢ₂λₜ + εᵢₜ
```

Where:
- `yᵢₜ` = observed score for person *i* at time *t*
- `ηᵢ₁` = person *i*'s latent intercept
- `ηᵢ₂` = person *i*'s latent slope
- `λₜ` = factor loading for time *t* (fixed, not estimated)
- `εᵢₜ` = residual for person *i* at time *t*

#### Distribution of Latent Factors

The intercept and slope are assumed bivariate normal:

```
[ηᵢ₁]     [α₁]   [ψ₁₁  ψ₁₂]
[ηᵢ₂] ~ N([α₂], [ψ₂₁  ψ₂₂])
```

Where:
- `α₁` = intercept mean
- `α₂` = slope mean
- `ψ₁₁` = intercept variance
- `ψ₂₂` = slope variance
- `ψ₁₂ = ψ₂₁` = intercept-slope covariance

#### Residual Distribution

```
εᵢₜ ~ N(0, θₜ)
```

Residuals are typically assumed independent across time (no autocorrelation) and independent of the latent factors.

#### Model-Implied Covariance Matrix

The model implies a specific covariance structure among observed variables. For 5 waves with loadings λ = [0, 1, 2, 3, 4]:

```
Σ = ΛΨΛ' + Θ
```

Where:
- `Λ` = factor loading matrix (5 × 2)
- `Ψ` = latent factor covariance matrix (2 × 2)
- `Θ` = residual covariance matrix (diagonal if residuals uncorrelated)

#### Identification

This section describes identification for a **linear LGCM with two latent factors (intercept and slope)** and **freely estimated residual variances** at each wave. This is the default parameterization in lavaan.

**Model assumptions:**
- Intercept loadings: all fixed to 1
- Slope loadings: fixed to time codes (e.g., 0, 1, 2, 3, 4)
- Residual variances: freely estimated at each wave (heterogeneous)

A linear LGCM with *T* time points has:
- **Data points**: *T* observed means + *T(T+1)/2* unique covariances/variances = *T* + *T(T+1)/2*
- **Parameters**: 2 latent means + 3 latent (co)variances [Var(I), Var(S), Cov(I,S)] + *T* residual variances = 5 + *T*

**Degrees of freedom**: [*T* + *T(T+1)/2*] − [5 + *T*] = *T(T+1)/2* − 5

| Waves (T) | Data Points | Parameters | df | Model Status |
|-----------|-------------|------------|-----|--------------|
| 3 | 3 + 6 = 9 | 5 + 3 = 8 | 1 | Over-identified (minimal fit test) |
| 4 | 4 + 10 = 14 | 5 + 4 = 9 | 5 | Over-identified (good fit test) |
| 5 | 5 + 15 = 20 | 5 + 5 = 10 | 10 | Over-identified (robust fit test) |
| 6 | 6 + 21 = 27 | 5 + 6 = 11 | 16 | Over-identified |

**Note on equal residual variance models:** If you constrain residual variances to be equal across waves (a common simplification), you estimate only 1 residual variance parameter instead of T. This changes the parameter count to 6 regardless of T, increasing degrees of freedom:
- T = 3 with equal residuals: df = 9 − 6 = 3
- T = 4 with equal residuals: df = 14 − 6 = 8
- T = 5 with equal residuals: df = 20 − 6 = 14

---

### FAQ & Common Pitfalls

#### Pitfall 1: Misinterpreting the Intercept After Recentering

**The mistake**: Changing time coding and forgetting that the intercept now refers to a different time point.

**Example**: Study A codes time as 0,1,2,3,4; Study B codes time as -2,-1,0,1,2. Their intercept means differ, but you conclude the samples started at different levels.

**Reality**: The intercepts refer to different time points (Wave 1 vs. Wave 3). They're not comparable without adjustment.

**Fix**: Always report what the intercept represents. Ensure consistent centering when comparing across studies.

---

#### Pitfall 2: Ignoring Slope Variance

**The mistake**: Reporting only the slope mean and concluding "people improved."

**Example**: Slope mean = 2.0 (p < .001). Conclusion: "Treatment led to improvement."

**Problem**: If slope variance = 4 (SD = 2), many individuals have slopes near 0 or negative. "On average" doesn't mean everyone.

**Fix**: Always report slope variance. Estimate proportion with positive slopes:

```r
# If slope mean = 2, slope SD = 2
# P(slope > 0) ≈ P(Z > -1) ≈ 0.84
# About 16% may not be improving
```

---

#### Pitfall 3: Confusing Factor Means with Individual Predictions

**The mistake**: Treating the intercept mean as every person's starting point.

**Example**: "Participants started at 50 and increased to 58."

**Reality**: 50 and 58 are *averages*. Individual trajectories vary widely.

**Fix**: Report means as averages. Report variances to convey spread. Visualize individual trajectories.

---

#### Pitfall 4: Assuming Good Fit = Correct Model

**The mistake**: CFI = 0.98, RMSEA = 0.03, so the model is "correct."

**Problem**: Good fit means the model *adequately describes* the covariance structure. It doesn't mean growth is truly linear, that you've found the true process, or that there are no confounds.

**Fix**: Good fit is necessary but not sufficient. Consider alternatives, examine residuals, interpret within theory.

---

#### Pitfall 5: Ignoring Residual Autocorrelation

**The mistake**: Assuming residuals are independent across time without checking.

**Signs of trouble**:
- Large modification indices for residual covariances
- Poor fit despite sensible specification
- Substantive reason to expect carryover

**Fix**: Check modification indices. Consider autoregressive structures or latent change score models if needed.

---

#### Pitfall 6: Overfitting with Too Few Time Points

**The mistake**: Fitting a quadratic model with 4 time points (0 df) and concluding "quadratic growth."

**Problem**: Just-identified models fit perfectly by construction—there's no test of fit.

**Fix**: Ensure positive degrees of freedom. With 4 waves, linear is testable; quadratic is just-identified.

---

#### Pitfall 7: Misinterpreting the Intercept-Slope Correlation {#pitfall-7}

**The mistake**: Negative correlation means high scorers "decline."

**Example**: r(intercept, slope) = -0.30. Conclusion: "High starters got worse."

**Reality**: Negative correlation means high starters grow *slower*, not that they decline. If slope mean = 2 and SD = 1, even someone 1 SD below average has slope ≈ 1.7—still positive.

**Fix**: Combine correlation with means and variances to understand the full picture.

---

#### Pitfall 8: Not Accounting for Missing Data Mechanism

**The mistake**: Using FIML and assuming everything is fine.

**Problem**: FIML assumes MAR. If dropout depends on unobserved values (MNAR), estimates may be biased.

**Fix**: Examine who drops out and why. Include auxiliary variables. Conduct sensitivity analyses. Acknowledge assumptions.

---

#### Pitfall 9: Over-interpreting Non-Significant Variance

**The mistake**: Slope variance p = .08, so "everyone changes at the same rate."

**Problem**: You may be underpowered. p = .08 isn't strong evidence of zero.

**Fix**: Report the estimate and confidence interval, not just p-value.

---

#### Pitfall 10: Forgetting to Look at the Data

**The mistake**: Jumping straight to modeling without visualization.

**You might miss**: Outliers, nonlinearity, subgroups, data errors.

**Fix**: Always plot spaghetti plots first.

---

#### Pre-Flight Checklist

Before finalizing results:

- [ ] Does my intercept refer to the time point I think it does?
- [ ] Have I reported slope variance, not just the mean?
- [ ] Am I describing averages as averages?
- [ ] Have I checked for residual autocorrelation?
- [ ] Does my model have degrees of freedom for a fit test?
- [ ] Is my interpretation of the intercept-slope correlation correct?
- [ ] Have I considered the missing data mechanism?
- [ ] Did I actually look at the data?

---

### Cheat Sheet

#### Model Diagram

```
              ┌─────────────┐          ┌─────────────┐
              │  Intercept  │          │    Slope    │
              │     (I)     │◄────────►│     (S)     │
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

Intercept loadings: all = 1
Slope loadings: 0, 1, 2, 3, 4 (encode time)
```

---

#### lavaan Syntax Template

```r
library(lavaan)

model <- '
  # Intercept factor (loadings = 1)
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5

  # Slope factor (loadings = time)
  slope =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
'

fit <- growth(model, data = data_wide)
summary(fit, fit.measures = TRUE, standardized = TRUE)
```

---

#### Parameter Quick Reference

| Parameter | Symbol | Meaning |
|-----------|--------|----------|
| Intercept mean | μᵢ | Average starting level (at Time=0) |
| Slope mean | μₛ | Average change per time unit |
| Intercept variance | ψᵢᵢ | Individual differences in starting level |
| Slope variance | ψₛₛ | Individual differences in change rate |
| I-S covariance | ψᵢₛ | Relationship between start and change |
| Residual variance | θ | Unexplained variance at each wave |

---

#### Fit Index Guidelines

| Index | Good | Acceptable |
|-------|------|------------|
| χ² p-value | > .05 | > .01 |
| CFI | ≥ .95 | ≥ .90 |
| RMSEA | ≤ .06 | ≤ .08 |
| SRMR | ≤ .08 | ≤ .10 |

*Report multiple indices; don't rely on one.*

---

#### Time Coding Options

| Centering | Loadings | Intercept Meaning |
|-----------|----------|-------------------|
| Wave 1 (standard) | 0, 1, 2, 3, 4 | Score at Wave 1 |
| Midpoint (Wave 3) | -2, -1, 0, 1, 2 | Score at Wave 3 |
| Final wave | -4, -3, -2, -1, 0 | Score at Wave 5 |
| Actual months | 0, 3, 6, 12, 24 | Score at baseline |

---

#### Minimum Requirements

| Aspect | Requirement |
|--------|-------------|
| Time points | 3+ (4+ recommended for robust fit testing) |
| Sample size | 100+ (simple); 200+ (complex) |
| Data format | Wide (one row per person) |
| Estimation | ML (default); MLR if non-normal |

---

#### Common Syntax Variants

**Equal residual variances:**
```r
y1 ~~ rv*y1
y2 ~~ rv*y2
y3 ~~ rv*y3
y4 ~~ rv*y4
y5 ~~ rv*y5
```

**Fixed slope (no variance):**
```r
slope ~~ 0*slope
intercept ~~ 0*slope
```

**Predictor of growth:**
```r
slope ~ predictor
intercept ~ predictor
```

---

#### Model Comparison

```r
# Nested models (likelihood ratio test)
anova(fit_constrained, fit_full)

# Any models (information criteria)
AIC(fit1); AIC(fit2)  # Lower = better
BIC(fit1); BIC(fit2)  # Lower = better
```

---

#### Quick Diagnostics

```r
# Fit indices
fitmeasures(fit, c("chisq","df","pvalue","cfi","rmsea","srmr"))

# Parameter estimates
parameterEstimates(fit)

# Modification indices
modindices(fit, sort = TRUE, minimum.value = 10)
```

---

#### Checklist: Before Running

- [ ] Data in wide format
- [ ] Variable names match syntax
- [ ] Time coding reflects actual spacing
- [ ] Missingness examined
- [ ] Trajectories visualized

#### Checklist: Before Reporting

- [ ] Model fit reported (χ², CFI, RMSEA, SRMR)
- [ ] All parameters reported with SEs
- [ ] Variances interpreted (not just means)
- [ ] Time coding stated
- [ ] Limitations acknowledged

---

### Advanced Extensions

Linear LGCM is the foundation. These extensions address more complex questions.

---

#### Quadratic Growth

**What**: Adds acceleration/deceleration to the trajectory.

**When**: Change is nonlinear—improvement slows, or decline accelerates.

**Loadings**:
- Intercept: 1, 1, 1, 1, 1
- Linear: 0, 1, 2, 3, 4
- Quadratic: 0, 1, 4, 9, 16

**Requirements**: 4+ waves (5+ for testable fit)

```r
model_quad <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5
  quad      =~ 0*y1 + 1*y2 + 4*y3 + 9*y4 + 16*y5
'
```

---

#### Latent Basis (Freed Loading) Models

**What**: Let data reveal the shape of change instead of imposing linearity.

**When**: Functional form unknown.

**Approach**: Fix first and last loadings; estimate intermediate.

```r
model_basis <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + y2 + y3 + y4 + 1*y5
'
```

---

#### Piecewise Growth

**What**: Different slopes for different phases.

**When**: Rapid treatment phase, slower follow-up.

```r
model_piecewise <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope1    =~ 0*y1 + 1*y2 + 2*y3 + 2*y4 + 2*y5
  slope2    =~ 0*y1 + 0*y2 + 0*y3 + 1*y4 + 2*y5
'
```

---

#### Time-Varying Covariates

**What**: Predictors measured at each wave affect that wave's outcome.

**When**: Control for state variables or test concurrent effects.

**Approach**: Add regressions from each covariate to each outcome.

---

#### Time-Invariant Predictors

**What**: Baseline characteristics predict trajectory parameters.

**When**: Does treatment predict faster improvement? Does gender predict starting level?

**Example: Treatment Predicting Growth**

```r
# Assume 'treatment' is coded 0 = control, 1 = treatment
# and exists in your data_wide dataframe

model_predictor <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  # Predict growth factors from treatment
  intercept ~ treatment
  slope     ~ treatment
'

fit_pred <- growth(model_predictor, data = data_wide, missing = "fiml")
summary(fit_pred, fit.measures = TRUE, standardized = TRUE)
```

**Interpreting the coefficients:**

- The coefficient on `treatment` in the **intercept equation** tells you how much higher (or lower) treated participants start at Time 0 compared to controls.
- The coefficient on `treatment` in the **slope equation** tells you how much faster (or slower) treated participants change per time unit.

**Example interpretation:**

| Parameter | Estimate | Interpretation |
|-----------|----------|----------------|
| intercept ~ treatment = -2.5 | Treated group starts 2.5 points lower than control at baseline |
| slope ~ treatment = 0.8 | Treated group improves 0.8 points more per wave than control |

If the slope coefficient is positive and significant, treatment accelerates improvement. If negative, treatment slows improvement (or accelerates decline, depending on context).

**Adding multiple predictors:**

```r
model_multi <- '
  intercept =~ 1*y1 + 1*y2 + 1*y3 + 1*y4 + 1*y5
  slope     =~ 0*y1 + 1*y2 + 2*y3 + 3*y4 + 4*y5

  # Multiple predictors
  intercept ~ treatment + age + baseline_severity
  slope     ~ treatment + age + baseline_severity
'
```

---

#### Multigroup LGCM

**What**: Separate models for different groups; compare parameters.

**When**: Test whether growth differs by group (treatment vs. control, male vs. female).

**Approach**: Fit constrained model (equal across groups), then freed model. Compare fit.

---

#### Growth Mixture Models (GMM)

**What**: Identify latent subgroups with different trajectory patterns.

**When**: Not everyone follows the same pattern—some improve, some decline, some stay flat.

**Caution**: Complex, sample-hungry, sensitive to specifications. Use with care.

---

#### Parallel Process Models

**What**: Model two variables' growth simultaneously; relate their trajectories.

**When**: Does change in anxiety correlate with change in depression?

---

#### Latent Change Score Models

**What**: Focus on change between adjacent time points.

**When**: Dynamic questions—does level predict subsequent change?

**Feature**: Can test coupling between variables.

---

### Recommended Resources

#### Books

| Book | Focus |
|------|-------|
| Grimm, Ram, & Estabrook (2017). *Growth Modeling*. Guilford. | Comprehensive coverage of SEM and MLM approaches |
| Little (2013). *Longitudinal Structural Equation Modeling*. Guilford. | SEM perspective, measurement issues |
| Bollen & Curran (2006). *Latent Curve Models*. Wiley. | Classic SEM treatment |
| Singer & Willett (2003). *Applied Longitudinal Data Analysis*. Oxford. | MLM perspective, excellent pedagogy |
| McArdle & Nesselroade (2014). *Longitudinal Data Analysis Using SEM*. APA. | Lifespan developmental focus |

#### Key Articles

| Article | Contribution |
|---------|---------------|
| Bollen, K. A., & Curran, P. J. (2006). *Latent curve models: A structural equation perspective*. Wiley. | Foundational SEM framework for growth modeling |
| Curran, P. J., Obeidat, K., & Losardo, D. (2010). Twelve frequently asked questions about growth curve modeling. *Journal of Cognition and Development, 11*(2), 121–136. | Accessible FAQ comparing LGCM and MLM approaches |
| Preacher, K. J., Wichman, A. L., MacCallum, R. C., & Briggs, N. E. (2008). *Latent growth curve modeling*. Sage. | Practical guidance on centering and time coding |
| McNeish, D., & Matta, T. (2018). Differentiating between mixed-effects and latent-curve approaches to growth modeling. *Behavior Research Methods, 50*(4), 1398–1414. | Clarifies when approaches differ in practice |

#### Online Tutorials

- **lavaan**: https://lavaan.ugent.be/tutorial/
- **Mplus User's Guide**: Chapter on growth models
- **Curran-Bauer Analytics**: YouTube channel, growth modeling workshops
- **QuantDev tutorials**: https://quantdev.ssri.psu.edu/

---

### Software Options

#### R Packages

| Package | Strengths | Notes |
|---------|-----------|-------|
| **lavaan** | Free, flexible, active development | Primary recommendation for this tutorial |
| **OpenMx** | Very flexible, matrix specification | Steeper learning curve |
| **semTools** | Extensions for lavaan (measurement invariance, etc.) | Companion to lavaan |
| **nlme** / **lme4** | MLM approach to growth | Equivalent results for basic models |
| **lcmm** | Latent class / mixture growth models | Specialized |

#### Commercial Software

| Software | Strengths | Notes |
|----------|-----------|-------|
| **Mplus** | Gold standard, extensive documentation, mixture models | Licensed, not free |
| **LISREL** | Historic, powerful | Steeper learning curve |
| **EQS** | User-friendly interface | Less common now |
| **AMOS** | GUI-based, integrates with SPSS | Limited flexibility |
| **Stata (sem)** | Good documentation, integrated with Stata workflow | Licensed |

#### Python

| Package | Notes |
|---------|-------|
| **semopy** | SEM in Python, active development |
| **statsmodels** | Mixed models (MLM approach) |

#### Choosing Software

For learning and most research purposes, **lavaan in R** is the recommended choice:
- Free and open source
- Syntax similar to Mplus
- Active community support
- Extensive online resources
- Handles most standard models

Use **Mplus** when you need:
- Complex mixture models
- Categorical latent variables
- Intensive Monte Carlo simulations
- Maximum flexibility

Use **MLM packages** (lme4, nlme) when:
- You're already in that ecosystem
- You want random effects inference
- You have intensive longitudinal data

---

*End of Reference section.*

---

*End of Document*
