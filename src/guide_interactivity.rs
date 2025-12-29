//! Guide interactivity island for anchor navigation and copy buttons.
//!
//! This island hydrates on guide pages to provide:
//! - Auto-expand collapsible modules when navigating to anchors inside them
//! - Copy buttons for code blocks (unified behavior with tutorial copy buttons)
//! - Smooth scrolling behavior with reduced-motion support
//!
//! ## Future Unification Note
//! The copy button logic here mirrors `CopyCodeButton` in copy_code_button.rs.
//! If the shared behavior becomes more complex, consider extracting a shared
//! clipboard helper module that both can use.

use leptos::prelude::*;

/// Island component for guide page interactivity.
///
/// Handles:
/// 1. **Anchor navigation**: Auto-expands `<details class="tutorial-module">` when
///    the target element is inside a collapsed module
/// 2. **Copy buttons**: Injects copy buttons into code blocks with `guide-code-*` IDs
/// 3. **Smooth scrolling**: Scrolls to anchor targets smoothly (respects reduced-motion)
#[island]
pub fn GuideInteractivity() -> impl IntoView {
    // Run setup effects on mount (client-side only)
    #[cfg(target_arch = "wasm32")]
    {
        // Effect that runs once on mount
        Effect::new(move |_| {
            setup_anchor_handling();
            setup_copy_buttons();

            // Handle initial hash on page load
            handle_current_hash();
        });
    }

    // This island is purely for side effects, return empty view
}

// ============================================================================
// Copy Button Setup
// ============================================================================

/// Set up copy buttons for all code blocks.
#[cfg(target_arch = "wasm32")]
fn setup_copy_buttons() {
    use leptos::wasm_bindgen::closure::Closure;
    use leptos::wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    // Find all code block wrappers
    let wrappers = document.query_selector_all(".code-block-wrapper");
    let Ok(wrappers) = wrappers else {
        return;
    };

    for i in 0..wrappers.length() {
        let Some(wrapper) = wrappers.get(i) else {
            continue;
        };
        let Some(wrapper_el) = wrapper.dyn_ref::<web_sys::HtmlElement>() else {
            continue;
        };

        // Get the code ID from data attribute
        let Some(code_id) = wrapper_el.get_attribute("data-code-id") else {
            continue;
        };

        // Check if copy button already exists
        if wrapper_el
            .query_selector(".copy-btn")
            .ok()
            .flatten()
            .is_some()
        {
            continue;
        }

        // Create copy button with proper accessibility attributes
        let button = match document.create_element("button") {
            Ok(el) => el,
            Err(_) => continue,
        };

        let _ = button.set_attribute("type", "button");
        let _ = button.set_attribute("class", "copy-btn guide-copy-btn");
        let _ = button.set_attribute("title", "Copy code");
        let _ = button.set_attribute("aria-label", "Copy code to clipboard");
        set_copy_button_default_state(&button);

        // Add click handler
        let code_id_clone = code_id.clone();
        let button_clone = button.clone();
        let click_handler = Closure::wrap(Box::new(move |_: web_sys::Event| {
            copy_code_to_clipboard(&code_id_clone, &button_clone);
        }) as Box<dyn Fn(web_sys::Event)>);

        let _ = button.add_event_listener_with_callback(
            "click",
            click_handler.as_ref().unchecked_ref(),
        );
        click_handler.forget();

        // Insert button at the beginning of the wrapper
        let _ = wrapper_el.insert_before(&button, wrapper_el.first_child().as_ref());
    }
}

/// Set button to default "Copy" state.
#[cfg(target_arch = "wasm32")]
fn set_copy_button_default_state(button: &web_sys::Element) {
    button.set_inner_html(
        r#"<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/>
        </svg>
        <span class="ml-1.5">Copy</span>"#,
    );
    let _ = button.set_attribute("title", "Copy code");
    let _ = button.set_attribute("aria-label", "Copy code to clipboard");
}

