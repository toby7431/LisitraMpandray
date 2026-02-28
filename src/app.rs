use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use web_sys::window;

use crate::{
    components::{navbar::Navbar, sky_canvas::SkyCanvas, titlebar::TitleBar, year_toast::YearToast},
    models::year_summary::YearSummary,
    pages::{
        accueil::Accueil, archives::Archives, cathekomens::Cathekomens,
        communiants::Communiants,
    },
    services::db_service,
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


}

// ─── Contextes globaux ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct ThemeCtx {
    pub theme: RwSignal<Theme>,
}

/// Contexte pour le toast de clôture annuelle.
/// `data` contient le résumé de l'année venant d'être clôturée, ou `None`.
#[derive(Clone, Copy)]
pub struct ToastCtx {
    pub data: RwSignal<Option<YearSummary>>,
}

// ─── Helpers ────────────────────────────────────────────────────────────────

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

pub fn apply_theme_to_dom(theme: Theme, with_transition: bool) {
    let dark = match theme {
        Theme::Dark   => true,
        Theme::Light  => false,
        Theme::System => system_prefers_dark(),
    };
    if let Some(html) = window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        if with_transition {
            let _ = html.class_list().add_1("theme-transitioning");
            let html2 = html.clone();
            leptos::task::spawn_local(async move {
                sleep_ms(900).await;
                let _ = html2.class_list().remove_1("theme-transitioning");
            });
        }
        if dark {
            let _ = html.class_list().add_1("dark");
            let _ = html.class_list().remove_1("light");
        } else {
            let _ = html.class_list().remove_1("dark");
            let _ = html.class_list().add_1("light");
        }
    }
}

/// Attendre `ms` millisecondes (non-bloquant, WASM-compatible).
async fn sleep_ms(ms: u32) {
    use js_sys::Promise;
    use wasm_bindgen_futures::JsFuture;
    let p = Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &resolve,
                ms as i32,
            )
            .unwrap();
    });
    let _ = JsFuture::from(p).await;
}

// ─── Composant racine ───────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let initial = load_theme();
    apply_theme_to_dom(initial, false); // pas de transition au premier rendu

    let theme = RwSignal::new(initial);
    provide_context(ThemeCtx { theme });

    // Réagit à chaque changement de thème → DOM + localStorage
    // `old.is_some()` = false au premier run, true ensuite → transition seulement lors des bascules
    Effect::new(move |old: Option<()>| {
        let t = theme.get();
        save_theme(t);
        apply_theme_to_dom(t, old.is_some());
    });

    // ── Toast clôture annuelle ───────────────────────────────────────────────
    let toast_data: RwSignal<Option<YearSummary>> = RwSignal::new(None);
    provide_context(ToastCtx { data: toast_data });

    // Vérification immédiate au lancement, puis toutes les 24h
    leptos::task::spawn_local(async move {
        if let Ok(Some(s)) = db_service::check_and_close_previous_year().await {
            toast_data.set(Some(s));
        }
        loop {
            sleep_ms(86_400_000).await; // 24 heures
            if let Ok(Some(s)) = db_service::check_and_close_previous_year().await {
                toast_data.set(Some(s));
            }
        }
    });

    view! {
        <Router>
            // ── Couche 0 : ciel animé (fixed, derrière tout) ──────────────────
            <SkyCanvas />

            // ── Couche 1 : barre de titre personnalisée ───────────────────────
            <TitleBar />

            // ── Couche 2 : contenu scrollable (démarre sous la titlebar) ──────
            <div style="position:fixed;top:36px;left:0;right:0;bottom:0;z-index:10;overflow-y:auto;"
                 class="flex flex-col min-h-full">
                <Navbar />
                <main class="flex-1 container mx-auto px-3 sm:px-4 py-4 sm:py-8 max-w-6xl w-full">
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

            // ── Toast cloche (au-dessus de tout, z-50) ────────────────────────
            <YearToast />
        </Router>
    }
}
