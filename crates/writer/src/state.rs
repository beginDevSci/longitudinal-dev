use crate::domain::*;
use crate::validation::*;
use chrono::Utc;
use leptos::prelude::*;

/// Global editor state managed with Leptos signals
#[derive(Debug, Clone, Copy)]
pub struct EditorState {
    pub tutorial: RwSignal<Tutorial>,
    pub validation_issues: RwSignal<Vec<ValidationIssue>>,
    pub is_dirty: RwSignal<bool>,
    pub last_saved: RwSignal<Option<String>>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            tutorial: RwSignal::new(Tutorial::new()),
            validation_issues: RwSignal::new(Vec::new()),
            is_dirty: RwSignal::new(false),
            last_saved: RwSignal::new(None),
        }
    }

    /// Load a tutorial from JSON
    pub fn load_tutorial(&self, tutorial: Tutorial) {
        self.tutorial.set(tutorial);
        self.is_dirty.set(false);
        self.validate();
    }

    /// Update tutorial and mark as dirty
    pub fn update_tutorial<F>(&self, f: F)
    where
        F: FnOnce(&mut Tutorial),
    {
        self.tutorial.update(|t| {
            f(t);
            t.update_timestamp();
        });
        self.is_dirty.set(true);
        self.validate();
    }

    /// Run validation
    pub fn validate(&self) {
        let tutorial = self.tutorial.get();
        let issues = TutorialValidator::validate(&tutorial);
        self.validation_issues.set(issues);
    }

    /// Check if can export
    pub fn can_export(&self) -> bool {
        let tutorial = self.tutorial.get();
        TutorialValidator::can_export(&tutorial)
    }

    /// Get tutorial as JSON
    pub fn to_json(&self) -> Result<String, String> {
        let tutorial = self.tutorial.get();
        serde_json::to_string_pretty(&tutorial).map_err(|e| e.to_string())
    }

    /// Load from JSON string
    pub fn from_json(&self, json: &str) -> Result<(), String> {
        let tutorial: Tutorial = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.load_tutorial(tutorial);
        Ok(())
    }

    /// Mark as saved
    pub fn mark_saved(&self) {
        self.is_dirty.set(false);
        let now = Utc::now();
        self.last_saved
            .set(Some(now.format("%Y-%m-%d %H:%M:%S").to_string()));
    }

    /// Get completeness percentage (0.0 to 1.0)
    pub fn completeness(&self) -> f32 {
        let tutorial = self.tutorial.get();
        let mut completed = 0;
        let mut total = 0;

        // Check metadata
        total += 2;
        if !tutorial.title.is_empty() {
            completed += 1;
        }
        if !tutorial.author.name.is_empty() {
            completed += 1;
        }

        // Check sections have content
        for section in &tutorial.sections {
            total += 1;
            if section.has_content() {
                completed += 1;
            }
        }

        if total == 0 {
            0.0
        } else {
            completed as f32 / total as f32
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}