/// Set button to "Copied!" success state.
#[cfg(target_arch = "wasm32")]
fn set_copy_button_success_state(button: &web_sys::Element) {
    button.set_inner_html(
        r#"<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M5 13l4 4L19 7"/>
        </svg>
        <span class="ml-1.5">Copied!</span>"#,
    );
    let _ = button.set_attribute("title", "Copied!");
    let _ = button.set_attribute("aria-label", "Code copied to clipboard");
}

/// Set button to error state.
#[cfg(target_arch = "wasm32")]
fn set_copy_button_error_state(button: &web_sys::Element) {
    button.set_inner_html(
        r#"<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 18L18 6M6 6l12 12"/>
        </svg>
        <span class="ml-1.5">Error</span>"#,
    );
    let _ = button.set_attribute("title", "Failed to copy");
    let _ = button.set_attribute("aria-label", "Failed to copy code");
}

/// Copy code content to clipboard.
#[cfg(target_arch = "wasm32")]
fn copy_code_to_clipboard(code_id: &str, button: &web_sys::Element) {
    use leptos::wasm_bindgen::closure::Closure;
    use leptos::wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    // Get the pre element by ID
    let Some(pre_element) = document.get_element_by_id(code_id) else {
        return;
    };

    // Get text content
    let Some(text) = pre_element.text_content() else {
        return;
    };

    // Copy to clipboard
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();
    let promise = clipboard.write_text(&text);

    // Handle success
    let button_clone = button.clone();
    let success_closure = Closure::wrap(Box::new(move |_: leptos::wasm_bindgen::JsValue| {
        set_copy_button_success_state(&button_clone);

        // Reset after 2 seconds (matches tutorial behavior)
        let button_reset = button_clone.clone();
        let timeout_closure = Closure::wrap(Box::new(move || {
            set_copy_button_default_state(&button_reset);
        }) as Box<dyn Fn()>);

        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_closure.as_ref().unchecked_ref(),
                2000,
            );
            timeout_closure.forget();
        }
    }) as Box<dyn FnMut(leptos::wasm_bindgen::JsValue)>);

    // Handle error - show error state, then reset after 2 seconds
    let button_error = button.clone();
    let error_closure = Closure::wrap(Box::new(move |_err: leptos::wasm_bindgen::JsValue| {
        set_copy_button_error_state(&button_error);

        // Reset after 2 seconds
        let button_reset = button_error.clone();
        let timeout_closure = Closure::wrap(Box::new(move || {
            set_copy_button_default_state(&button_reset);
        }) as Box<dyn Fn()>);

        if let Some(win) = web_sys::window() {
            let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                timeout_closure.as_ref().unchecked_ref(),
                2000,
            );
            timeout_closure.forget();
        }
    }) as Box<dyn FnMut(leptos::wasm_bindgen::JsValue)>);

    let _ = promise.then2(&success_closure, &error_closure);
    success_closure.forget();
    error_closure.forget();
}

// ============================================================================
// Anchor Navigation
// ============================================================================

/// Set up anchor navigation handling.
#[cfg(target_arch = "wasm32")]
fn setup_anchor_handling() {
    use leptos::wasm_bindgen::closure::Closure;
    use leptos::wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    // Handle hashchange events (for back/forward navigation)
    let hashchange_handler = Closure::wrap(Box::new(move |_: web_sys::Event| {
        handle_current_hash();
    }) as Box<dyn Fn(web_sys::Event)>);

    let _ = window.add_event_listener_with_callback(
        "hashchange",
        hashchange_handler.as_ref().unchecked_ref(),
    );
    hashchange_handler.forget();

    // Intercept anchor link clicks to prevent double-click issue
    let anchor_links = document.query_selector_all("a[href^='#']");
    if let Ok(links) = anchor_links {
        for i in 0..links.length() {
            if let Some(link) = links.get(i) {
                let click_handler = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    handle_anchor_click(event);
                }) as Box<dyn Fn(web_sys::Event)>);

                let _ = link.add_event_listener_with_callback(
                    "click",
                    click_handler.as_ref().unchecked_ref(),
                );
                click_handler.forget();
            }
        }
    }
}

