# Generate LMM Guide Figures
# Figure 10: Shrinkage Demonstration (simulated)
# Figure 11: ICC Visualization

library(ggplot2)
library(dplyr)
library(gridExtra)

set.seed(42)

# Output directory
output_dir <- "public/images/guides/lmm"
dir.create(output_dir, recursive = TRUE, showWarnings = FALSE)

# =============================================================================
# FIGURE 10: Shrinkage Demonstration
# Simulating the shrinkage effect without fitting actual mixed model
# =============================================================================

# Simulate OLS estimates with varying reliability
n_persons <- 30

# Generate "true" person-level parameters
true_intercepts <- rnorm(n_persons, mean = 50, sd = 8)
true_slopes <- rnorm(n_persons, mean = 2, sd = 1.2)

# OLS estimates = true + noise (more noise for unreliable estimates)
reliability <- runif(n_persons, 0.4, 0.95)  # Varying reliability
n_obs <- round(3 + reliability * 5)  # More obs = more reliable

ols_intercepts <- true_intercepts + rnorm(n_persons, sd = 12 * (1 - reliability))
ols_slopes <- true_slopes + rnorm(n_persons, sd = 2 * (1 - reliability))

# Grand means
grand_intercept <- mean(ols_intercepts)
grand_slope <- mean(ols_slopes)

# BLUP estimates = shrunk toward grand mean
# Shrinkage factor depends on reliability
shrinkage_factor <- 0.3 + 0.5 * (1 - reliability)  # More shrinkage for low reliability

blup_intercepts <- (1 - shrinkage_factor) * ols_intercepts + shrinkage_factor * grand_intercept
blup_slopes <- (1 - shrinkage_factor) * ols_slopes + shrinkage_factor * grand_slope

estimates <- data.frame(
  person_id = 1:n_persons,
  ols_intercept = ols_intercepts,
  ols_slope = ols_slopes,
  blup_intercept = blup_intercepts,
  blup_slope = blup_slopes,
  n_obs = n_obs,
  reliability = reliability
)

# Create shrinkage plot for intercepts
p_shrinkage_intercept <- ggplot(estimates, aes(x = ols_intercept, y = blup_intercept)) +
  geom_abline(slope = 1, intercept = 0, linetype = "solid", color = "gray50", linewidth = 0.5) +
  geom_vline(xintercept = grand_intercept, linetype = "dotted", color = "firebrick", linewidth = 0.8) +
  geom_hline(yintercept = grand_intercept, linetype = "dotted", color = "steelblue", linewidth = 0.8) +
  geom_segment(aes(xend = blup_intercept, yend = blup_intercept),
               arrow = arrow(length = unit(0.1, "cm"), type = "closed"),
               color = "gray40", alpha = 0.5) +
  geom_point(aes(size = n_obs), color = "steelblue", alpha = 0.7) +
  scale_size_continuous(range = c(2, 5), name = "N Obs") +
  labs(
    x = "OLS Intercept Estimate",
    y = "BLUP Intercept Estimate",
    title = "Intercept Shrinkage"
  ) +
  theme_minimal(base_size = 11) +
  theme(
    plot.title = element_text(size = 12, face = "bold"),
    legend.position = "bottom"
  ) +
  coord_fixed()

# Create shrinkage plot for slopes
p_shrinkage_slope <- ggplot(estimates, aes(x = ols_slope, y = blup_slope)) +
  geom_abline(slope = 1, intercept = 0, linetype = "solid", color = "gray50", linewidth = 0.5) +
  geom_vline(xintercept = grand_slope, linetype = "dotted", color = "firebrick", linewidth = 0.8) +
  geom_hline(yintercept = grand_slope, linetype = "dotted", color = "darkorange", linewidth = 0.8) +
  geom_segment(aes(xend = blup_slope, yend = blup_slope),
               arrow = arrow(length = unit(0.1, "cm"), type = "closed"),
               color = "gray40", alpha = 0.5) +
  geom_point(aes(size = n_obs), color = "darkorange", alpha = 0.7) +
  scale_size_continuous(range = c(2, 5), name = "N Obs") +
  labs(
    x = "OLS Slope Estimate",
    y = "BLUP Slope Estimate",
    title = "Slope Shrinkage"
  ) +
  theme_minimal(base_size = 11) +
  theme(
    plot.title = element_text(size = 12, face = "bold"),
    legend.position = "bottom"
  ) +
  coord_fixed()

# Combine plots using gridExtra
png(file.path(output_dir, "lmm_fig10_shrinkage.png"), width = 1000, height = 500, res = 100)
grid.arrange(
  p_shrinkage_intercept,
  p_shrinkage_slope,
  ncol = 2,
  top = grid::textGrob(
    "Shrinkage in Mixed Models: OLS vs BLUP Estimates\nPoints pulled toward grand mean (horizontal/vertical dotted lines)",
    gp = grid::gpar(fontsize = 14, fontface = "bold")
  )
)
dev.off()

