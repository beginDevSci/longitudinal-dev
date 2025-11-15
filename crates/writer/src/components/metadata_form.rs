use crate::EditorState;
use leptos::prelude::*;

/// Metadata editor form component
///
/// Allows editing tutorial metadata:
/// - Title and author
/// - Method classification (family, engine, covariates, outcome_type)
/// - Date and tags
#[component]
pub fn MetadataForm(editor_state: EditorState) -> impl IntoView {
    view! {
        <div class="metadata-form max-w-3xl">
            <h2 class="section-title mb-6">
                "Tutorial Metadata"
            </h2>

            <div class="space-y-6">
                {/* Title */}
                <div class="form-group">
                    <label for="title" class="form-label">
                        "Title" <span class="text-red-500">"*"</span>
                    </label>
                    <input
                        type="text"
                        id="title"
                        class="form-input"
                        placeholder="Enter tutorial title..."
                        prop:value=move || editor_state.tutorial.get().title
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            editor_state.tutorial.update(|t| {
                                t.title = value;
                            });
                            editor_state.is_dirty.set(true);
                        }
                    />
                    <p class="form-help">
                        "The main title of your tutorial"
                    </p>
                </div>

                {/* Author */}
                <div class="form-group">
                    <label for="author-name" class="form-label">
                        "Author Name"
                    </label>
                    <input
                        type="text"
                        id="author-name"
                        class="form-input"
                        placeholder="Your name..."
                        prop:value=move || editor_state.tutorial.get().author.name
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            editor_state.tutorial.update(|t| {
                                t.author.name = value;
                            });
                            editor_state.is_dirty.set(true);
                        }
                    />
                </div>

                {/* Method Classification */}
                <div class="border-t border-neutral-200 dark:border-neutral-700 pt-6">
                    <h3 class="text-lg font-semibold text-primary mb-4">
                        "Method Classification"
                    </h3>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {/* Family */}
                        <div class="form-group">
                            <label for="family" class="form-label">
                                "Family"
                            </label>
                            <select
                                id="family"
                                class="form-select"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.family = if value.is_empty() {
                                            None
                                        } else {
                                            Some(value)
                                        };
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            >
                                <option value="" selected=move || editor_state.tutorial.get().metadata.family.is_none()>
                                    "-- Select --"
                                </option>
                                <option value="LGCM" selected=move || editor_state.tutorial.get().metadata.family.as_deref() == Some("LGCM")>
                                    "LGCM"
                                </option>
                                <option value="GLMM" selected=move || editor_state.tutorial.get().metadata.family.as_deref() == Some("GLMM")>
                                    "GLMM"
                                </option>
                                <option value="LMM" selected=move || editor_state.tutorial.get().metadata.family.as_deref() == Some("LMM")>
                                    "LMM"
                                </option>
                                <option value="SEM" selected=move || editor_state.tutorial.get().metadata.family.as_deref() == Some("SEM")>
                                    "SEM"
                                </option>
                                <option value="LCGA" selected=move || editor_state.tutorial.get().metadata.family.as_deref() == Some("LCGA")>
                                    "LCGA"
                                </option>
                                <option value="LCSM" selected=move || editor_state.tutorial.get().metadata.family.as_deref() == Some("LCSM")>
                                    "LCSM"
                                </option>
                            </select>
                            <p class="form-help">
                                "Statistical method family"
                            </p>
                        </div>

                        {/* Family Label */}
                        <div class="form-group">
                            <label for="family_label" class="form-label">
                                "Family Label"
                            </label>
                            <input
                                id="family_label"
                                class="form-input"
                                r#type="text"
                                placeholder="e.g., Latent Growth Curve Models (LGCM)"
                                value=move || {
                                    editor_state
                                        .tutorial
                                        .get()
                                        .metadata
                                        .family_label
                                        .clone()
                                        .unwrap_or_default()
                                }
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.family_label = if value.trim().is_empty() {
                                            None
                                        } else {
                                            Some(value.trim().to_string())
                                        };
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            />
                            <p class="form-help">
                                "Optional human-friendly display name for the family"
                            </p>
                        </div>

                        {/* Engine */}
                        <div class="form-group">
                            <label for="engine" class="form-label">
                                "Engine"
                            </label>
                            <select
                                id="engine"
                                class="form-select"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.engine = if value.is_empty() {
                                            None
                                        } else {
                                            Some(value)
                                        };
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            >
                                <option value="" selected=move || editor_state.tutorial.get().metadata.engine.is_none()>
                                    "-- Select --"
                                </option>
                                <option value="lavaan" selected=move || editor_state.tutorial.get().metadata.engine.as_deref() == Some("lavaan")>
                                    "lavaan"
                                </option>
                                <option value="lme4" selected=move || editor_state.tutorial.get().metadata.engine.as_deref() == Some("lme4")>
                                    "lme4"
                                </option>
                                <option value="glmmTMB" selected=move || editor_state.tutorial.get().metadata.engine.as_deref() == Some("glmmTMB")>
                                    "glmmTMB"
                                </option>
                                <option value="OpenMx" selected=move || editor_state.tutorial.get().metadata.engine.as_deref() == Some("OpenMx")>
                                    "OpenMx"
                                </option>
                                <option value="lcmm" selected=move || editor_state.tutorial.get().metadata.engine.as_deref() == Some("lcmm")>
                                    "lcmm"
                                </option>
                            </select>
                            <p class="form-help">
                                "R package or software used"
                            </p>
                        </div>

                        {/* Covariates */}
                        <div class="form-group">
                            <label for="covariates" class="form-label">
                                "Covariates"
                            </label>
                            <select
                                id="covariates"
                                class="form-select"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.covariates = if value.is_empty() {
                                            None
                                        } else {
                                            Some(value)
                                        };
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            >
                                <option value="" selected=move || editor_state.tutorial.get().metadata.covariates.is_none()>
                                    "-- Select --"
                                </option>
                                <option value="None" selected=move || editor_state.tutorial.get().metadata.covariates.as_deref() == Some("None")>
                                    "None"
                                </option>
                                <option value="TIC" selected=move || editor_state.tutorial.get().metadata.covariates.as_deref() == Some("TIC")>
                                    "TIC (Time-Invariant)"
                                </option>
                                <option value="TVC" selected=move || editor_state.tutorial.get().metadata.covariates.as_deref() == Some("TVC")>
                                    "TVC (Time-Varying)"
                                </option>
                                <option value="Both" selected=move || editor_state.tutorial.get().metadata.covariates.as_deref() == Some("Both")>
                                    "Both TIC & TVC"
                                </option>
                            </select>
                            <p class="form-help">
                                "Covariate type"
                            </p>
                        </div>

                        {/* Outcome Type */}
                        <div class="form-group md:col-span-2">
                            <label for="outcome-type" class="form-label">
                                "Outcome Type"
                            </label>
                            <select
                                id="outcome-type"
                                class="form-select"
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.outcome_type = if value.is_empty() {
                                            None
                                        } else {
                                            Some(value)
                                        };
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            >
                                <option value="" selected=move || editor_state.tutorial.get().metadata.outcome_type.is_none()>
                                    "-- Select --"
                                </option>
                                <option value="Continuous" selected=move || editor_state.tutorial.get().metadata.outcome_type.as_deref() == Some("Continuous")>
                                    "Continuous"
                                </option>
                                <option value="Count" selected=move || editor_state.tutorial.get().metadata.outcome_type.as_deref() == Some("Count")>
                                    "Count"
                                </option>
                                <option value="Binary" selected=move || editor_state.tutorial.get().metadata.outcome_type.as_deref() == Some("Binary")>
                                    "Binary"
                                </option>
                                <option value="Categorical" selected=move || editor_state.tutorial.get().metadata.outcome_type.as_deref() == Some("Categorical")>
                                    "Categorical"
                                </option>
                            </select>
                            <p class="form-help">
                                "Type of outcome variable"
                            </p>
                        </div>
                    </div>
                </div>

                {/* Date and Tags */}
                <div class="border-t border-neutral-200 dark:border-neutral-700 pt-6">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {/* Date */}
                        <div class="form-group">
                            <label for="date" class="form-label">
                                "Date"
                            </label>
                            <input
                                type="date"
                                id="date"
                                class="form-input"
                                prop:value=move || {
                                    editor_state.tutorial.get().metadata.date_iso
                                        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string())
                                }
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.date_iso = Some(value);
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            />
                        </div>

                        {/* Tags */}
                        <div class="form-group">
                            <label for="tags" class="form-label">
                                "Tags"
                            </label>
                            <input
                                type="text"
                                id="tags"
                                class="form-input"
                                placeholder="tag1, tag2, tag3"
                                prop:value=move || {
                                    editor_state.tutorial.get().metadata.tags
                                        .join(", ")
                                }
                                on:input=move |ev| {
                                    let value = event_target_value(&ev);
                                    editor_state.tutorial.update(|t| {
                                        t.metadata.tags = if value.is_empty() {
                                            Vec::new()
                                        } else {
                                            value.split(',').map(|s| s.trim().to_string()).collect()
                                        };
                                    });
                                    editor_state.is_dirty.set(true);
                                }
                            />
                            <p class="form-help">
                                "Comma-separated tags"
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
