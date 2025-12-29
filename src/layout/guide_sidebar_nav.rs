//! Guide sidebar navigation component.
//!
//! Renders a hierarchical outline for intra-guide navigation.
//! This is separate from LeftNav (tutorial navigation) to maintain
//! scope isolation between guides and tutorials.

use crate::models::guide::OutlineNode;
use leptos::prelude::*;

/// Sidebar navigation for guide pages.
///
/// Renders a hierarchical outline extracted from H2/H3/H4 headings.
/// - Sticky positioning within viewport
/// - Hierarchical tree with indentation
/// - Click to jump to section anchors
#[component]
pub fn GuideSidebarNav(outline: Vec<OutlineNode>) -> impl IntoView {
    if outline.is_empty() {
        return ().into_any();
    }

    view! {
        <nav class="guide-sidebar-nav" aria-label="Guide sections">
            <div class="guide-sidebar-header">
                <span class="guide-sidebar-title">"On this page"</span>
            </div>
            <ul class="guide-sidebar-list">
                {outline.iter().map(|node| {
                    view! { <OutlineItem node=node.clone() /> }
                }).collect_view()}
            </ul>
        </nav>
    }.into_any()
}

/// Renders a single outline item with its children.
#[component]
fn OutlineItem(node: OutlineNode) -> impl IntoView {
    let has_children = !node.children.is_empty();
    let href = format!("#{}", node.id);
    let level_class = format!("guide-sidebar-item level-{}", node.level);

    view! {
        <li class=level_class>
            <a href=href class="guide-sidebar-link">
                {node.title.clone()}
            </a>
            {if has_children {
                view! {
                    <ul class="guide-sidebar-children">
                        {node.children.iter().map(|child| {
                            view! { <OutlineItem node=child.clone() /> }
                        }).collect_view()}
                    </ul>
                }.into_any()
            } else {
                ().into_any()
            }}
        </li>
    }
}
