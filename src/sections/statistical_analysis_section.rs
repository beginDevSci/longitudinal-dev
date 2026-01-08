//! Statistical Analysis section renderer v2 - Flexible block-based system.
//!
//! Renders a section with fully flexible content blocks:
//! - Code blocks: Display code snippets with syntax highlighting
//! - Output blocks: Display results (text, tables, or images)
//! - Note blocks: Interpretation callout cards
//! - Viewer blocks: Interactive WebGPU visualizations (with static fallback)
//!
//! All blocks are repeatable and author-controlled ordering.

use leptos::prelude::*;

use crate::models::statistical_analysis::{
    CodeData, ContentBlock, NoteData, OutputData, OutputFormat, StatsModel, ViewerData,
};
use crate::ui::StatsTaskCard;

#[component]
pub fn StatisticalAnalysisSection(model: StatsModel) -> impl IntoView {
    view! {
        <section
            id="statistical-analysis"
            data-testid="section:stats-v2"
            class="mt-8 md:mt-12 lg:mt-16"
            aria-labelledby="stats-title"
        >
            <div class="card">
                <header class="max-w-prose mb-6">
                    <h2 id="stats-title" class="section-title text-balance tracking-tight">
                        "Statistical Analysis"
                    </h2>
                </header>

                // Render content blocks in order
                {model
                    .content_blocks
                    .into_iter()
                    .enumerate()
                    .map(|(idx, block)| {
                        let block_key = format!("block-{idx}");
                        render_content_block(block, block_key, idx)
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

/// Render a single content block based on its type
fn render_content_block(block: ContentBlock, key: String, index: usize) -> impl IntoView {
    match block {
        ContentBlock::Code(data) => render_code_block(data, key, index).into_any(),
        ContentBlock::Output(data) => render_output_block(data, key, index).into_any(),
        ContentBlock::Note(data) => render_note_block(data, key).into_any(),
        ContentBlock::Viewer(data) => render_viewer_block(data, key).into_any(),
    }
}

/// Render a code block using StatsTaskCard with syntax highlighting
fn render_code_block(data: CodeData, key: String, index: usize) -> impl IntoView {
    let title_id = format!("{key}-title");
    // All code blocks are open by default in Statistical Analysis section
    let is_open = data.default_open.unwrap_or(true);
    let lang = data.language.to_string();
    let fname = data.filename.map(|f| f.to_string());

    view! {
        <div class="mt-6" data-testid={format!("stats-v2:code:{key}")}>
            {if let Some(filename_str) = fname {
                view! {
                    <StatsTaskCard
                        title=data.title.to_string()
                        code=data.content.clone()
                        title_id=title_id.clone()
                        language=lang.clone()
                        filename=filename_str
                        task_index=index
                        default_open=is_open
                    />
                }.into_any()
            } else {
                view! {
                    <StatsTaskCard
                        title=data.title.to_string()
                        code=data.content.clone()
                        title_id=title_id.clone()
                        language=lang.clone()
                        task_index=index
                        default_open=is_open
                    />
                }.into_any()
            }}
        </div>
    }
}

/// Render an output block (text, table, or image)
fn render_output_block(data: OutputData, key: String, index: usize) -> impl IntoView {
    match data.format {
        OutputFormat::Text => render_text_output(data, key, index).into_any(),
        OutputFormat::Table => render_table_output(data, key).into_any(),
        OutputFormat::Image => render_image_output(data, key).into_any(),
    }
}

/// Render text output using StatsTaskCard
fn render_text_output(data: OutputData, key: String, index: usize) -> impl IntoView {
    let title_id = format!("{key}-title");

    view! {
        <div class="mt-6" data-testid={format!("stats-v2:output-text:{key}")}>
            <StatsTaskCard
                title="Output".to_string()
                code=data.content.clone()
                title_id=title_id
                task_index=index
            />
            {data.caption.map(|cap| view! {
                <div class="mt-2 text-sm text-muted text-center">
                    {cap.to_string()}
                </div>
            })}
        </div>
    }
}

/// Render table output
fn render_table_output(data: OutputData, key: String) -> impl IntoView {
    view! {
        <div class="mt-6" data-testid={format!("stats-v2:output-table:{key}")}>
            <div class="card table-wrapper">
                <div inner_html={data.content.to_string()} />
            </div>
            {data.caption.map(|cap| view! {
                <div class="mt-2 text-sm text-muted text-center">
                    {cap.to_string()}
                </div>
            })}
        </div>
    }
}

/// Render image output
fn render_image_output(data: OutputData, key: String) -> impl IntoView {
    let alt_text = data
        .alt
        .as_ref()
        .map(|a| a.to_string())
        .unwrap_or_else(|| "Analysis output image".to_string());

    view! {
        <div class="mt-6" data-testid={format!("stats-v2:output-image:{key}")}>
            <figure class="figure-frame flex justify-center">
                <img
                    src={data.content.to_string()}
                    alt={alt_text}
                    class="max-w-3xl w-full"
                    loading="lazy"
                />
            </figure>
            {data.caption.map(|cap| view! {
                <figcaption class="figure-caption">
                    {cap.to_string()}
                </figcaption>
            })}
        </div>
    }
}

/// Render interpretation note as callout card with rich HTML content
fn render_note_block(data: NoteData, key: String) -> impl IntoView {
    view! {
        <div
            data-testid={format!("stats-v2:note:{key}")}
            role="note"
            class="mt-6 callout callout--interpretation note-card"
        >
            <div class="mx-auto mb-4 icon-circle flex items-center justify-center">
                <svg aria-hidden="true" class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                <span class="sr-only">"Interpretation"</span>
            </div>
            <div class="panel-title">{data.title.to_string()}</div>
            <div class="note-content prose prose-sm mt-4" inner_html={data.content.to_string()} />
        </div>
    }
}

/// Render interactive viewer block using the brain_viewer_facade.
///
/// When the `webgpu-viewer` feature is enabled, renders the interactive
/// BrainViewerIsland component. When disabled, renders a static fallback.
fn render_viewer_block(data: ViewerData, key: String) -> impl IntoView {
    // Convert model ViewerData (Cow<str>) to facade ViewerData (String)
    #[cfg(feature = "webgpu-viewer")]
    {
        use brain_viewer_facade::{
            BrainViewerIsland, ViewerData as FacadeViewerData, ViewerOverrides as FacadeOverrides,
        };

        let facade_data = FacadeViewerData {
            manifest_path: data.manifest_path.to_string(),
            overrides: FacadeOverrides {
                analysis: data.overrides.analysis.map(|s| s.to_string()),
                statistic: data.overrides.statistic.map(|s| s.to_string()),
                volume_idx: data.overrides.volume_idx,
                colormap: data.overrides.colormap.map(|s| s.to_string()),
                threshold: data.overrides.threshold,
                hemisphere: data.overrides.hemisphere.map(|s| s.to_string()),
            },
            caption: data.caption.map(|s| s.to_string()),
            fallback_image: data.fallback_image.map(|s| s.to_string()),
            fallback_alt: data.fallback_alt.map(|s| s.to_string()),
            auto_start: data.auto_start,
        };

        view! {
            <div
                data-testid={format!("stats-v2:viewer:{key}")}
                class="mt-6"
            >
                <BrainViewerIsland data=facade_data />
            </div>
        }
        .into_any()
    }

    // Static fallback when webgpu-viewer feature is disabled
    #[cfg(not(feature = "webgpu-viewer"))]
    {
        let fallback_alt = data
            .fallback_alt
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_else(|| "Interactive visualization (static fallback)".to_string());

        let caption = data.caption.map(|c| c.to_string());

        view! {
            <div
                data-testid={format!("stats-v2:viewer:{key}")}
                class="mt-6"
            >
                <div class="viewer-container rounded-lg border border-default bg-subtle overflow-hidden">
                    {if let Some(fallback_src) = data.fallback_image {
                        view! {
                            <figure class="figure-frame relative">
                                <img
                                    src={fallback_src.to_string()}
                                    alt={fallback_alt.clone()}
                                    class="w-full"
                                    loading="lazy"
                                />
                                <div class="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 hover:opacity-100 transition-opacity duration-200">
                                    <div class="text-center text-white p-4">
                                        <svg class="w-12 h-12 mx-auto mb-2 opacity-80" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/>
                                        </svg>
                                        <p class="text-sm font-medium">"Interactive viewer requires webgpu-viewer feature"</p>
                                    </div>
                                </div>
                            </figure>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-col items-center justify-center p-12 text-center">
                                <svg class="w-16 h-16 text-muted mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/>
                                </svg>
                                <p class="text-lg font-medium text-secondary mb-2">"Interactive Brain Viewer"</p>
                                <p class="text-sm text-muted max-w-md">
                                    "Enable the webgpu-viewer feature to view interactive 3D brain surfaces."
                                </p>
                            </div>
                        }.into_any()
                    }}
                </div>
                {caption.map(|cap| view! {
                    <figcaption class="figure-caption mt-2">
                        {cap}
                    </figcaption>
                })}
            </div>
        }
        .into_any()
    }
}
