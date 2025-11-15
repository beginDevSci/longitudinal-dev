use leptos::prelude::*;

/// Step pill (matches FilterPill style from Statistical Analysis section).
#[component]
pub fn StepPill<T>(num: u8, label: T, #[prop(default = true)] active: bool) -> impl IntoView
where
    T: AsRef<str>,
{
    let label = label.as_ref().to_string();
    let _num = num; // Keep parameter for compatibility but don't use it
    let _active = active; // Keep parameter for compatibility but don't use it

    view! {
        <span class="pill">{label}</span>
    }
}

/// Task card (title, filename, code, actions).
#[component]
pub fn PrepTaskCard<T, F, C, A>(
    title: T,
    filename: F,
    code_lines: C,
    actions: A,
    #[prop(default = 0)] task_index: usize,
) -> impl IntoView
where
    T: AsRef<str>,
    F: AsRef<str>,
    C: IntoIterator,
    C::Item: AsRef<str>,
    A: IntoIterator,
    A::Item: AsRef<str>,
{
    let title_id = format!("prep-task-title-{task_index}");
    let code_id = format!("prep-code-{task_index}");
    let title = title.as_ref().to_string();
    let filename = filename.as_ref().to_string();
    let code_joined = code_lines
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    // Collect actions outside view! and render them before the macro
    // Take only the first action for single button layout
    let action_nodes = actions
        .into_iter()
        .take(1)
        .map(|a| {
            let action_text = a.as_ref().to_string();
            view! {
                <span class="badge">
                    {action_text}
                </span>
            }
        })
        .collect_view();

    view! {
        <article data-testid="prep:task" class="card">
            <div class="flex items-start justify-between mb-2">
                <div id=title_id.clone() class="panel-title">{title}</div>
                <div class="flex gap-2">
                    {action_nodes}
                </div>
            </div>

            <div class="supporting-text font-mono tracking-widest uppercase mb-4">
                {filename}
            </div>

            <div class="panel relative" aria-labelledby=title_id>
                <div class="absolute top-2 right-2 z-10">
                    <crate::CopyCodeButton code_id=code_id.clone() />
                </div>
                <pre id=code_id class="overflow-x-auto"><code>{code_joined}</code></pre>
            </div>
        </article>
    }
}