/// Handle anchor link clicks.
#[cfg(target_arch = "wasm32")]
fn handle_anchor_click(event: web_sys::Event) {
    use leptos::wasm_bindgen::JsCast;

    let Some(target) = event.target() else {
        return;
    };
    let Some(anchor) = target.dyn_ref::<web_sys::HtmlAnchorElement>() else {
        return;
    };

    let href = anchor.get_attribute("href").unwrap_or_default();
    if !href.starts_with('#') {
        return;
    }

    let target_id = &href[1..];
    if target_id.is_empty() {
        return;
    }

    // Prevent default navigation
    event.prevent_default();

    // Navigate to anchor (this expands modules and scrolls)
    navigate_to_anchor(target_id);

    // Update URL hash using pushState (doesn't trigger hashchange)
    if let Some(window) = web_sys::window() {
        if let Ok(history) = window.history() {
            let _ = history.push_state_with_url(
                &leptos::wasm_bindgen::JsValue::NULL,
                "",
                Some(&href),
            );
        }
    }
}

/// Handle the current URL hash (for page load and back/forward navigation).
#[cfg(target_arch = "wasm32")]
fn handle_current_hash() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(hash) = window.location().hash() else {
        return;
    };

    if hash.is_empty() || hash == "#" {
        return;
    }

    let target_id = &hash[1..]; // Remove leading #
    navigate_to_anchor(target_id);
}

/// Navigate to an anchor, expanding any parent modules first.
#[cfg(target_arch = "wasm32")]
fn navigate_to_anchor(target_id: &str) {
    use leptos::wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };

    // Find the target element
    let Some(target_element) = document.get_element_by_id(target_id) else {
        return;
    };

    // Check if target is inside a collapsed <details class="tutorial-module">
    expand_parent_modules(&target_element);

    // Check user's motion preference
    let prefers_reduced_motion = check_prefers_reduced_motion(&window);

    // Small delay to allow DOM updates after expanding
    let target_id_owned = target_id.to_string();
    let scroll_closure = leptos::wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) = document.get_element_by_id(&target_id_owned) {
                    // Scroll to element, respecting reduced-motion preference
                    let options = web_sys::ScrollIntoViewOptions::new();
                    if prefers_reduced_motion {
                        options.set_behavior(web_sys::ScrollBehavior::Instant);
                    } else {
                        options.set_behavior(web_sys::ScrollBehavior::Smooth);
                    }
                    options.set_block(web_sys::ScrollLogicalPosition::Start);
                    element.scroll_into_view_with_scroll_into_view_options(&options);
                }
            }
        }
    }) as Box<dyn Fn()>);

    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        scroll_closure.as_ref().unchecked_ref(),
        50,
    );
    scroll_closure.forget();
}

/// Check if user prefers reduced motion.
#[cfg(target_arch = "wasm32")]
fn check_prefers_reduced_motion(window: &web_sys::Window) -> bool {
    window
        .match_media("(prefers-reduced-motion: reduce)")
        .ok()
        .flatten()
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

/// Expand any parent <details class="tutorial-module"> elements.
#[cfg(target_arch = "wasm32")]
fn expand_parent_modules(element: &web_sys::Element) {
    let mut current = element.parent_element();

    while let Some(parent) = current {
        // Check if this is a tutorial-module details element
        if parent.tag_name().to_lowercase() == "details" {
            if let Some(class_list) = parent.get_attribute("class") {
                if class_list.contains("tutorial-module") {
                    // Expand it by setting the open attribute
                    let _ = parent.set_attribute("open", "");
                }
            }
        }

        current = parent.parent_element();
    }
}
