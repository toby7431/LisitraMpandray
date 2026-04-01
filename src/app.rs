use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::{
    components::{navbar::Navbar, sky_canvas::SkyCanvas, titlebar::TitleBar, year_toast::YearToast},
    models::year_summary::YearSummary,
    pages::{
        accueil::Accueil, archives::Archives, cathekomens::Cathekomens,
        communiants::Communiants, setup::SetupPage,
    },
    services::{config_service, db_service},
    theme::{apply_theme_to_dom, load_theme, save_theme, ThemeCtx, ToastCtx},
    utils::sleep_ms,
};

// ─── Contexte de configuration ───────────────────────────────────────────────

/// Partagé via provide_context pour que la Navbar puisse déclencher
/// une reconfiguration sans dépendance directe vers App.
#[derive(Clone, Copy)]
pub struct ConfigCtx {
    pub is_configured: RwSignal<Option<bool>>,
}

// ─── Application principale (après configuration) ────────────────────────────

#[component]
fn MainApp() -> impl IntoView {
    let toast_data: RwSignal<Option<YearSummary>> = RwSignal::new(None);
    provide_context(ToastCtx { data: toast_data });

    leptos::task::spawn_local(async move {
        if let Ok(Some(s)) = db_service::check_and_close_previous_year().await {
            toast_data.set(Some(s));
        }
        loop {
            sleep_ms(86_400_000).await;
            if let Ok(Some(s)) = db_service::check_and_close_previous_year().await {
                toast_data.set(Some(s));
            }
        }
    });

    view! {
        <Router>
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
            <YearToast />
        </Router>
    }
}

// ─── Composant racine ────────────────────────────────────────────────────────

#[component]
pub fn App() -> impl IntoView {
    let initial = load_theme();
    apply_theme_to_dom(initial, false);

    let theme = RwSignal::new(initial);
    provide_context(ThemeCtx { theme });

    Effect::new(move |old: Option<()>| {
        let t = theme.get();
        save_theme(t);
        apply_theme_to_dom(t, old.is_some());
    });

    // None = chargement, Some(false) = non configuré, Some(true) = configuré
    let is_configured: RwSignal<Option<bool>> = RwSignal::new(None);

    // Fournir le signal au reste de l'arbre (Navbar en a besoin)
    provide_context(ConfigCtx { is_configured });

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            match config_service::get_config().await {
                Ok(Some(_)) => is_configured.set(Some(true)),
                _           => is_configured.set(Some(false)),
            }
        });
    });

    view! {
        <SkyCanvas />
        <TitleBar />

        {move || match is_configured.get() {
            None => view! {
                <div class="fixed inset-0 flex items-center justify-center z-20">
                    <p class="text-blue-900 dark:text-blue-100 text-lg font-medium animate-pulse">
                        "Chargement…"
                    </p>
                </div>
            }.into_any(),

            Some(false) => view! {
                <SetupPage is_configured />
            }.into_any(),

            Some(true) => view! {
                <MainApp />
            }.into_any(),
        }}
    }
}
