---
title: "Generalized Linear Mixed Models"
slug: "glmm"
description: "Extend mixed models to non-continuous outcomes — binary, count, and ordinal — while preserving individual-level trajectories."
category: "mixed-models"
tags: ["GLMM", "guide", "glmmTMB", "lme4"]
guide_type: "overview"
---

## When Outcomes Aren't Continuous

Linear mixed models assume a continuous, normally distributed outcome. But many longitudinal questions involve outcomes that aren't continuous:

- Did the participant use a substance? (**binary**: yes/no)
- How many days did they use in the past month? (**count**: 0, 1, 2, …)
- What was their symptom severity rating? (**ordinal**: mild, moderate, severe)

Fitting a standard LMM to these outcomes creates real problems:

| Outcome Type | What Goes Wrong with LMM |
|-------------|--------------------------|
| **Binary** | Predicted probabilities outside [0, 1]; wrong variance structure |
| **Count** | Ignores floor at zero; assumes symmetric errors around the mean |
| **Ordinal** | Treats category distances as equal; misrepresents the outcome scale |

These aren't minor technical issues — they distort your estimates, standard errors, and conclusions. A model that predicts a -15% probability of substance use isn't just aesthetically wrong; it's structurally misspecified.

**Generalized Linear Mixed Models (GLMM)** solve this by combining two ideas:

1. **Generalized linear models (GLM)**: Map the outcome to an appropriate scale through a *link function*, and model variance as a function of the mean
2. **Mixed models**: Include random effects to capture individual-level variation over time

The result: you get individual trajectories for non-continuous outcomes, with proper handling of the outcome's distributional properties.

---

> [!tip] **Before You Continue**
>
> If you're comfortable with linear mixed models (LMM), you already understand the core idea — fixed effects for population averages, random effects for individual deviations. GLMM adds one layer: a link function that transforms the outcome scale so that the linear predictor maps correctly to the outcome's natural range.
>
> If LMM is unfamiliar, start with the [LMM Overview](/guides/lmm) first.

---

## What GLMM Provides

GLMM extends the LMM framework to handle the distributional properties of non-continuous outcomes:

### Appropriate Outcome Modeling

Each outcome type gets a distribution that matches its properties — Bernoulli for binary, Poisson or negative binomial for counts, cumulative logit for ordinal. The model respects natural boundaries (probabilities between 0 and 1, counts ≥ 0) and correctly specifies how variance relates to the mean.

### Individual Trajectories on the Latent Scale

Random effects operate on the *link scale* (e.g., log-odds for binary, log for counts), not the observed scale. Each person gets their own intercept and slope on this transformed scale, which then maps back to probabilities or expected counts through the inverse link function.

### Conditional Interpretation

GLMM estimates are **conditional** — they describe the effect of a predictor *for a specific individual*, holding their random effects constant. This is the natural quantity when you care about within-person change: "How does *this person's* probability of substance use change over time?"

This contrasts with **marginal** (population-averaged) approaches like GEE, which answer: "How does the *average probability* change over time?" The distinction matters because averaging nonlinear functions gives different results than applying the function to averages.

### Flexible Predictor Framework

Like LMM, you can include:

- **Time-invariant predictors**: Do males and females differ in their trajectories?
- **Time-varying predictors**: Does current stress predict this wave's outcome?
- **Cross-level interactions**: Does the effect of stress on the outcome differ across people?

---

## When GLMM is Appropriate

GLMM works well when you have:

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ waves (fewer limits random effects structure) |
| **Non-continuous outcome** | Binary, count, ordinal, or proportions |
| **Interest in individual differences** | Not just average trends — person-level trajectories |
| **Adequate sample** | 50+ clusters minimum; 100+ recommended for complex models |
| **Conditional effects needed** | You want subject-specific (not population-averaged) estimates |

If you only need **population-averaged** effects and don't care about individual trajectories, consider [GEE](/guides/gee) instead — it's simpler, makes fewer assumptions, and directly estimates marginal effects.

---

## Key Components

### The Link Function

The link function is what makes GLMM "generalized." It connects the linear predictor (the part that looks like a regression equation) to the expected value of the outcome:

