use leptos::prelude::*;

/// Helper function to get icon SVG based on badge type
fn get_icon_for_badge(badge: &str) -> impl IntoView {
    match badge {
        "DOCS" => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
        }.into_any(),
        "VIGNETTE" => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
            </svg>
        }.into_any(),
        "BOOK" => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
            </svg>
        }.into_any(),
        "PAPER" => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
            </svg>
        }.into_any(),
        "TOOL" => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
            </svg>
        }.into_any(),
        "VIDEO" => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"/>
            </svg>
        }.into_any(),
        _ => view! {
            <svg aria-hidden="true" class="w-full h-full" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
            </svg>
        }.into_any(),
    }
}

/// Compact resource row for Additional Resources section.
/// Shows icon, title, badge, description, and link in a scannable horizontal layout.
#[component]
pub fn ResourceIntroCard<T1, T2, T3>(
    title: T1,
    body: T2,
    badge_upper: T3,
    url: Option<String>,
) -> impl IntoView
where
    T1: AsRef<str>,
    T2: AsRef<str>,
    T3: AsRef<str>,
{
    let title = title.as_ref().to_string();
    let body = body.as_ref().to_string();
    let badge = badge_upper.as_ref().to_string();

    let icon = get_icon_for_badge(&badge);

    view! {
        <div class="resource-row" data-testid="resources:item">
            {/* Header row: icon + title + badge */}
            <div class="resource-row-header">
                <div class="resource-row-icon">
                    {icon}
                </div>
                <h3 class="resource-row-title">{title.clone()}</h3>
                <div class="pill resource-row-badge">
                    <span class="font-mono tracking-widest uppercase">{badge}</span>
                </div>
            </div>

            {/* Body row: description + link */}
            <div class="resource-row-body">
                <p class="resource-row-description">{body}</p>
                {url.map(|resource_url| view! {
                    <a
                        href={resource_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="resource-row-link"
                        aria-label={format!("Visit resource: {title}")}
                    >
                        <svg aria-hidden="true" class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.8">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                        </svg>
                        <span>"Visit Resource"</span>
                    </a>
                })}
            </div>
        </div>
    }
}
