//! Barre de titre personnalisée — remplace la décoration native Windows.
//! La zone centrale est draggable via `data-tauri-drag-region`.
//! Les boutons invoquent minimize / toggle_maximize / close via Tauri.
use leptos::prelude::*;

use crate::services::db_service;

#[component]
pub fn TitleBar() -> impl IntoView {
    // Suit l'état maximisé pour afficher la bonne icône (restore vs maximize)
    let is_maximized = RwSignal::new(false);

    let on_minimize = move |_| {
        leptos::task::spawn_local(async move {
            let _ = db_service::minimize_window().await;
        });
    };

    let on_maximize = move |_| {
        leptos::task::spawn_local(async move {
            let _ = db_service::toggle_maximize().await;
            is_maximized.update(|m| *m = !*m);
        });
    };

    let on_close = move |_| {
        leptos::task::spawn_local(async move {
            let _ = db_service::close_window().await;
        });
    };

    view! {
        <div
            style="position:fixed;top:0;left:0;right:0;height:36px;z-index:10000;"
            class="flex items-stretch select-none \
                   bg-white/95 dark:bg-gray-950/95 backdrop-blur-sm \
                   border-b border-gray-200/70 dark:border-gray-800/70"
        >
            // ── Zone draggable (logo + titre) ──────────────────────────────────
            <div
                data-tauri-drag-region="true"
                class="flex items-center gap-2 px-4 flex-1 h-full cursor-default"
            >
                // Croix d'église miniature
                <svg xmlns="http://www.w3.org/2000/svg"
                    class="w-[14px] h-[14px] text-blue-600 dark:text-blue-400 shrink-0"
                    fill="currentColor" viewBox="0 0 24 24">
                    <path d="M11 2v7H4a1 1 0 0 0 0 2h7v11a1 1 0 0 0 2 0V11h7a1 1 0 0 0 0-2h-7V2a1 1 0 0 0-2 0Z"/>
                </svg>
                <span class="text-[11px] font-semibold tracking-wide \
                              text-gray-600 dark:text-gray-400">
                    "Église Gestion"
                </span>
            </div>

            // ── Boutons de contrôle ─────────────────────────────────────────────
            <div class="flex items-stretch" style="-webkit-app-region:no-drag">

                // ── Minimiser ──
                <button
                    on:click=on_minimize
                    title="Minimiser"
                    class="group w-[46px] flex items-center justify-center \
                           text-gray-500 dark:text-gray-500 \
                           hover:bg-gray-200/80 dark:hover:bg-gray-700/80 \
                           hover:text-gray-900 dark:hover:text-white \
                           transition-colors duration-100"
                >
                    // — trait horizontal (style Windows 11)
                    <svg width="10" height="10" viewBox="0 0 10 10"
                         fill="currentColor" xmlns="http://www.w3.org/2000/svg">
                        <rect x="0" y="4.25" width="10" height="1.5" rx="0.5"/>
                    </svg>
                </button>

                // ── Maximiser / Restaurer ──
                <button
                    on:click=on_maximize
                    title=move || if is_maximized.get() { "Restaurer" } else { "Maximiser" }
                    class="group w-[46px] flex items-center justify-center \
                           text-gray-500 dark:text-gray-500 \
                           hover:bg-gray-200/80 dark:hover:bg-gray-700/80 \
                           hover:text-gray-900 dark:hover:text-white \
                           transition-colors duration-100"
                >
                    {move || if is_maximized.get() {
                        // ⧉ Restaurer — deux carrés superposés
                        view! {
                            <svg width="10" height="10" viewBox="0 0 10 10"
                                 fill="none" stroke="currentColor" stroke-width="1.2"
                                 stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">
                                <rect x="2.5" y="0.5" width="7" height="7" rx="0.5"/>
                                <path d="M0.5 2.5v7h7" stroke-linecap="round"/>
                            </svg>
                        }.into_any()
                    } else {
                        // □ Maximiser — un carré
                        view! {
                            <svg width="10" height="10" viewBox="0 0 10 10"
                                 fill="none" stroke="currentColor" stroke-width="1.2"
                                 stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg">
                                <rect x="0.5" y="0.5" width="9" height="9" rx="0.5"/>
                            </svg>
                        }.into_any()
                    }}
                </button>

                // ── Fermer ──
                <button
                    on:click=on_close
                    title="Fermer"
                    class="group w-[46px] flex items-center justify-center \
                           text-gray-500 dark:text-gray-500 \
                           hover:bg-red-500 dark:hover:bg-red-600 \
                           hover:text-white dark:hover:text-white \
                           transition-colors duration-100"
                >
                    // ✕ Croix
                    <svg width="10" height="10" viewBox="0 0 10 10"
                         fill="none" stroke="currentColor" stroke-width="1.4"
                         stroke-linecap="round" xmlns="http://www.w3.org/2000/svg">
                        <path d="M1 1l8 8M9 1L1 9"/>
                    </svg>
                </button>
            </div>
        </div>
    }
}
