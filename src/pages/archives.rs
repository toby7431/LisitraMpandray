use leptos::prelude::*;
use crate::{models::membre::Membre, services::db_service};

#[component]
pub fn Archives() -> impl IntoView {
    let membres: RwSignal<Vec<Membre>> = RwSignal::new(vec![]);
    let loading = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    let charger = move || {
        loading.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::get_membres().await {
                Ok(liste) => {
                    let archives: Vec<_> = liste
                        .into_iter()
                        .filter(|m| m.statut == "Archive")
                        .collect();
                    membres.set(archives);
                }
                Err(e) => erreur.set(Some(e)),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| charger());

    view! {
        <div class="animate-fade-in space-y-6">

            <div>
                <h1 class="text-2xl font-bold text-gray-800 dark:text-white">
                    "📦 Archives"
                </h1>
                <p class="text-gray-500 dark:text-gray-400 text-sm mt-1">
                    "Membres archivés (inactifs ou partis)"
                </p>
            </div>

            {move || erreur.get().map(|e| view! {
                <div class="p-4 bg-red-50 dark:bg-red-900/30 border border-red-200 \
                            dark:border-red-700 rounded-xl text-red-700 dark:text-red-300 text-sm">
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
                } else if membres.get().is_empty() {
                    view! {
                        <div class="text-center py-20 text-gray-400 dark:text-gray-500">
                            <div class="text-5xl mb-4">"📦"</div>
                            <p class="text-lg font-medium">"Aucune archive"</p>
                            <p class="text-sm mt-1">
                                "Les membres archivés apparaîtront ici."
                            </p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                                    overflow-hidden shadow-sm">
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="bg-gray-50 dark:bg-gray-900/50 \
                                               border-b border-gray-100 dark:border-gray-700">
                                        <th class="text-left px-4 py-3 font-semibold \
                                                   text-gray-600 dark:text-gray-400">"Nom"</th>
                                        <th class="text-left px-4 py-3 font-semibold \
                                                   text-gray-600 dark:text-gray-400">"Prénom"</th>
                                        <th class="text-left px-4 py-3 font-semibold \
                                                   text-gray-600 dark:text-gray-400 hidden sm:table-cell">
                                            "Type"
                                        </th>
                                        <th class="text-left px-4 py-3 font-semibold \
                                                   text-gray-600 dark:text-gray-400 hidden md:table-cell">
                                            "Adhésion"
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
                                                       hover:bg-gray-50/80 \
                                                       dark:hover:bg-gray-700/30 \
                                                       transition-colors duration-150 opacity-75">
                                                <td class="px-4 py-3 font-medium \
                                                           text-gray-700 dark:text-gray-200">
                                                    {m.nom}
                                                </td>
                                                <td class="px-4 py-3 text-gray-500 \
                                                           dark:text-gray-400">
                                                    {m.prenom}
                                                </td>
                                                <td class="px-4 py-3 text-gray-500 \
                                                           dark:text-gray-400 hidden sm:table-cell">
                                                    <span class="px-2 py-0.5 rounded-full text-xs \
                                                                 bg-gray-100 dark:bg-gray-700 \
                                                                 text-gray-600 dark:text-gray-300">
                                                        {m.type_membre}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-gray-500 \
                                                           dark:text-gray-400 hidden md:table-cell">
                                                    {m.date_adhesion}
                                                </td>
                                                <td class="px-4 py-3 text-right">
                                                    <button class="text-xs text-gray-500 \
                                                                   dark:text-gray-400 \
                                                                   hover:text-blue-600 \
                                                                   dark:hover:text-blue-400 \
                                                                   hover:underline font-medium \
                                                                   transition-colors duration-150">
                                                        "Restaurer"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }}

        </div>
    }
}
