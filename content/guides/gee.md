---
title: "Generalized Estimating Equations"
slug: "gee"
description: "Estimate population-averaged effects for correlated data without modeling individual trajectories."
category: "mixed-models"
tags: ["GEE", "guide", "geepack", "marginal"]
guide_type: "overview"
---

## When You Care About the Population, Not the Person

Mixed models — both LMM and GLMM — estimate **conditional** effects: what happens for a *specific individual*, accounting for their random effect. This is the right question when individual trajectories matter.

But sometimes they don't. Sometimes the question is purely about the population:

- Does this intervention reduce the **average prevalence** of substance use?
- Is the **population rate** of emergency visits declining over time?
- Across all participants, does the **average count** differ by treatment group?

These are **marginal** questions — they ask about averages, not individuals. You don't need to model person-specific trajectories to answer them. You just need to account for the fact that repeated measures on the same person are correlated.

**Generalized Estimating Equations (GEE)** do exactly this. Instead of specifying a full probability model for the data (as GLMM does), GEE takes a semi-parametric approach:

1. Specify how the mean relates to predictors (the regression model)
2. Specify a **working correlation structure** to approximate within-person dependence
3. Use a **sandwich estimator** to produce valid standard errors *even if the working correlation is wrong*

The result: consistent, efficient estimates of population-averaged effects with robust inference — and far fewer assumptions than GLMM.

---

> [!tip] **Before You Continue**
>
> GEE and GLMM answer fundamentally different questions. Neither is "better" — they target different estimands. If you need individual trajectories, random effect variances, or subject-specific predictions, use [GLMM](/guides/glmm). If you need population-averaged effects with minimal distributional assumptions, read on.

---

## What GEE Provides

### Population-Averaged (Marginal) Effects

GEE estimates what happens to the **average person in the population** — not a specific individual. For binary outcomes, the coefficient describes how the population prevalence changes with a predictor. For counts, it describes how the population rate changes.

This distinction matters most for non-linear models. In a logistic GLMM, the odds ratio is conditional — it applies to a specific person holding their random effect constant. In GEE, the odds ratio is marginal — it describes the population-level association. The marginal OR is always closer to 1.0 than the conditional OR because averaging over heterogeneous individuals attenuates the effect.

### Robust Standard Errors

GEE's signature feature is the **sandwich (Huber-White) estimator**. It produces valid standard errors and confidence intervals even when the working correlation structure is misspecified. This is a remarkable property: you can get the correlation structure wrong and still get valid inference for the regression coefficients.

The trade-off: if the working correlation is close to the truth, robust SEs are slightly less efficient than model-based (naive) SEs. If it's far from the truth, robust SEs protect you while naive SEs don't.

### Minimal Distributional Assumptions

GEE doesn't require a full likelihood specification. You need:

1. A correct mean model (how predictors relate to the outcome)
2. A variance function (how variance relates to the mean)
3. A working correlation structure (can be wrong — robust SEs cover you)

You do **not** need:

- A correctly specified random effects distribution
- The true correlation structure
- Normality of anything

This makes GEE particularly attractive when you distrust distributional assumptions or when the random effects distribution is genuinely non-normal.

### Straightforward Interpretation

GEE coefficients have the same interpretation as coefficients from a standard GLM fit to independent data — just with appropriate standard errors for the correlation. No random effects to condition on, no link-scale complications for marginal summaries.

---

## When GEE is Appropriate

| Requirement | Guideline |
|-------------|-----------|
| **Repeated measures** | 3+ waves (GEE works with 2, but limited) |
| **Interest in population averages** | Not individual trajectories |
| **Adequate clusters** | 40+ individuals minimum; 100+ recommended for robust SEs |
| **Outcome type** | Binary, count, continuous, ordinal |
| **Missing data mechanism** | MCAR or approximately MCAR (see below) |

### When to Prefer GEE Over GLMM

| Scenario | GEE | GLMM |
|----------|-----|------|
| Research question is about population averages | ✓ | |
| You distrust random effects distributional assumptions | ✓ | |
| You want model-robust inference | ✓ | |
| You need individual predictions or random effects | | ✓ |
| You need to model individual heterogeneity | | ✓ |
| You have informative dropout (MAR) | | ✓ |
| You want likelihood-based model comparison | | ✓ |

