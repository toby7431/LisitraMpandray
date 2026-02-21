/// Modal d'ajout de cotisation + couche confetti.
use js_sys::{Date, Function, Math, Promise};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{models::contribution::ContributionInput, services::db_service};

// ─── Palette confetti ─────────────────────────────────────────────────────────

const CONFETTI_COLORS: &[&str] = &[
    "#f59e0b", "#10b981", "#3b82f6", "#ef4444", "#8b5cf6",
    "#ec4899", "#14b8a6", "#f97316", "#06b6d4", "#a3e635",
    "#fbbf24", "#34d399", "#60a5fa", "#f87171", "#a78bfa",
];

struct Piece {
    x:        f64,   // left en %
    color:    &'static str,
    size:     u32,   // px
    delay:    u32,   // ms
    duration: u32,   // ms
}

impl Piece {
    fn random() -> Self {
        let ci = (Math::random() * CONFETTI_COLORS.len() as f64) as usize;
        Self {
            x:        Math::random() * 98.0,
            color:    CONFETTI_COLORS[ci],
            size:     (Math::random() * 7.0 + 5.0) as u32,
            delay:    (Math::random() * 700.0) as u32,
            duration: (Math::random() * 900.0 + 1600.0) as u32,
        }
    }

    fn style_str(&self) -> String {
        format!(
            "position:fixed;\
             left:{x:.1}%;\
             top:-20px;\
             width:{s}px;height:{s}px;\
             background:{c};\
             border-radius:2px;\
             pointer-events:none;\
             z-index:9999;\
             animation:\
               confetti-fall {dur}ms ease-in {del}ms both,\
               confetti-sway {sw}ms ease-in-out {del}ms both;",
            x = self.x, s = self.size, c = self.color,
            dur = self.duration, del = self.delay,
            sw  = self.duration / 2,
        )
    }
}

// ─── Helper async ─────────────────────────────────────────────────────────────

async fn sleep_ms(ms: u32) {
    let promise = Promise::new(&mut |resolve: Function, _: Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .unwrap();
    });
    let _ = JsFuture::from(promise).await;
}

// ─── Formatage du montant ─────────────────────────────────────────────────────

/// Insère des espaces fins comme séparateurs de milliers.
fn fmt_thousands(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut r = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            r.push('\u{202f}'); // espace fine insécable
        }
        r.push(c);
    }
    r
}

/// Formate la saisie brute en "1 234,50".
///
/// Accepte uniquement chiffres + virgule ; virgule unique ; 2 décimales max.
pub fn fmt_amount(raw: &str) -> String {
    let mut int_s = String::new();
    let mut dec_s = String::new();
    let mut has_comma = false;

    for c in raw.chars() {
        if c.is_ascii_digit() {
            if has_comma {
                if dec_s.len() < 2 { dec_s.push(c); }
            } else {
                int_s.push(c);
            }
        } else if c == ',' && !has_comma {
            has_comma = true;
        }
    }

    let int_fmt = fmt_thousands(&int_s);
    if has_comma { format!("{},{}", int_fmt, dec_s) } else { int_fmt }
}

/// "1 234,50" (espace fine) → "1234.50" pour le backend.
fn amount_to_backend(display: &str) -> String {
    display
        .chars()
        .filter(|&c| c.is_ascii_digit() || c == ',')
        .collect::<String>()
        .replace(',', ".")
}

/// Date d'aujourd'hui au format "YYYY-MM-DD".
fn today() -> String {
    let d = Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        d.get_full_year() as i32,
        d.get_month() + 1,
        d.get_date()
    )
}

// ─── Couche Confetti ──────────────────────────────────────────────────────────

