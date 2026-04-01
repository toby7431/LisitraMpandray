/// Barre de navigation avec 4 onglets et indicateur glissant animé.
/// L'indicateur utilise `transform: translateX(n × 100%)` sur un élément
/// de largeur 25% — la transition CSS utilise un cubic-bezier « ressort ».

/// Logo embarqué en data URI — élimine tout problème de chemin ou CSP.
const LOGO_SRC: &str = include_str!("../../assets/logo_data_uri.txt");

use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::use_location,
};

use crate::app::ConfigCtx;
use crate::components::icons::{
    IconArchive, IconBookOpen, IconCross, IconHome,
};
use crate::components::theme_switcher::ThemeSwitcher;
use crate::services::config_service;

struct Tab {
    label: &'static str,
    path:  &'static str,
}

const TABS: &[Tab] = &[
    Tab { label: "Accueil",      path: "/"            },
    Tab { label: "Communiants",  path: "/communiants" },
    Tab { label: "Cathécomènes", path: "/cathekomens" },
    Tab { label: "Archives",     path: "/archives"    },
];

fn tab_icon(i: usize) -> impl IntoView {
    match i {
        0 => view! { <IconHome     class="w-4 h-4" /> }.into_any(),
        1 => view! { <IconCross    class="w-4 h-4" /> }.into_any(),
        2 => view! { <IconBookOpen class="w-4 h-4" /> }.into_any(),
        3 => view! { <IconArchive  class="w-4 h-4" /> }.into_any(),
        _ => view! { <span /> }.into_any(),
    }
}

fn active_index(pathname: &str) -> usize {
    TABS.iter()
        .position(|t| {
            if t.path == "/" {
                pathname == "/"
            } else {
                pathname.starts_with(t.path)
            }
        })
        .unwrap_or(0)
}

#[component]
pub fn Navbar() -> impl IntoView {
    let location = use_location();
    let idx = Memo::new(move |_| active_index(&location.pathname.get()));

    // Contexte de configuration partagé depuis App
    let config_ctx = use_context::<ConfigCtx>().expect("ConfigCtx manquant");

    // État de la confirmation de reconfiguration
    let confirming = RwSignal::new(false);
    let resetting  = RwSignal::new(false);

    let on_reset_confirm = move |_| {
        resetting.set(true);
        leptos::task::spawn_local(async move {
            match config_service::reset_config().await {
                Ok(_) => {
                    config_ctx.is_configured.set(Some(false));
                }
                Err(_) => {
                    // En cas d'erreur, on referme juste la confirmation
                    confirming.set(false);
                }
            }
            resetting.set(false);
        });
    };

    view! {
        <header class="sticky top-0 z-50 \
                       bg-white/80 dark:bg-gray-900/80 \
                       backdrop-blur-md \
                       border-b border-gray-200 dark:border-gray-700 \
                       shadow-sm">
            <div class="container mx-auto px-3 sm:px-4 max-w-6xl">
                <div class="flex items-center justify-between h-14 sm:h-16 gap-2">

                    // ── Logo / Titre ───────────────────────────────────────────
                    <div class="flex items-center gap-2 shrink-0">
                        <img src=LOGO_SRC class="w-6 h-6 sm:w-7 sm:h-7 object-cover rounded" alt="Logo" />
                        <div class="leading-tight hidden xs:block sm:block">
                            <p class="font-bold text-gray-800 dark:text-white text-xs sm:text-sm md:text-base">
                                "FJKM Ambalavao Isotry"
                            </p>
                        </div>
                    </div>

                    // ── Onglets ────────────────────────────────────────────────
                    <nav class="flex overflow-x-auto scrollbar-none flex-1 justify-center"
                         aria-label="Navigation principale">
                        <div class="relative flex">
                            {TABS
                                .iter()
                                .enumerate()
                                .map(|(i, tab)| {
                                    let path  = tab.path;
                                    let label = tab.label;
                                    view! {
                                        <A
                                            href=path
                                            attr:class=move || {
                                                let base = "nav-tab flex items-center gap-1.5 px-2 sm:px-4 \
                                                            py-4 sm:py-5 text-xs sm:text-sm font-medium \
                                                            whitespace-nowrap shrink-0";
                                                if idx.get() == i {
                                                    format!("{base} text-blue-600 dark:text-blue-400")
                                                } else {
                                                    format!("{base} text-gray-500 dark:text-gray-400 \
                                                             hover:text-blue-500 dark:hover:text-blue-300")
                                                }
                                            }
                                        >
                                            {tab_icon(i)}
                                            <span class="hidden sm:inline">{label}</span>
                                        </A>
                                    }
                                })
                                .collect_view()}

                            // ── Indicateur glissant ────────────────────────────
                            <div
                                class="nav-indicator"
                                style=move || {
                                    format!("transform: translateX({}%);", idx.get() * 100)
                                }
                            />
                        </div>
                    </nav>

                    // ── Droite : thème + reconfigurer ─────────────────────────
                    <div class="shrink-0 flex items-center gap-1">

                        // ── Bouton reconfigurer (normal / confirmation) ─────────
                        {move || if confirming.get() {
                            // ── Mode confirmation ──────────────────────────────
                            view! {
                                <div class="flex items-center gap-1 \
                                            bg-orange-50 dark:bg-orange-900/30 \
                                            border border-orange-200 dark:border-orange-700 \
                                            rounded-lg px-2 py-1">
                                    <span class="text-xs text-orange-700 dark:text-orange-300 whitespace-nowrap">
                                        "Reconfigurer ?"
                                    </span>
                                    <button
                                        class="text-xs font-semibold px-2 py-0.5 rounded \
                                               bg-orange-500 hover:bg-orange-600 text-white \
                                               transition-colors disabled:opacity-50"
                                        disabled=move || resetting.get()
                                        on:click=on_reset_confirm
                                    >
                                        {move || if resetting.get() { "…" } else { "Oui" }}
                                    </button>
                                    <button
                                        class="text-xs font-semibold px-2 py-0.5 rounded \
                                               bg-gray-200 dark:bg-gray-700 \
                                               text-gray-700 dark:text-gray-300 \
                                               hover:bg-gray-300 dark:hover:bg-gray-600 \
                                               transition-colors"
                                        on:click=move |_| confirming.set(false)
                                    >
                                        "Non"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            // ── Bouton icône engrenage ─────────────────────────
                            view! {
                                <button
                                    title="Reconfigurer le mode réseau"
                                    class="p-1.5 rounded-lg \
                                           text-gray-400 dark:text-gray-500 \
                                           hover:text-orange-500 dark:hover:text-orange-400 \
                                           hover:bg-orange-50 dark:hover:bg-orange-900/20 \
                                           transition-colors"
                                    on:click=move |_| confirming.set(true)
                                >
                                    // ⚙ Engrenage
                                    <svg xmlns="http://www.w3.org/2000/svg"
                                         class="w-4 h-4" fill="none" viewBox="0 0 24 24"
                                         stroke="currentColor" stroke-width="1.8">
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                        <path stroke-linecap="round" stroke-linejoin="round"
                                              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                    </svg>
                                </button>
                            }.into_any()
                        }}

                        <ThemeSwitcher />
                    </div>

                </div>
            </div>
        </header>
    }
}
