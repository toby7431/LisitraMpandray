/// Modal formulaire — créer ou modifier un membre.
use leptos::portal::Portal;
use leptos::prelude::*;

use crate::{
    components::{
        icons::{IconX},
        phone_input::PhoneInput,
    },
    models::member::MemberInput,
    services::db_service,
};

const LABEL: &str = "block text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1";
const INPUT: &str = "w-full px-3 py-2 text-sm \
                     bg-gray-50 dark:bg-gray-700/60 \
                     border border-gray-200 dark:border-gray-600 \
                     rounded-xl text-gray-800 dark:text-white \
                     placeholder-gray-400 dark:placeholder-gray-500 \
                     focus:outline-none focus:ring-2 focus:ring-blue-400 transition";

/// Modal formulaire de création / modification d'un membre.
///
/// Les signaux du formulaire (`f_*`) sont définis dans `MemberPage` et passés ici
/// car `RwSignal<T>` est `Copy` — aucune allocation supplémentaire.
#[component]
pub fn MemberForm(
    /// Signal d'ouverture du modal.
    open:        RwSignal<bool>,
    /// `Some(id)` en mode édition, `None` en création.
    edit_id:     RwSignal<Option<i64>>,
    /// Type de membre ("Communiant" | "Cathekomen").
    member_type: &'static str,
    /// Classes Tailwind du bouton de soumission (couleur principale).
    btn_class:   &'static str,
    /// Incrémenter pour déclencher un rechargement de liste.
    refresh_ctr: RwSignal<u32>,
    /// Signal d'erreur flottante.
    notif_error: RwSignal<Option<String>>,
    // ── Signaux de champs ────────────────────────────────────────────────────
    f_carte:     RwSignal<String>,
    f_nom:       RwSignal<String>,
    f_adresse:   RwSignal<String>,
    f_telephone: RwSignal<String>,
    f_travail:   RwSignal<String>,
    f_genre:     RwSignal<String>,
    f_loading:   RwSignal<bool>,
) -> impl IntoView {

    let soumettre = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let phone_val = f_telephone.get();
        let phone = if phone_val.trim() == "+261" || phone_val.trim().len() <= 5 {
            None
        } else {
            Some(phone_val.trim().to_string())
        };

        let input = MemberInput {
            card_number: f_carte.get().trim().to_string(),
            full_name:   f_nom.get().trim().to_string(),
            address:     { let t = f_adresse.get().trim().to_string(); if t.is_empty() { None } else { Some(t) } },
            phone,
            job:         { let t = f_travail.get().trim().to_string(); if t.is_empty() { None } else { Some(t) } },
            gender:      f_genre.get(),
            member_type: member_type.to_string(),
        };
        f_loading.set(true);
        let eid = edit_id.get();
        leptos::task::spawn_local(async move {
            let res = if let Some(id) = eid {
                db_service::update_member(id, &input).await.map(|_| ())
            } else {
                db_service::create_member(&input).await.map(|_| ())
            };
            match res {
                Ok(_) => {
                    open.set(false);
                    refresh_ctr.update(|n| *n += 1);
                }
                Err(e) => notif_error.set(Some(e)),
            }
            f_loading.set(false);
        });
    };

    let is_edit    = move || edit_id.get().is_some();
    let modal_title = move || if is_edit() { "Modifier le membre" } else { "Nouveau membre" };

    view! {
        <Portal>
        <div
            style="position:fixed;top:0;left:0;right:0;bottom:0;z-index:9999;\
                   display:flex;align-items:center;justify-content:center;padding:1rem;"
            class="overlay-fade bg-black/40 dark:bg-black/60 backdrop-blur-sm"
            on:click=move |ev| {
                if ev.target() == ev.current_target() {
                    leptos::task::spawn_local(async move { open.set(false); });
                }
            }
        >
            <div class="modal-pop bg-white dark:bg-gray-800 rounded-2xl shadow-2xl \
                        w-full max-w-lg max-h-[90vh] overflow-y-auto \
                        border border-gray-100 dark:border-gray-700">

                <div class="flex items-center justify-between px-6 pt-5 pb-4 \
                            border-b border-gray-100 dark:border-gray-700">
                    <h2 class="text-base font-bold text-gray-800 dark:text-white">
                        {modal_title}
                    </h2>
                    <button
                        on:click=move |_| { leptos::task::spawn_local(async move { open.set(false); }); }
                        class="text-gray-400 hover:text-gray-600 \
                               dark:hover:text-gray-200 transition-colors \
                               p-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                <form on:submit=soumettre class="px-6 py-5 space-y-4">
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class=LABEL>"N° carte *"</label>
                            <input
                                type="text" required
                                placeholder="ex : C-0042"
                                class=INPUT
                                prop:value=move || f_carte.get()
                                on:input=move |ev| f_carte.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class=LABEL>"Genre *"</label>
                            <select
                                class=INPUT
                                prop:value=move || f_genre.get()
                                on:change=move |ev| f_genre.set(event_target_value(&ev))
                            >
                                <option value="M">"Masculin"</option>
                                <option value="F">"Féminin"</option>
                            </select>
                        </div>
                    </div>

                    <div>
                        <label class=LABEL>"Nom complet *"</label>
                        <input
                            type="text" required
                            placeholder="Prénom Nom"
                            class=INPUT
                            prop:value=move || f_nom.get()
                            on:input=move |ev| f_nom.set(event_target_value(&ev))
                        />
                    </div>

                    <div>
                        <label class=LABEL>"Adresse"</label>
                        <input
                            type="text"
                            placeholder="Quartier, ville…"
                            class=INPUT
                            prop:value=move || f_adresse.get()
                            on:input=move |ev| f_adresse.set(event_target_value(&ev))
                        />
                    </div>

                    <div>
                        <label class=LABEL>"Téléphone"</label>
                        <PhoneInput value=f_telephone class=INPUT />
                    </div>

                    <div>
                        <label class=LABEL>"Travail / Emploi"</label>
                        <input
                            type="text"
                            placeholder="Enseignant, Commerçant…"
                            class=INPUT
                            prop:value=move || f_travail.get()
                            on:input=move |ev| f_travail.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="flex gap-3 justify-end pt-1">
                        <button
                            type="button"
                            on:click=move |_| { leptos::task::spawn_local(async move { open.set(false); }); }
                            class="btn-ripple px-4 py-2 text-sm font-medium \
                                   text-gray-600 dark:text-gray-300 \
                                   bg-gray-100 dark:bg-gray-700 \
                                   hover:bg-gray-200 dark:hover:bg-gray-600 \
                                   rounded-xl transition-colors"
                        >
                            "Annuler"
                        </button>
                        <button
                            type="submit"
                            disabled=move || f_loading.get()
                            class=format!("btn-ripple px-4 py-2 text-sm font-semibold \
                                           text-white {} rounded-xl \
                                           disabled:opacity-60 disabled:cursor-wait \
                                           transition-colors shadow-sm", btn_class)
                        >
                            {move || if f_loading.get() { "Enregistrement…" } else { "Enregistrer" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
        </Portal>
    }
}