cat("Saved: lmm_fig10_shrinkage.png\n")

# =============================================================================
# FIGURE 11: ICC Visualization
# =============================================================================

set.seed(123)

# High ICC scenario (ICC ≈ 0.65)
n_persons_icc <- 20
n_time <- 4

between_var_high <- 100
within_var_high <- 50

icc_high <- between_var_high / (between_var_high + within_var_high)

persons_high <- data.frame(
  person_id = 1:n_persons_icc,
  person_mean = rnorm(n_persons_icc, mean = 50, sd = sqrt(between_var_high))
)

data_high <- expand.grid(
  person_id = 1:n_persons_icc,
  time = 1:n_time
) %>%
  left_join(persons_high, by = "person_id") %>%
  mutate(y = person_mean + rnorm(n(), sd = sqrt(within_var_high)))

# Low ICC scenario (ICC ≈ 0.15)
between_var_low <- 20
within_var_low <- 120

icc_low <- between_var_low / (between_var_low + within_var_low)

persons_low <- data.frame(
  person_id = 1:n_persons_icc,
  person_mean = rnorm(n_persons_icc, mean = 50, sd = sqrt(between_var_low))
)

data_low <- expand.grid(
  person_id = 1:n_persons_icc,
  time = 1:n_time
) %>%
  left_join(persons_low, by = "person_id") %>%
  mutate(y = person_mean + rnorm(n(), sd = sqrt(within_var_low)))

# Create variance partition data for stacked bar chart
variance_data <- data.frame(
  Component = factor(rep(c("Between-Person", "Within-Person"), 2),
                     levels = c("Within-Person", "Between-Person")),
  ICC_Level = rep(c("High ICC (0.65)", "Low ICC (0.14)"), each = 2),
  Proportion = c(0.65, 0.35, 0.14, 0.86)
)

# Stacked bar for variance partitioning
p_variance <- ggplot(variance_data, aes(x = ICC_Level, y = Proportion, fill = Component)) +
  geom_bar(stat = "identity", width = 0.6) +
  geom_text(aes(label = paste0(round(Proportion * 100), "%")),
            position = position_stack(vjust = 0.5),
            color = "white", fontface = "bold", size = 4) +
  scale_fill_manual(values = c("Between-Person" = "steelblue", "Within-Person" = "coral")) +
  labs(
    title = "Variance Partitioning",
    y = "Proportion of Total Variance",
    x = NULL
  ) +
  theme_minimal(base_size = 11) +
  theme(
    plot.title = element_text(size = 12, face = "bold"),
    legend.position = "bottom",
    legend.title = element_blank()
  ) +
  coord_flip()

# Spaghetti plot for high ICC
p_high_icc <- ggplot(data_high, aes(x = time, y = y, group = person_id, color = factor(person_id))) +
  geom_line(alpha = 0.7, linewidth = 0.8) +
  geom_point(size = 1.5) +
  scale_color_manual(values = rainbow(n_persons_icc), guide = "none") +
  labs(
    title = paste0("High ICC (", round(icc_high, 2), ")"),
    subtitle = "People differ; each person consistent",
    x = "Time",
    y = "Score"
  ) +
  theme_minimal(base_size = 11) +
  theme(
    plot.title = element_text(size = 12, face = "bold"),
    plot.subtitle = element_text(size = 9, color = "gray40")
  ) +
  ylim(0, 100)

# Spaghetti plot for low ICC
p_low_icc <- ggplot(data_low, aes(x = time, y = y, group = person_id, color = factor(person_id))) +
  geom_line(alpha = 0.7, linewidth = 0.8) +
  geom_point(size = 1.5) +
  scale_color_manual(values = rainbow(n_persons_icc), guide = "none") +
  labs(
    title = paste0("Low ICC (", round(icc_low, 2), ")"),
    subtitle = "People similar; lots of noise",
    x = "Time",
    y = "Score"
  ) +
  theme_minimal(base_size = 11) +
  theme(
    plot.title = element_text(size = 12, face = "bold"),
    plot.subtitle = element_text(size = 9, color = "gray40")
  ) +
  ylim(0, 100)

# Combine ICC plots using gridExtra
png(file.path(output_dir, "lmm_fig11_icc.png"), width = 1000, height = 800, res = 100)
grid.arrange(
  p_variance,
  arrangeGrob(p_high_icc, p_low_icc, ncol = 2),
  nrow = 2,
  heights = c(1, 1.5),
  top = grid::textGrob(
    "Understanding the Intraclass Correlation Coefficient (ICC)\nHigh ICC = stable individual differences | Low ICC = occasion-specific variability",
    gp = grid::gpar(fontsize = 14, fontface = "bold")
  )
)
dev.off()

cat("Saved: lmm_fig11_icc.png\n")

cat("\nAll figures generated successfully!\n")
