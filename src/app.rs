use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use web_sys::window;

use crate::{
    components::{navbar::Navbar, sky_canvas::SkyCanvas},
    pages::{
        accueil::Accueil, archives::Archives, cathekomens::Cathekomens,
        communiants::Communiants,
    },
};

// ─── Thème ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light  => "light",
            Theme::Dark   => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dark"   => Theme::Dark,
            "system" => Theme::System,
            _        => Theme::Light,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Theme::Light  => "Lumineux",
            Theme::Dark   => "Sombre",
            Theme::System => "Système",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Theme::Light  => "☀️",
            Theme::Dark   => "🌙",
            Theme::System => "💻",
        }
    }
}

// ─── Contexte global ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct ThemeCtx {
    pub theme: RwSignal<Theme>,
}

// ─── Helpers localStorage ───────────────────────────────────────────────────

fn load_theme() -> Theme {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("eglise_theme").ok().flatten())
        .map(|v| Theme::from_str(&v))
        .unwrap_or(Theme::System)
}

fn save_theme(theme: Theme) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item("eglise_theme", theme.as_str());
    }
}

fn system_prefers_dark() -> bool {
    window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mq| mq.matches())
        .unwrap_or(false)
}

pub fn apply_theme_to_dom(theme: Theme) {
    let dark = match theme {
        Theme::Dark   => true,
        Theme::Light  => false,
        Theme::System => system_prefers_dark(),
    };
    if let Some(html) = window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        if dark {
            let _ = html.class_list().add_1("dark");
            let _ = html.class_list().remove_1("light");
        } else {
            let _ = html.class_list().remove_1("dark");
            let _ = html.class_list().add_1("light");
        }
    }
}

// ─── Composant racine ───────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let initial = load_theme();
    apply_theme_to_dom(initial);

    let theme = RwSignal::new(initial);
    provide_context(ThemeCtx { theme });

    // Réagit à chaque changement de thème → DOM + localStorage
    Effect::new(move |_| {
        let t = theme.get();
        save_theme(t);
        apply_theme_to_dom(t);
    });

    view! {
        <Router>
            // Ciel en arrière-plan (canvas fixe)
            <SkyCanvas />

            // Couche contenu
            <div class="relative z-10 flex flex-col min-h-screen">
                <Navbar />
                <main class="flex-1 container mx-auto px-4 py-8 max-w-6xl">
                    <Routes fallback=|| {
                        view! {
                            <p class="text-center text-gray-500 dark:text-gray-400 mt-20 text-lg">
                                "Page introuvable"
                            </p>
                        }
                    }>
                        <Route path=path!("/")             view=Accueil />
                        <Route path=path!("/communiants")  view=Communiants />
                        <Route path=path!("/cathekomens")  view=Cathekomens />
                        <Route path=path!("/archives")     view=Archives />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
