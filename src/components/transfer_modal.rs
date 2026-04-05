/// Modal de confirmation de transfert de membres (Cathécomènes → Communiants).
use leptos::prelude::*;

use crate::components::{icons::{IconCross, IconInfo}, modal_wrapper::ModalWrapper};

/// Modal de confirmation avant le transfert de membres sélectionnés.
#[allow(unused_variables)]
#[component]
pub fn TransferModal(
    /// Signal d'ouverture du modal.
    open:             RwSignal<bool>,
    /// `true` quand la requête de transfert est en cours.
    loading:          RwSignal<bool>,
    /// IDs des membres sélectionnés (pour afficher le compteur).
    selected:         RwSignal<Vec<i64>>,
    /// Type cible du transfert (ex: "Communiant").
    _transfer_to:      &'static str,
    /// Callback déclenché quand l'utilisateur confirme.
    on_confirm:       Callback<()>,
) -> impl IntoView {
    view! {
        <ModalWrapper card_class="max-w-sm overflow-hidden">
                // En-tête coloré
                <div class="bg-gradient-to-r from-amber-500 to-orange-500 px-6 py-5">
                    <div class="text-center">
                        <div class="flex justify-center mb-2">
                            <IconCross class="w-10 h-10 text-white" />
                        </div>
                        <h2 class="text-base font-bold text-white">
                            "Hekena ny famindra"
                        </h2>
                    </div>
                </div>
                // Corps
                <div class="px-6 py-5 space-y-4">
                    <p class="text-sm text-gray-700 dark:text-gray-300 text-center">
                        {move || {
                            let n = selected.get().len();
                            format!(
                                "Hamindra mpikambana {} ho any amin'ny Mpandray ?",
                                n
                            )
                        }}
                    </p>
                    <div class="flex items-start gap-2 p-3 \
                                bg-amber-50 dark:bg-amber-900/20 \
                                border border-amber-200 dark:border-amber-700/50 \
                                rounded-xl text-xs text-amber-700 dark:text-amber-300">
                        <IconInfo class="w-4 h-4 shrink-0 mt-0.5" />
                        <span>
                            "Ny rakitra dia mitazona ny fifandraisana amin'ireo mpikambana — \
                             voatahiry ny tantarany."
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
                            "Foana"
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
                                view! { <span>"Famindra mandeha…"</span> }.into_any()
                            } else {
                                view! {
                                    <span class="flex items-center gap-1.5">
                                        <IconCross class="w-4 h-4" />
                                        "Hekena"
                                    </span>
                                }.into_any()
                            }}
                        </button>
                    </div>
                </div>
        </ModalWrapper>
    }
}