```
g(μᵢₜ) = Xᵢₜβ + Zᵢₜbᵢ
```

Where g(·) is the link function, transforming the outcome's natural scale to one where linear modeling makes sense.

| Outcome | Distribution | Link | g(μ) | Inverse: μ = |
|---------|-------------|------|------|--------------|
| Binary | Bernoulli | Logit | log(μ/(1-μ)) | 1/(1+e⁻ˣ) |
| Count | Poisson | Log | log(μ) | eˣ |
| Count (overdispersed) | Negative Binomial | Log | log(μ) | eˣ |
| Ordinal | Multinomial | Cumulative logit | log(P(Y≤k)/(1-P(Y≤k))) | — |

**Why not just use a linear link?** Consider a binary outcome. A linear model predicts:

```
P(y = 1) = β₀ + β₁·time
```

If β₀ = 0.8 and β₁ = 0.1, by Time 3 you predict P = 1.1. The logit link prevents this by mapping probabilities to the entire real line:

```
log(P/(1-P)) = β₀ + β₁·time
```

Now the linear predictor can take any value, and the inverse logit maps it back to [0, 1].

### Random Effects Structure

Random effects in GLMM work the same way as in LMM — they capture individual deviations from population averages. The key difference is that these deviations operate on the **link scale**:

| Component | What It Captures |
|-----------|-----------------|
| **Random intercept** | Person-specific baseline level (on link scale) |
| **Random slope** | Person-specific rate of change (on link scale) |
| **Random effect variance** | How much individuals differ |
| **Random effect covariance** | Relationship between starting level and change |

**Critical insight**: A random intercept in a logistic GLMM captures person-level variation in *log-odds*, not in probability. Two people with random intercepts of +1 and -1 differ by 2 units on the log-odds scale, but the probability difference depends on where they are on the curve — the logistic function is nonlinear, so the same log-odds difference produces different probability differences at different baseline levels.

### Variance Structure

Unlike LMM where residual variance is a free parameter, GLMM outcome distributions have variance tied to the mean:

| Distribution | Variance |
|-------------|----------|
| Bernoulli | μ(1-μ) — largest at p = 0.5 |
| Poisson | μ — variance equals the mean |
| Negative Binomial | μ + μ²/θ — allows overdispersion |

This mean-variance relationship is built into the model. You don't estimate a separate residual variance for binary or Poisson outcomes — it's determined by the distribution. When the data show *more* variance than the distribution implies (overdispersion), you need a distribution that accommodates it, such as the negative binomial for counts.

---

<details>
<summary><strong>Mathematical Foundations</strong> (optional formal notation)</summary>

### The GLMM Equation

For person *i* at time *t*:

**Conditional model (given random effects):**

```
g(μᵢₜ) = ηᵢₜ = x'ᵢₜβ + z'ᵢₜbᵢ
```

Where:
- **g(·)** = link function
- **μᵢₜ** = E(yᵢₜ | bᵢ) = conditional expected value
- **x'ᵢₜβ** = fixed effects (population-level)
- **z'ᵢₜbᵢ** = random effects (person-level deviations)

**Distribution of random effects:**

```
bᵢ ~ N(0, G)
```

Where G is the random effects covariance matrix:

```
     [σ²₀₀  σ₀₁]
G =  [σ₀₁   σ²₁₁]
```

For a random intercept and slope model:
- σ²₀₀ = variance of random intercepts
- σ²₁₁ = variance of random slopes
- σ₀₁ = covariance between intercepts and slopes

### Conditional Distribution

```
yᵢₜ | bᵢ ~ f(μᵢₜ, φ)
```

Where f is the specified distribution (Bernoulli, Poisson, Negative Binomial) and φ is the dispersion parameter (if applicable).

### Marginal Likelihood

The marginal likelihood integrates over the random effects:

```
L(β, G, φ) = ∏ᵢ ∫ [∏ₜ f(yᵢₜ | bᵢ, β, φ)] · g(bᵢ | G) dbᵢ
```

