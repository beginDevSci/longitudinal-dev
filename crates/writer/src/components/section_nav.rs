use crate::{SectionType, ValidationIssue, ValidationLevel};
use leptos::ev;
use leptos::prelude::*;

/// Section navigation sidebar component
///
/// Displays navigation to:
/// - Metadata (default)
/// - All 6 main sections
/// - Validation status indicators for each section
#[component]
pub fn SectionNav(
    current_section: RwSignal<Option<SectionType>>,
    validation_issues: Memo<Vec<ValidationIssue>>,
) -> impl IntoView {
    // Helper to check if section has errors or warnings
    let section_status = move |section_name: &str| -> (bool, bool) {
        let issues = validation_issues.get();
        let errors = issues.iter().any(|i| {
            i.level == ValidationLevel::Error && i.section.as_deref() == Some(section_name)
        });
        let warnings = issues.iter().any(|i| {
            i.level == ValidationLevel::Warning && i.section.as_deref() == Some(section_name)
        });
        (errors, warnings)
    };

    view! {
        <nav class="section-nav p-4">
            <div class="space-y-1">
                {/* Metadata */}
                <NavItem
                    label="Metadata"
                    is_active=move || current_section.get().is_none()
                    has_error=move || section_status("metadata").0
                    has_warning=move || section_status("metadata").1
                    on_click=move |_| current_section.set(None)
                />

                <div class="border-t border-neutral-200 dark:border-neutral-700 my-2"></div>

                {/* Sections */}
                <NavItem
                    label="Overview"
                    is_active=move || current_section.get() == Some(SectionType::Overview)
                    has_error=move || section_status("Overview").0
                    has_warning=move || section_status("Overview").1
                    on_click=move |_| current_section.set(Some(SectionType::Overview))
                />

                <NavItem
                    label="Data Access"
                    is_active=move || current_section.get() == Some(SectionType::DataAccess)
                    has_error=move || section_status("Data Access").0
                    has_warning=move || section_status("Data Access").1
                    on_click=move |_| current_section.set(Some(SectionType::DataAccess))
                />

                <NavItem
                    label="Data Preparation"
                    is_active=move || current_section.get() == Some(SectionType::DataPreparation)
                    has_error=move || section_status("Data Preparation").0
                    has_warning=move || section_status("Data Preparation").1
                    on_click=move |_| current_section.set(Some(SectionType::DataPreparation))
                />

                <NavItem
                    label="Statistical Analysis"
                    is_active=move || current_section.get() == Some(SectionType::StatisticalAnalysis)
                    has_error=move || section_status("Statistical Analysis").0
                    has_warning=move || section_status("Statistical Analysis").1
                    on_click=move |_| current_section.set(Some(SectionType::StatisticalAnalysis))
                />

                <NavItem
                    label="Discussion"
                    is_active=move || current_section.get() == Some(SectionType::Discussion)
                    has_error=move || section_status("Discussion").0
                    has_warning=move || section_status("Discussion").1
                    on_click=move |_| current_section.set(Some(SectionType::Discussion))
                />

                <NavItem
                    label="Additional Resources"
                    is_active=move || current_section.get() == Some(SectionType::AdditionalResources)
                    has_error=move || section_status("Additional Resources").0
                    has_warning=move || section_status("Additional Resources").1
                    on_click=move |_| current_section.set(Some(SectionType::AdditionalResources))
                />
            </div>
        </nav>
    }
}

/// Individual navigation item
#[component]
fn NavItem<F, A, E, W>(
    label: &'static str,
    is_active: A,
    has_error: E,
    has_warning: W,
    on_click: F,
) -> impl IntoView
where
    F: Fn(ev::MouseEvent) + Send + Sync + 'static,
    A: Fn() -> bool + Send + Sync + Clone + 'static,
    E: Fn() -> bool + Send + Sync + Clone + 'static,
    W: Fn() -> bool + Send + Sync + Clone + 'static,
{
    let is_active_clone = is_active.clone();
    let has_error_clone = has_error.clone();
    let has_error_clone2 = has_error.clone();
    let has_warning_clone = has_warning.clone();
    let has_warning_clone2 = has_warning.clone();

    view! {
        <button
            class=move || {
                let mut classes = vec![
                    "nav-item w-full text-left px-3 py-2 rounded-lg transition-colors flex items-center justify-between",
                ];
                if is_active_clone() {
                    classes.push("active");
                    classes.push("bg-accent-100 dark:bg-accent-900/30 text-accent-700 dark:text-accent-300");
                } else {
                    classes.push("hover:bg-neutral-100 dark:hover:bg-neutral-700/50");
                }
                if has_error_clone() {
                    classes.push("error");
                }
                if has_warning_clone() {
                    classes.push("warning");
                }
                classes.join(" ")
            }
            on:click=on_click
        >
            <span class="text-sm font-medium">
                {label}
            </span>

            {/* Status indicator */}
            {move || {
                if has_error_clone2() {
                    view! {
                        <span class="w-2 h-2 rounded-full bg-red-500"></span>
                    }.into_any()
                } else if has_warning_clone2() {
                    view! {
                        <span class="w-2 h-2 rounded-full bg-amber-500"></span>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </button>
    }
}
