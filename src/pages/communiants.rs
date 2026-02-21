use leptos::prelude::*;
use crate::{models::membre::Membre, services::db_service};

#[component]
pub fn Communiants() -> impl IntoView {
    // Signal pour la liste des membres
    let membres: RwSignal<Vec<Membre>> = RwSignal::new(vec![]);
    let loading = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    // Chargement initial depuis Tauri
    let charger = move || {
        loading.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::get_membres().await {
                Ok(liste) => {
                    let communiants: Vec<_> = liste
                        .into_iter()
                        .filter(|m| m.type_membre == "Communiant" && m.statut == "Actif")
                        .collect();
                    membres.set(communiants);
                }
                Err(e) => erreur.set(Some(e)),
            }
            loading.set(false);
        });
    };

    // Charger au montage
    Effect::new(move |_| charger());

    view! {
        <div class="animate-fade-in space-y-6">

            // ── En-tête ────────────────────────────────────────────────────
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800 dark:text-white flex items-center gap-2">
                        "✝️ Communiants"
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 text-sm mt-1">
                        "Membres communiants actifs de l'église"
                    </p>
                </div>
                <button
                    class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white \
                           rounded-lg text-sm font-medium transition-colors duration-200 \
                           flex items-center gap-2"
                    // TODO: ouvrir modal ajout
                >
                    "➕ Ajouter"
                </button>
            </div>

            // ── Feedback ───────────────────────────────────────────────────
            {move || erreur.get().map(|e| view! {
                <div class="p-4 bg-red-50 dark:bg-red-900/30 border border-red-200 \
                            dark:border-red-700 rounded-xl text-red-700 dark:text-red-300 text-sm">
                    "⚠️ " {e}
                </div>
            })}

            // ── Tableau ────────────────────────────────────────────────────
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex justify-center py-16">
                            <div class="w-8 h-8 border-4 border-blue-500 border-t-transparent \
                                        rounded-full animate-spin" />
                        </div>
                    }.into_any()
                } else if membres.get().is_empty() {
                    view! {
                        <div class="text-center py-20 text-gray-400 dark:text-gray-500">
                            <div class="text-5xl mb-4">"✝️"</div>
                            <p class="text-lg font-medium">"Aucun communiant enregistré"</p>
                            <p class="text-sm mt-1">"Ajoutez le premier communiant avec le bouton ci-dessus."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <MembreTable membres=membres />
                    }.into_any()
                }
            }}

        </div>
    }
}

#[component]
fn MembreTable(membres: RwSignal<Vec<Membre>>) -> impl IntoView {
    view! {
        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                    overflow-hidden shadow-sm">
            <table class="w-full text-sm">
                <thead>
                    <tr class="bg-gray-50 dark:bg-gray-900/50 border-b \
                               border-gray-100 dark:border-gray-700">
                        <th class="text-left px-4 py-3 font-semibold \
                                   text-gray-600 dark:text-gray-400">"Nom"</th>
                        <th class="text-left px-4 py-3 font-semibold \
                                   text-gray-600 dark:text-gray-400">"Prénom"</th>
                        <th class="text-left px-4 py-3 font-semibold \
                                   text-gray-600 dark:text-gray-400 hidden md:table-cell">
                            "Téléphone"
                        </th>
                        <th class="text-left px-4 py-3 font-semibold \
                                   text-gray-600 dark:text-gray-400 hidden lg:table-cell">
                            "Adhésion"
                        </th>
                        <th class="px-4 py-3" />
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || membres.get()
                        key=|m| m.id
                        children=|m| view! { <MembreLigne membre=m /> }
                    />
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn MembreLigne(membre: Membre) -> impl IntoView {
    view! {
        <tr class="border-b border-gray-50 dark:border-gray-700/50 \
                   hover:bg-blue-50/50 dark:hover:bg-blue-900/10 \
                   transition-colors duration-150">
            <td class="px-4 py-3 font-medium text-gray-800 dark:text-white">
                {membre.nom.clone()}
            </td>
            <td class="px-4 py-3 text-gray-600 dark:text-gray-300">
                {membre.prenom.clone()}
            </td>
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400 hidden md:table-cell">
                {membre.telephone.clone().unwrap_or_else(|| "—".into())}
            </td>
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400 hidden lg:table-cell">
                {membre.date_adhesion.clone()}
            </td>
            <td class="px-4 py-3 text-right">
                <button
                    class="text-xs text-blue-600 dark:text-blue-400 \
                           hover:underline font-medium"
                >
                    "Voir"
                </button>
            </td>
        </tr>
    }
}