/// Couche fixe qui affiche les confettis quand `active` passe à `true`.
/// Se désactive automatiquement après l'animation.
#[component]
pub fn ConfettiLayer(active: RwSignal<bool>) -> impl IntoView {
    let pieces: RwSignal<Vec<String>> = RwSignal::new(vec![]);

    Effect::new(move |_| {
        if !active.get() {
            return;
        }
        // Génère 60 pièces
        let styles: Vec<String> = (0..60).map(|_| Piece::random().style_str()).collect();
        pieces.set(styles);

        // Efface après la dernière animation (max ~2.4 s + 0.7 s délai = 3.1 s)
        leptos::task::spawn_local(async move {
            sleep_ms(3400).await;
            active.set(false);
            pieces.set(vec![]);
        });
    });

    view! {
        <div style="pointer-events:none;position:fixed;inset:0;z-index:9998;overflow:hidden;">
            {move || pieces.get().into_iter().map(|css_str| {
                view! { <div attr:style={css_str} /> }
            }).collect_view()}
        </div>
    }
}

// ─── Modal Cotisation ─────────────────────────────────────────────────────────

const LABEL: &str = "block text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1";
const INPUT: &str = "w-full px-3 py-2 text-sm \
                     bg-gray-50 dark:bg-gray-700/60 \
                     border border-gray-200 dark:border-gray-600 \
                     rounded-xl text-gray-800 dark:text-white \
                     placeholder-gray-400 dark:placeholder-gray-500 \
                     focus:outline-none focus:ring-2 focus:ring-emerald-400 transition";

