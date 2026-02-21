use leptos::prelude::*;
use crate::{models::member::Member, services::db_service};

#[component]
pub fn Cathekomens() -> impl IntoView {
    let membres: RwSignal<Vec<Member>> = RwSignal::new(vec![]);
    let loading = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    let charger = move || {
        loading.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::get_members_by_type("Cathekomen").await {
                Ok(liste) => membres.set(liste),
                Err(e)    => erreur.set(Some(e)),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| charger());

    view! {
        <div class="animate-fade-in space-y-4 sm:space-y-6">

            <div class="flex flex-wrap items-start sm:items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl sm:text-2xl font-bold text-gray-800 dark:text-white">
                        "📖 Cathécomènes"
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 text-xs sm:text-sm mt-0.5 sm:mt-1">
                        "Membres en cours de formation catéchétique"
                    </p>
                </div>
                <button
                    class="px-3 sm:px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white \
                           rounded-lg text-xs sm:text-sm font-medium transition-colors \
                           duration-200 flex items-center gap-1.5 shrink-0"
                >
                    "➕ Ajouter"
                </button>
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
                            <div class="w-8 h-8 border-4 border-emerald-500 \
                                        border-t-transparent rounded-full animate-spin" />
                        </div>
                    }.into_any()
                } else if membres.get().is_empty() {
                    view! {
                        <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                                    text-center py-16 sm:py-20 \
                                    text-gray-400 dark:text-gray-500">
                            <div class="text-4xl sm:text-5xl mb-3">"📖"</div>
                            <p class="text-base sm:text-lg font-medium">
                                "Aucun cathécomène enregistré"
                            </p>
                            <p class="text-xs sm:text-sm mt-1">
                                "Ajoutez le premier cathécomène avec le bouton ci-dessus."
                            </p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                                    overflow-hidden shadow-sm">

                            // ── Tableau (md+) ──────────────────────────────────
                            <div class="hidden md:block overflow-x-auto">
                                <table class="w-full text-sm">
                                    <thead>
                                        <tr class="bg-gray-50/80 dark:bg-gray-900/50 \
                                                   border-b border-gray-100 dark:border-gray-700">
                                            <th class="text-left px-4 py-3 font-semibold \
                                                       text-gray-600 dark:text-gray-400">
                                                "N° Carte"
                                            </th>
                                            <th class="text-left px-4 py-3 font-semibold \
                                                       text-gray-600 dark:text-gray-400">
                                                "Nom complet"
                                            </th>
                                            <th class="text-left px-4 py-3 font-semibold \
                                                       text-gray-600 dark:text-gray-400 \
                                                       hidden lg:table-cell">
                                                "Téléphone"
                                            </th>
                                            <th class="px-4 py-3" />
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || membres.get()
                                            key=|m| m.id
                                            children=|m| view! {
                                                <tr class="border-b border-gray-50 \
                                                           dark:border-gray-700/50 \
                                                           hover:bg-emerald-50/50 \
                                                           dark:hover:bg-emerald-900/10 \
                                                           transition-colors duration-150">
                                                    <td class="px-4 py-3 text-xs font-mono \
                                                               text-gray-500 dark:text-gray-400">
                                                        {m.card_number}
                                                    </td>
                                                    <td class="px-4 py-3 font-medium \
                                                               text-gray-800 dark:text-white">
                                                        {m.full_name}
                                                    </td>
                                                    <td class="px-4 py-3 text-gray-500 \
                                                               dark:text-gray-400 \
                                                               hidden lg:table-cell">
                                                        {m.phone.unwrap_or_else(|| "—".into())}
                                                    </td>
                                                    <td class="px-4 py-3 text-right">
                                                        <button class="text-xs text-emerald-600 \
                                                                       dark:text-emerald-400 \
                                                                       hover:underline font-medium">
                                                            "Voir"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        />
                                    </tbody>
                                </table>
                            </div>

                            // ── Cartes (mobile) ────────────────────────────────
                            <div class="md:hidden divide-y divide-gray-100 dark:divide-gray-700">
                                <For
                                    each=move || membres.get()
                                    key=|m| m.id
                                    children=|m| view! {
                                        <div class="flex items-center justify-between px-4 py-3 \
                                                    hover:bg-emerald-50/40 \
                                                    dark:hover:bg-emerald-900/10 \
                                                    transition-colors duration-150">
                                            <div class="min-w-0">
                                                <p class="font-medium text-gray-800 \
                                                           dark:text-white text-sm truncate">
                                                    {m.full_name}
                                                </p>
                                                <p class="text-xs font-mono text-gray-400 \
                                                           dark:text-gray-500 mt-0.5">
                                                    {m.card_number}
                                                </p>
                                            </div>
                                            <button class="text-xs text-emerald-600 \
                                                           dark:text-emerald-400 \
                                                           hover:underline font-medium \
                                                           ml-3 shrink-0">
                                                "Voir"
                                            </button>
                                        </div>
                                    }
                                />
                            </div>

                        </div>
                    }.into_any()
                }
            }}

        </div>
    }
}
