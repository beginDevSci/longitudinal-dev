use leptos::prelude::*;

use crate::base_path;
use crate::layout::TopNav;

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
pub fn SiteLayout(
    options: leptos::config::LeptosOptions,
    #[prop(optional, into)] canonical_url: Option<String>,
    children: Children,
) -> impl IntoView {
    // Always compute base path (defaults to "/" if SITE_BASE_PATH not set)
    let base = base_path::base_path();
    let base_trimmed = base_path::base_path_trimmed();
    let css_url = base_path::join("pkg/blog.css");

    view! {
        <!DOCTYPE html>
        <html lang="en" data-theme="dracula" class="dark">
            <head>
                // FOUC prevention: run before any paint to honor user preference
                <script>
                    "(function(){const theme=localStorage.getItem('theme')||'dracula';document.documentElement.setAttribute('data-theme',theme);if(theme==='dark'||theme==='dracula'){document.documentElement.classList.add('dark')}else{document.documentElement.classList.remove('dark')}})()"
                </script>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                // Always render base tag for consistent path resolution
                <base href=base/>
                // Use base path for stylesheet URL
                <link rel="stylesheet" href=css_url/>
                // KaTeX CSS for math rendering (guides use server-side KaTeX)
                <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css" crossorigin="anonymous"/>
                // Canonical URL for SEO (only rendered if provided)
                {canonical_url.map(|url| view! { <link rel="canonical" href=url/> })}
                <AutoReload options=options.clone()/>
                // Pass trimmed base path to HydrationScripts for modulepreload/WASM URLs
                <HydrationScripts options islands=true root=base_trimmed/>
            </head>
            <body>
                <div class="min-h-screen flex flex-col">
                    <TopNav/>
                    <div class="flex-1 flex flex-col">
                        {children()}
                    </div>
                </div>
            </body>
        </html>
    }
}
