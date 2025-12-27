//! Statistical Analysis section renderer v2 - Flexible block-based system.
//!
//! Renders a section with fully flexible content blocks:
//! - Code blocks: Display code snippets with syntax highlighting
//! - Output blocks: Display results (text, tables, or images)
//! - Note blocks: Interpretation callout cards
//!
//! All blocks are repeatable and author-controlled ordering.

use leptos::prelude::*;

use crate::models::statistical_analysis::{
    CodeData, ContentBlock, NoteData, OutputData, OutputFormat, StatsModel,
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
