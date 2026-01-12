use leptos::prelude::*;

use crate::base_path;

/// ABCD Study Overview page - information about the study, data access, and resources.
#[component]
pub fn AbcdOverviewPage() -> impl IntoView {
    let abcd_href = base_path::join("abcd/");

    view! {
        <main class="min-h-screen bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950 text-slate-100 px-6 py-16">
            <article class="max-w-3xl mx-auto space-y-10">
                <section class="space-y-4">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"ABCD Study Overview"</p>
                    <h1 class="text-3xl font-bold text-slate-100">"The ABCD Study"</h1>
                    <p class="text-lg text-slate-200 leading-relaxed">
                        "The "
                        <a
                            href="https://abcdstudy.org"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-teal-400 hover:text-teal-300 transition-colors font-medium"
                        >
                            "Adolescent Brain Cognitive Development (ABCD) Study"
                        </a>
                        " is the largest long-term study of brain development and child health in the United States."
                    </p>
                </section>

                <section class="space-y-4">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"Study Overview"</p>
                    <p class="text-slate-200 leading-relaxed">
                        "The ABCD Study follows over 11,000 youth from ages 9-10 into early adulthood, examining the impact of biological, environmental, and behavioral factors on development. This landmark study provides unprecedented insights into adolescent brain development."
                    </p>

                    <div class="space-y-3 mt-6">
                        <p class="text-sm font-medium text-slate-300">"Key Features"</p>
                        <ul class="space-y-2 text-slate-300 text-sm">
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Sample Size"</span>" — Over 11,000 participants from 21 research sites across the U.S."</span>
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Longitudinal Design"</span>" — Annual follow-ups supporting developmental trajectory analysis"</span>
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Multimodal Data"</span>" — Neuroimaging, cognitive, clinical, environmental, and behavioral measures"</span>
                            </li>
                            <li class="flex items-start gap-2">
                                <span class="text-teal-400 mt-1">"•"</span>
                                <span><span class="font-medium text-slate-200">"Public Access"</span>" — Open datasets available through the NIH Brain Development Cohorts"</span>
                            </li>
                        </ul>
                    </div>
                </section>

                <section class="space-y-4">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"Data Access"</p>
                    <p class="text-slate-200 leading-relaxed">
                        "ABCD Study data is available through the "
                        <a
                            href="https://nbdc-datahub.org"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="text-teal-400 hover:text-teal-300 transition-colors"
                        >
                            "NIH Brain Development Cohorts (NBDC) Data Hub"
                        </a>
                        "."
                    </p>

                    <div class="space-y-3">
                        <p class="text-sm font-medium text-slate-300">"Getting Started"</p>
                        <ol class="space-y-1.5 text-slate-300 text-sm pl-4">
                            <li>"1. Create an account on the NBDC Data Hub"</li>
                            <li>"2. Review the data use terms and documentation"</li>
                            <li>"3. Request access to the ABCD dataset"</li>
                            <li>"4. Download data and explore analysis examples"</li>
                        </ol>
                    </div>
                </section>

                <section class="space-y-4">
                    <p class="text-sm uppercase tracking-[0.3em] text-teal-300">"Resources"</p>
                    <div class="grid gap-4 sm:grid-cols-2">
                        <a
                            href="https://abcdstudy.org"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="block p-4 rounded-lg bg-slate-900/60 border border-slate-800 hover:border-teal-600/50 transition-colors"
                        >
                            <p class="font-medium text-slate-200">"Official ABCD Study Website"</p>
                            <p class="text-sm text-slate-400 mt-1">"Study information and news"</p>
                        </a>
                        <a
                            href="https://nbdc-datahub.org"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="block p-4 rounded-lg bg-slate-900/60 border border-slate-800 hover:border-teal-600/50 transition-colors"
                        >
                            <p class="font-medium text-slate-200">"NBDC Data Hub"</p>
                            <p class="text-sm text-slate-400 mt-1">"Access ABCD Study data"</p>
                        </a>
                        <a
                            href="https://wiki.abcdstudy.org"
                            target="_blank"
                            rel="noopener noreferrer"
                            class="block p-4 rounded-lg bg-slate-900/60 border border-slate-800 hover:border-teal-600/50 transition-colors"
                        >
                            <p class="font-medium text-slate-200">"ABCD Study Wiki"</p>
                            <p class="text-sm text-slate-400 mt-1">"Documentation and protocols"</p>
                        </a>
                        <a
                            href=abcd_href
                            class="block p-4 rounded-lg bg-slate-900/60 border border-slate-800 hover:border-teal-600/50 transition-colors"
                        >
                            <p class="font-medium text-slate-200">"ABCD Analyses"</p>
                            <p class="text-sm text-slate-400 mt-1">"Example analyses using ABCD data"</p>
                        </a>
                    </div>
                </section>

                <blockquote class="border-l-4 border-teal-600/50 pl-4 py-2 text-slate-400 text-sm italic">
                    "Please note that this site is not officially associated with or endorsed by the ABCD Study, and all content is provided by the community for educational purposes."
                </blockquote>
            </article>
        </main>
    }
}