This integral generally has no closed-form solution, requiring numerical approximation (Laplace, adaptive Gauss-Hermite quadrature, or MCMC).

</details>

---

## Model Specification in R

Two primary packages handle GLMM in R, each with different strengths:

### glmmTMB

The more flexible option, supporting negative binomial, zero-inflated, and other distributions:

```r
library(glmmTMB)

# Binary outcome — random intercept
fit <- glmmTMB(outcome ~ time + predictor + (1 | id),
               data = df, family = binomial)

# Count outcome — negative binomial, random intercept + slope
fit <- glmmTMB(outcome ~ time + predictor + (time | id),
               data = df, family = nbinom2)

# Zero-inflated count
fit <- glmmTMB(outcome ~ time + (time | id),
               zi = ~ time,
               data = df, family = nbinom2)
```

### lme4::glmer

The more established option, well-supported but limited to exponential family distributions:

```r
library(lme4)

# Binary outcome — random intercept
fit <- glmer(outcome ~ time + predictor + (1 | id),
             data = df, family = binomial)

# Poisson count — random intercept + slope
fit <- glmer(outcome ~ time + predictor + (time | id),
             data = df, family = poisson)
```

### Which to Choose?

| Feature | glmmTMB | lme4::glmer |
|---------|---------|-------------|
| Negative binomial | Yes (nbinom1, nbinom2) | Via `MASS::glmer.nb` |
| Zero-inflation | Yes (built-in `zi` formula) | No |
| Estimation | TMB (Laplace) | Laplace or AGQ |
| Adaptive GH quadrature | No | Yes (`nAGQ > 1`) |
| Speed | Generally faster | Varies |
| Ecosystem | Growing | Mature (merTools, effects, etc.) |

**Recommendation**: Use `glmmTMB` for count models (better NB and zero-inflation support) and `lme4::glmer` for binary models when you want AGQ estimation or the mature `lme4` tooling ecosystem.

---

## Interpretation

GLMM parameters live on the link scale. Interpretation requires transforming back to the natural scale — and understanding that these are **conditional** (subject-specific) effects.

### Binary Outcomes (Logistic GLMM)

On the link scale, coefficients are **log-odds**. Exponentiate for odds ratios:

| Parameter | Link Scale | Natural Scale |
|-----------|-----------|---------------|
| Intercept | Log-odds at time = 0 | Probability = logit⁻¹(β₀) |
| Time coefficient | Change in log-odds per unit time | Odds ratio = exp(β₁) |
| Predictor coefficient | Difference in log-odds | Odds ratio = exp(β) |

**Example**: β₁ = -0.3 for time → OR = exp(-0.3) = 0.74. For a given individual, the odds of the outcome decrease by 26% per time unit.

**Conditional vs. marginal**: The OR from a GLMM is *conditional* — it applies to a specific person. The marginal OR (from GEE) is typically closer to 1.0 because averaging over heterogeneous individuals attenuates the effect. The more random effect variance, the larger this gap.

### Count Outcomes (Log-link GLMM)

Coefficients are on the **log scale**. Exponentiate for incidence rate ratios (IRR):

| Parameter | Link Scale | Natural Scale |
|-----------|-----------|---------------|
| Intercept | Log expected count at time = 0 | Expected count = exp(β₀) |
| Time coefficient | Change in log count per unit time | IRR = exp(β₁) |
| Predictor coefficient | Difference in log count | IRR = exp(β) |

**Example**: β₁ = 0.15 for time → IRR = exp(0.15) = 1.16. For a given individual, the expected count increases by 16% per time unit.

### Converting to Probability Scale (Binary)

For presentation, convert log-odds to predicted probabilities:

```r
# Predicted probabilities at the population level (random effects = 0)
newdata <- data.frame(time = 0:4, predictor = mean(df$predictor))
newdata$prob <- predict(fit, newdata = newdata, type = "response",
                        re.form = NA)
```

Setting `re.form = NA` gives predictions at the "average" individual (random effects = 0). These are conditional predictions at the random effect mean, *not* marginal (population-averaged) predictions — the distinction matters when random effect variance is large.

---

## Conditional vs. Marginal: Why It Matters

