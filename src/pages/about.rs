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
                        "Longitudinal.dev is a collaborative, community-driven resource hub for longitudinal data science, developed by members of the "
                        <a
                            href="https://abcdstudy.org"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-teal-400 hover:text-teal-300 transition-colors font-medium"
                        >
                            "ABCD Study® Biostatistics Working Group"
                        </a>
                        ". Content includes tutorials, open-source tools, code examples, and documentation."
                    </p>
                </section>

                <blockquote class="border-l-4 border-teal-600/50 pl-4 py-2 text-slate-400 text-sm italic">
                    "Please note that this project is not officially associated with or endorsed by the ABCD Study®, and all opinions expressed within are solely those of the project maintainers."
                </blockquote>

                <section class="space-y-6">
                    <div class="space-y-4">
                        <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"The ABCD Study"</p>
                        <p class="text-slate-200 leading-relaxed">
                            "The "
                            <a
                                href="https://abcdstudy.org"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-teal-400 hover:text-teal-300 transition-colors"
                            >
                                "Adolescent Brain Cognitive Development (ABCD) Study®"
                            </a>
                            " is the largest long-term study of brain development and child health in the United States. It follows over 11,000 youth from ages 9-10 into early adulthood, examining the impact of biological, environmental, and behavioral factors on development."
                        </p>
                    </div>

                    <div class="space-y-3">
                        <p class="text-sm font-medium text-slate-300">"Key Features"</p>
                        <ul class="space-y-2 text-slate-300 text-sm">
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Public Access"</span>" — Open datasets available through the NIH Brain Development Cohorts"</span>
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Longitudinal Design"</span>" — Annual follow-ups supporting developmental trajectory analysis"</span>
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Multimodal Data"</span>" — Neuroimaging, cognitive, clinical, environmental, and behavioral measures"</span>
                            </li>
                        </ul>
                    </div>

                    <div class="space-y-3">
                        <p class="text-sm font-medium text-slate-300">"Data Access"</p>
                        <p class="text-slate-300 text-sm leading-relaxed">
                            "ABCD Study data is available through the "
                            <a
                                href="https://nbdc-datahub.org"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-teal-400 hover:text-teal-300 transition-colors"
                            >
                                "NIH Brain Development Cohorts (NBDC) Data Hub"
                            </a>
                            ". To get started:"
                        </p>
                        <ol class="space-y-1.5 text-slate-300 text-sm pl-4">
                            <li>"1. Create an account on the NBDC Data Hub"</li>
                            <li>"2. Review the data use terms and documentation"</li>
                            <li>"3. Request access to the ABCD dataset"</li>
                            <li>"4. Explore tutorials on this site for analysis guidance"</li>
                        </ol>
                    </div>
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
