/// Toast de notification — clôture automatique d'une année.
///
/// Affiché en bas à droite quand `ToastCtx.data` passe à `Some(YearSummary)`.
/// Auto-dismiss après 8 s (7.6 s affichage + 0.4 s animation de sortie).
use leptos::prelude::*;
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;

use crate::app::ToastCtx;
use crate::components::icons::{IconBell, IconX};

// ── Helper local ──────────────────────────────────────────────────────────────

async fn sleep_ms(ms: u32) {
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

/// Formate une chaîne Decimal en "1 234 567 Ar" (partie entière uniquement).
fn format_ariary(amount_str: &str) -> String {
    let n = amount_str.parse::<f64>().unwrap_or(0.0) as i64;
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }
    format!("{} Ar", result)
}

// ── Composant ─────────────────────────────────────────────────────────────────

#[component]
pub fn YearToast() -> impl IntoView {
    let ctx = use_context::<ToastCtx>().expect("ToastCtx manquant");
    let visible  = RwSignal::new(false);
    let exiting  = RwSignal::new(false);

    // Réagit quand une clôture vient d'être effectuée
    Effect::new(move |_| {
        if ctx.data.get().is_some() {
            visible.set(true);
            exiting.set(false);
            // Auto-dismiss
            leptos::task::spawn_local(async move {
                sleep_ms(7_600).await;
                exiting.set(true);
                sleep_ms(400).await;
                visible.set(false);
                ctx.data.set(None);
            });
        }
    });

    let close = move |_| {
        if !exiting.get() {
            exiting.set(true);
            leptos::task::spawn_local(async move {
                sleep_ms(400).await;
                visible.set(false);
                ctx.data.set(None);
            });
        }
    };

    move || {
        if !visible.get() {
            return view! { <div /> }.into_any();
        }
        let summary = match ctx.data.get() {
            Some(s) => s,
            None    => return view! { <div /> }.into_any(),
        };
        let year  = summary.year;
        let total = format_ariary(&summary.total);
        let note  = summary.note.clone();

        let wrapper_cls = if exiting.get() {
            "fixed bottom-6 right-6 z-50 w-80 rounded-2xl shadow-2xl overflow-hidden toast-exit"
        } else {
            "fixed bottom-6 right-6 z-50 w-80 rounded-2xl shadow-2xl overflow-hidden toast-enter"
        };
        let progress_cls = if exiting.get() {
            "h-full bg-amber-500"
        } else {
            "h-full bg-amber-500 toast-progress"
        };

        view! {
            <div class={wrapper_cls}>
                // ── Bande ambre : icône + titre + bouton fermer ────────────────
                <div class="bg-gradient-to-r from-amber-500 to-orange-400 \
                            px-4 py-3 flex items-center gap-3">
                    <div class="bell-ring select-none shrink-0">
                        <IconBell class="w-6 h-6 text-white" />
                    </div>
                    <div class="flex-1 min-w-0">
                        <p class="text-white font-bold text-sm leading-tight">
                            "Année clôturée automatiquement"
                        </p>
                        <p class="text-amber-100 text-xs mt-0.5">
                            {year.to_string()}
                        </p>
                    </div>
                    <button
                        on:click=close
                        class="text-white/70 hover:text-white flex-shrink-0 \
                               transition-colors duration-150 p-0.5 rounded"
                        aria-label="Fermer"
                    >
                        <IconX class="w-4 h-4" />
                    </button>
                </div>

                // ── Corps : total + note ───────────────────────────────────────
                <div class="bg-white dark:bg-gray-800 px-4 py-3">
                    <p class="text-xs text-gray-500 dark:text-gray-400 mb-1">
                        "Total archivé"
                    </p>
                    <p class="text-lg font-bold text-gray-800 dark:text-white font-mono">
                        {total}
                    </p>
                    {note.map(|n| view! {
                        <p class="text-xs text-gray-400 dark:text-gray-500 mt-1.5 italic \
                                  leading-snug line-clamp-2">
                            {n}
                        </p>
                    })}
                </div>

                // ── Barre de progression ───────────────────────────────────────
                <div class="h-1 bg-amber-100 dark:bg-amber-900/30">
                    <div class={progress_cls} style="width:100%" />
                </div>
            </div>
        }
        .into_any()
    }
}
