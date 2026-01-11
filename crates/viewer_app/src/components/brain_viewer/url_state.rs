//! URL query parameter parsing for initial viewer state.
//!
//! Extracts view configuration from URL query strings to enable shareable links.
//! This is a pure parsing module with no signal or rendering dependencies.

use crate::types::{Hemisphere, LayoutMode, Statistic};

/// Initial state parsed from URL query parameters.
///
/// All fields are optional - `None` means use the default value.
#[derive(Debug, Default)]
pub struct InitialUrlState {
    pub hemisphere: Option<Hemisphere>,
    pub statistic: Option<Statistic>,
    pub volume_idx: Option<u32>,
    pub layout: Option<LayoutMode>,
    pub threshold: Option<f32>,
    pub region_selection_enabled: Option<bool>,
    pub color_mode: Option<String>,
    pub parc_display_mode: Option<String>,
}

/// Parse URL query parameters into initial state values.
///
/// Supported parameters:
/// - `hemi`: "lh" or "rh" for hemisphere selection
/// - `stat`: "tstat", "beta", "conTlp", "logp", "sigma2", "Chi2", "Chi2lp"
/// - `vol`: volume/contrast index (integer)
/// - `layout`: "single", "side", "side-by-side", "stacked"
/// - `thr`: threshold value (float)
/// - `region`: "1" or "true" to enable region selection
/// - `color`: "overlay" or "parcellation"
/// - `parc_display`: "fill", "edges", or "fill_edges"
///
/// # Example
///
/// ```ignore
/// let state = parse_url_query_params("hemi=lh&stat=tstat&vol=2&thr=2.5");
/// assert_eq!(state.hemisphere, Some(Hemisphere::Left));
/// assert_eq!(state.threshold, Some(2.5));
/// ```
pub fn parse_url_query_params(query: &str) -> InitialUrlState {
    let mut state = InitialUrlState::default();

    // Strip leading '?' if present
    let query = query.trim_start_matches('?');

    if query.is_empty() {
        return state;
    }

    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");

        match key {
            "hemi" => {
                state.hemisphere = match value {
                    "lh" => Some(Hemisphere::Left),
                    "rh" => Some(Hemisphere::Right),
                    _ => None,
                };
            }
            "stat" => {
                state.statistic = Some(match value {
                    "beta" => Statistic::Beta,
                    "conTlp" | "logp" => Statistic::LogP,
                    "sigma2" => Statistic::Sigma2,
                    "Chi2" => Statistic::Chi2,
                    "Chi2lp" => Statistic::Chi2lp,
                    _ => Statistic::TStat,
                });
            }
            "vol" => {
                if let Ok(v) = value.parse::<u32>() {
                    state.volume_idx = Some(v);
                }
            }
            "layout" => {
                state.layout = Some(match value {
                    "side" | "side-by-side" => LayoutMode::SideBySide,
                    "stacked" => LayoutMode::Stacked,
                    _ => LayoutMode::Single,
                });
            }
            "thr" => {
                if let Ok(t) = value.parse::<f32>() {
                    state.threshold = Some(t);
                }
            }
            "region" => {
                state.region_selection_enabled = Some(value == "1" || value == "true");
            }
            "color" => {
                state.color_mode = Some(if value == "parcellation" {
                    "parcellation".to_string()
                } else {
                    "overlay".to_string()
                });
            }
            "parc_display" => {
                state.parc_display_mode = Some(
                    match value {
                        "edges" => "edges",
                        "fill_edges" => "fill_edges",
                        _ => "fill",
                    }
                    .to_string(),
                );
            }
            _ => {}
        }
    }

    state
}

/// State values for building a shareable URL.
///
/// This is the inverse of `InitialUrlState` - used when generating share links.
#[derive(Debug, Clone)]
pub struct ShareableState {
    pub hemisphere: Hemisphere,
    pub statistic: Statistic,
    pub volume_idx: u32,
    pub layout: LayoutMode,
    pub threshold: Option<f32>,
    pub region_selection_enabled: bool,
}

