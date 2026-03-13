/// Squelette commun aux modals : Portal → overlay → card.
///
/// Utilisé par `MemberForm`, `TransferModal` et `ContributionModal`.
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
pub fn ModalWrapper(
    /// Callback appelé quand l'utilisateur clique sur l'overlay (backdrop).
    /// Si `None`, le clic sur le backdrop n'a aucun effet.
    #[prop(optional)]
    on_close: Option<Callback<()>>,
    /// Classes Tailwind supplémentaires pour la card (max-w, overflow, etc.).
    card_class: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Portal>
            <div
                style="position:fixed;top:0;left:0;right:0;bottom:0;z-index:9999;\
                       display:flex;align-items:center;justify-content:center;padding:1rem;"
                class="overlay-fade bg-black/40 dark:bg-black/60 backdrop-blur-sm"
                on:click=move |ev| {
                    if let Some(cb) = on_close {
                        if ev.target() == ev.current_target() {
                            cb.run(());
                        }
                    }
                }
            >
                <div class=format!(
                    "modal-pop bg-white dark:bg-gray-800 rounded-2xl shadow-2xl \
                     w-full border border-gray-100 dark:border-gray-700 {card_class}"
                )>
                    {children()}
                </div>
            </div>
        </Portal>
    }
}
