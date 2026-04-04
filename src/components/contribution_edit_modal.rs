/// Modal de modification d'une cotisation avec PIN et motif.
use leptos::prelude::*;

use crate::{
    components::icons::{IconAlertTriangle, IconX},
    models::contribution::{Contribution, ContributionEditInput, ContributionWithMember},
    services::db_service,
    utils::format_ariary,
};

#[component]
pub fn ContributionEditModal(
    /// Cotisation à modifier
    contribution: ContributionWithMember,
    /// Appelé avec la cotisation mise à jour après sauvegarde
    on_saved:  Callback<Contribution>,
    /// Appelé si l'utilisateur annule
    on_cancel: Callback<()>,
) -> impl IntoView {
    let contrib = StoredValue::new(contribution);

    let date_val   = RwSignal::new(contrib.get_value().payment_date.clone());
    let period_val = RwSignal::new(contrib.get_value().period.clone());
    let amount_val = RwSignal::new(contrib.get_value().amount.clone());
    let pin_val    = RwSignal::new(String::new());
    let reason_val = RwSignal::new(String::new());

    let saving  = RwSignal::new(false);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    let on_submit = move |_| {
        let id     = contrib.get_value().id;
        let input  = ContributionEditInput {
            payment_date: date_val.get(),
            period:       period_val.get(),
            amount:       amount_val.get(),
            pin:          pin_val.get(),
            reason:       reason_val.get(),
        };
        saving.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::update_contribution(id, &input).await {
                Ok(updated) => on_saved.run(updated),
                Err(e)      => {
                    erreur.set(Some(e));
                    saving.set(false);
                }
            }
        });
    };

    view! {
        // Fond sombre
        <div class="fixed inset-0 z-50 flex items-center justify-center p-4 \
                    bg-black/50 backdrop-blur-sm animate-fade-in">
            <div class="w-full max-w-md bg-white dark:bg-gray-900 rounded-2xl \
                        shadow-2xl overflow-hidden">

                // ── En-tête ───────────────────────────────────────────────────
                <div class="flex items-center justify-between px-6 py-4 \
                            border-b border-gray-200 dark:border-gray-700">
                    <h2 class="text-base font-semibold \
                               text-gray-900 dark:text-white">
                        "Hanova ny rakitra"
                    </h2>
                    <button
                        class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 \
                               dark:hover:text-gray-200 hover:bg-gray-100 \
                               dark:hover:bg-gray-800 transition-colors"
                        on:click=move |_| on_cancel.run(())
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // ── Valeurs actuelles (référence) ─────────────────────────────
                <div class="px-6 pt-4">
                    <p class="text-xs font-medium uppercase tracking-wide \
                               text-gray-400 dark:text-gray-500 mb-2">
                        "Soatoavina ankehitriny"
                    </p>
                    <div class="bg-gray-50 dark:bg-gray-800 rounded-xl px-4 py-3 \
                                text-sm text-gray-600 dark:text-gray-300 \
                                grid grid-cols-3 gap-2">
                        <div>
                            <span class="text-xs text-gray-400">"Daty"</span>
                            <p class="font-medium">{contrib.get_value().payment_date}</p>
                        </div>
                        <div>
                            <span class="text-xs text-gray-400">"Vanim-potoana"</span>
                            <p class="font-medium">{contrib.get_value().period}</p>
                        </div>
                        <div>
                            <span class="text-xs text-gray-400">"Vola"</span>
                            <p class="font-mono font-semibold">
                                {format_ariary(&contrib.get_value().amount)}
                            </p>
                        </div>
                    </div>
                </div>

                // ── Formulaire ────────────────────────────────────────────────
                <div class="px-6 py-4 space-y-3">

                    // Erreur
                    {move || erreur.get().map(|e| view! {
                        <div class="flex items-start gap-2 p-3 rounded-xl \
                                    bg-red-50 dark:bg-red-900/30 \
                                    border border-red-200 dark:border-red-700 \
                                    text-red-700 dark:text-red-300 text-sm">
                            <IconAlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
                            <span>{e}</span>
                        </div>
                    })}

                    // Nouvelle date
                    <div>
                        <label class="block text-xs font-medium \
                                      text-gray-600 dark:text-gray-400 mb-1">
                            "Daty vaovao"
                        </label>
                        <input
                            type="date"
                            class="w-full px-3 py-2 text-sm rounded-xl \
                                   bg-white dark:bg-gray-800 \
                                   border border-gray-200 dark:border-gray-600 \
                                   text-gray-800 dark:text-gray-200 \
                                   focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            prop:value=move || date_val.get()
                            on:input=move |ev| date_val.set(event_target_value(&ev))
                        />
                    </div>

                    // Nouveau vanim-potoana
                    <div>
                        <label class="block text-xs font-medium \
                                      text-gray-600 dark:text-gray-400 mb-1">
                            "Vanim-potoana vaovao"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 text-sm rounded-xl \
                                   bg-white dark:bg-gray-800 \
                                   border border-gray-200 dark:border-gray-600 \
                                   text-gray-800 dark:text-gray-200 \
                                   focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            prop:value=move || period_val.get()
                            on:input=move |ev| period_val.set(event_target_value(&ev))
                        />
                    </div>

                    // Nouvelle vola
                    <div>
                        <label class="block text-xs font-medium \
                                      text-gray-600 dark:text-gray-400 mb-1">
                            "Vola vaovao (Ariary)"
                        </label>
                        <input
                            type="text"
                            inputmode="decimal"
                            class="w-full px-3 py-2 text-sm rounded-xl \
                                   bg-white dark:bg-gray-800 \
                                   border border-gray-200 dark:border-gray-600 \
                                   text-gray-800 dark:text-gray-200 \
                                   focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            prop:value=move || amount_val.get()
                            on:input=move |ev| amount_val.set(event_target_value(&ev))
                        />
                    </div>

                    // Antony (raison)
                    <div>
                        <label class="block text-xs font-medium \
                                      text-gray-600 dark:text-gray-400 mb-1">
                            "Antony fanovanana"
                        </label>
                        <input
                            type="text"
                            placeholder="Diso ny vola, sns…"
                            class="w-full px-3 py-2 text-sm rounded-xl \
                                   bg-white dark:bg-gray-800 \
                                   border border-gray-200 dark:border-gray-600 \
                                   text-gray-800 dark:text-gray-200 \
                                   placeholder-gray-400 dark:placeholder-gray-500 \
                                   focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            prop:value=move || reason_val.get()
                            on:input=move |ev| reason_val.set(event_target_value(&ev))
                        />
                    </div>

                    // PIN
                    <div>
                        <label class="block text-xs font-medium \
                                      text-gray-600 dark:text-gray-400 mb-1">
                            "Code PIN admin"
                        </label>
                        <input
                            type="password"
                            inputmode="numeric"
                            maxlength="20"
                            placeholder="••••"
                            class="w-full px-3 py-2 text-sm rounded-xl \
                                   bg-white dark:bg-gray-800 \
                                   border border-gray-200 dark:border-gray-600 \
                                   text-gray-800 dark:text-gray-200 \
                                   placeholder-gray-400 dark:placeholder-gray-500 \
                                   focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            prop:value=move || pin_val.get()
                            on:input=move |ev| pin_val.set(event_target_value(&ev))
                        />
                    </div>

                </div>

                // ── Boutons ───────────────────────────────────────────────────
                <div class="px-6 pb-5 flex justify-end gap-3">
                    <button
                        class="px-4 py-2 text-sm font-medium rounded-xl \
                               text-gray-600 dark:text-gray-300 \
                               bg-gray-100 dark:bg-gray-800 \
                               hover:bg-gray-200 dark:hover:bg-gray-700 \
                               transition-colors"
                        on:click=move |_| on_cancel.run(())
                        disabled=move || saving.get()
                    >
                        "Foana"
                    </button>
                    <button
                        class="px-4 py-2 text-sm font-semibold rounded-xl \
                               bg-blue-600 hover:bg-blue-700 text-white \
                               disabled:opacity-50 disabled:cursor-not-allowed \
                               transition-colors"
                        on:click=on_submit
                        disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Mitahiry…" } else { "Tehirizina" }}
                    </button>
                </div>

            </div>
        </div>
    }
}
