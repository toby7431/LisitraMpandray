/// Gestion du thème clair/sombre/système.
///
/// Contient l'enum `Theme`, les contextes Leptos `ThemeCtx` / `ToastCtx`
/// et les helpers DOM (lecture/écriture localStorage, application au <html>).
use leptos::prelude::*;
use web_sys::window;

use crate::{models::year_summary::YearSummary, utils::sleep_ms};

// ─── Enum Thème ──────────────────────────────────────────────────────────────

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

// ─── Contextes Leptos ────────────────────────────────────────────────────────

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

// ─── Helpers DOM ─────────────────────────────────────────────────────────────

pub(crate) fn load_theme() -> Theme {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("eglise_theme").ok().flatten())
        .map(|v| Theme::from_str(&v))
        .unwrap_or(Theme::System)
}

pub(crate) fn save_theme(theme: Theme) {
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
