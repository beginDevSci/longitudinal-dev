use leptos::prelude::*;

/// Copy-to-clipboard button island for code blocks
///
/// This island hydrates independently on each code block to provide
/// copy functionality without hydrating the entire code block.
///
/// The code content exists in the DOM already (SSG-rendered), so we
/// query it by ID to avoid duplicating it in the WASM bundle.
#[island]
pub fn CopyCodeButton(
    /// The ID of the pre element containing the code to copy
    #[allow(unused_variables)] // Used only in wasm32 builds
    code_id: String,
) -> impl IntoView {
    let (copied, _set_copied) = signal(false);
    let (error, _set_error) = signal(false);

    let copy_action = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::closure::Closure;
            use leptos::wasm_bindgen::JsCast;

            // Reset error state
            _set_error.set(false);

            // Get the code element from the DOM
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(pre_element) = document.get_element_by_id(&code_id) {
                        // Get the text content from the code block
                        if let Some(text) = pre_element.text_content() {
                            // Get clipboard from navigator
                            let navigator = window.navigator();
                            let clipboard = navigator.clipboard();

                            // Write text to clipboard (returns a Promise)
                            let promise = clipboard.write_text(&text);

                            // Handle the promise
                            let set_copied_clone = _set_copied;
                            let set_error_clone = _set_error;

                            let success_closure =
                                Closure::wrap(Box::new(move |_: leptos::wasm_bindgen::JsValue| {
                                    set_copied_clone.set(true);

                                    // Reset copied state after 2 seconds
                                    let set_copied_reset = set_copied_clone;
                                    let timeout_closure = Closure::wrap(Box::new(move || {
                                        set_copied_reset.set(false);
                                    })
                                        as Box<dyn Fn()>);

                                    if let Some(win) = web_sys::window() {
                                        let _ = win
                                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                                timeout_closure.as_ref().unchecked_ref(),
                                                2000,
                                            );
                                        timeout_closure.forget();
                                    }
                                })
                                    as Box<dyn FnMut(leptos::wasm_bindgen::JsValue)>);

                            let error_closure = Closure::wrap(Box::new(
                                move |_err: leptos::wasm_bindgen::JsValue| {
                                    set_error_clone.set(true);
                                },
                            )
                                as Box<dyn FnMut(leptos::wasm_bindgen::JsValue)>);

                            let _ = promise.then2(&success_closure, &error_closure);

                            success_closure.forget();
                            error_closure.forget();
                        }
                    }
                }
            }
        }
    };

    view! {
        <button
            type="button"
            class="copy-btn"
            on:click=copy_action
            title=move || if copied.get() { "Copied!" } else if error.get() { "Failed to copy" } else { "Copy code" }
            aria-label="Copy code to clipboard"
        >
            <Show
                when=move || copied.get()
                fallback=move || view! {
                    <Show
                        when=move || error.get()
                        fallback=|| view! {
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
                            </svg>
                            <span class="ml-1.5">"Copy"</span>
                        }
                    >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/>
                        </svg>
                        <span class="ml-1.5">"Error"</span>
                    </Show>
                }
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M5 13l4 4L19 7"/>
                </svg>
                <span class="ml-1.5">"Copied!"</span>
            </Show>
        </button>
    }
}
