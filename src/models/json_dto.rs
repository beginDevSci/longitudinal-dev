use serde::Deserialize;

// Note: Version enums removed - all tutorials now use V2 format exclusively

#[derive(Deserialize, Debug, Clone)]
pub struct JsonPost {
    pub title: String,
    #[serde(default = "default_layout")]
    pub layout: Option<String>,
    #[serde(default)]
    pub metadata: Option<crate::models::post::PostMetadata>,
    pub overview: JsonOverview,
    pub data_access: JsonDataAccess,
    pub data_preparation: crate::models::data_preparation::DataPrepModel,
    pub statistical_analysis: crate::models::statistical_analysis::StatsModel,
    pub discussion: JsonDiscussion,
    pub additional_resources: JsonResources,
}

fn default_layout() -> Option<String> {
    None
}

/// Represents a stat item that can be either a simple string (auto-labeled)
/// or an object with custom label and value
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JsonStatItem {
    /// Simple string value with auto-assigned label based on position
    Simple(String),
    /// Object with optional custom label and required value
    WithLabel {
        #[serde(default)]
        label: Option<String>,
        value: String,
    },
}

/// Represents a feature item that can be either a simple string (auto-labeled)
/// or an object with custom heading and text
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum JsonFeatureItem {
    /// Simple string value with auto-assigned heading based on position
    Simple(String),
    /// Object with optional custom heading and required text
    WithHeading {
        #[serde(default)]
        heading: Option<String>,
        text: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonOverview {
    pub summary: String,
    #[serde(default)]
    pub stats_panel: Option<JsonStatsPanel>,
    #[serde(default)]
    pub features_panel: Option<JsonFeaturesPanel>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonStatsPanel {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub items: Vec<JsonStatItem>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonFeaturesPanel {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub items: Vec<JsonFeatureItem>,
}

impl JsonOverview {
    pub(crate) fn into_overview_model(self) -> crate::models::post::OverviewModel {
        use crate::models::overview::{
            FeatureCard, FeaturesPanelData, StatRow, StatsPanelData, FEATURES_PANEL_LABELS,
            STATS_PANEL_LABELS,
        };
        use crate::models::post::OverviewModel;
        use std::borrow::Cow;

        // Convert stats_panel (1-10 stats with auto-assigned or custom labels)
        let stats_panel = self.stats_panel.map(|panel| {
            let stats_rows: Vec<StatRow> = panel
                .items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    match item {
                        // Simple string: use auto-assigned label based on position
                        JsonStatItem::Simple(value) => {
                            let label = STATS_PANEL_LABELS.get(i).copied().unwrap_or("Stat:");
                            StatRow {
                                label: Cow::Borrowed(label),
                                value: Cow::Owned(value),
                                delta: None,
                            }
                        }
                        // Object with custom label: use provided label or auto-assign if None
                        JsonStatItem::WithLabel { label, value } => {
                            let final_label = label.map(Cow::Owned).unwrap_or_else(|| {
                                let auto_label =
                                    STATS_PANEL_LABELS.get(i).copied().unwrap_or("Stat:");
                                Cow::Borrowed(auto_label)
                            });
                            StatRow {
                                label: final_label,
                                value: Cow::Owned(value),
                                delta: None,
                            }
                        }
                    }
                })
                .collect();

            StatsPanelData {
                title: panel.title.map(Cow::Owned),
                rows: stats_rows,
            }
        });

        // Convert features_panel (1-5 features with auto-assigned or custom headings)
        let features_panel = self.features_panel.map(|panel| {
            let features_vec: Vec<FeatureCard> = panel
                .items
                .into_iter()
                .enumerate()
                .map(|(i, item)| {
                    match item {
                        // Simple string: use auto-assigned heading based on position
                        JsonFeatureItem::Simple(text) => {
                            let heading =
                                FEATURES_PANEL_LABELS.get(i).copied().unwrap_or("Feature");
                            FeatureCard {
                                heading: Cow::Borrowed(heading),
                                lines: vec![Cow::Owned(text)],
                            }
                        }
                        // Object with custom heading: use provided heading or auto-assign if None
                        JsonFeatureItem::WithHeading { heading, text } => {
                            let final_heading = heading.map(Cow::Owned).unwrap_or_else(|| {
                                let auto_heading =
                                    FEATURES_PANEL_LABELS.get(i).copied().unwrap_or("Feature");
                                Cow::Borrowed(auto_heading)
                            });
                            FeatureCard {
                                heading: final_heading,
                                lines: vec![Cow::Owned(text)],
                            }
                        }
                    }
                })
                .collect();

            FeaturesPanelData {
                title: panel.title.map(Cow::Owned),
                cards: features_vec,
            }
        });

        OverviewModel {
            summary_paragraphs: vec![Cow::Owned(self.summary)],
            stats_panel,
            features_panel,
        }
    }
}

/// Data Access item from JSON
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsonDataAccessItem {
    Collapsible {
        title: String,
        content: String,
        #[serde(default = "default_true_json")]
        open: bool,
    },
    Prose {
        content: String,
    },
}

fn default_true_json() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonDataAccess {
    #[serde(default)]
    pub items: Vec<JsonDataAccessItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prose: Option<String>,
}

impl JsonDataAccess {
    pub(crate) fn into_data_access_model(self) -> crate::models::data_access::DataAccessModel {
        use crate::models::data_access::{DataAccessItem, DataAccessModel};
        use std::borrow::Cow;

        // Convert JSON items to model items
        let items: Vec<DataAccessItem> = self
            .items
            .into_iter()
            .map(|json_item| match json_item {
                crate::models::json_dto::JsonDataAccessItem::Collapsible {
                    title,
                    content,
                    open,
                } => DataAccessItem::Collapsible {
                    title: Cow::Owned(title),
                    content: Cow::Owned(content),
                    open,
                },
                crate::models::json_dto::JsonDataAccessItem::Prose { content } => {
                    DataAccessItem::Prose {
                        content: Cow::Owned(content),
                    }
                }
            })
            .collect();

        DataAccessModel {
            items,
            prose: self.prose.map(Cow::Owned),
        }
    }
}

// Note: V1 JSON structs and conversion functions removed - all tutorials now use V2 format

#[derive(Deserialize, Debug, Clone)]
pub struct JsonDiscussionItem {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonDiscussion {
    /// Structured items from H2 headings (preferred)
    #[serde(default)]
    pub items: Vec<JsonDiscussionItem>,
    /// Fallback paragraphs (for backward compatibility)
    #[serde(default)]
    pub paragraphs: Vec<String>,
}

impl JsonDiscussion {
    pub(crate) fn into_discussion_model(self) -> crate::models::discussion::DiscussionModel {
        use crate::models::discussion::{DiscussionItem, DiscussionModel};
        use std::borrow::Cow;

        DiscussionModel {
            items: self
                .items
                .into_iter()
                .map(|item| DiscussionItem {
                    title: Cow::Owned(item.title),
                    content: Cow::Owned(item.content),
                })
                .collect(),
            paragraphs: self.paragraphs.into_iter().map(Cow::Owned).collect(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonResources {
    /// Exactly 3 cards (validated by schema)
    pub cards: Vec<JsonResourceCard>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonResourceCard {
    pub title: String,
    /// Required badge label (uppercase)
    pub badge: String,
    pub body: String,
    /// Optional URL to the resource
    pub url: Option<String>,
}

impl JsonResources {
    pub(crate) fn into_resources_model(
        self,
    ) -> crate::models::additional_resources::ResourcesModel {
        use crate::models::additional_resources::{ResourceCard, ResourcesModel};
        use std::borrow::Cow;

        ResourcesModel {
            items: self
                .cards
                .into_iter()
                .map(|c| {
                    let url = c.url.map(Cow::Owned);
                    let show_chevron = url.is_some();
                    ResourceCard {
                        title: Cow::Owned(c.title),
                        body: Cow::Owned(c.body),
                        badge_upper: Cow::Owned(c.badge),
                        url,
                        show_chevron,
                    }
                })
                .collect(),
        }
    }
}
