use leptos::prelude::*;

/// Editor modal island for submitting page edit suggestions
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
    let (edits, set_edits) = signal(String::new());
    let (notes, set_notes) = signal(String::new());
    let (contact, set_contact) = signal(String::new());
    let (honeypot, set_honeypot) = signal(String::new()); // Must stay empty
    let (status_message, set_status_message) = signal(String::from(""));
    let (is_submitting, set_is_submitting) = signal(false);
    let (submission_state, set_submission_state) = signal(String::from("idle")); // idle, success, error

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

    // Load from localStorage when modal opens, or use prefill if no saved draft
    #[cfg(target_arch = "wasm32")]
    let slug_clone = slug.clone();
    #[cfg(target_arch = "wasm32")]
    let prefill_clone = prefill_markdown.clone();
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if show_modal.get() {
                if let Some(win) = web_sys::window() {
                    if let Ok(Some(storage)) = win.local_storage() {
                        let key_prefix = format!("edit_suggestion_{}_", slug_clone);

                        // Load edits from localStorage or use prefill
                        if let Ok(Some(saved_edits)) = storage.get_item(&format!("{}edits", key_prefix)) {
                            set_edits.set(saved_edits);
                            set_status_message.set("Draft restored from auto-save".to_string());
                        } else {
                            // Use prefill markdown if no saved draft
                            set_edits.set(prefill_clone.clone());
                            set_status_message.set("Showing page markdown - edit as needed".to_string());
                        }

                        // Load notes
                        if let Ok(Some(saved_notes)) = storage.get_item(&format!("{}notes", key_prefix)) {
                            set_notes.set(saved_notes);
                        }
                        // Load contact
                        if let Ok(Some(saved_contact)) = storage.get_item(&format!("{}contact", key_prefix)) {
                            set_contact.set(saved_contact);
                        }
                    }
                }
            }
        });
    }

    // Auto-save to localStorage as user types
    #[cfg(target_arch = "wasm32")]
    let slug_clone2 = slug.clone();
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if show_modal.get() {
                let key_prefix = format!("edit_suggestion_{}_", slug_clone2);
                if let Some(win) = web_sys::window() {
                    if let Ok(Some(storage)) = win.local_storage() {
                        let _ = storage.set_item(&format!("{}edits", key_prefix), &edits.get());
                        let _ = storage.set_item(&format!("{}notes", key_prefix), &notes.get());
                        let _ = storage.set_item(&format!("{}contact", key_prefix), &contact.get());
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

    // Clear localStorage after successful submission
    #[cfg(target_arch = "wasm32")]
    let slug_clone3 = slug.clone();
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            if submission_state.get() == "success" {
                if let Some(win) = web_sys::window() {
                    if let Ok(Some(storage)) = win.local_storage() {
                        let key_prefix = format!("edit_suggestion_{}_", slug_clone3);
                        let _ = storage.remove_item(&format!("{}edits", key_prefix));
                        let _ = storage.remove_item(&format!("{}notes", key_prefix));
                        let _ = storage.remove_item(&format!("{}contact", key_prefix));
                    }
                }
            }
        });
    }

    // Submit handler using Action to avoid FnOnce issues
    let slug_for_action = slug.clone();
    let page_url_for_action = page_url.clone();
    let baseline_hash_for_action = baseline_hash.clone();

    let submit_action = Action::new(move |_: &()| {
        let slug_submit = slug_for_action.clone();
        let page_url_submit = page_url_for_action.clone();
        let baseline_hash_submit = baseline_hash_for_action.clone();
        let edits_val = edits.get().trim().to_string();

        async move {
            // Validate edits not empty
            if edits_val.is_empty() {
                set_status_message.set("Please provide your suggested changes".to_string());
                set_submission_state.set("error".to_string());
                return;
            }

            // Check honeypot
            if !honeypot.get().is_empty() {
                set_status_message.set("Submission failed validation".to_string());
                set_submission_state.set("error".to_string());
                return;
            }

            set_is_submitting.set(true);
            set_status_message.set("Submitting suggestion...".to_string());
            set_submission_state.set("idle".to_string());

        // Create payload
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast;
            use leptos::wasm_bindgen::JsValue;
            use serde_json::json;
            use wasm_bindgen_futures::JsFuture;

            let spawn_local = leptos::task::spawn_local;

            let payload = json!({
                "slug": slug_submit,
                "page_url": page_url_submit,
                "edits": edits_val,
                "notes": notes.get(),
                "contact": contact.get(),
                "baseline_hash": baseline_hash_submit,
                "website": honeypot.get()
            });

            spawn_local(async move {
                let result: Result<(), JsValue> = async {
                    let window = leptos::web_sys::window().ok_or(JsValue::from_str("No window"))?;

                    let mut opts = leptos::web_sys::RequestInit::new();
                    opts.set_method("POST");
                    opts.set_mode(leptos::web_sys::RequestMode::Cors);

                    let headers = leptos::web_sys::Headers::new()?;
                    headers.set("Content-Type", "application/json")?;
                    opts.set_headers(&headers);

                    let body_str = JsValue::from_str(&payload.to_string());
                    opts.set_body(&body_str);

                    let request = leptos::web_sys::Request::new_with_str_and_init(
                        "https://suggestions-api.swh004.workers.dev/api/suggestions",
                        &opts
                    )?;

                    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
                    let resp: leptos::web_sys::Response = resp_value.dyn_into()?;

                    if resp.ok() {
                        set_submission_state.set("success".to_string());
                        set_status_message.set("Thank you! Your suggestion has been submitted.".to_string());
                        set_edits.set(String::new());
                        set_notes.set(String::new());
                        set_contact.set(String::new());

                        // Close modal after brief delay to show success message
                        set_timeout(
                            move || {
                                set_show_modal.set(false);
                                set_submission_state.set("idle".to_string());
                                set_status_message.set(String::new());
                            },
                            std::time::Duration::from_millis(2000),
                        );
                    } else {
                        let text_promise = resp.text()?;
                        let text_value = JsFuture::from(text_promise).await?;
                        let error_msg = text_value.as_string().unwrap_or_else(|| "Submission failed".to_string());
                        set_submission_state.set("error".to_string());
                        set_status_message.set(error_msg);
                    }

                    Ok(())
                }.await;

                if let Err(e) = result {
                    set_submission_state.set("error".to_string());
                    set_status_message.set(format!("Network error: {:?}", e));
                }

                set_is_submitting.set(false);
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            set_is_submitting.set(false);
        }
        }
    });

    let close_action = move || {
        if submission_state.get() != "success" {
            // Keep draft if not successfully submitted
        }
        set_show_modal.set(false);
        set_submission_state.set("idle".to_string());
        set_status_message.set(String::new());
    };

    // Prevent clicks inside modal from closing it
    let stop_propagation = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation();
    };

    // Check if submit should be disabled
    let submit_disabled = move || {
        edits.get().trim().is_empty() || is_submitting.get()
    };

    let slug_display = slug.clone();
    let page_url_display = page_url.clone();

    view! {
        <Show when=move || show_modal.get()>
            <div
                class="fixed inset-0 z-[100000] flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
                on:click=move |_| close_action()
            >
                <div
                    class="w-full max-w-4xl max-h-[90vh] flex flex-col bg-white dark:bg-gray-900 rounded-xl shadow-2xl overflow-hidden border border-gray-200 dark:border-gray-700"
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
                                <h3 class="text-xl font-semibold text-gray-900 dark:text-gray-100">"Suggest Page Edit"</h3>
                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                                    {slug_display.clone()}
                                </p>
                            </div>
                        </div>
                        <button
                            class="p-2 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors group"
                            on:click=move |_| close_action()
                            title="Close"
                        >
                            <svg class="w-5 h-5 text-gray-400 group-hover:text-gray-600 dark:text-gray-500 dark:group-hover:text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>

                    // Form content
                    <div class="flex-1 overflow-y-auto p-6 space-y-4">
                        // Page info display
                        <div class="text-sm text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 rounded-lg p-3">
                            <div class="font-medium mb-1">"Page:"</div>
                            <div class="text-xs break-all">{page_url_display.clone()}</div>
                        </div>

                        // Edits textarea (required)
                        <div>
                            <label for="edits" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Suggested Changes"
                                <span class="text-red-500">"*"</span>
                            </label>
                            <textarea
                                id="edits"
                                class="w-full h-48 px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-gray-100 font-mono text-sm leading-relaxed outline-none focus:ring-2 focus:ring-primary/50 resize-y"
                                placeholder="Describe the changes you'd like to suggest...

Examples:
- Fix typo in section X
- Update code example to use newer syntax
- Clarify explanation of concept Y"
                                prop:value=move || edits.get()
                                on:input=move |ev| {
                                    set_edits.set(event_target_value(&ev));
                                }
                            />
                        </div>

                        // Notes textarea (optional)
                        <div>
                            <label for="notes" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Additional Notes"
                                <span class="text-xs text-gray-500 ml-2">"(optional)"</span>
                            </label>
                            <textarea
                                id="notes"
                                class="w-full h-24 px-4 py-3 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-gray-100 text-sm leading-relaxed outline-none focus:ring-2 focus:ring-primary/50 resize-y"
                                placeholder="Any additional context or notes..."
                                prop:value=move || notes.get()
                                on:input=move |ev| {
                                    set_notes.set(event_target_value(&ev));
                                }
                            />
                        </div>

                        // Contact input (optional)
                        <div>
                            <label for="contact" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                "Contact"
                                <span class="text-xs text-gray-500 ml-2">"(optional - email or GitHub handle)"</span>
                            </label>
                            <input
                                type="text"
                                id="contact"
                                class="w-full px-4 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-gray-100 text-sm outline-none focus:ring-2 focus:ring-primary/50"
                                placeholder="your@email.com or @github-username"
                                prop:value=move || contact.get()
                                on:input=move |ev| {
                                    set_contact.set(event_target_value(&ev));
                                }
                            />
                        </div>

                        // Honeypot field (hidden)
                        <input
                            type="text"
                            name="website"
                            tabindex="-1"
                            autocomplete="off"
                            style="position: absolute; left: -9999px; width: 1px; height: 1px;"
                            prop:value=move || honeypot.get()
                            on:input=move |ev| {
                                set_honeypot.set(event_target_value(&ev));
                            }
                        />

                        // Status message
                        <Show when=move || !status_message.get().is_empty()>
                            <div class={move || {
                                let base = "px-4 py-3 rounded-lg text-sm";
                                match submission_state.get().as_str() {
                                    "success" => format!("{} bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-200", base),
                                    "error" => format!("{} bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-200", base),
                                    _ => format!("{} bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-200", base),
                                }
                            }}>
                                {move || status_message.get()}
                            </div>
                        </Show>
                    </div>

                    // Footer with actions
                    <div class="flex items-center justify-between px-6 py-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
                        <div class="text-xs text-gray-500 dark:text-gray-400">
                            "Changes auto-save as you type"
                        </div>
                        <div class="flex items-center gap-3">
                            <button
                                class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
                                on:click=move |_| close_action()
                            >
                                {move || if submission_state.get() == "success" { "Close" } else { "Cancel" }}
                            </button>
                            <Show when=move || submission_state.get() != "success">
                                <button
                                    class="px-6 py-2 text-sm font-medium text-white bg-primary hover:bg-primary/90 rounded-lg transition-colors shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-primary"
                                    on:click=move |_| { let _ = submit_action.dispatch(()); }
                                    disabled=submit_disabled
                                >
                                    {move || if is_submitting.get() { "Submitting..." } else { "Submit Suggestion" }}
                                </button>
                            </Show>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
