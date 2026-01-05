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

// ============================================================================
// PageSectionHeader - Section header for Resources/Tools pages
// ============================================================================

/// Page section header with title, description, and optional controls slot.
/// Used for Resources and Tools page sections (Books, Videos, Languages, etc.)
#[component]
pub fn PageSectionHeader(
    title: &'static str,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional)] id: Option<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-4 mb-6">
            <div>
                <h2 id=id class="text-2xl font-bold text-primary">{title}</h2>
                {description.map(|desc| {
                    view! { <p class="mt-1 text-secondary">{desc}</p> }
                })}
            </div>
            {children.map(|c| {
                view! { <div class="flex items-center gap-2">{c()}</div> }
            })}
        </header>
    }
}

// ============================================================================
// CardShell - Unified interactive card wrapper
// ============================================================================

/// Interactive card wrapper for Resources/Tools pages.
/// Uses CSS custom properties for consistent styling across themes.
///
/// # Props
/// - `href`: Optional URL - if provided, renders as an `<a>` tag
/// - `external`: Whether link opens in new tab (default: true for external links)
/// - `class`: Additional CSS classes to apply
/// - `children`: Card content
#[component]
pub fn CardShell(
    #[prop(optional, into)] href: Option<String>,
    #[prop(default = true)] external: bool,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base_class = "resource-card";
    let full_class = match &class {
        Some(c) => format!("{} {}", base_class, c),
        None => base_class.to_string(),
    };

    match href {
        Some(url) => {
            if external {
                view! {
                    <a
                        href=url
                        target="_blank"
                        rel="noopener noreferrer"
                        class=full_class
                    >
                        {children()}
                    </a>
                }
                .into_any()
            } else {
                view! {
                    <a href=url class=full_class>
                        {children()}
                    </a>
                }
                .into_any()
            }
        }
        None => {
            view! {
                <div class=full_class>
                    {children()}
                </div>
            }
            .into_any()
        }
    }
}

/// Card media container - for images, logos, icons
/// Provides consistent aspect ratio and background
#[component]
pub fn CardMedia(
    #[prop(optional, into)] aspect: Option<String>,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    // Aspect ratio classes: "3/4" for book covers, "video" for 16:9, "square" for 1:1
    let aspect_class = match aspect.as_deref() {
        Some("3/4") => "aspect-[3/4]",
        Some("video") => "aspect-video",
        Some("square") => "aspect-square",
        Some("logo") => "h-24", // Fixed height for tool logos
        _ => "aspect-video",     // Default 16:9
    };

    let full_class = format!("resource-card__media {}", aspect_class);
    let full_class = match &class {
        Some(c) => format!("{} {}", full_class, c),
        None => full_class,
    };

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

/// Card content area - flexible container for text
#[component]
pub fn CardContent(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "resource-card__content";
    let full_class = match &class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

/// Card title - consistent heading style
#[component]
pub fn CardTitle(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "font-semibold text-primary group-hover:text-accent transition-colors line-clamp-2";
    let full_class = match &class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };

    view! {
        <h3 class=full_class>
            {children()}
        </h3>
    }
}

/// Card description - secondary text with line clamping
#[component]
pub fn CardDescription(
    #[prop(default = 2)] lines: u8,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let clamp_class = match lines {
        1 => "line-clamp-1",
        2 => "line-clamp-2",
        3 => "line-clamp-3",
        _ => "line-clamp-2",
    };

    let base = format!("text-sm text-secondary {}", clamp_class);
    let full_class = match &class {
        Some(c) => format!("{} {}", base, c),
        None => base,
    };

    view! {
        <p class=full_class>
            {children()}
        </p>
    }
}

/// Card meta - tertiary information (author, source, etc.)
#[component]
pub fn CardMeta(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "text-sm text-tertiary";
    let full_class = match &class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };

    view! {
        <p class=full_class>
            {children()}
        </p>
    }
}

/// Card footer - pinned to bottom with mt-auto
#[component]
pub fn CardFooter(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let base = "resource-card__footer";
    let full_class = match &class {
        Some(c) => format!("{} {}", base, c),
        None => base.to_string(),
    };

    view! {
        <div class=full_class>
            {children()}
        </div>
    }
}

/// Badge/pill for metadata (format, access type, etc.)
#[derive(Clone, Copy, Debug)]
pub enum BadgeVariant {
    Default,
    Accent,
    Success,
    Warning,
}

#[component]
pub fn Badge(
    #[prop(default = BadgeVariant::Default)] variant: BadgeVariant,
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let variant_class = match variant {
        BadgeVariant::Default => "bg-subtle text-secondary border-default",
        BadgeVariant::Accent => "bg-accent/10 text-accent border-accent/30",
        BadgeVariant::Success => "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-300 border-emerald-300 dark:border-emerald-700",
        BadgeVariant::Warning => "bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300 border-amber-300 dark:border-amber-700",
    };

    let base = format!(
        "inline-block px-2 py-0.5 text-xs font-medium rounded border {}",
        variant_class
    );
    let full_class = match &class {
        Some(c) => format!("{} {}", base, c),
        None => base,
    };

    view! {
        <span class=full_class>
            {children()}
        </span>
    }
}
