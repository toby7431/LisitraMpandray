use leptos::prelude::*;
use crate::{models::membre::Membre, services::db_service};

#[component]
pub fn Cathekomens() -> impl IntoView {
    let membres: RwSignal<Vec<Membre>> = RwSignal::new(vec![]);
    let loading = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    let charger = move || {
        loading.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::get_membres().await {
                Ok(liste) => {
                    let cathekomens: Vec<_> = liste
                        .into_iter()
                        .filter(|m| m.type_membre == "Cathekomen" && m.statut == "Actif")
                        .collect();
                    membres.set(cathekomens);
                }
                Err(e) => erreur.set(Some(e)),
            }
            loading.set(false);
        });
    };

    Effect::new(move |_| charger());

    view! {
        <div class="animate-fade-in space-y-6">

            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-white">
                        "📖 Cathécomènes"
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 text-sm mt-1">
                        "Membres en cours de formation catéchétique"
                    </p>
                </div>
                <button
                    class="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white \
                           rounded-lg text-sm font-medium transition-colors duration-200"
                >
                    "➕ Ajouter"
                </button>
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
                            <div class="w-8 h-8 border-4 border-emerald-500 \
                                        border-t-transparent rounded-full animate-spin" />
                        </div>
                    }.into_any()
                } else if membres.get().is_empty() {
                    view! {
                        <div class="text-center py-20 text-gray-400 dark:text-gray-500">
                            <div class="text-5xl mb-4">"📖"</div>
                            <p class="text-lg font-medium">"Aucun cathécomène enregistré"</p>
                            <p class="text-sm mt-1">
                                "Ajoutez le premier cathécomène avec le bouton ci-dessus."
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
                                                   text-gray-600 dark:text-gray-400 hidden md:table-cell">
                                            "Date naissance"
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
                                                <td class="px-4 py-3 font-medium \
                                                           text-gray-800 dark:text-white">
                                                    {m.nom}
                                                </td>
                                                <td class="px-4 py-3 text-gray-600 \
                                                           dark:text-gray-300">
                                                    {m.prenom}
                                                </td>
                                                <td class="px-4 py-3 text-gray-500 \
                                                           dark:text-gray-400 \
                                                           hidden md:table-cell">
                                                    {m.date_naissance.unwrap_or_else(|| "—".into())}
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
                    }.into_any()
                }
            }}

        </div>
    }
}