---

## Conceptual Foundations

### The Estimating Equation

GEE is based on solving a set of **estimating equations** — generalizations of the score equations from maximum likelihood. For each person *i* with observations at times *t* = 1, …, nᵢ:

```
U(β) = Σᵢ D'ᵢ Vᵢ⁻¹ (yᵢ - μᵢ) = 0
```

Where:
- **Dᵢ** = ∂μᵢ/∂β — how the mean responds to parameter changes
- **Vᵢ** = A^(1/2) R(α) A^(1/2) — the "working" covariance matrix
- **A** = diagonal matrix of variance functions
- **R(α)** = working correlation matrix
- **yᵢ - μᵢ** = residuals

This looks complex, but the intuition is simple: find β values that make the weighted residuals equal zero on average, where the weights account for the correlation structure.

### Working Correlation Structures

The working correlation matrix R(α) specifies how observations within a person are related. GEE offers several options:

| Structure | Pattern | Assumption | Parameters |
|-----------|---------|------------|------------|
| **Independence** | All off-diagonals = 0 | No within-person correlation | 0 |
| **Exchangeable** | All pairs equally correlated | Compound symmetry | 1 (α) |
| **AR(1)** | Correlation decays with lag | Autoregressive | 1 (α) |
| **Unstructured** | Each pair has its own correlation | No pattern assumed | T(T−1)/2 |

```
Independence:       Exchangeable:       AR(1):              Unstructured:
[1 0 0 0]          [1 α α α]          [1  α  α² α³]       [1   α₁₂ α₁₃ α₁₄]
[0 1 0 0]          [α 1 α α]          [α  1  α  α²]       [α₁₂ 1   α₂₃ α₂₄]
[0 0 1 0]          [α α 1 α]          [α² α  1  α ]       [α₁₃ α₂₃ 1   α₃₄]
[0 0 0 1]          [α α α 1]          [α³ α² α  1 ]       [α₁₄ α₂₄ α₃₄ 1  ]
```

**Choosing a structure:**

- **Independence**: When correlations are weak or you rely entirely on robust SEs. Surprisingly often a reasonable starting point.
- **Exchangeable**: When all time pairs are roughly equally correlated — common in cluster-randomized studies. Default choice for many applications.
- **AR(1)**: When adjacent measurements are more correlated than distant ones — natural for time-series-like data.
- **Unstructured**: When you have few time points (≤ 5) and enough clusters to estimate all pairwise correlations. Most flexible but most parameters.

> [!note] **The working correlation doesn't have to be right**
>
> With robust SEs, the choice of working correlation affects *efficiency* (precision) but not *consistency* (validity). A badly wrong working correlation gives wider confidence intervals than necessary, but they still have correct coverage. A close-to-correct working correlation gives tighter intervals.

### The Sandwich Estimator

GEE's robustness comes from how it estimates the covariance of β̂:

```
Var(β̂) = B⁻¹ M B⁻¹
```

Where:
- **B** = model-based (naive) information matrix — what you'd get if the working correlation were correct
- **M** = "meat" — empirical correction based on observed residuals

The "bread-meat-bread" sandwich structure means:
- If the working correlation is correct: B⁻¹ M B⁻¹ ≈ B⁻¹ (sandwich ≈ naive)
- If the working correlation is wrong: B⁻¹ alone is invalid, but the sandwich correction fixes it

**Small-sample adjustment**: With fewer than 40–50 clusters, the sandwich estimator can underestimate standard errors. Use bias-corrected variants (e.g., `sandwich = "MD"` or `sandwich = "KC"` in some packages).

---

<details>
<summary><strong>Mathematical Foundations</strong> (optional formal notation)</summary>

### The GEE Framework

For person *i* with nᵢ observations, the marginal mean is:

```
g(μᵢₜ) = x'ᵢₜβ
```

Where g(·) is the link function (same choices as GLMM: logit, log, identity).

The marginal variance is:

```
Var(yᵢₜ) = φ · v(μᵢₜ)
```

Where v(·) is the variance function and φ is the dispersion parameter.

### Estimating Equations

The GEE estimator β̂ solves:

