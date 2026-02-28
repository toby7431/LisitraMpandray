use js_sys::{Date, Function, Math, Promise};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::components::icons::PageIcon;
use crate::services::db_service;

// ─── Versets bibliques — sélection aléatoire à chaque ouverture ──────────────

const VERSES: &[(&str, &str)] = &[
    ("Jean 3:16",
     "Car Dieu a tant aimé le monde qu'il a donné son Fils unique, afin que \
      quiconque croit en lui ne périsse point, mais qu'il ait la vie éternelle."),
    ("Philippiens 4:13",
     "Je puis tout par celui qui me fortifie."),
    ("Psaume 23:1",
     "L'Éternel est mon berger : je ne manquerai de rien."),
    ("Romains 8:28",
     "Nous savons, du reste, que toutes choses concourent au bien de ceux \
      qui aiment Dieu."),
    ("Josué 1:9",
     "Sois fort et courageux ! Ne te frappe pas de terreur et ne t'effraie pas, \
      car l'Éternel, ton Dieu, est avec toi dans tout ce que tu entreprendras."),
    ("Matthieu 11:28",
     "Venez à moi, vous tous qui êtes fatigués et chargés, \
      et je vous donnerai du repos."),
    ("Proverbes 3:5-6",
     "Confie-toi en l'Éternel de tout ton cœur, et ne t'appuie pas sur ta sagesse ; \
      reconnais-le dans toutes tes voies, et il aplanira tes sentiers."),
    ("Ésaïe 40:31",
     "Ceux qui se confient en l'Éternel renouvellent leur force. \
      Ils prennent le vol comme les aigles."),
    ("Psaume 46:2",
     "Dieu est pour nous un refuge et un appui, \
      un secours qui ne manque jamais dans la détresse."),
    ("1 Corinthiens 13:13",
     "Maintenant ces trois choses demeurent : la foi, l'espérance, la charité ; \
      mais la plus grande de ces choses, c'est la charité."),
];

// ─── Formatage des montants en Ariary ────────────────────────────────────────

fn format_ariary(n: i64) -> String {
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        // Insère une espace tous les 3 chiffres en partant de la droite
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }
    format!("{} Ar", result)
}

// ─── Helpers async ────────────────────────────────────────────────────────────

async fn sleep_ms(ms: u32) {
    let promise = Promise::new(&mut |resolve: Function, _: Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .unwrap();
    });
    let _ = JsFuture::from(promise).await;
}

async fn animate_count(signal: RwSignal<usize>, target: usize) {
    if target == 0 {
        return;
    }
    let steps: usize = 30;
    for i in 1..=steps {
        signal.set(target * i / steps);
        sleep_ms(15).await;
    }
    signal.set(target);
}

async fn animate_count_i64(signal: RwSignal<i64>, target: i64) {
    if target <= 0 {
        signal.set(target);
        return;
    }
    let steps: i64 = 40;
    for i in 1..=steps {
        signal.set(target * i / steps);
        sleep_ms(15).await;
    }
    signal.set(target);
}

// ─── Composant principal ──────────────────────────────────────────────────────

