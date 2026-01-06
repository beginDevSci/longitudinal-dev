use leptos::prelude::*;
#[derive(Clone, Copy, Debug)]
pub enum Tone {
    Default,
    Info,
    Warning,
}

/// Section title with optional subtitle and icon
#[component]
pub fn SectionHeader(
    title: &'static str,
    #[prop(default = "")] subtitle: &'static str,
    #[prop(default = "")] id: &'static str,
) -> impl IntoView {
    // Extract section prefix from ID for subtitle test-id (e.g., "data-access-title" → "da")
    let subtitle_testid = if id.starts_with("data-access") && !subtitle.is_empty() {
        "da:subtitle"
    } else {
        ""
    };

    view! {
        <header class="flex items-center gap-3 mb-6">
            <div>
                <h2 id=id class="section-title">
                    {title}
                </h2>
                {(!subtitle.is_empty()).then(|| {
                    let testid_attr = if !subtitle_testid.is_empty() { subtitle_testid } else { "" };
                    if !testid_attr.is_empty() {
                        view! { <p class="supporting-text" data-testid=testid_attr>{subtitle}</p> }.into_any()
                    } else {
                        view! { <p class="supporting-text" data-testid="">{subtitle}</p> }.into_any()
                    }
                })}
            </div>
        </header>
    }
}

/// Card with optional tone modifier
#[component]
pub fn Card(children: Children, #[prop(default = Tone::Default)] tone: Tone) -> impl IntoView {
    let class_str: &str = match tone {
        Tone::Info => "card tone-info",
        Tone::Warning => "card tone-warning",
        Tone::Default => "card",
    };

    view! {
        <div class=class_str>
            {children()}
        </div>
    }
}

/// Callout with border accent
#[component]
pub fn Callout(title: &'static str, tone: Tone, children: Children) -> impl IntoView {
    let class_str: &str = match tone {
        Tone::Info => "callout tone-info",
        Tone::Warning => "callout tone-warning",
        Tone::Default => "callout",
    };

    view! {
        <div class=class_str>
            <h3 class="panel-title mb-2">{title}</h3>
            {children()}
        </div>
    }
}

// ============================================================================
// Table Primitive
// ============================================================================

pub struct Col {
    pub header: &'static str,
    pub align: Option<&'static str>, // "left" | "center" | "right"
}

pub type Row = &'static [&'static str];

/// Static SSR-only table with accessible markup
#[component]
pub fn TableStatic(
    columns: &'static [Col],
    rows: &'static [Row],
    #[prop(optional)] caption: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto py-6">
            <table class="min-w-full text-sm border-collapse">
                {caption.map(|c| view! { <caption class="panel-title text-left mb-2">{c}</caption> })}
                <thead>
                    <tr class="border-b">
                        {columns
                            .iter()
                            .map(|col| {
                                let align = col.align.unwrap_or("left");
                                view! {
                                    <th
                                        scope="col"
                                        class={
                                    match align {
                                        "left" => "px-4 py-2 text-left font-semibold",
                                        "center" => "px-4 py-2 text-center font-semibold",
                                        "right" => "px-4 py-2 text-right font-semibold",
                                        _ => "px-4 py-2 text-left font-semibold",
                                    }
                                }
                                    >
                                        {col.header}
                                    </th>
                                }
                            })
                            .collect::<Vec<_>>()}

                    </tr>
                </thead>
                <tbody>
                    {rows
                        .iter()
                        .map(|row| {
                            view! {
                                <tr class="border-b border-table">
                                    {row
                                        .iter()
                                        .enumerate()
                                        .map(|(i, cell)| {
                                            let align = columns
                                                .get(i)
                                                .and_then(|c| c.align)
                                                .unwrap_or("left");
                                            view! {
                                                <td class={
                                                match align {
                                                    "left" => "px-4 py-2 text-left",
                                                    "center" => "px-4 py-2 text-center",
                                                    "right" => "px-4 py-2 text-right",
                                                    _ => "px-4 py-2 text-left",
                                                }
                                            }>{*cell}</td>
                                            }
                                        })
                                        .collect::<Vec<_>>()}

                                </tr>
                            }
                        })
                        .collect::<Vec<_>>()}

                </tbody>
            </table>
        </div>
    }
}

// ============================================================================
// Control Bar (CSS-only tabs)
// ============================================================================

pub struct Tab {
    pub id: &'static str,
    pub label: &'static str,
}

/// CSS-only tab bar using radio inputs
#[component]
pub fn ControlBar(
    tabs: &'static [Tab],
    selected_id: &'static str,
    group_name: &'static str,
) -> impl IntoView {
    view! {
        <div class="control-bar mb-6" role="tablist">
            {tabs
                .iter()
                .map(|tab| {
                    let is_selected = tab.id == selected_id;
                    view! {
                        <label class="inline-flex items-center cursor-pointer">
                            <input
                                type="radio"
                                name=group_name
                                value=tab.id
                                checked=is_selected
                                class="peer sr-only"
                            />
                            <span class="tab-label peer-checked:bg-accent-subtle peer-checked:border-accent peer-checked:font-semibold">
                                {tab.label}
                            </span>
                        </label>
                    }
                })
                .collect::<Vec<_>>()}

        </div>
    }
}
