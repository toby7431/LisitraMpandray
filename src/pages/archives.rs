use leptos::prelude::*;
use crate::{models::year_summary::YearSummary, services::db_service};

/// Page Archives — affiche les résumés annuels des cotisations.
/// (Le concept d'archivage de membres est remplacé par les résumés financiers par année.)
#[component]
pub fn Archives() -> impl IntoView {
    let summaries: RwSignal<Vec<YearSummary>> = RwSignal::new(vec![]);
    let loading = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    let charger = move || {
        loading.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::get_year_summaries().await {
                Ok(liste) => summaries.set(liste),
                Err(e)    => erreur.set(Some(e)),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| charger());

    view! {
        <div class="animate-fade-in space-y-4 sm:space-y-6">

            <div>
                <h1 class="text-xl sm:text-2xl font-bold text-gray-800 dark:text-white">
                    "📦 Résumés annuels"
                </h1>
                <p class="text-gray-500 dark:text-gray-400 text-xs sm:text-sm mt-0.5 sm:mt-1">
                    "Totaux des cotisations par année — recalculés automatiquement"
                </p>
            </div>

            {move || erreur.get().map(|e| view! {
                <div class="p-3 sm:p-4 bg-red-50 dark:bg-red-900/30 \
                            border border-red-200 dark:border-red-700 \
                            rounded-xl text-red-700 dark:text-red-300 text-sm">
                    "⚠️ " {e}
                </div>
            })}

            {move || {
                if loading.get() {
                    view! {
                        <div class="flex justify-center py-16">
                            <div class="w-8 h-8 border-4 border-gray-400 \
                                        border-t-transparent rounded-full animate-spin" />
                        </div>
                    }.into_any()
                } else if summaries.get().is_empty() {
                    view! {
                        <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                                    text-center py-16 sm:py-20 \
                                    text-gray-400 dark:text-gray-500">
                            <div class="text-4xl sm:text-5xl mb-3">"📦"</div>
                            <p class="text-base sm:text-lg font-medium">
                                "Aucun résumé annuel"
                            </p>
                            <p class="text-xs sm:text-sm mt-1">
                                "Les résumés apparaissent automatiquement dès la première cotisation enregistrée."
                            </p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                            <For
                                each=move || summaries.get()
                                key=|s| s.year
                                children=|s| view! { <YearCard summary=s /> }
                            />
                        </div>
                    }.into_any()
                }
            }}

        </div>
    }
}

#[component]
fn YearCard(summary: YearSummary) -> impl IntoView {
    let is_closed = summary.closed_at.is_some();
    let badge_class = if is_closed {
        "px-2 py-0.5 rounded-full text-xs bg-gray-100 dark:bg-gray-700 \
         text-gray-500 dark:text-gray-400"
    } else {
        "px-2 py-0.5 rounded-full text-xs bg-emerald-100 dark:bg-emerald-900/40 \
         text-emerald-700 dark:text-emerald-300"
    };

    view! {
        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                    p-5 shadow-sm hover:shadow-md transition-shadow duration-200">
            <div class="flex items-start justify-between mb-3">
                <span class="text-2xl font-bold text-gray-800 dark:text-white">
                    {summary.year.to_string()}
                </span>
                <span class=badge_class>
                    {if is_closed { "Clôturé" } else { "Ouvert" }}
                </span>
            </div>

            <p class="text-xs text-gray-500 dark:text-gray-400 mb-1">"Total cotisations"</p>
            <p class="text-xl font-semibold text-gray-800 dark:text-white font-mono">
                {format!("{} Ar", summary.total)}
            </p>

            {summary.note.map(|n| view! {
                <p class="mt-2 text-xs text-gray-500 dark:text-gray-400 italic truncate">
                    {n}
                </p>
            })}

            {summary.closed_at.map(|dt| view! {
                <p class="mt-1 text-xs text-gray-400 dark:text-gray-500">
                    "Clôturé le " {dt.chars().take(10).collect::<String>()}
                </p>
            })}
        </div>
    }
}
