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
    Tab { label: "Accueil",       path: "/",             icon: "🏠" },
    Tab { label: "Communiants",   path: "/communiants",  icon: "✝️"  },
    Tab { label: "Cathécomènes",  path: "/cathekomens",  icon: "📖" },
    Tab { label: "Archives",      path: "/archives",     icon: "📦" },
];

fn active_index(pathname: &str) -> usize {
    // Correspondance exacte pour "/" pour éviter les faux positifs
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
            <div class="container mx-auto px-4 max-w-6xl">
                <div class="flex items-center justify-between h-16">

                    // ── Logo / Titre ───────────────────────────────────────
                    <div class="flex items-center gap-3 shrink-0">
                        <span class="text-2xl" aria-hidden="true">"⛪"</span>
                        <div class="leading-tight">
                            <p class="font-bold text-gray-800 dark:text-white text-sm sm:text-base">
                                "Église Gestion"
                            </p>
                            <p class="text-xs text-gray-500 dark:text-gray-400 hidden sm:block">
                                "Madagascar"
                            </p>
                        </div>
                    </div>

                    // ── Onglets ───────────────────────────────────────────
                    <nav class="relative flex" aria-label="Navigation principale">
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
                                            let base = "flex items-center gap-1.5 px-3 sm:px-5 py-5 \
                                                        text-sm font-medium transition-colors duration-200";
                                            if idx.get() == i {
                                                format!("{base} text-blue-600 dark:text-blue-400")
                                            } else {
                                                format!("{base} text-gray-600 dark:text-gray-300 \
                                                         hover:text-blue-500 dark:hover:text-blue-300")
                                            }
                                        }
                                    >
                                        <span class="text-base leading-none hidden sm:inline">{icon}</span>
                                        <span>{label}</span>
                                    </A>
                                }
                            })
                            .collect_view()}

                        // ── Indicateur glissant ───────────────────────────
                        // Largeur = 25% du conteneur (100% / 4 onglets)
                        // translateX(n × 100%) déplace de n × 25% du parent
                        <div
                            class="nav-indicator"
                            style=move || {
                                format!("transform: translateX({}%);", idx.get() * 100)
                            }
                        />
                    </nav>

                    // ── Sélecteur de thème ────────────────────────────────
                    <ThemeSwitcher />

                </div>
            </div>
        </header>
    }
}
