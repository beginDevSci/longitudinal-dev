use leptos::prelude::*;

use crate::base_path;

/// Global site wrapper providing HTML shell, head, and body structure.
///
/// This component:
/// - Wraps all pages with consistent <html>, <head>, <body> structure
/// - Injects HydrationScripts with islands=true for selective hydration
/// - Contains no client code or islands
/// - Accepts children to render in the <body>
///
/// Usage:
/// ```rust,ignore
/// view! { <SiteLayout options><PostLayout post/></SiteLayout> }
/// ```
#[component]
pub fn SiteLayout(options: leptos::config::LeptosOptions, children: Children) -> impl IntoView {
    // Always compute base path (defaults to "/" if SITE_BASE_PATH not set)
    let base = base_path::base_path();
    let base_trimmed = base_path::base_path_trimmed();
    let css_url = base_path::join("pkg/blog.css");

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // Always render base tag for consistent path resolution
                <base href=base/>
                // Use base path for stylesheet URL
                <link rel="stylesheet" href=css_url/>
                <AutoReload options=options.clone()/>
                // Pass trimmed base path to HydrationScripts for modulepreload/WASM URLs
                <HydrationScripts options islands=true root=base_trimmed/>
            </head>
            <body>
                {children()}
            </body>
        </html>
    }
}
