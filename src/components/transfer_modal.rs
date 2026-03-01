/// Modal de confirmation de transfert de membres (Cathécomènes → Communiants).
use leptos::portal::Portal;
use leptos::prelude::*;

use crate::components::icons::{IconCross, IconInfo};

/// Modal de confirmation avant le transfert de membres sélectionnés.
#[component]
pub fn TransferModal(
    /// Signal d'ouverture du modal.
    open:             RwSignal<bool>,
    /// `true` quand la requête de transfert est en cours.
    loading:          RwSignal<bool>,
    /// IDs des membres sélectionnés (pour afficher le compteur).
    selected:         RwSignal<Vec<i64>>,
    /// Type cible du transfert (ex: "Communiant").
    transfer_to:      &'static str,
    /// Callback déclenché quand l'utilisateur confirme.
    on_confirm:       Callback<()>,
) -> impl IntoView {
    view! {
        <Portal>
        <div
            style="position:fixed;top:0;left:0;right:0;bottom:0;z-index:9999;\
                   display:flex;align-items:center;justify-content:center;padding:1rem;"
            class="overlay-fade bg-black/40 dark:bg-black/60 backdrop-blur-sm"
        >
            <div class="modal-pop bg-white dark:bg-gray-800 rounded-2xl shadow-2xl \
                        w-full max-w-sm border border-gray-100 dark:border-gray-700 \
                        overflow-hidden">
                // En-tête coloré
                <div class="bg-gradient-to-r from-amber-500 to-orange-500 px-6 py-5">
                    <div class="text-center">
                        <div class="flex justify-center mb-2">
                            <IconCross class="w-10 h-10 text-white" />
                        </div>
                        <h2 class="text-base font-bold text-white">
                            "Confirmer le transfert"
                        </h2>
                    </div>
                </div>
                // Corps
                <div class="px-6 py-5 space-y-4">
                    <p class="text-sm text-gray-700 dark:text-gray-300 text-center">
                        {move || {
                            let n = selected.get().len();
                            format!(
                                "Transférer {n} membre{} vers les {}s ?",
                                if n > 1 { "s" } else { "" },
                                transfer_to
                            )
                        }}
                    </p>
                    <div class="flex items-start gap-2 p-3 \
                                bg-amber-50 dark:bg-amber-900/20 \
                                border border-amber-200 dark:border-amber-700/50 \
                                rounded-xl text-xs text-amber-700 dark:text-amber-300">
                        <IconInfo class="w-4 h-4 shrink-0 mt-0.5" />
                        <span>
                            "Les contributions restent liées à ces membres — \
                             leur historique est préservé."
                        </span>
                    </div>
                    <div class="flex gap-3">
                        <button
                            type="button"
                            disabled=move || loading.get()
                            on:click=move |_| open.set(false)
                            class="btn-ripple flex-1 px-4 py-2.5 text-sm font-medium \
                                   text-gray-600 dark:text-gray-300 \
                                   bg-gray-100 dark:bg-gray-700 \
                                   hover:bg-gray-200 dark:hover:bg-gray-600 \
                                   disabled:opacity-50 rounded-xl transition-colors"
                        >
                            "Annuler"
                        </button>
                        <button
                            type="button"
                            disabled=move || loading.get()
                            on:click=move |_| on_confirm.run(())
                            class="btn-ripple flex-1 px-4 py-2.5 text-sm font-semibold \
                                   text-white bg-amber-500 hover:bg-amber-600 \
                                   disabled:opacity-60 disabled:cursor-wait \
                                   rounded-xl transition-colors shadow-sm"
                        >
                            {move || if loading.get() {
                                view! { <span>"Transfert en cours…"</span> }.into_any()
                            } else {
                                view! {
                                    <span class="flex items-center gap-1.5">
                                        <IconCross class="w-4 h-4" />
                                        "Confirmer"
                                    </span>
                                }.into_any()
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
        </Portal>
    }
}