This distinction is fundamental and often misunderstood. In a linear model, conditional and marginal effects are identical. In a GLMM, they aren't — because the link function is nonlinear.

**Conditional (GLMM)**: "For a specific person, how does a one-unit increase in X change their log-odds?"

**Marginal (GEE)**: "Across the population, how does a one-unit increase in X change the average probability?"

```
Conditional effects from GLMM:
  Person A (random intercept = -2): P changes from 0.12 → 0.18
  Person B (random intercept =  0): P changes from 0.50 → 0.62
  Person C (random intercept = +2): P changes from 0.88 → 0.93

Same log-odds change, different probability changes.

Population average (marginal): P changes from 0.50 → 0.58
```

Neither is "right" — they answer different questions. Use GLMM when individual-level effects matter. Use GEE when you want population-level summaries. See the [GEE Overview](/guides/gee) for the marginal approach.

---

## Estimation Methods

Unlike LMM, where maximum likelihood has a closed-form solution for the random effects integral, GLMM requires numerical approximation:

| Method | How It Works | Trade-off |
|--------|-------------|-----------|
| **Laplace approximation** | Second-order Taylor expansion of the integrand | Fast; default in glmmTMB and glmer. Can be inaccurate with few observations per cluster or binary outcomes with small cluster sizes |
| **Adaptive Gauss-Hermite (AGQ)** | Numerical integration with quadrature points | More accurate for binary outcomes; slow with multiple random effects. Only in `glmer(nAGQ = k)` |
| **MCMC (Bayesian)** | Full posterior sampling | Most flexible; handles complex models well. Requires prior specification and convergence diagnostics |

**Practical guidance**: Start with Laplace (the default). For binary outcomes with small cluster sizes (< 5 observations per person), compare results with AGQ (`nAGQ = 7` or higher). If results differ meaningfully, trust AGQ. For complex models where AGQ is infeasible (multiple random effects), Laplace is your best frequentist option.

---

## Practical Considerations

### Overdispersion

Overdispersion means the data show more variance than the assumed distribution predicts. It's common with count data and doesn't apply to binary outcomes (Bernoulli variance is fully determined by the mean).

**Detection**: Compare the ratio of the Pearson residual deviance to residual degrees of freedom. Values substantially above 1.0 suggest overdispersion.

```r
# Quick overdispersion check
overdisp_ratio <- sum(residuals(fit, type = "pearson")^2) / df.residual(fit)
```

**Solutions**:
- Switch from Poisson to negative binomial (adds a dispersion parameter)
- Add an observation-level random effect (OLRE) to a Poisson model
- Use quasi-Poisson (not available in mixed model packages; use NB instead)

### Zero-Inflation

When count data have more zeros than the count distribution predicts, consider a zero-inflated model. This models two processes: (1) a binary process determining whether the count is structurally zero, and (2) a count process for non-structural zeros and positive counts.

```r
# Zero-inflated negative binomial
fit_zi <- glmmTMB(count ~ time + (time | id),
                  zi = ~ 1,  # constant zero-inflation probability
                  family = nbinom2, data = df)
```

Test whether zero-inflation is needed by comparing AIC/BIC of the standard vs. zero-inflated model.

### Convergence

GLMM convergence issues are more common than in LMM because of the nonlinear likelihood surface. Common strategies:

- **Simplify random effects**: Start with random intercept only; add random slopes if supported by the data
- **Rescale predictors**: Center continuous predictors and ensure time is on a reasonable scale
- **Try different optimizers**: `glmmTMB` and `lme4` support multiple optimizers
- **Check for separation**: In binary models, if a predictor perfectly predicts the outcome in some subgroup, the model can't converge

### Sample Size Considerations

GLMM generally needs more data than LMM because:
- Information per observation is lower (binary outcomes carry less information than continuous)
- Estimation involves numerical integration
- Random slope models require sufficient within-person variation

**Rules of thumb** (binary outcomes):
- Random intercept only: 50+ clusters, 5+ observations per cluster
- Random intercept + slope: 100+ clusters, 5+ observations per cluster
- These are minimums — more is always better for stable estimates

