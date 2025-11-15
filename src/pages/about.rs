use leptos::prelude::*;

/// About page content describing the mission and community focus.
#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950 text-slate-100 px-6 py-16">
            <article class="max-w-3xl mx-auto space-y-10">
                <section class="space-y-4">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"About"</p>
                    <p class="text-lg text-slate-200 leading-relaxed">
                        "This open-source project is a collaborative, and community-driven resource hub dedicated to longitudinal data science. Content includes tutorials, open-source tools, code examples, and documentation."
                    </p>
                </section>

                <section class="space-y-4">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"Community and Collaboration"</p>
                    <p class="text-slate-200 leading-relaxed">
                        "Open knowledge fosters innovation and accelerates scientific progress. This platform is built upon the collective expertise and shared contributions of its community."
                    </p>
                    <p class="text-slate-200 leading-relaxed">
                        "We encourage participation: by sharing tools, tutorials, or research insights, you drive the continuous improvement and expansion of this platform. Those interested in sharing content, submitting improvements, or getting involved are welcome to review our contribution guide."
                    </p>
                    <p class="text-slate-200 leading-relaxed">
                        "We extend our gratitude to all contributors and supporters whose involvement is essential to this project."
                    </p>
                </section>

                <section class="space-y-3 bg-slate-900/60 p-6 rounded-2xl border border-slate-800 shadow-lg shadow-slate-950/40">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"Contact Us"</p>
                    <p class="text-slate-200">
                        "Please reach out to us at support@longitudinal.dev or connect via our community forums and social media channels for questions or suggestions."
                    </p>
                </section>
            </article>
        </main>
    }
}