/// Modal d'ajout de cotisation pour un membre.
#[component]
pub fn ContributionModal(
    /// ID du membre concerné.
    membre_id:       i64,
    /// Nom affiché dans le titre du modal.
    membre_nom:      String,
    /// Signal d'ouverture — ferme le modal quand `false`.
    open:            RwSignal<bool>,
    /// Incrémenter pour rafraîchir la liste des membres.
    refresh_ctr:     RwSignal<u32>,
    /// Passe à `true` pour déclencher les confettis.
    confetti_active: RwSignal<bool>,
) -> impl IntoView {
    // ── Champs du formulaire ──────────────────────────────────────────────────
    let f_date:    RwSignal<String>         = RwSignal::new(today());
    let f_period:  RwSignal<String>         = RwSignal::new(String::new());
    let f_erreur:  RwSignal<Option<String>> = RwSignal::new(None);
    let f_loading: RwSignal<bool>           = RwSignal::new(false);

    // Montant : stocke la chaîne formatée "1 234,50" directement
    let f_amount:    RwSignal<String>              = RwSignal::new(String::new());
    let amount_node: NodeRef<leptos::html::Input>  = NodeRef::new();

    // ── Gestion du montant ────────────────────────────────────────────────────
    let on_amount_input = move |_| {
        let el = match amount_node.get() { Some(e) => e, None => return };
        let raw = el.value();
        let formatted = fmt_amount(&raw);
        f_amount.set(formatted.clone());
        el.set_value(&formatted);
        let pos = formatted.len() as u32;
        let _ = el.set_selection_range(pos, pos);
    };

    // Empêche toute saisie autre que chiffres et virgule
    let on_amount_keydown = move |ev: web_sys::KeyboardEvent| {
        let k = ev.key();
        let allowed = k.len() > 1  // touches de contrôle (Backspace, ArrowLeft…)
            || k.chars().all(|c| c.is_ascii_digit())
            || k == ",";
        if !allowed { ev.prevent_default(); }
    };

    // ── Soumission ────────────────────────────────────────────────────────────
    let soumettre = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let amount_backend = amount_to_backend(&f_amount.get());
        if amount_backend.is_empty() || amount_backend == "." {
            f_erreur.set(Some("Veuillez saisir un montant valide.".into()));
            return;
        }

        let input = ContributionInput {
            member_id:    membre_id,
            payment_date: f_date.get(),
            period:       f_period.get().trim().to_string(),
            amount:       amount_backend,
        };

        f_loading.set(true);
        f_erreur.set(None);

        leptos::task::spawn_local(async move {
            match db_service::create_contribution(&input).await {
                Ok(_) => {
                    open.set(false);
                    refresh_ctr.update(|n| *n += 1);
                    confetti_active.set(true);
                }
                Err(e) => f_erreur.set(Some(e)),
            }
            f_loading.set(false);
        });
    };

    // ─── Vue ──────────────────────────────────────────────────────────────────
    view! {
        // Overlay
        <div
            class="fixed inset-0 z-50 flex items-center justify-center p-4 \
                   bg-black/40 dark:bg-black/60 backdrop-blur-sm"
            on:click=move |ev| {
                if ev.target() == ev.current_target() { open.set(false); }
            }
        >
            // Panneau
            <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl \
                        w-full max-w-md border border-gray-100 dark:border-gray-700 \
                        overflow-hidden">

                // ── En-tête ──────────────────────────────────────────────────
                <div class="flex items-center justify-between px-6 pt-5 pb-4 \
                            border-b border-gray-100 dark:border-gray-700">
                    <div>
                        <h2 class="text-base font-bold text-gray-800 dark:text-white">
                            "Nouvelle cotisation"
                        </h2>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            {membre_nom.clone()}
                        </p>
                    </div>
                    <button
                        on:click=move |_| open.set(false)
                        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 \
                               text-xl leading-none transition-colors"
                    >
                        "✕"
                    </button>
                </div>

                // ── Formulaire ───────────────────────────────────────────────
                <form on:submit=soumettre class="px-6 py-5 space-y-4">

                    // Date + Période côte à côte
                    <div class="grid grid-cols-2 gap-3">
                        <div>
                            <label class=LABEL>"Date *"</label>
                            <input
                                type="date" required
                                class=INPUT
                                prop:value=move || f_date.get()
                                on:input=move |ev| f_date.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class=LABEL>"Période *"</label>
                            <input
                                type="text" required
                                placeholder="ex : 2025"
                                class=INPUT
                                prop:value=move || f_period.get()
                                on:input=move |ev| f_period.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    // Montant
                    <div>
                        <label class=LABEL>"Montant (Ariary) *"</label>
                        <div class="relative">
                            <input
                                type="text"
                                inputmode="decimal"
                                placeholder="0"
                                node_ref=amount_node
                                class="w-full pl-3 pr-16 py-2 text-sm \
                                       bg-gray-50 dark:bg-gray-700/60 \
                                       border border-gray-200 dark:border-gray-600 \
                                       rounded-xl text-gray-800 dark:text-white font-mono \
                                       placeholder-gray-400 dark:placeholder-gray-500 \
                                       focus:outline-none focus:ring-2 focus:ring-emerald-400 transition"
                                prop:value=move || f_amount.get()
                                on:input=on_amount_input
                                on:keydown=on_amount_keydown
                            />
                            <span class="absolute right-3 top-1/2 -translate-y-1/2 \
                                         text-xs font-semibold \
                                         text-gray-400 dark:text-gray-500 select-none">
                                "Ar"
                            </span>
                        </div>
                        // Aperçu du montant formaté
                        {move || {
                            let v = f_amount.get();
                            (!v.is_empty()).then(|| view! {
                                <p class="mt-1 text-xs text-emerald-600 dark:text-emerald-400 \
                                           font-mono font-semibold">
                                    {format!("{}\u{202f}Ar", v)}
                                </p>
                            })
                        }}
                    </div>

                    // Erreur
                    {move || f_erreur.get().map(|e| view! {
                        <div class="p-3 bg-red-50 dark:bg-red-900/30 \
                                    border border-red-200 dark:border-red-700 \
                                    rounded-xl text-red-700 dark:text-red-300 text-xs">
                            "⚠️ " {e}
                        </div>
                    })}

                    // Boutons
                    <div class="flex gap-3 justify-end pt-1">
                        <button
                            type="button"
                            on:click=move |_| open.set(false)
                            class="px-4 py-2 text-sm font-medium \
                                   text-gray-600 dark:text-gray-300 \
                                   bg-gray-100 dark:bg-gray-700 \
                                   hover:bg-gray-200 dark:hover:bg-gray-600 \
                                   rounded-xl transition-colors"
                        >
                            "Annuler"
                        </button>
                        <button
                            type="submit"
                            disabled=move || f_loading.get()
                            class="px-4 py-2 text-sm font-semibold text-white \
                                   bg-emerald-600 hover:bg-emerald-700 \
                                   disabled:opacity-60 disabled:cursor-wait \
                                   rounded-xl transition-colors shadow-sm"
                        >
                            {move || if f_loading.get() { "Enregistrement…" } else { "💾 Enregistrer" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
