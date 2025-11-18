use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::EditPageButton;

/// Table of Contents item for navigation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TocItem {
    pub id: String,
    pub title: String,
    pub level: u8, // 1 = h1, 2 = h2 (main sections)
}

/// Code download data for the Downloads section
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CodeDownloadData {
    pub markdown: String,
    pub r_code: Option<String>,
    pub python_code: Option<String>,
}

/// Right-side Table of Contents with navigation, contribute, and downloads sections.
///
/// This component provides:
/// - "On This Page" navigation links to main sections
/// - "Contribute" section with edit/discussion links
/// - "Downloads" section for code files (Markdown, R, Python)
///
/// Styling uses semantic theme tokens defined in style/input.css:
/// - `.aside-container` - Dark background with transparency and blur
/// - `.aside-heading` - Section headings
/// - `.aside-link` - Navigation links with teal accent on hover
/// - `.aside-action` - Action links (edit/discuss) with icons
/// - `.download-btn` - Download buttons with teal hover state
///
/// All colors integrate with the main teal theme system.
#[island]
pub fn TableOfContents(
    toc_items: Vec<TocItem>,
    #[prop(optional)] download_data: Option<CodeDownloadData>,
    #[prop(optional)] repo_url: Option<String>,
    slug: String,
) -> impl IntoView {
    let download_data = download_data.unwrap_or_default();
    let repo_url = repo_url.unwrap_or_default();
    let has_downloads = !download_data.markdown.is_empty();

    let (is_collapsed, set_is_collapsed) = signal(false);

    view! {
        <aside
            class="hidden md:block sticky top-6 h-[calc(100vh-3rem)]"
            style=move || {
                let width = if is_collapsed.get() { 48 } else { 280 };
                format!("width: {width}px;")
            }
        >
            <div class="relative h-full">
                <div
                    id="toc-panel"
                    class=move || {
                        format!(
                            "right-aside-panel {}",
                            if is_collapsed.get() {
                                "right-aside-panel--collapsed"
                            } else {
                                ""
                            }
                        )
                    }
                >
                    <div class="aside-container h-full">
                        <nav aria-label="On this page" class="h-full overflow-y-auto py-2">
                        // On This Page section
                        <div class="px-3">
                            <div class="aside-heading mb-1">"On this page"</div>
                            <ul class="space-y-0.5">
                                {toc_items
                                    .into_iter()
                                    .map(|item| {
                                        let padding_left = match item.level {
                                            1 => 4,
                                            2 => 12,
                                            _ => 20,
                                        };
                                        view! {
                                            <li>
                                                <TocLink
                                                    target_id=item.id
                                                    title=item.title
                                                    padding_left=padding_left
                                                />
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>

                        // Contribute section
                        <div class="aside-divider"/>
                        <div class="px-3 pb-2">
                            <div class="aside-heading">
                                "Contribute"
                            </div>
                            <ul class="space-y-1">
                                {if !repo_url.is_empty() {
                                    view! {
                                        <>
                                            {if crate::config::ENABLE_SUGGESTIONS {
                                                view! {
                                                    <li>
                                                        <EditPageButton slug=slug.clone()/>
                                                    </li>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }}
                                            <li>
                                                <button
                                                    type="button"
                                                    class="aside-action aside-action--disabled"
                                                    disabled
                                                    aria-disabled="true"
                                                    title="Discussion forum coming soon"
                                                >
                                                    <svg
                                                        class="aside-action-icon"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        viewBox="0 0 24 24"
                                                    >
                                                        <path
                                                            stroke-linecap="round"
                                                            stroke-linejoin="round"
                                                            stroke-width="1.8"
                                                            d="M8 10h8M8 14h5M21 12c0 4.418-4.03 8-9 8-1.084 0-2.12-.172-3.082-.488L3 20l1.488-5.918C4.172 13.12 4 12.084 4 11c0-4.418 4.03-8 9-8s9 3.582 9 9z"
                                                        />
                                                    </svg>
                                                    <span>"Join the discussion"</span>
                                                </button>
                                            </li>
                                        </>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}
                            </ul>
                        </div>

                        // Downloads section
                        {if has_downloads {
                            view! {
                                <>
                                    <div class="aside-divider"/>
                                    <div class="px-3 pb-2">
                                        <div class="aside-heading">
                                            "Downloads"
                                        </div>
                                        <div class="px-2">
                                            <div class="py-2">
                                                <div class="text-xs mb-2" style="color: var(--color-aside-text-secondary);">
                                                    "Formats:"
                                                </div>
                                                <div class="flex gap-2 flex-wrap">
                                                    // Markdown download button
                                                    <DownloadButton
                                                        content=download_data.markdown.clone()
                                                        filename="code.md".to_string()
                                                        label="Markdown".to_string()
                                                        icon_type="markdown".to_string()
                                                    />

                                                    // R download button
                                                    {download_data
                                                        .r_code
                                                        .as_ref()
                                                        .map(|code| {
                                                            view! {
                                                                <DownloadButton
                                                                    content=code.clone()
                                                                    filename="code.R".to_string()
                                                                    label="R".to_string()
                                                                    icon_type="r".to_string()
                                                                />
                                                            }
                                                        })}

                                                    // Python download button
                                                    {download_data
                                                        .python_code
                                                        .as_ref()
                                                        .map(|code| {
                                                            view! {
                                                                <DownloadButton
                                                                    content=code.clone()
                                                                    filename="code.py".to_string()
                                                                    label="Python".to_string()
                                                                    icon_type="python".to_string()
                                                                />
                                                            }
                                                        })}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </>
                            }.into_any()
                        } else {
                            ().into_any()
                        }}
                        </nav>
                    </div>
                </div>
                <button
                    type="button"
                    class=move || {
                        format!(
                            "right-aside-toggle {}",
                            if is_collapsed.get() {
                                "right-aside-toggle--collapsed"
                            } else {
                                ""
                            }
                        )
                    }
                    on:click=move |_| {
                        set_is_collapsed.update(|state| *state = !*state);
                    }
                    aria-label=move || {
                        if is_collapsed.get() {
                            "Expand supplemental panel".to_string()
                        } else {
                            "Collapse supplemental panel".to_string()
                        }
                    }
                    aria-expanded=move || (!is_collapsed.get()).to_string()
                    aria-controls="toc-panel"
                >
                    <svg
                        class="right-aside-toggle__icon"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                    >
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M9 5l7 7-7 7"/>
                    </svg>
                </button>
            </div>
        </aside>
    }
}

/// Download button component for code files (interactive island)
#[island]
pub fn DownloadButton(
    content: String,
    filename: String,
    label: String,
    icon_type: String,
) -> impl IntoView {
    let download_action = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::Array;
            use wasm_bindgen::JsCast;
            use wasm_bindgen::JsValue;
            use web_sys::{Blob, HtmlAnchorElement, Url};

            // Create blob from content
            let array = Array::new();
            array.push(&JsValue::from_str(&content));

            if let Ok(blob) = Blob::new_with_str_sequence(&array) {
                // Get window and document
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        // Create hidden anchor element
                        if let Ok(element) = document.create_element("a") {
                            if let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() {
                                // Create object URL
                                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                                    anchor.set_href(&url);
                                    anchor.set_download(&filename);
                                    let _ = anchor.set_attribute("style", "display: none;");

                                    // Trigger download
                                    anchor.click();

                                    // Cleanup
                                    let _ = Url::revoke_object_url(&url);
                                }
                            }
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = &content;
            let _ = &filename;
            leptos::logging::warn!("Download only available in browser");
        }
    };

    let label_for_fallback = label.clone();
    let title_text = format!("Download as {label}");

    view! {
        <button
            class="download-btn"
            on:click=download_action
            title=title_text
        >
            {match icon_type.as_str() {
                "markdown" => {
                    view! { <img src="images/ui/markdown-logo.svg" alt="Markdown" class="download-btn-icon download-btn-icon--invert"/> }
                        .into_any()
                }
                "r" => {
                    view! { <img src="images/ui/r-logo.svg" alt="R" class="download-btn-icon"/> }.into_any()
                }
                "python" => {
                    view! { <img src="images/ui/python-logo.svg" alt="Python" class="download-btn-icon"/> }
                        .into_any()
                }
                _ => {
                    view! { <span class="text-xs font-medium" style="color: var(--color-text-secondary);">{label_for_fallback}</span> }
                        .into_any()
                }
            }}
        </button>
    }
}

/// TOC link component with JavaScript navigation (interactive island)
///
/// Handles anchor navigation properly when <base href="/"> is set
/// by using JavaScript scrollIntoView instead of relying on browser's
/// default anchor behavior.
#[island]
pub fn TocLink(target_id: String, title: String, padding_left: i32) -> impl IntoView {
    // Clone target_id for use in both the closure and the view
    let target_id_clone = target_id.clone();

    let scroll_to_section = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default();

        #[cfg(target_arch = "wasm32")]
        {
            use leptos::web_sys::window;

            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Some(target) = document.get_element_by_id(&target_id_clone) {
                        // Use scrollIntoView with smooth behavior via JS
                        // This is more reliable than web-sys bindings
                        let _ = js_sys::Reflect::set(
                            &js_sys::Object::new(),
                            &"behavior".into(),
                            &"smooth".into(),
                        );

                        target.scroll_into_view_with_bool(true);

                        // Update URL hash without triggering navigation
                        if let Some(history) = window.history().ok() {
                            let _ = history.replace_state_with_url(
                                &wasm_bindgen::JsValue::NULL,
                                "",
                                Some(&format!("#{}", target_id_clone)),
                            );
                        }
                    }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = &target_id_clone;
            leptos::logging::warn!("TOC navigation only available in browser");
        }
    };

    view! {
        <a
            href=format!("#{}", target_id)
            class="aside-link truncate"
            style=format!("padding-left: {}px; padding-right: 8px;", padding_left)
            on:click=scroll_to_section
        >
            {title}
        </a>
    }
}
