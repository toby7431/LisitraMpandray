/// Barre de navigation avec 4 onglets et indicateur glissant animé.
/// L'indicateur utilise `transform: translateX(n × 100%)` sur un élément
/// de largeur 25% — la transition CSS utilise un cubic-bezier « ressort ».
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::use_location,
};

use crate::components::icons::{
    IconArchive, IconBookOpen, IconChurch, IconCross, IconHome,
};
use crate::components::theme_switcher::ThemeSwitcher;

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
                        <IconChurch class="w-6 h-6 sm:w-7 sm:h-7 text-blue-600 dark:text-blue-400" />
                        <div class="leading-tight hidden xs:block sm:block">
                            <p class="font-bold text-gray-800 dark:text-white text-xs sm:text-sm md:text-base">
                                "Église Gestion"
                            </p>
                            <p class="text-xs text-gray-500 dark:text-gray-400 hidden md:block">
                                "Madagascar"
                            </p>
                        </div>
                    </div>

                    // ── Onglets ────────────────────────────────────────────────
                    <nav class="relative flex overflow-x-auto scrollbar-none flex-1 justify-center"
                         aria-label="Navigation principale">
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
                                        // Icône SVG — hérite la couleur du parent via currentColor
                                        {tab_icon(i)}
                                        // Label masqué sur mobile, visible à partir de sm
                                        <span class="hidden sm:inline">{label}</span>
                                    </A>
                                }
                            })
                            .collect_view()}

                        // ── Indicateur glissant ────────────────────────────────
                        <div
                            class="nav-indicator"
                            style=move || {
                                format!("transform: translateX({}%);", idx.get() * 100)
                            }
                        />
                    </nav>

                    // ── Sélecteur de thème ─────────────────────────────────────
                    <div class="shrink-0">
                        <ThemeSwitcher />
                    </div>

                </div>
            </div>
        </header>
    }
}
