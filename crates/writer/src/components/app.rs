use super::{MetadataForm, SectionEditor, SectionNav};
use crate::{EditorState, MarkdownExporter, Tutorial, TutorialValidator};
use leptos::prelude::*;

/// Main Writer application component
///
/// Provides the shell for the tutorial writer with:
/// - Header with title and export button
/// - Left sidebar for section navigation
/// - Main content area for editing
/// - Autosave indicator and validation status
#[island]
pub fn WriterApp() -> impl IntoView {
    // Initialize editor state
    let editor_state = EditorState::new();

    // Load draft from localStorage on mount
    Effect::new(move |_| {
        if let Some(tutorial) = load_from_local_storage() {
            editor_state.tutorial.set(tutorial);
        }
    });

    // Autosave every 30 seconds
    Effect::new(move |_| {
        let interval = gloo_timers::callback::Interval::new(30_000, move || {
            if editor_state.is_dirty.get() {
                let tutorial = editor_state.tutorial.get();
                save_to_local_storage(&tutorial);
                editor_state
                    .last_saved
                    .set(Some(chrono::Utc::now().to_rfc3339()));
                editor_state.is_dirty.set(false);
            }
        });

        // Keep interval alive
        std::mem::forget(interval);
    });

    // Validation
    let validation_issues = Memo::new(move |_| {
        let tutorial = editor_state.tutorial.get();
        TutorialValidator::validate(&tutorial)
    });

    let can_export = Memo::new(move |_| {
        let tutorial = editor_state.tutorial.get();
        TutorialValidator::can_export(&tutorial)
    });

    // Current section (default to metadata)
    let current_section = RwSignal::new(None);

    view! {
        <div class="writer-app min-h-screen bg-neutral-50 dark:bg-neutral-900">
            {/* Header */}
            <header class="bg-white dark:bg-neutral-800 border-b border-neutral-200 dark:border-neutral-700 sticky top-0 z-10">
                <div class="max-w-screen-2xl mx-auto px-6 py-4 flex items-center justify-between">
                    <div class="flex items-center gap-4">
                        <h1 class="text-2xl font-bold text-primary">
                            "Tutorial Writer"
                        </h1>

                        {/* Autosave indicator */}
                        {move || {
                            if let Some(last_saved) = editor_state.last_saved.get() {
                                view! {
                                    <span class="text-sm text-muted">
                                        "Saved " {format_time_ago(&last_saved)}
                                    </span>
                                }.into_any()
                            } else if editor_state.is_dirty.get() {
                                view! {
                                    <span class="text-sm text-amber-600 dark:text-amber-400">
                                        "Unsaved changes"
                                    </span>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </div>

                    <div class="flex items-center gap-4">
                        {/* Validation status */}
                        {move || {
                            let issues = validation_issues.get();
                            let error_count = issues.iter().filter(|i| i.level == crate::ValidationLevel::Error).count();
                            let warning_count = issues.iter().filter(|i| i.level == crate::ValidationLevel::Warning).count();

                            view! {
                                <div class="flex items-center gap-2 text-sm">
                                    {if error_count > 0 {
                                        view! {
                                            <span class="text-red-600 dark:text-red-400">
                                                {error_count} " error" {if error_count != 1 { "s" } else { "" }}
                                            </span>
                                        }.into_any()
                                    } else if warning_count > 0 {
                                        view! {
                                            <span class="text-amber-600 dark:text-amber-400">
                                                {warning_count} " warning" {if warning_count != 1 { "s" } else { "" }}
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="text-green-600 dark:text-green-400">
                                                "✓ Ready to export"
                                            </span>
                                        }.into_any()
                                    }}
                                </div>
                            }
                        }}

                        {/* Export button */}
                        <button
                            class="btn btn-primary"
                            class:opacity-50=move || !can_export.get()
                            disabled=move || !can_export.get()
                            on:click=move |_| {
                                if can_export.get() {
                                    let tutorial = editor_state.tutorial.get();
                                    if let Ok(markdown) = MarkdownExporter::export(&tutorial) {
                                        download_markdown(&markdown, &tutorial.title);
                                    }
                                }
                            }
                        >
                            "Export Markdown"
                        </button>
                    </div>
                </div>
            </header>

            {/* Main content */}
            <div class="flex max-w-screen-2xl mx-auto">
                {/* Left sidebar - Section navigation */}
                <aside class="w-64 bg-white dark:bg-neutral-800 border-r border-neutral-200 dark:border-neutral-700 min-h-[calc(100vh-73px)] sticky top-[73px]">
                    <SectionNav
                        current_section=current_section
                        validation_issues=validation_issues
                    />
                </aside>

                {/* Main editor area */}
                <main class="flex-1 p-6">
                    {move || {
                        match current_section.get() {
                            None => view! { <MetadataForm editor_state=editor_state /> }.into_any(),
                            Some(section_type) => view! {
                                <SectionEditor
                                    section_type=section_type
                                    editor_state=editor_state
                                />
                            }.into_any(),
                        }
                    }}
                </main>
            </div>
        </div>
    }
}

/// Load tutorial from localStorage
fn load_from_local_storage() -> Option<Tutorial> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let json = storage.get_item("writer_draft").ok()??;
    serde_json::from_str(&json).ok()
}

/// Save tutorial to localStorage
fn save_to_local_storage(tutorial: &Tutorial) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(tutorial) {
                let _ = storage.set_item("writer_draft", &json);
            }
        }
    }
}

/// Format time ago (simple version)
fn format_time_ago(_iso_string: &str) -> String {
    // For MVP, just show "moments ago"
    "moments ago".to_string()
}

/// Download markdown file
fn download_markdown(content: &str, title: &str) {
    use wasm_bindgen::JsCast;

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            // Create blob
            let array = js_sys::Array::new();
            array.push(&wasm_bindgen::JsValue::from_str(content));

            let blob_options = web_sys::BlobPropertyBag::new();
            blob_options.set_type("text/markdown;charset=utf-8");

            if let Ok(blob) =
                web_sys::Blob::new_with_str_sequence_and_options(&array, &blob_options)
            {
                // Create download link
                if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                    if let Some(a) = document
                        .create_element("a")
                        .ok()
                        .and_then(|e| e.dyn_into::<web_sys::HtmlAnchorElement>().ok())
                    {
                        a.set_href(&url);
                        a.set_download(&format!("{}.md", title.to_lowercase().replace(" ", "-")));
                        a.click();
                        web_sys::Url::revoke_object_url(&url).ok();
                    }
                }
            }
        }
    }
}
