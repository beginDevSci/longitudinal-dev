use leptos::prelude::*;

use crate::base_path;

/// Toolkit hub page - landing page linking to learning resources and software tools.
#[component]
pub fn ToolkitPage() -> impl IntoView {
    let learning_href = base_path::join("toolkit/learning/");
    let software_href = base_path::join("toolkit/software/");

    view! {
        <main class="min-h-screen bg-surface">
            // Hero section
            <section class="relative overflow-hidden bg-subtle">
                <div class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 lg:py-10">
                    <h1 class="text-4xl md:text-5xl font-bold text-primary">"Toolkit"</h1>
                    <p class="mt-3 text-lg md:text-xl text-secondary max-w-3xl">
                        "Curated resources and tools for longitudinal data science research."
                    </p>
                </div>
            </section>

            // Cards section
            <section class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
                <div class="grid gap-8 md:grid-cols-2">
                    // Learning Resources Card
                    <a
                        href=learning_href
                        class="group block p-8 rounded-2xl bg-subtle border border-default hover:border-accent/50 transition-all duration-300 hover:shadow-lg"
                    >
                        <div class="flex items-start gap-4">
                            <div class="flex-shrink-0 w-12 h-12 rounded-xl bg-accent/10 flex items-center justify-center text-2xl">
                                <span class="text-accent">"📚"</span>
                            </div>
                            <div class="flex-1">
                                <h2 class="text-2xl font-bold text-primary group-hover:text-accent transition-colors">
                                    "R Learning Resources"
                                </h2>
                                <p class="mt-2 text-secondary">
                                    "Books, video courses, interactive tutorials, and cheatsheets for learning R programming and data analysis."
                                </p>
                                <div class="mt-4 flex items-center gap-2 text-accent font-medium">
                                    <span>"Browse resources"</span>
                                    <span class="group-hover:translate-x-1 transition-transform">"→"</span>
                                </div>
                            </div>
                        </div>
                    </a>

                    // Software Tools Card
                    <a
                        href=software_href
                        class="group block p-8 rounded-2xl bg-subtle border border-default hover:border-accent/50 transition-all duration-300 hover:shadow-lg"
                    >
                        <div class="flex items-start gap-4">
                            <div class="flex-shrink-0 w-12 h-12 rounded-xl bg-accent/10 flex items-center justify-center text-2xl">
                                <span class="text-accent">"🔧"</span>
                            </div>
                            <div class="flex-1">
                                <h2 class="text-2xl font-bold text-primary group-hover:text-accent transition-colors">
                                    "Software & Tools"
                                </h2>
                                <p class="mt-2 text-secondary">
                                    "Programming languages, IDEs, version control, data formats, notebooks, and databases for data science workflows."
                                </p>
                                <div class="mt-4 flex items-center gap-2 text-accent font-medium">
                                    <span>"Browse tools"</span>
                                    <span class="group-hover:translate-x-1 transition-transform">"→"</span>
                                </div>
                            </div>
                        </div>
                    </a>
                </div>
            </section>
        </main>
    }
}