#[component]
pub fn Accueil() -> impl IntoView {
    let verse_idx = (Math::random() * VERSES.len() as f64) as usize % VERSES.len();
    let (verse_ref, verse_text) = VERSES[verse_idx];

    let current_year = Date::new_0().get_full_year() as i32;

    // Signaux d'affichage animés
    let communiants_display: RwSignal<usize> = RwSignal::new(0);
    let cathekumens_display: RwSignal<usize> = RwSignal::new(0);
    let contributions_display: RwSignal<i64> = RwSignal::new(0);

    // Chargement + animation au montage
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(list) = db_service::get_members_by_type("Communiant").await {
                animate_count(communiants_display, list.len()).await;
            }
            if let Ok(list) = db_service::get_members_by_type("Cathekomen").await {
                animate_count(cathekumens_display, list.len()).await;
            }
            if let Ok(Some(summary)) = db_service::get_year_summary(current_year).await {
                if let Ok(total) = summary.total.parse::<f64>() {
                    animate_count_i64(contributions_display, total as i64).await;
                }
            }
        });
    });

    view! {
        <div class="animate-fade-in space-y-6 sm:space-y-10">

            // ── Verset du jour ─────────────────────────────────────────────────
            <section class="text-center px-4 pt-8 sm:pt-12 md:pt-16 pb-2">

                // Étiquette discrète
                // Clair : blue-800 sur ciel bleu → contraste ~6:1 ✓
                // Sombre : indigo-300 sur ardoise → contraste ~8:1 ✓
                <p class="text-[0.65rem] sm:text-xs font-semibold uppercase \
                           tracking-[0.25em] mb-4 \
                           text-blue-800 dark:text-indigo-300 \
                           select-none">
                    "✦ Verset du jour ✦"
                </p>

                // Séparateur ornemental
                <div class="flex items-center justify-center gap-2 mb-6 sm:mb-8">
                    <div class="h-px w-8 sm:w-12 \
                                bg-blue-800/25 dark:bg-indigo-400/45" />
                    <span class="text-blue-700/55 dark:text-indigo-400/65 text-xs">
                        "✝"
                    </span>
                    <div class="h-px w-8 sm:w-12 \
                                bg-blue-800/25 dark:bg-indigo-400/45" />
                </div>

                // Citation animée — grand titre avec shimmer + glow + respiration
                <blockquote class="verse-animate max-w-xs sm:max-w-xl md:max-w-2xl \
                                   lg:max-w-3xl mx-auto">
                    <p class="grand-titre font-bold italic \
                               text-2xl sm:text-3xl md:text-4xl lg:text-5xl \
                               leading-snug sm:leading-snug">
                        {format!("« {} »", verse_text)}
                    </p>
                    // Référence : casse naturelle, pas de majuscules imposées
                    <footer class="verse-ref mt-5 sm:mt-6 \
                                   text-xs sm:text-sm md:text-base \
                                   font-medium tracking-wide">
                        "— " {verse_ref}
                    </footer>
                </blockquote>

            </section>

            // ── Cartes de statistiques ─────────────────────────────────────────
            <section class="grid grid-cols-1 sm:grid-cols-2 gap-4 \
                            max-w-2xl mx-auto w-full px-4">

                <StatCard
                    icon="cross"
                    title="Communiants"
                    subtitle="Membres actifs"
                    color_class="from-blue-500 to-indigo-600"
                    count=communiants_display
                />

                <StatCard
                    icon="book"
                    title="Cathécomènes"
                    subtitle="En formation"
                    color_class="from-emerald-500 to-teal-600"
                    count=cathekumens_display
                />

            </section>

            // ── Cotisations de l'année en cours ───────────────────────────────
            <section class="max-w-2xl mx-auto w-full px-4 pb-6">
                <div class="rounded-2xl \
                            border border-amber-100 dark:border-amber-900/40 \
                            bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                            px-6 py-5 shadow-sm \
                            flex items-center justify-between gap-4">
                    <div>
                        <p class="text-xs font-semibold \
                                   text-amber-500 dark:text-amber-400 \
                                   uppercase tracking-widest">
                            {format!("Cotisations {}", current_year)}
                        </p>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            "Total encaissé cette année"
                        </p>
                    </div>
                    <p class="text-2xl sm:text-3xl font-bold font-mono \
                               text-gray-800 dark:text-white shrink-0">
                        {move || format_ariary(contributions_display.get())}
                    </p>
                </div>
            </section>

        </div>
    }
}

// ─── Carte statistique ────────────────────────────────────────────────────────

#[component]
fn StatCard(
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    color_class: &'static str,
    count: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                    rounded-2xl p-5 sm:p-6 shadow-sm \
                    border border-gray-100 dark:border-gray-700 \
                    flex flex-col items-center gap-3 \
                    hover:shadow-md transition-shadow duration-200">

            <div class=format!(
                "w-12 h-12 sm:w-14 sm:h-14 rounded-xl \
                 bg-gradient-to-br {color_class} \
                 flex items-center justify-center shadow-sm"
            )>
                <PageIcon name=icon class="w-7 h-7 text-white" />
            </div>

            <div class="text-center">
                <p class="font-semibold text-gray-800 dark:text-white text-sm sm:text-base">
                    {title}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{subtitle}</p>
            </div>

            <p class="text-3xl sm:text-4xl font-bold text-gray-800 dark:text-white tabular-nums">
                {move || count.get().to_string()}
            </p>
        </div>
    }
}
