use leptos::prelude::*;

/// Note panel for Discussion section (Insights or Limitations).
#[component]
pub fn NotePanel<T, B>(title: T, bullets: B, icon: &'static str) -> impl IntoView
where
    T: AsRef<str>,
    B: IntoIterator,
    B::Item: AsRef<str>,
{
    let title = title.as_ref().to_string();
    let bullets: Vec<String> = bullets
        .into_iter()
        .map(|b| b.as_ref().to_string())
        .collect();
    let icon_svg = match icon {
        "warning" => view! {
            <svg aria-hidden="true" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8"
                    d="M12 9v4m0 4h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
            </svg>
        },
        _ => view! {
            <svg aria-hidden="true" class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8"
                    d="M12 17.27l5.18 3.05-1.4-5.99L20 9.24l-6.09-.52L12 3l-1.91 5.72L4 9.24l4.22 5.09-1.4 5.99L12 17.27z"/>
            </svg>
        },
    };

    // Render bullet list outside view!
    let bullet_items = bullets
        .into_iter()
        .map(|b| {
            view! { <li>{b}</li> }
        })
        .collect_view();

    view! {
        <aside role="note" class="card">
            <div class="flex items-start gap-3">
                <div class="icon-circle">
                    {icon_svg}
                    <span class="sr-only">Icon</span>
                </div>
                <div class="flex-1">
                    <div class="text-base font-semibold text-primary">{title}</div>
                    <ul class="mt-3 list-disc pl-6 space-y-2 text-base text-secondary">
                        {bullet_items}
                    </ul>                </div>
            </div>
        </aside>
    }
}
