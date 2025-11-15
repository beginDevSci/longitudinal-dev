use crate::{Block, EditorState, SectionType};
use leptos::prelude::*;

/// Section content editor component
///
/// For MVP, provides a simple textarea for editing section content.
/// Future: Structured block editors for code, output, notes, etc.
#[component]
pub fn SectionEditor(section_type: SectionType, editor_state: EditorState) -> impl IntoView {
    // Get current section content
    let section_content = Memo::new(move |_| {
        let tutorial = editor_state.tutorial.get();
        tutorial
            .sections
            .iter()
            .find(|s| s.section_type == section_type)
            .map(|s| {
                // Convert blocks to simple text for MVP
                s.blocks
                    .iter()
                    .filter_map(|block| match block {
                        Block::Paragraph { content } => Some(content.clone()),
                        Block::List { items, .. } => Some(items.join("\n- ")),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            })
            .unwrap_or_default()
    });

    // Helper to get section display name
    let section_name = match section_type {
        SectionType::Overview => "Overview",
        SectionType::DataAccess => "Data Access",
        SectionType::DataPreparation => "Data Preparation",
        SectionType::StatisticalAnalysis => "Statistical Analysis",
        SectionType::Discussion => "Discussion",
        SectionType::AdditionalResources => "Additional Resources",
    };

    // Helper to get section description
    let section_description = match section_type {
        SectionType::Overview => {
            "Provide an overview of the tutorial, including key features and statistics."
        }
        SectionType::DataAccess => "Describe data requirements and access methods.",
        SectionType::DataPreparation => "Detail the data preparation steps, including code blocks.",
        SectionType::StatisticalAnalysis => {
            "Explain the statistical analysis, including code and output."
        }
        SectionType::Discussion => "Discuss key insights and limitations.",
        SectionType::AdditionalResources => "Provide additional resources and references.",
    };

    view! {
        <div class="section-editor max-w-4xl">
            <div class="mb-6">
                <h2 class="section-title">
                    {section_name}
                </h2>
                <p class="text-sm text-muted mt-1">
                    {section_description}
                </p>
            </div>

            <div class="space-y-4">
                {/* Character count */}
                <div class="flex justify-between items-center text-sm text-muted">
                    <span>
                        "Content"
                    </span>
                    <span>
                        {move || {
                            let content = section_content.get();
                            format!("{} characters", content.len())
                        }}
                    </span>
                </div>

                {/* Textarea editor */}
                <textarea
                    class="form-textarea font-mono"
                    placeholder=move || format!("Enter {section_name} content here...\n\nFor now, use simple paragraphs. Structured blocks (code, output, notes) coming in Phase 2.")
                    rows="20"
                    on:input=move |ev| {
                        let value = event_target_value(&ev);

                        // Update section content
                        editor_state.tutorial.update(|t| {
                            if let Some(section) = t.sections.iter_mut().find(|s| s.section_type == section_type) {
                                // For MVP, store as paragraphs
                                section.blocks = if value.trim().is_empty() {
                                    vec![]
                                } else {
                                    value.split("\n\n")
                                        .filter(|p| !p.trim().is_empty())
                                        .map(|p| Block::Paragraph {
                                            content: p.trim().to_string(),
                                        })
                                        .collect()
                                };
                            }
                        });

                        editor_state.is_dirty.set(true);
                    }
                >
                    {move || section_content.get()}
                </textarea>

                {/* Help text */}
                <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
                    <h4 class="text-sm font-semibold text-blue-900 dark:text-blue-100 mb-2">
                        "💡 MVP Editor"
                    </h4>
                    <p class="text-sm text-blue-800 dark:text-blue-200">
                        "This is a simple text editor for the MVP. Use paragraphs separated by blank lines. "
                        "In future phases, you'll be able to add:"
                    </p>
                    <ul class="list-disc list-inside text-sm text-blue-800 dark:text-blue-200 mt-2 space-y-1">
                        <li>"Code blocks with syntax highlighting"</li>
                        <li>"Output blocks with tables and images"</li>
                        <li>"Note blocks with custom titles"</li>
                        <li>"Custom labels and markers"</li>
                    </ul>
                </div>

                {/* Section-specific guidance */}
                {match section_type {
                    SectionType::Overview => view! {
                        <div class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-4">
                            <h4 class="text-sm font-semibold text-amber-900 dark:text-amber-100 mb-2">
                                "Overview Tips"
                            </h4>
                            <ul class="list-disc list-inside text-sm text-amber-800 dark:text-amber-200 space-y-1">
                                <li>"Start with a brief summary paragraph"</li>
                                <li>"Highlight key features and benefits"</li>
                                <li>"Include important statistics or metrics"</li>
                            </ul>
                        </div>
                    }.into_any(),
                    SectionType::DataPreparation | SectionType::StatisticalAnalysis => view! {
                        <div class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-4">
                            <h4 class="text-sm font-semibold text-amber-900 dark:text-amber-100 mb-2">
                                "Code Section Tips"
                            </h4>
                            <ul class="list-disc list-inside text-sm text-amber-800 dark:text-amber-200 space-y-1">
                                <li>"Describe what the code will do"</li>
                                <li>"Explain key parameters and options"</li>
                                <li>"Note: Structured code blocks coming in Phase 2"</li>
                            </ul>
                        </div>
                    }.into_any(),
                    _ => view! { <span></span> }.into_any(),
                }}
            </div>
        </div>
    }
}