---

## Common Pitfalls

**Interpreting coefficients on the wrong scale** — Reporting β = -0.5 as "a decrease of 0.5" without specifying the scale. *Reality:* GLMM coefficients are on the link scale (log-odds, log). Always specify: "The log-odds decreased by 0.5, corresponding to an odds ratio of 0.61."

**Treating conditional estimates as marginal** — Using GLMM odds ratios to make population-level statements like "the intervention reduced prevalence by X%." *Reality:* GLMM gives conditional (subject-specific) effects. Marginal effects are attenuated. Use GEE or marginalize explicitly if you need population-averaged estimates.

**Ignoring overdispersion in count models** — Fitting a Poisson GLMM without checking whether variance exceeds the mean. *Reality:* Overdispersed Poisson models produce standard errors that are too small and p-values that are too optimistic. Always check and use negative binomial if needed.

**Over-specifying random effects** — Adding random slopes for every predictor because "that's what the theory says." *Reality:* Complex random effects structures frequently fail to converge with realistic sample sizes. Start simple. The data will tell you what they can support.

**Confusing Laplace and AGQ accuracy** — Assuming the default Laplace approximation is always sufficient. *Reality:* For binary outcomes with few repeated measures (< 5 per person), Laplace can produce biased estimates. Compare with `nAGQ = 7` in `glmer` to check.

**Ignoring the mean-variance relationship** — Adding an observation-level random effect to a binary model "for overdispersion." *Reality:* Bernoulli variance is fully determined by the mean. An OLRE in a binary GLMM changes the model's meaning (it becomes a beta-binomial-like model), not just its variance.

**Presenting odds ratios without context** — Reporting OR = 2.5 as "2.5 times more likely." *Reality:* OR = 2.5 means 2.5 times the *odds*, not 2.5 times the *probability*. The probability ratio depends on the baseline probability. At P = 0.01, OR = 2.5 roughly doubles the probability; at P = 0.50, it shifts probability to 0.71.

**Using AIC to compare across distribution families** — Comparing AIC between a Poisson model and a negative binomial model fit with different packages or likelihoods. *Reality:* AIC comparisons are only valid when models are fit to the same data using the same likelihood. Comparing a Poisson `glmer` to a NB `glmmTMB` requires care — use the same package for both.

**Fitting ordinal outcomes as continuous** — Treating a 4-point Likert scale as continuous in a standard LMM because "it's close enough." *Reality:* With few categories (< 5), the equal-interval assumption is strong and may distort conclusions. Use a cumulative link mixed model (`ordinal::clmm`) or treat with caution.

**Forgetting to examine predicted probabilities** — Interpreting the model entirely through odds ratios without ever plotting predicted probabilities across time. *Reality:* Odds ratios are constant across the predictor range, but probability changes are not. A plot of predicted probabilities reveals the practical significance of effects and makes results accessible to non-statistical audiences.

---

## Summary

GLMM extends the mixed model framework to handle outcomes that aren't continuous:

- **Link functions** transform the outcome scale so linear modeling applies — logit for binary, log for counts
- **Random effects** operate on the link scale, giving each person their own trajectory in log-odds or log-counts
- **Conditional interpretation** means effects are subject-specific — they describe what happens for a given individual, not the population average
- **Variance is tied to the mean** for most GLMM distributions — overdispersion and zero-inflation require explicit modeling
- **Estimation is approximate** — Laplace is fast but may be inaccurate for small clusters with binary outcomes; AGQ is more precise but slower

The conceptual leap from LMM to GLMM is smaller than it appears. If you understand random intercepts and slopes in a linear model, you understand them in a GLMM — they just live on a transformed scale.

---

## Next Steps

<div style="display: flex; gap: 1rem; flex-wrap: wrap; margin-top: 1rem;">

**[Walkthrough: Worked Example →](/guides/glmm-walkthrough)**
Step-by-step R code to simulate data, fit binary and count GLMMs, and interpret results.

**[Quick Reference →](/guides/glmm-reference)**
Syntax cheat sheets, distribution tables, diagnostics checklist, and troubleshooting.

</div>
