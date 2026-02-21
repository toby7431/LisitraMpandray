use leptos::prelude::*;
use crate::{models::membre::Membre, services::db_service};

#[component]
pub fn Communiants() -> impl IntoView {
    let membres: RwSignal<Vec<Membre>> = RwSignal::new(vec![]);
    let loading = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

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

    Effect::new(move |_| charger());

    view! {
        <div class="animate-fade-in space-y-4 sm:space-y-6">

            // ── En-tête ────────────────────────────────────────────────────────
            <div class="flex flex-wrap items-start sm:items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl sm:text-2xl font-bold text-gray-800 dark:text-white \
                                flex items-center gap-2">
                        "✝️ Communiants"
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 text-xs sm:text-sm mt-0.5 sm:mt-1">
                        "Membres communiants actifs de l'église"
                    </p>
                </div>
                <button
                    class="px-3 sm:px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white \
                           rounded-lg text-xs sm:text-sm font-medium transition-colors \
                           duration-200 flex items-center gap-1.5 shrink-0"
                >
                    "➕ Ajouter"
                </button>
            </div>

            // ── Feedback erreur ────────────────────────────────────────────────
            {move || erreur.get().map(|e| view! {
                <div class="p-3 sm:p-4 bg-red-50 dark:bg-red-900/30 \
                            border border-red-200 dark:border-red-700 \
                            rounded-xl text-red-700 dark:text-red-300 text-sm">
                    "⚠️ " {e}
                </div>
            })}

            // ── Contenu principal ──────────────────────────────────────────────
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex justify-center py-16">
                            <div class="w-8 h-8 border-4 border-blue-500 \
                                        border-t-transparent rounded-full animate-spin" />
                        </div>
                    }.into_any()
                } else if membres.get().is_empty() {
                    view! {
                        <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                                    text-center py-16 sm:py-20 \
                                    text-gray-400 dark:text-gray-500">
                            <div class="text-4xl sm:text-5xl mb-3 sm:mb-4">"✝️"</div>
                            <p class="text-base sm:text-lg font-medium">
                                "Aucun communiant enregistré"
                            </p>
                            <p class="text-xs sm:text-sm mt-1">
                                "Ajoutez le premier communiant avec le bouton ci-dessus."
                            </p>
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
        // Sur mobile : liste de cartes ; sur md+ : tableau
        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                    overflow-hidden shadow-sm">

            // ── Vue tableau (md et plus) ───────────────────────────────────────
            <div class="hidden md:block overflow-x-auto">
                <table class="w-full text-sm">
                    <thead>
                        <tr class="bg-gray-50/80 dark:bg-gray-900/50 \
                                   border-b border-gray-100 dark:border-gray-700">
                            <th class="text-left px-4 py-3 font-semibold \
                                       text-gray-600 dark:text-gray-400">"Nom"</th>
                            <th class="text-left px-4 py-3 font-semibold \
                                       text-gray-600 dark:text-gray-400">"Prénom"</th>
                            <th class="text-left px-4 py-3 font-semibold \
                                       text-gray-600 dark:text-gray-400">"Téléphone"</th>
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
                            children=|m| view! { <MembreLigneTable membre=m /> }
                        />
                    </tbody>
                </table>
            </div>

            // ── Vue carte (moins de md) ────────────────────────────────────────
            <div class="md:hidden divide-y divide-gray-100 dark:divide-gray-700">
                <For
                    each=move || membres.get()
                    key=|m| m.id
                    children=|m| view! { <MembreCarte membre=m /> }
                />
            </div>

        </div>
    }
}

#[component]
fn MembreLigneTable(membre: Membre) -> impl IntoView {
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
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400">
                {membre.telephone.clone().unwrap_or_else(|| "—".into())}
            </td>
            <td class="px-4 py-3 text-gray-500 dark:text-gray-400 hidden lg:table-cell">
                {membre.date_adhesion.clone()}
            </td>
            <td class="px-4 py-3 text-right">
                <button class="text-xs text-blue-600 dark:text-blue-400 \
                               hover:underline font-medium">
                    "Voir"
                </button>
            </td>
        </tr>
    }
}

#[component]
fn MembreCarte(membre: Membre) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between px-4 py-3 \
                    hover:bg-blue-50/40 dark:hover:bg-blue-900/10 \
                    transition-colors duration-150">
            <div class="min-w-0">
                <p class="font-medium text-gray-800 dark:text-white text-sm truncate">
                    {format!("{} {}", membre.nom, membre.prenom)}
                </p>
                {membre.telephone.map(|t| view! {
                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{t}</p>
                })}
            </div>
            <button class="text-xs text-blue-600 dark:text-blue-400 \
                           hover:underline font-medium ml-3 shrink-0">
                "Voir"
            </button>
        </div>
    }
}
