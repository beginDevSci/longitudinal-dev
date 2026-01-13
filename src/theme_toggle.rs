use leptos::prelude::*;

/// Theme toggle island component
///
/// Provides a dropdown menu to switch between light, dark, and dracula themes.
/// Persists user preference to localStorage and applies theme via data-theme attribute.
#[island]
pub fn ThemeToggle() -> impl IntoView {
    let (current_theme, set_current_theme) = signal("dracula".to_string());
    let (menu_open, set_menu_open) = signal(false);

    // On mount: read theme from localStorage and apply it
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        let class_list = html.class_list();

                        // Try to read from localStorage
                        if let Ok(Some(storage)) = window.local_storage() {
                            if let Ok(Some(stored_theme)) = storage.get_item("theme") {
                                set_current_theme.set(stored_theme.clone());
                                let _ = html.set_attribute("data-theme", &stored_theme);

                                // Set dark class if needed
                                if stored_theme == "dark" || stored_theme == "dracula" {
                                    let _ = class_list.add_1("dark");
                                } else {
                                    let _ = class_list.remove_1("dark");
                                }
                                return;
                            }
                        }

                        // Default to dracula if no stored preference
                        let _ = html.set_attribute("data-theme", "dracula");
                        let _ = class_list.add_1("dark");
                        set_current_theme.set("dracula".to_string());
                    }
                }
            }
        }
    });

    // Handler to change theme
    let change_theme = move |new_theme: &'static str| {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::window;

            if let Some(window) = window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        // Update HTML attribute
                        let _ = html.set_attribute("data-theme", new_theme);

                        // Also set/remove dark class for Tailwind dark: variant support
                        let class_list = html.class_list();
                        if new_theme == "dark" || new_theme == "dracula" {
                            let _ = class_list.add_1("dark");
                        } else {
                            let _ = class_list.remove_1("dark");
                        }

                        // Save to localStorage
                        if let Ok(Some(storage)) = window.local_storage() {
                            let _ = storage.set_item("theme", new_theme);
                        }

                        // Update state
                        set_current_theme.set(new_theme.to_string());
                        set_menu_open.set(false);
                    }
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            set_current_theme.set(new_theme.to_string());
            set_menu_open.set(false);
        }
    };

    // Helper to get theme icon/label
    let theme_icon = move |theme: &str| -> &'static str {
        match theme {
            "light" => "☀️",
            "dark" => "🌙",
            "dracula" => "🧛",
            _ => "🎨",
        }
    };

    let theme_label = move |theme: &str| -> &'static str {
        match theme {
            "light" => "Light",
            "dark" => "Dark",
            "dracula" => "Dracula",
            _ => "Theme",
        }
    };

    view! {
        <div class="relative inline-block text-left">
            <button
                type="button"
                class="top-nav-theme-toggle"
                on:click=move |_| set_menu_open.update(|open| *open = !*open)
                aria-haspopup="true"
                aria-expanded=move || menu_open.get()
            >
                <span class="text-base">{move || theme_icon(&current_theme.get())}</span>
                <span class="text-secondary">{move || theme_label(&current_theme.get())}</span>
                <svg
                    class="w-4 h-4 text-secondary transition-transform duration-200"
                    class:rotate-180=move || menu_open.get()
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 9l-7 7-7-7"/>
                </svg>
            </button>

            <Show when=move || menu_open.get()>
                <div class="absolute right-0 mt-2 w-48 rounded-lg shadow-md border border-default bg-surface overflow-hidden z-50">
                    <div class="py-1">
                        <button
                            type="button"
                            class="w-full text-left px-4 py-2 text-sm flex items-center gap-3 hover:bg-subtle transition-colors duration-150"
                            class:bg-accent-subtle=move || current_theme.get() == "light"
                            class:font-semibold=move || current_theme.get() == "light"
                            on:click=move |_| change_theme("light")
                        >
                            <span class="text-lg">"☀️"</span>
                            <span class="text-primary">"Light"</span>
                            <Show when=move || current_theme.get() == "light">
                                <svg class="w-4 h-4 ml-auto text-accent" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                </svg>
                            </Show>
                        </button>

                        <button
                            type="button"
                            class="w-full text-left px-4 py-2 text-sm flex items-center gap-3 hover:bg-subtle transition-colors duration-150"
                            class:bg-accent-subtle=move || current_theme.get() == "dark"
                            class:font-semibold=move || current_theme.get() == "dark"
                            on:click=move |_| change_theme("dark")
                        >
                            <span class="text-lg">"🌙"</span>
                            <span class="text-primary">"Dark"</span>
                            <Show when=move || current_theme.get() == "dark">
                                <svg class="w-4 h-4 ml-auto text-accent" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                </svg>
                            </Show>
                        </button>

                        <button
                            type="button"
                            class="w-full text-left px-4 py-2 text-sm flex items-center gap-3 hover:bg-subtle transition-colors duration-150"
                            class:bg-accent-subtle=move || current_theme.get() == "dracula"
                            class:font-semibold=move || current_theme.get() == "dracula"
                            on:click=move |_| change_theme("dracula")
                        >
                            <span class="text-lg">"🧛"</span>
                            <span class="text-primary">"Dracula"</span>
                            <Show when=move || current_theme.get() == "dracula">
                                <svg class="w-4 h-4 ml-auto text-accent" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                </svg>
                            </Show>
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
