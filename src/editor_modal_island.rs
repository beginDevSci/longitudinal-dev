use leptos::prelude::*;

/// Editor modal island that appears at page level (centered)
///
/// Listens for "open-editor-modal" custom event from EditPageButton.
/// Renders with fixed positioning at the document root level for proper centering.
///
/// Props:
/// - slug: Tutorial slug for identification
/// - page_url: Full URL of the page being edited
/// - prefill_markdown: Rendered markdown content to prefill the editor
/// - baseline_hash: Hash of the prefilled content for change detection
#[island]
pub fn EditorModalIsland(
    slug: String,
    page_url: String,
    prefill_markdown: String,
    baseline_hash: String,
) -> impl IntoView {
    let (show_modal, set_show_modal) = signal(false);
    let (content, set_content) = signal(String::new());
    let (notes, set_notes) = signal(String::new());
    let (contact, set_contact) = signal(String::new());
    let (status, set_status) = signal(String::from("Ready"));

    // Listen for custom event to open modal
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::closure::Closure;
        use leptos::wasm_bindgen::JsCast;

        Effect::new(move |_| {
            if let Some(window) = web_sys::window() {
                let set_show = set_show_modal;
                let closure = Closure::wrap(Box::new(move |_evt: web_sys::Event| {
                    set_show.set(true);
                }) as Box<dyn Fn(web_sys::Event)>);

                let _ = window.add_event_listener_with_callback(
                    "open-editor-modal",
                    closure.as_ref().unchecked_ref(),
                );

                // Keep closure alive
                closure.forget();
            }
        });
    }

    // Load from localStorage or prefill when modal opens
    let prefill = prefill_markdown.clone();
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if show_modal.get() {
                if let Some(win) = web_sys::window() {
                    if let Ok(Some(storage)) = win.local_storage() {
                        // Try to load from localStorage first (for draft recovery)
                        if let Ok(Some(saved)) = storage.get_item("blog_editor_content") {
                            set_content.set(saved);
                            set_status.set("Loaded from auto-save".to_string());
                            return;
                        }
                    }
                }
                // Otherwise prefill with the rendered markdown
                set_content.set(prefill.clone());
                set_status.set("Ready to edit".to_string());
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // SSR: just set prefill
        let _ = show_modal.get(); // track dependency
        set_content.set(prefill.clone());
    }

    // Auto-save to localStorage as user types
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            let text = content.get();
            if !text.is_empty() && show_modal.get() {
                if let Some(win) = web_sys::window() {
                    if let Ok(Some(storage)) = win.local_storage() {
                        let _ = storage.set_item("blog_editor_content", &text);
                        set_status.set("Auto-saved".to_string());
                    }
                }
            }
        });
    }

    // Body scroll lock when modal is open
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(body) = doc.body() {
                    let style = body.style();
                    if show_modal.get() {
                        let _ = style.set_property("overflow", "hidden");
                    } else {
                        let _ = style.remove_property("overflow");
                    }
                }
            }
        });
    }

    // Toolbar helpers
    let wrap_selection = move |prefix: &str, suffix: &str| {
        let val = content.get();
        let new_val = format!("{prefix}{val}{suffix}");
        set_content.set(new_val);
    };

    let insert_heading = move |level: u8| {
        let hashes = "#".repeat(level as usize);
        let val = content.get();
        let new_val = if val.is_empty() {
            format!("{hashes} ")
        } else {
            format!("{val}\n{hashes} ")
        };
        set_content.set(new_val);
    };

    let save_action = move || {
        set_status.set("Saved!".to_string());
        set_show_modal.set(false);
    };

    let close_action = move || {
        set_show_modal.set(false);
    };

    // Prevent clicks inside modal from closing it
    let stop_propagation = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    view! {
        <Show when=move || show_modal.get()>
            <div
                class="fixed inset-0 z-[100000] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
                on:click=move |_| close_action()
            >
                <div
                    class="w-full max-w-5xl max-h-[90vh] flex flex-col bg-white dark:bg-gray-900 rounded-xl shadow-2xl overflow-hidden border border-gray-200 dark:border-gray-700"
                    on:click=stop_propagation
                >
                    // Header
                    <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
                        <div class="flex items-center gap-3">
                            <div class="p-2 bg-primary/10 rounded-lg">
                                <svg class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                </svg>
                            </div>
                            <div>
                                <h3 class="text-xl font-semibold text-gray-900 dark:text-gray-100">"Edit Page"</h3>
                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{move || status.get()}</p>
                            </div>
                        </div>
                        <button
                            class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors group"
                            on:click=move |_| close_action()
                            title="Close editor"
                        >
                            <svg class="w-5 h-5 text-gray-400 group-hover:text-gray-600 dark:text-gray-500 dark:group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    // Toolbar
                    <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
                        <div class="flex gap-1">
                            <button
                                class="px-3 py-1.5 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors font-semibold"
                                title="Bold"
                                on:click=move |_| wrap_selection("**", "**")
                            >
                                "B"
                            </button>
                            <button
                                class="px-3 py-1.5 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors italic"
                                title="Italic"
                                on:click=move |_| wrap_selection("*", "*")
                            >
                                "I"
                            </button>
                            <button
                                class="px-3 py-1.5 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors font-mono"
                                title="Code"
                                on:click=move |_| wrap_selection("`", "`")
                            >
                                "<>"
                            </button>
                        </div>

                        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"/>

                        <div class="flex gap-1">
                            <button
                                class="px-2 py-1.5 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                title="Heading 1"
                                on:click=move |_| insert_heading(1)
                            >
                                "H1"
                            </button>
                            <button
                                class="px-2 py-1.5 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                title="Heading 2"
                                on:click=move |_| insert_heading(2)
                            >
                                "H2"
                            </button>
                            <button
                                class="px-2 py-1.5 text-sm rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                                title="Heading 3"
                                on:click=move |_| insert_heading(3)
                            >
                                "H3"
                            </button>
                        </div>

                        <div class="ml-auto text-xs text-gray-500 dark:text-gray-400">
                            "Changes save automatically"
                        </div>
                    </div>

                    // Editor area
                    <div class="flex-1 overflow-hidden">
                        <textarea
                            class="w-full h-full min-h-[400px] p-6 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed outline-none resize-none"
                            placeholder="Start typing your content here...

You can use markdown formatting:
- **bold** or *italic*
- # Headings
- `code`
- And more!"
                            prop:value=move || content.get()
                            on:input=move |ev| {
                                set_content.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    // Footer with actions
                    <div class="flex items-center justify-between px-6 py-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
                        <div class="text-xs text-gray-500 dark:text-gray-400">
                            "Your changes are saved locally in your browser"
                        </div>
                        <div class="flex items-center gap-3">
                            <button
                                class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
                                on:click=move |_| close_action()
                            >
                                "Cancel"
                            </button>
                            <button
                                class="px-6 py-2 text-sm font-medium text-white bg-primary hover:bg-primary/90 rounded-lg transition-colors shadow-lg hover:shadow-xl"
                                on:click=move |_| save_action()
                            >
                                "Done"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
