//! Guide navigation tabs for switching between overview and walkthrough pages.
//!
//! Provides a horizontal tab bar that appears below the guide header,
//! allowing users to navigate between related guide sections.

use crate::base_path;
use leptos::prelude::*;

/// Information about a guide tab.
#[derive(Clone)]
pub struct GuideTabInfo {
    /// The method slug (e.g., "lgcm-pilot")
    pub method_slug: String,
    /// Current guide type: "overview" or "tutorial"
    pub current_type: String,
}

impl GuideTabInfo {
    /// Create tab info from a guide's metadata.
    ///
    /// For hub pages, the method_slug is the guide's own slug.
    /// For tutorial/reference pages, it's derived from parent_method.
    pub fn from_guide(
        slug: &str,
        guide_type: Option<&str>,
        parent_method: Option<&str>,
    ) -> Option<Self> {
        let current_type = match guide_type.unwrap_or("overview") {
            "hub" => "overview",
            "walkthrough" => "tutorial",
            other => other,
        }
        .to_string();

        // Determine the method slug
        let method_slug = match current_type.as_str() {
            "overview" => slug.to_string(),
            "tutorial" | "reference" => {
                // Use parent_method if available, otherwise derive from slug
                parent_method
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        // Fallback: strip known suffixes
                        slug.trim_end_matches("-walkthrough")
                            .trim_end_matches("-tutorial")
                            .trim_end_matches("-reference")
                            .to_string()
                    })
            }
            _ => return None,
        };

        Some(Self {
            method_slug,
            current_type,
        })
    }

    /// Get the URL for the overview page.
    pub fn hub_url(&self) -> String {
        base_path::join(&format!("guides/{}/", self.method_slug))
    }

    /// Get the URL for the tutorial page.
    pub fn tutorial_url(&self) -> String {
        base_path::join(&format!("guides/{}-walkthrough/", self.method_slug))
    }
}

/// Tab definition for rendering.
struct TabDef {
    label: &'static str,
    type_id: &'static str,
    url: String,
}

/// Navigation tabs for switching between guide sections.
///
/// Renders a horizontal tab bar with Overview and Worked Example tabs.
/// The current page's tab is highlighted.
#[component]
pub fn GuideNavTabs(info: GuideTabInfo) -> impl IntoView {
    let current_type = info.current_type.clone();

    let tabs = vec![
        TabDef {
            label: "Overview",
            type_id: "overview",
            url: info.hub_url(),
        },
        TabDef {
            label: "Worked Example",
            type_id: "tutorial",
            url: info.tutorial_url(),
        },
    ];

    view! {
        <nav class="guide-nav-tabs" aria-label="Guide sections">
            <div class="guide-nav-tabs-container">
                {tabs.into_iter().map(|tab| {
                    let is_active = tab.type_id == current_type;
                    let class = if is_active {
                        "guide-nav-tab guide-nav-tab--active"
                    } else {
                        "guide-nav-tab"
                    };

                    if is_active {
                        // Current page - render as non-clickable span
                        view! {
                            <span class=class aria-current="page">
                                {tab.label}
                            </span>
                        }.into_any()
                    } else {
                        // Other pages - render as link
                        view! {
                            <a href={tab.url} class=class>
                                {tab.label}
                            </a>
                        }.into_any()
                    }
                }).collect_view()}
            </div>
        </nav>
    }
}