/// Build a URL query string from the current view state.
///
/// This is the inverse of `parse_url_query_params`.
///
/// # Example
///
/// ```ignore
/// let state = ShareableState {
///     hemisphere: Hemisphere::Left,
///     statistic: Statistic::TStat,
///     volume_idx: 2,
///     layout: LayoutMode::Single,
///     threshold: Some(2.5),
///     region_selection_enabled: false,
/// };
/// let query = build_share_query_string(&state);
/// assert!(query.contains("hemi=lh"));
/// assert!(query.contains("thr=2.500"));
/// ```
pub fn build_share_query_string(state: &ShareableState) -> String {
    let hemi_str = match state.hemisphere {
        Hemisphere::Left => "lh",
        Hemisphere::Right => "rh",
    };

    let stat_str = match state.statistic {
        Statistic::TStat => "tstat",
        Statistic::Beta => "beta",
        Statistic::LogP => "conTlp",
        Statistic::Sigma2 => "sigma2",
        Statistic::Chi2 => "Chi2",
        Statistic::Chi2lp => "Chi2lp",
    };

    let layout_str = match state.layout {
        LayoutMode::Single => "single",
        LayoutMode::SideBySide => "side",
        LayoutMode::Stacked => "stacked",
    };

    let mut params = vec![
        format!("hemi={}", hemi_str),
        format!("stat={}", stat_str),
        format!("vol={}", state.volume_idx),
        format!("layout={}", layout_str),
        format!("region={}", if state.region_selection_enabled { "1" } else { "0" }),
    ];

    if let Some(t) = state.threshold {
        params.push(format!("thr={:.3}", t));
    }

    params.join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_query() {
        let state = parse_url_query_params("");
        assert!(state.hemisphere.is_none());
        assert!(state.threshold.is_none());
    }

    #[test]
    fn test_hemisphere_parsing() {
        let state = parse_url_query_params("hemi=lh");
        assert_eq!(state.hemisphere, Some(Hemisphere::Left));

        let state = parse_url_query_params("hemi=rh");
        assert_eq!(state.hemisphere, Some(Hemisphere::Right));
    }

    #[test]
    fn test_threshold_parsing() {
        let state = parse_url_query_params("thr=2.5");
        assert_eq!(state.threshold, Some(2.5));
    }

    #[test]
    fn test_multiple_params() {
        let state = parse_url_query_params("hemi=lh&stat=beta&vol=3&thr=1.5");
        assert_eq!(state.hemisphere, Some(Hemisphere::Left));
        assert_eq!(state.statistic, Some(Statistic::Beta));
        assert_eq!(state.volume_idx, Some(3));
        assert_eq!(state.threshold, Some(1.5));
    }

    #[test]
    fn test_strips_leading_question_mark() {
        let state = parse_url_query_params("?hemi=rh");
        assert_eq!(state.hemisphere, Some(Hemisphere::Right));
    }

    #[test]
    fn test_build_share_query_string() {
        let state = ShareableState {
            hemisphere: Hemisphere::Left,
            statistic: Statistic::TStat,
            volume_idx: 2,
            layout: LayoutMode::Single,
            threshold: Some(2.5),
            region_selection_enabled: false,
        };
        let query = build_share_query_string(&state);
        assert!(query.contains("hemi=lh"));
        assert!(query.contains("stat=tstat"));
        assert!(query.contains("vol=2"));
        assert!(query.contains("layout=single"));
        assert!(query.contains("region=0"));
        assert!(query.contains("thr=2.500"));
    }

    #[test]
    fn test_build_share_query_no_threshold() {
        let state = ShareableState {
            hemisphere: Hemisphere::Right,
            statistic: Statistic::Beta,
            volume_idx: 0,
            layout: LayoutMode::SideBySide,
            threshold: None,
            region_selection_enabled: true,
        };
        let query = build_share_query_string(&state);
        assert!(query.contains("hemi=rh"));
        assert!(query.contains("stat=beta"));
        assert!(query.contains("layout=side"));
        assert!(query.contains("region=1"));
        assert!(!query.contains("thr="));
    }
}
