//! Data Access section model with collapsible items.

use serde::Deserialize;
use std::borrow::Cow;

/// Data Access item - either collapsible or prose
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DataAccessItem {
    /// Collapsible section with title and content
    Collapsible {
        title: Cow<'static, str>,
        content: Cow<'static, str>,
        #[serde(default = "default_true")]
        open: bool,
    },
    /// Plain prose/HTML content
    Prose { content: Cow<'static, str> },
}

fn default_true() -> bool {
    true
}

/// Root Data Access model
#[derive(Clone, Debug, Deserialize)]
pub struct DataAccessModel {
    /// List of data access items (collapsible sections or prose)
    #[serde(default)]
    pub items: Vec<DataAccessItem>,
    /// Fallback for backward compatibility - plain prose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prose: Option<Cow<'static, str>>,
}

impl DataAccessModel {
    /// Validate the model
    pub fn validate(&self) {
        // No specific validation needed
    }
}
