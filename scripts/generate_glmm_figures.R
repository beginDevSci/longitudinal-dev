# Generate GLMM Guide Figures
# Figure 01: Binary Probability Trajectories (spaghetti plot on probability scale)

library(ggplot2)
library(dplyr)
library(MASS)

set.seed(42)

# Output directory
output_dir <- "public/images/guides/glmm"
dir.create(output_dir, recursive = TRUE, showWarnings = FALSE)

# =============================================================================
# FIGURE 01: Binary Probability Trajectories
# Individual logistic trajectories showing between-person variability
# in probability of a binary outcome over time
# =============================================================================

n_persons <- 150
time_points <- 0:4

# Fixed effects (on log-odds scale)
beta_0 <- 0.8    # Average log-odds at baseline (~69% probability)
beta_1 <- -0.35  # Average change in log-odds per time unit (decreasing)

# Random effects covariance matrix (on log-odds scale)
# Intercept SD ~1.2 gives meaningful spread in baseline probability
# Slope SD ~0.2 gives some divergence
tau <- matrix(c(1.44, -0.06,
                -0.06, 0.04), nrow = 2)

# Generate random effects
re <- mvrnorm(n_persons, c(0, 0), tau)

# Inverse logit function
inv_logit <- function(x) 1 / (1 + exp(-x))

# Generate probability trajectories (no residual noise — these are the
# person-specific probabilities, not observed binary outcomes)
traj_data <- expand.grid(id = 1:n_persons, time = time_points) %>%
  mutate(
    log_odds = (beta_0 + re[id, 1]) + (beta_1 + re[id, 2]) * time,
    prob = inv_logit(log_odds)
  )

# Mean trajectory on probability scale
mean_traj <- data.frame(time = time_points) %>%
  mutate(
    log_odds = beta_0 + beta_1 * time,
    prob = inv_logit(log_odds)
  )

p_binary <- ggplot(traj_data, aes(x = time, y = prob)) +
  geom_line(aes(group = id), alpha = 0.10, color = "gray50", linewidth = 0.4) +
  geom_line(data = mean_traj, aes(y = prob),
            color = "#3B82F6", linewidth = 1.8) +
  geom_point(data = mean_traj, aes(y = prob),
             color = "#3B82F6", size = 3.5) +
  scale_x_continuous(breaks = 0:4, labels = paste("Wave", 1:5)) +
  scale_y_continuous(limits = c(0, 1), breaks = seq(0, 1, 0.25),
                     labels = scales::percent_format()) +
  labs(
    x = "Time",
    y = "Probability",
    title = "Individual Probability Trajectories (Binary GLMM)",
    subtitle = paste0("N = ", n_persons,
                      " participants | Gray = individual | Blue = mean trajectory")
  ) +
  theme_minimal(base_size = 13) +
  theme(
    plot.title = element_text(size = 15, face = "bold"),
    plot.subtitle = element_text(size = 11, color = "gray40"),
    panel.grid.minor = element_blank()
  )

ggsave(file.path(output_dir, "glmm_fig01_binary_trajectories.png"),
       p_binary, width = 8, height = 5, dpi = 150)

cat("Saved: glmm_fig01_binary_trajectories.png\n")
cat("\nAll GLMM figures generated successfully!\n")
