# Generate GEE Guide Figures
# Figure 01: Population-Averaged Trends
# Shows what GEE models: marginal group-level trends with individual
# heterogeneity faded in the background

library(ggplot2)
library(dplyr)
library(MASS)

set.seed(42)

# Output directory
output_dir <- "public/images/guides/gee"
dir.create(output_dir, recursive = TRUE, showWarnings = FALSE)

# =============================================================================
# FIGURE 01: Population-Averaged Trends (Binary Outcome)
# Faded individual probability trajectories with bold group-averaged curves
# Visually communicates GEE's focus: the population average, not individuals
# =============================================================================

n_persons <- 200
time_points <- 0:4

# Fixed effects (log-odds scale)
beta_0 <- -0.2       # Baseline log-odds (control group)
beta_time <- -0.30    # Time effect
beta_trt <- -0.80     # Treatment effect
ri_sd <- 1.0          # Random intercept SD

# Generate person-level data
person_df <- data.frame(
  id = 1:n_persons,
  treatment = rep(c(0, 1), each = n_persons / 2),
  ri = rnorm(n_persons, 0, ri_sd)
)

# Inverse logit
inv_logit <- function(x) 1 / (1 + exp(-x))

# Individual probability trajectories
traj_data <- expand.grid(id = 1:n_persons, time = time_points) %>%
  left_join(person_df, by = "id") %>%
  mutate(
    log_odds = beta_0 + beta_time * time + beta_trt * treatment + ri,
    prob = inv_logit(log_odds),
    group = factor(treatment, labels = c("Control", "Treatment"))
  )

# Population-averaged proportions (observed, what GEE targets)
pop_avg <- traj_data %>%
  group_by(time, group) %>%
  summarise(prop = mean(prob), .groups = "drop")

p_gee <- ggplot() +
  # Individual trajectories (faded — what GEE does NOT model)
  geom_line(data = traj_data,
            aes(x = time, y = prob, group = id, color = group),
            alpha = 0.06, linewidth = 0.3) +
  # Population-averaged curves (bold — what GEE DOES model)
  geom_line(data = pop_avg,
            aes(x = time, y = prop, color = group),
            linewidth = 2) +
  geom_point(data = pop_avg,
             aes(x = time, y = prop, color = group),
             size = 3.5) +
  scale_color_manual(
    values = c("Control" = "#3B82F6", "Treatment" = "#F97316"),
    name = "Group"
  ) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  scale_y_continuous(limits = c(0, 1), breaks = seq(0, 1, 0.25),
                     labels = scales::percent_format()) +
  labs(
    x = "Time",
    y = "Probability",
    title = "Population-Averaged Trends (What GEE Models)",
    subtitle = paste0("N = ", n_persons,
                      " participants | Faded = individual trajectories",
                      " | Bold = population averages")
  ) +
  theme_minimal(base_size = 13) +
  theme(
    plot.title = element_text(size = 15, face = "bold"),
    plot.subtitle = element_text(size = 11, color = "gray40"),
    panel.grid.minor = element_blank(),
    legend.position = "bottom"
  )

ggsave(file.path(output_dir, "gee_fig01_population_averaged.png"),
       p_gee, width = 8, height = 5, dpi = 150)

cat("Saved: gee_fig01_population_averaged.png\n")
cat("\nAll GEE figures generated successfully!\n")
