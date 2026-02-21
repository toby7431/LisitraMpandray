/// Barre de navigation avec 4 onglets et indicateur glissant animé.
/// L'indicateur utilise `transform: translateX(n × 100%)` sur un élément
/// de largeur 25% — la transition CSS utilise un cubic-bezier « ressort ».
use leptos::prelude::*;
use leptos_router::{
    components::A,
    hooks::use_location,
};

use crate::components::theme_switcher::ThemeSwitcher;

struct Tab {
    label: &'static str,
    path:  &'static str,
    icon:  &'static str,
}

const TABS: &[Tab] = &[
    Tab { label: "Accueil",      path: "/",            icon: "🏠" },
    Tab { label: "Communiants",  path: "/communiants", icon: "✝️"  },
    Tab { label: "Cathécomènes", path: "/cathekomens", icon: "📖" },
    Tab { label: "Archives",     path: "/archives",    icon: "📦" },
];

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
                        <span class="text-xl sm:text-2xl" aria-hidden="true">"⛪"</span>
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
                                let icon  = tab.icon;
                                view! {
                                    <A
                                        href=path
                                        attr:class=move || {
                                            let base = "flex items-center gap-1 px-2 sm:px-4 py-4 sm:py-5 \
                                                        text-xs sm:text-sm font-medium transition-colors \
                                                        duration-200 whitespace-nowrap shrink-0";
                                            if idx.get() == i {
                                                format!("{base} text-blue-600 dark:text-blue-400")
                                            } else {
                                                format!("{base} text-gray-600 dark:text-gray-300 \
                                                         hover:text-blue-500 dark:hover:text-blue-300")
                                            }
                                        }
                                    >
                                        // Icône toujours visible
                                        <span class="text-base leading-none">{icon}</span>
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
