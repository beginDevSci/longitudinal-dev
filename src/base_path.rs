/// Base path utilities for GitHub Pages deployment
///
/// When deploying to GitHub Pages at a subpath (e.g., /longitudinal-dev/),
/// all URLs need to be prefixed with that base path. This module provides helpers
/// to ensure consistent base path handling throughout the app.
/// Get the base path from environment variable, defaulting to "/"
///
/// The base path is set via SITE_BASE_PATH environment variable during build.
/// - For local development: "/" (default)
/// - For GitHub Pages: "/longitudinal-dev/"
pub fn base_path() -> String {
    base_path_impl(None)
}

/// Internal implementation that accepts optional override for testing
fn base_path_impl(override_path: Option<&str>) -> String {
    override_path
        .map(String::from)
        .unwrap_or_else(|| std::env::var("SITE_BASE_PATH").unwrap_or_else(|_| "/".to_string()))
}

/// Get the base path without trailing slash (for use in props)
///
/// HydrationScripts root prop expects no trailing slash
pub fn base_path_trimmed() -> String {
    base_path_trimmed_impl(None)
}

/// Internal implementation that accepts optional override for testing
fn base_path_trimmed_impl(override_path: Option<&str>) -> String {
    let path = base_path_impl(override_path);
    path.trim_end_matches('/').to_string()
}

/// Join base path with a relative URL
///
/// Ensures proper path joining without double slashes.
/// Examples:
/// - join("/longitudinal-dev/", "posts/foo") -> "/longitudinal-dev/posts/foo"
/// - join("/", "posts/foo") -> "/posts/foo"
pub fn join(relative: &str) -> String {
    join_impl(relative, None)
}

/// Internal implementation that accepts optional base path override for testing
fn join_impl(relative: &str, base_override: Option<&str>) -> String {
    let base = base_path_impl(base_override);
    let relative = relative.trim_start_matches('/');

    if base == "/" {
        format!("/{relative}")
    } else {
        format!(
            "{}{}",
            base.trim_end_matches('/'),
            if relative.is_empty() { "" } else { "/" }
        )
        .to_string()
            + relative
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_with_root() {
        // Use internal _impl functions with explicit base path to avoid env var conflicts
        assert_eq!(join_impl("posts/foo", Some("/")), "/posts/foo");
        assert_eq!(join_impl("/posts/foo", Some("/")), "/posts/foo");
        assert_eq!(join_impl("", Some("/")), "/");
    }

    #[test]
    fn test_join_with_subpath() {
        // Use internal _impl functions with explicit base path to avoid env var conflicts
        assert_eq!(
            join_impl("posts/foo", Some("/longitudinal-dev/")),
            "/longitudinal-dev/posts/foo"
        );
        assert_eq!(
            join_impl("/posts/foo", Some("/longitudinal-dev/")),
            "/longitudinal-dev/posts/foo"
        );
        assert_eq!(
            join_impl("", Some("/longitudinal-dev/")),
            "/longitudinal-dev"
        );
    }

    #[test]
    fn test_base_path_trimmed() {
        // Use internal _impl functions with explicit base path to avoid env var conflicts
        assert_eq!(
            base_path_trimmed_impl(Some("/longitudinal-dev/")),
            "/longitudinal-dev"
        );
        assert_eq!(base_path_trimmed_impl(Some("/")), "");
    }
}
