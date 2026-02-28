/// Page Archives — onglets par année, tableau des cotisations, bannière de clôture.
use leptos::prelude::*;

use crate::{
    components::icons::{
        IconAlertTriangle, IconArchive, IconFileText, IconLock, IconSearch,
    },
    models::{
        contribution::ContributionWithMember,
        year_summary::YearSummary,
    },
    services::db_service,
};

// ── Helpers locaux ────────────────────────────────────────────────────────────

fn format_ariary(amount_str: &str) -> String {
    let n = amount_str.parse::<f64>().unwrap_or(0.0) as i64;
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }
    format!("{} Ar", result)
}

/// Année courante depuis JS (WASM-compatible).
fn current_year() -> i32 {
    js_sys::Date::new_0().get_full_year() as i32
}

// ── Composant principal ───────────────────────────────────────────────────────

#[component]
pub fn Archives() -> impl IntoView {
    let cur_year = current_year();

    // Liste des résumés annuels (triés DESC par le backend)
    let summaries: RwSignal<Vec<YearSummary>> = RwSignal::new(vec![]);
    // Cotisations de l'année sélectionnée
    let contributions: RwSignal<Vec<ContributionWithMember>> = RwSignal::new(vec![]);
    // État de chargement
    let loading_sum  = RwSignal::new(true);
    let loading_cont = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    // Année sélectionnée (défaut : année courante)
    let selected_year: RwSignal<i32> = RwSignal::new(cur_year);
    // Recherche par nom de membre
    let recherche: RwSignal<String> = RwSignal::new(String::new());

    // ── Charger les résumés au montage ────────────────────────────────────────
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            loading_sum.set(true);
            match db_service::get_year_summaries().await {
                Ok(liste) => summaries.set(liste),
                Err(e)    => erreur.set(Some(e)),
            }
            loading_sum.set(false);
        });
    });

    // ── Charger les cotisations quand l'année change ──────────────────────────
    Effect::new(move |_| {
        let year = selected_year.get();
        recherche.set(String::new());
        leptos::task::spawn_local(async move {
            loading_cont.set(true);
            contributions.set(vec![]);
            match db_service::get_contributions_by_year_with_member(year).await {
                Ok(liste) => contributions.set(liste),
                Err(e)    => erreur.set(Some(e)),
            }
            loading_cont.set(false);
        });
    });

    // ── Liste des onglets : années DB + année courante si absente ─────────────
    let tab_years = Memo::new(move |_| {
        let mut years: Vec<i32> = summaries.get().iter().map(|s| s.year).collect();
        if !years.contains(&cur_year) {
            years.push(cur_year);
        }
        years.sort_unstable_by(|a, b| b.cmp(a));
        years
    });

    // ── Détail de l'année sélectionnée ───────────────────────────────────────
    let year_detail = Memo::new(move |_| {
        let sel = selected_year.get();
        summaries.get().into_iter().find(|s| s.year == sel)
    });

    // ── Cotisations filtrées par nom (déjà triées ASC par le backend) ────────────
    let filtered = Memo::new(move |_| {
        let q = recherche.get().to_lowercase();
        contributions.get()
            .into_iter()
            .filter(|c| c.member_name.to_lowercase().contains(&q))
            .collect::<Vec<_>>()
    });

    view! {
        <div class="animate-fade-in space-y-4 sm:space-y-6">

            // ── En-tête ───────────────────────────────────────────────────────
            <div>
                <h1 class="text-xl sm:text-2xl font-bold text-gray-800 dark:text-white \
                            flex items-center gap-2">
                    <IconArchive class="w-6 h-6 text-gray-600 dark:text-gray-400" />
                    "Archives — Cotisations par année"
                </h1>
                <p class="text-gray-500 dark:text-gray-400 text-xs sm:text-sm mt-0.5 sm:mt-1">
                    "Sélectionnez une année pour consulter les cotisations et le résumé annuel."
                </p>
            </div>

            // ── Message d'erreur ──────────────────────────────────────────────
            {move || erreur.get().map(|e| view! {
                <div class="p-3 sm:p-4 bg-red-50 dark:bg-red-900/30 \
                            border border-red-200 dark:border-red-700 \
                            rounded-xl text-red-700 dark:text-red-300 text-sm \
                            flex items-start gap-2">
                    <IconAlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
                    <span>{e}</span>
                </div>
            })}

            // ── Onglets d'années ──────────────────────────────────────────────
            {move || {
                if loading_sum.get() {
                    return view! {
                        <div class="flex gap-2">
                            <div class="h-9 w-20 bg-gray-200 dark:bg-gray-700 \
                                        rounded-full animate-pulse" />
                            <div class="h-9 w-20 bg-gray-200 dark:bg-gray-700 \
                                        rounded-full animate-pulse" />
                            <div class="h-9 w-20 bg-gray-200 dark:bg-gray-700 \
                                        rounded-full animate-pulse" />
                        </div>
                    }.into_any();
                }
                view! {
                    <div class="flex gap-2 overflow-x-auto pb-1">
                        {tab_years.get().into_iter().map(|y| {
                            let is_active  = y == selected_year.get();
                            let is_current = y == cur_year;
                            let detail = summaries.get().into_iter().find(|s| s.year == y);
                            let is_closed = detail
                                .as_ref()
                                .and_then(|d| d.closed_at.as_ref())
                                .is_some();

                            let btn_cls = if is_active {
                                "flex-shrink-0 px-4 py-1.5 rounded-full text-sm font-semibold \
                                 bg-blue-600 text-white shadow-sm transition-all duration-200"
                            } else {
                                "flex-shrink-0 px-4 py-1.5 rounded-full text-sm font-medium \
                                 bg-white/70 dark:bg-gray-800/70 \
                                 text-gray-700 dark:text-gray-300 \
                                 border border-gray-200 dark:border-gray-600 \
                                 hover:border-blue-400 dark:hover:border-blue-500 \
                                 hover:text-blue-600 dark:hover:text-blue-400 \
                                 transition-all duration-200 backdrop-blur"
                            };

                            view! {
                                <button
                                    class={btn_cls}
                                    on:click=move |_| selected_year.set(y)
                                >
                                    <span class="flex items-center gap-1">
                                        {if is_current && !is_closed {
                                            format!("{} ✦", y)
                                        } else {
                                            y.to_string()
                                        }}
                                        {is_closed.then(|| view! {
                                            <IconLock class="w-3 h-3 opacity-80" />
                                        })}
                                    </span>
                                </button>
                            }
                        }).collect_view()}
                    </div>
                }.into_any()
            }}

            // ── Barre de recherche ────────────────────────────────────────────
            <div class="relative w-full max-w-xs sm:max-w-sm">
                <span class="absolute left-3 top-1/2 -translate-y-1/2 \
                             text-gray-400 dark:text-gray-500 pointer-events-none">
                    <IconSearch class="w-4 h-4" />
                </span>
                <input
                    type="text"
                    placeholder="Rechercher un membre…"
                    class="w-full pl-9 pr-3 py-2 text-sm rounded-xl \
                           bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                           border border-gray-200 dark:border-gray-600 \
                           text-gray-800 dark:text-gray-200 \
                           placeholder-gray-400 dark:placeholder-gray-500 \
                           focus:outline-none focus:ring-2 focus:ring-blue-400/50 \
                           transition-all duration-200"
                    prop:value=move || recherche.get()
                    on:input=move |ev| recherche.set(event_target_value(&ev))
                />
            </div>

            // ── Contenu de l'année sélectionnée ──────────────────────────────
            {move || {
                let sel = selected_year.get();
                let detail = year_detail.get();
                let is_closed = detail.as_ref().and_then(|d| d.closed_at.as_ref()).is_some();

                view! {
                    <div class="space-y-4">

                        // ── Bannière clôture ──────────────────────────────────
                        {detail.clone().filter(|_| is_closed).map(|d| {
                            let total_fmt   = format_ariary(&d.total);
                            let closed_date = d.closed_at.as_deref()
                                .map(|dt| dt.chars().take(10).collect::<String>())
                                .unwrap_or_default();
                            let note = d.note.clone();
                            view! {
                                <div class="bg-gradient-to-r from-amber-50 to-orange-50 \
                                            dark:from-amber-900/20 dark:to-orange-900/20 \
                                            border border-amber-200 dark:border-amber-700/50 \
                                            rounded-2xl p-4 sm:p-5">
                                    <div class="flex flex-wrap items-start gap-3">
                                        <IconLock class="w-6 h-6 text-amber-700 dark:text-amber-400 shrink-0 mt-0.5" />
                                        <div class="flex-1 min-w-0">
                                            <p class="font-semibold \
                                                       text-amber-800 dark:text-amber-300">
                                                "Année " {sel.to_string()}
                                                " — clôturée le " {closed_date}
                                            </p>
                                            {note.map(|n| view! {
                                                <p class="text-sm \
                                                           text-amber-700 dark:text-amber-400 \
                                                           mt-0.5 italic">
                                                    {n}
                                                </p>
                                            })}
                                        </div>
                                        <div class="text-right flex-shrink-0">
                                            <p class="text-xs text-amber-600 dark:text-amber-400">
                                                "Total archivé"
                                            </p>
                                            <p class="text-xl font-bold font-mono \
                                                       text-amber-800 dark:text-amber-200">
                                                {total_fmt}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }
                        })}

                        // ── Badge "En cours" si année ouverte ─────────────────
                        {(!is_closed).then(|| {
                            let total_opt = detail.as_ref().map(|d| format_ariary(&d.total));
                            view! {
                                <div class="flex flex-wrap items-center justify-between gap-3 \
                                            bg-emerald-50/70 dark:bg-emerald-900/20 \
                                            border border-emerald-200 dark:border-emerald-700/50 \
                                            rounded-2xl px-4 py-3">
                                    <div class="flex items-center gap-2">
                                        <span class="w-2 h-2 rounded-full bg-emerald-500 \
                                                     animate-pulse inline-block" />
                                        <span class="text-sm font-medium \
                                                     text-emerald-700 dark:text-emerald-300">
                                            "Année " {sel.to_string()} " en cours"
                                        </span>
                                    </div>
                                    {total_opt.map(|t| view! {
                                        <span class="text-sm font-semibold font-mono \
                                                     text-emerald-700 dark:text-emerald-300">
                                            {t}
                                        </span>
                                    })}
                                </div>
                            }
                        })}

                        // ── Tableau des cotisations ───────────────────────────
                        {move || {
                            if loading_cont.get() {
                                return view! {
                                    <div class="flex justify-center py-12">
                                        <div class="w-7 h-7 border-4 border-blue-400 \
                                                    border-t-transparent rounded-full \
                                                    animate-spin" />
                                    </div>
                                }.into_any();
                            }
                            if filtered.get().is_empty() {
                                let (is_empty_data, msg, sub) = if contributions.get().is_empty() {
                                    (true, "Aucune cotisation enregistrée",
                                     format!("pour l'année {}", selected_year.get()))
                                } else {
                                    (false, "Aucun résultat",
                                     format!("aucun membre ne correspond à \"{}\"",
                                             recherche.get()))
                                };
                                return view! {
                                    <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                                rounded-2xl border border-gray-100 \
                                                dark:border-gray-700 text-center py-14 \
                                                text-gray-400 dark:text-gray-500">
                                        <div class="flex justify-center mb-3">
                                            {if is_empty_data {
                                                view! { <IconFileText class="w-10 h-10 text-gray-300 dark:text-gray-600" /> }.into_any()
                                            } else {
                                                view! { <IconSearch class="w-10 h-10 text-gray-300 dark:text-gray-600" /> }.into_any()
                                            }}
                                        </div>
                                        <p class="text-base font-medium">{msg}</p>
                                        <p class="text-xs mt-1">{sub}</p>
                                    </div>
                                }.into_any();
                            }
                            view! {
                                <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                            rounded-2xl border border-gray-100 \
                                            dark:border-gray-700 overflow-hidden shadow-sm">
                                    <div class="overflow-x-auto">
                                        <table class="w-full text-sm">
                                            <thead>
                                                <tr class="bg-gray-50/80 dark:bg-gray-700/50 \
                                                           text-gray-600 dark:text-gray-300 \
                                                           text-xs uppercase tracking-wide">
                                                    <th class="text-left px-4 py-3 font-semibold">
                                                        "Membre"
                                                    </th>
                                                    <th class="text-left px-4 py-3 font-semibold \
                                                               hidden sm:table-cell">
                                                        "Période"
                                                    </th>
                                                    <th class="text-right px-4 py-3 font-semibold">
                                                        "Montant"
                                                    </th>
                                                    <th class="text-right px-4 py-3 font-semibold \
                                                               hidden sm:table-cell">
                                                        "Date"
                                                    </th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-gray-100 \
                                                          dark:divide-gray-700/50">
                                                {filtered.get().into_iter().map(|c| {
                                                    let montant = format_ariary(&c.amount);
                                                    view! {
                                                        <tr class="tr-hover hover:bg-blue-50/40 \
                                                                   dark:hover:bg-blue-900/10 \
                                                                   transition-colors duration-150">
                                                            <td class="px-4 py-2.5 \
                                                                       text-gray-800 dark:text-gray-200 \
                                                                       font-medium">
                                                                {c.member_name}
                                                            </td>
                                                            <td class="px-4 py-2.5 \
                                                                       text-gray-500 dark:text-gray-400 \
                                                                       hidden sm:table-cell">
                                                                {c.period}
                                                            </td>
                                                            <td class="px-4 py-2.5 text-right \
                                                                       font-mono font-semibold \
                                                                       text-gray-800 dark:text-gray-100">
                                                                {montant}
                                                            </td>
                                                            <td class="px-4 py-2.5 text-right \
                                                                       text-gray-400 dark:text-gray-500 \
                                                                       text-xs hidden sm:table-cell">
                                                                {c.payment_date}
                                                            </td>
                                                        </tr>
                                                    }
                                                }).collect_view()}
                                            </tbody>
                                            // ── Pied de tableau : total ───────
                                            {move || {
                                                let total: f64 = filtered.get()
                                                    .iter()
                                                    .filter_map(|c| c.amount.parse::<f64>().ok())
                                                    .sum();
                                                let total_fmt = format_ariary(
                                                    &format!("{:.0}", total)
                                                );
                                                let count = filtered.get().len();
                                                view! {
                                                    <tfoot>
                                                        <tr class="bg-gray-50/80 dark:bg-gray-700/50 \
                                                                   border-t border-gray-200 \
                                                                   dark:border-gray-600">
                                                            <td class="px-4 py-2.5 text-xs \
                                                                       text-gray-500 dark:text-gray-400 \
                                                                       font-medium">
                                                                {count.to_string()} " cotisation(s)"
                                                            </td>
                                                            <td class="hidden sm:table-cell" />
                                                            <td class="px-4 py-2.5 text-right \
                                                                       font-mono font-bold \
                                                                       text-gray-800 dark:text-white">
                                                                {total_fmt}
                                                            </td>
                                                            <td class="hidden sm:table-cell" />
                                                        </tr>
                                                    </tfoot>
                                                }
                                            }}
                                        </table>
                                    </div>
                                </div>
                            }.into_any()
                        }}

                    </div>
                }
            }}

        </div>
    }
}
