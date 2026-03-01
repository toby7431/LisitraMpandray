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
        communiants::Communiants,
    },
    services::db_service,
    theme::{apply_theme_to_dom, load_theme, save_theme, ThemeCtx, ToastCtx},
    utils::sleep_ms,
};

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
