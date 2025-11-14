use leptos::prelude::*;

/// Non-interactive filter pill.
#[component]
pub fn FilterPill<T>(label: T) -> impl IntoView
where
    T: AsRef<str>,
{
    let label = label.as_ref().to_string();
    view! {
        <span class="pill">{label}</span>
    }
}

/// Uppercase metric card (summary block).
#[component]
pub fn UpperMetric<T1, T2>(label_upper: T1, description: T2) -> impl IntoView
where
    T1: AsRef<str>,
    T2: AsRef<str>,
{
    let label = label_upper.as_ref().to_string();
    let desc = description.as_ref().to_string();
    view! {
        <div data-testid="stats:metric" class="panel text-center">
            <div class="stat-label font-mono tracking-widest uppercase">{label}</div>
            <p class="stat-value mt-1">{desc}</p>
        </div>
    }
}

/// Simplified task card for Statistical Analysis section with syntax highlighting.
/// Includes collapsible functionality using CSS-only <details> element.
#[component]
pub fn StatsTaskCard<T1, T2, T3>(
    title: T1,
    code: T2,
    title_id: T3,
    #[prop(optional, into)] language: Option<String>,
    #[prop(optional)] filename: Option<String>,
    #[prop(optional)] task_index: Option<usize>,
    #[prop(optional, default = false)] default_open: bool,
) -> impl IntoView
where
    T1: AsRef<str>,
    T2: AsRef<str>,
    T3: AsRef<str>,
{
    let title = title.as_ref().to_string();
    let code_raw = code.as_ref();
    let title_id = title_id.as_ref().to_string();
    let lang = language.as_deref().unwrap_or("r");
    let file = filename.clone();

    // Generate unique ID for the code block
    let code_id = format!("stats-code-{}", task_index.unwrap_or(0));
    let expand_id = format!("expand-{}", task_index.unwrap_or(0));

    // Highlight code at build time (SSR) using syntect
    let highlighted_html = crate::syntax_highlight::highlight_code(code_raw, lang);

    // Count lines for the summary
    let line_count = code_raw.lines().count();

    // Show "expand all" link if content exceeds ~15 lines
    let show_expand_link = line_count > 15;

    // Check if metadata is available
    let has_metadata = file.is_some() || !lang.is_empty();
    let show_line_count_in_summary = !has_metadata;

    view! {
        <article data-testid="stats:task" class="card">
            <details class="code-collapsible" open=default_open>
                <summary class="code-summary">
                    <div class="flex items-center justify-between w-full">
                        <div id=title_id.clone() class="panel-title">{title}</div>
                        <div class="flex items-center gap-2">
                            <crate::CopyCodeButton code_id=code_id.clone() />
                            {show_line_count_in_summary.then(|| view! {
                                <span class="code-line-count text-xs text-muted font-mono">
                                    {line_count} " lines"
                                </span>
                            })}
                            <svg
                                aria-hidden="true"
                                class="code-chevron h-4 w-4"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 9l6 6 6-6"/>
                            </svg>
                        </div>
                    </div>
                </summary>
                <div class="code-content-wrapper">
                    {has_metadata.then(|| {
                        let is_r = lang.to_lowercase() == "r";
                        let lang_upper = lang.to_uppercase();
                        view! {
                            <div class="code-meta">
                                <Show
                                    when=move || is_r
                                    fallback=move || view! {
                                        <span class="pill pill--code">{lang_upper.clone()}</span>
                                    }
                                >
                                    <span class="pill pill--code">
                                        <img
                                            src="/images/ui/r-logo.svg"
                                            alt="R"
                                            class="h-3 w-3"
                                            aria-label="R programming language"
                                        />
                                    </span>
                                </Show>
                                {file.as_ref().map(|f| view! {
                                    <span class="code-filename">{f.clone()}</span>
                                })}
                                <span class="code-line-count">
                                    {line_count} " lines"
                                </span>
                            </div>
                        }
                    })}
                    <div class="panel relative mt-3">
                        <pre id=code_id class="overflow-x-auto">
                            <code inner_html=highlighted_html></code>
                        </pre>
                    </div>
                    {show_expand_link.then(|| view! {
                        <input type="checkbox" id=expand_id.clone() class="code-expand-checkbox" />
                        <label for=expand_id class="code-expand-toggle btn-secondary">
                            <span class="when-collapsed">"Show all " {line_count} " lines"</span>
                            <span class="when-expanded">"Show less"</span>
                        </label>
                    })}
                </div>
            </details>
        </article>
    }
}