```
U(β) = Σᵢ₌₁ᴺ D'ᵢ Vᵢ⁻¹ Sᵢ = 0
```

Where:
- Sᵢ = yᵢ − μᵢ(β) is the vector of residuals for person i
- Dᵢ = ∂μᵢ/∂β is the matrix of derivatives
- Vᵢ = φ Aᵢ^(1/2) R(α) Aᵢ^(1/2) is the working covariance
- Aᵢ = diag{v(μᵢ₁), …, v(μᵢₙᵢ)} contains variance function values
- R(α) is the working correlation matrix

### Consistency

Under correct specification of the mean model E(yᵢₜ) = g⁻¹(x'ᵢₜβ), β̂_GEE is consistent for β regardless of the working correlation R(α).

### Asymptotic Variance

```
√N(β̂ − β) → N(0, B⁻¹ M B⁻¹)
```

Where:
- B = Σᵢ D'ᵢ Vᵢ⁻¹ Dᵢ
- M = Σᵢ D'ᵢ Vᵢ⁻¹ Sᵢ S'ᵢ Vᵢ⁻¹ Dᵢ

This is the sandwich variance — valid under misspecification of R(α).

### QIC (Quasi-likelihood Information Criterion)

```
QIC = -2Q(β̂ᵣ; I) + 2 trace(Ω̂ᵢ V̂ᵣ)
```

Where Q is the quasi-likelihood evaluated under independence working correlation, and the penalty adjusts for model complexity. Lower QIC = better fit.

</details>

---

## Practical Considerations

### Missing Data: The MCAR Requirement

GEE's most important limitation is its assumption about missing data:

| Mechanism | Definition | GEE Valid? |
|-----------|-----------|------------|
| **MCAR** | Missingness is completely random — unrelated to any variables | Yes |
| **MAR** | Missingness depends on *observed* variables | Biased without correction |
| **MNAR** | Missingness depends on *unobserved* values | Biased |

Standard GEE assumes **MCAR** — the strongest and least realistic assumption. If participants who are getting worse are more likely to drop out (MAR), GEE estimates will be biased.

**Mitigations for MAR dropout:**
- Weighted GEE (WGEE): weight each observation by the inverse probability of being observed
- Multiple imputation + GEE: impute missing values, fit GEE to each imputed dataset, pool results
- Use GLMM instead — FIML/REML handles MAR naturally

### Number of Clusters

The sandwich estimator is an *asymptotic* result — it works well with many clusters. With few clusters:

| Clusters | Recommendation |
|----------|---------------|
| < 20 | Avoid GEE or use bias-corrected SEs with extreme caution |
| 20–40 | Use bias-corrected sandwich (Mancl-DeRouen or Kauermann-Carroll) |
| 40–100 | Standard sandwich is acceptable; bias-corrected is preferable |
| 100+ | Standard sandwich works well |

### Choosing a Correlation Structure

In practice, the choice rarely matters much for point estimates (they're consistent regardless). It affects efficiency:

1. **Start with exchangeable** — a safe default for most longitudinal data
2. Compare with AR(1) if you expect temporal decay
3. Use **QIC** (quasi-likelihood information criterion) to compare structures
4. Check that robust and naive SEs are reasonably close — large discrepancies suggest the working correlation is far from reality

```r
library(geepack)

fit_exch  <- geeglm(y ~ time + group, id = id, data = df,
                     family = binomial, corstr = "exchangeable")
fit_ar1   <- geeglm(y ~ time + group, id = id, data = df,
                     family = binomial, corstr = "ar1")
fit_indep <- geeglm(y ~ time + group, id = id, data = df,
                     family = binomial, corstr = "independence")
```

### GEE vs. GLMM: A Practical Comparison

For **linear models** (identity link), GEE and LMM give essentially the same fixed effects. The choice is about inference philosophy and assumptions.

For **non-linear models** (logit, log links), the estimands differ:

| Aspect | GEE (Marginal) | GLMM (Conditional) |
|--------|-----------------|---------------------|
| **Target** | Population average | Subject-specific |
| **Coefficients** | Marginal effects | Conditional effects |
| **OR/IRR magnitude** | Smaller (attenuated) | Larger |
| **Random effects** | Not estimated | Estimated |
| **Missing data** | MCAR required | MAR sufficient |
| **Model comparison** | QIC | AIC, BIC, LRT |
| **Distributional assumptions** | Fewer | More (RE distribution) |

---

## Common Pitfalls

**Treating GEE coefficients as conditional effects** — Interpreting the GEE odds ratio as if it applies to a specific person. *Reality:* GEE estimates marginal (population-averaged) effects. For binary outcomes, the marginal OR is always closer to 1.0 than the conditional OR from GLMM. Report them as "population-averaged."

**Ignoring the MCAR assumption with substantial dropout** — Using standard GEE when 30% of participants have incomplete data and dropout is related to the outcome. *Reality:* If dropout is MAR (depends on observed values), GEE is biased. Use weighted GEE, multiple imputation, or switch to GLMM with FIML.

**Using too few clusters** — Fitting GEE with 15 participants and trusting the sandwich SEs. *Reality:* The sandwich estimator needs large N (clusters, not total observations). With < 40 clusters, use bias-corrected variants or consider GLMM.

**Over-interpreting the working correlation** — Reporting the estimated exchangeable correlation (α = 0.45) as the "true" within-person correlation. *Reality:* The working correlation is a nuisance parameter used for efficiency. It's not a reliable estimate of the actual correlation structure, especially under misspecification.

**Comparing GEE and GLMM coefficients directly** — Noting that "the GEE odds ratio was 1.8 but the GLMM odds ratio was 2.4" and concluding one is wrong. *Reality:* They estimate different quantities. The marginal OR is *supposed* to be smaller than the conditional OR. Both can be correct for their respective targets.

**Using naive SEs instead of robust** — Reporting model-based standard errors without the sandwich correction. *Reality:* Naive SEs are only valid if the working correlation is exactly correct — an assumption you can't verify. Always report robust SEs unless you have strong reasons to believe the working correlation is correct.

**Fitting unstructured correlation with many time points** — Using `corstr = "unstructured"` with 10 waves and 80 participants. *Reality:* Unstructured requires estimating T(T-1)/2 = 45 correlation parameters. With limited clusters relative to time points, estimation becomes unstable. Use exchangeable or AR(1) instead.

**Neglecting to check robust vs. naive SE agreement** — Never comparing the two sets of standard errors. *Reality:* Large discrepancies (> 20%) signal that the working correlation is substantially misspecified. While robust SEs remain valid, this is a useful diagnostic — a well-chosen working correlation gives you more efficient estimates.

**Applying GEE to non-longitudinal clustered data without thought** — Using GEE for any clustered data (e.g., students within schools) without considering whether cluster-level effects matter. *Reality:* GEE estimates marginal effects averaging over clusters. If cluster-level variation is substantively important, a multilevel model is more informative.

**Forgetting that QIC isn't a likelihood-based criterion** — Using QIC as if it were AIC and expecting the same statistical properties. *Reality:* QIC is based on quasi-likelihood, not full likelihood. It's useful for comparing correlation structures and mean models within GEE, but it doesn't have the same theoretical grounding as AIC/BIC. Use it as a guide, not an oracle.

---

## Summary

GEE provides population-averaged estimates for correlated data with minimal distributional assumptions:

- **Marginal effects** describe population averages — how the outcome changes across the population, not for a specific individual
- **Working correlation structures** approximate within-person dependence; the choice affects efficiency but not consistency
- **Sandwich standard errors** produce valid inference even when the working correlation is wrong — GEE's defining feature
- **MCAR assumption** is GEE's main limitation — informative dropout biases estimates without correction
- **Fewer assumptions than GLMM** — no random effects distribution required — but also **fewer capabilities** (no individual predictions, no variance components)

The choice between GEE and GLMM isn't about which is "better" — it's about which question you're asking. Use GEE when the population average is the quantity of interest and you want robust, assumption-light inference.

---

## Next Steps

<div style="display: flex; gap: 1rem; flex-wrap: wrap; margin-top: 1rem;">

**[Walkthrough: Worked Example →](/guides/gee-walkthrough)**
Step-by-step R code to fit GEE models, compare correlation structures, and examine robust SEs.

**[Quick Reference →](/guides/gee-reference)**
Syntax cheat sheets, correlation structure tables, QIC usage, and troubleshooting.

</div>
