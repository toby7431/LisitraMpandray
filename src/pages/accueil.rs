use js_sys::{Function, Promise};
use leptos::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::services::db_service;

// ─── Versets bibliques ────────────────────────────────────────────────────────

const VERSES: &[(&str, &str)] = &[
    (
        "Jean 3:16",
        "Car Dieu a tant aimé le monde qu'il a donné son Fils unique, afin que \
         quiconque croit en lui ne périsse point, mais qu'il ait la vie éternelle.",
    ),
    (
        "Philippiens 4:13",
        "Je puis tout par celui qui me fortifie.",
    ),
    (
        "Psaume 23:1",
        "L'Éternel est mon berger : je ne manquerai de rien.",
    ),
    (
        "Romains 8:28",
        "Nous savons, du reste, que toutes choses concourent au bien de ceux qui aiment Dieu.",
    ),
    (
        "Josué 1:9",
        "Sois fort et courageux ! Ne te frappe pas de terreur et ne t'effraie pas, \
         car l'Éternel, ton Dieu, est avec toi dans tout ce que tu entreprendras.",
    ),
    (
        "Matthieu 11:28",
        "Venez à moi, vous tous qui êtes fatigués et chargés, et je vous donnerai du repos.",
    ),
    (
        "Proverbes 3:5-6",
        "Confie-toi en l'Éternel de tout ton cœur, et ne t'appuie pas sur ta sagesse ; \
         reconnais-le dans toutes tes voies, et il aplanira tes sentiers.",
    ),
    (
        "Ésaïe 40:31",
        "Ceux qui se confient en l'Éternel renouvellent leur force. \
         Ils prennent le vol comme les aigles.",
    ),
    (
        "Psaume 46:2",
        "Dieu est pour nous un refuge et un appui, un secours qui ne manque jamais dans la détresse.",
    ),
    (
        "1 Corinthiens 13:13",
        "Maintenant ces trois choses demeurent : la foi, l'espérance, la charité ; \
         mais la plus grande de ces choses, c'est la charité.",
    ),
];

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
    // Verset aléatoire — choisi une fois à la création du composant
    let verse_idx = (js_sys::Math::random() * VERSES.len() as f64) as usize % VERSES.len();
    let (verse_ref, verse_text) = VERSES[verse_idx];

    let current_year = js_sys::Date::new_0().get_full_year() as i32;

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
        <div class="animate-fade-in space-y-6 sm:space-y-8">

            // ── Verset du jour ─────────────────────────────────────────────────
            <section class="mx-auto max-w-2xl px-4 pt-6 sm:pt-10">
                <div class="relative rounded-2xl \
                            border border-indigo-100 dark:border-indigo-900/50 \
                            bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                            px-6 py-5 sm:px-8 sm:py-6 shadow-sm overflow-hidden">

                    // Barre d'accent verticale
                    <div class="absolute left-0 top-0 bottom-0 w-1 \
                                bg-gradient-to-b from-indigo-400 to-purple-500 \
                                rounded-l-2xl" />

                    <p class="text-xs font-semibold \
                               text-indigo-500 dark:text-indigo-400 \
                               uppercase tracking-widest mb-3 ml-2">
                        "Verset du jour"
                    </p>

                    <blockquote class="verse-animate ml-2">
                        <p class="text-base sm:text-lg \
                                  text-gray-700 dark:text-gray-200 \
                                  italic leading-relaxed">
                            {format!("« {} »", verse_text)}
                        </p>
                        <footer class="mt-2 text-xs sm:text-sm font-semibold \
                                       text-indigo-600 dark:text-indigo-400">
                            "— " {verse_ref}
                        </footer>
                    </blockquote>
                </div>
            </section>

            // ── Cartes de statistiques ─────────────────────────────────────────
            <section class="grid grid-cols-1 sm:grid-cols-2 gap-4 \
                            max-w-2xl mx-auto w-full px-4">

                <StatCard
                    icon="✝️"
                    title="Communiants"
                    subtitle="Membres actifs"
                    color_class="from-blue-500 to-indigo-600"
                    count=communiants_display
                />

                <StatCard
                    icon="📖"
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
                        {move || format!("{} Ar", contributions_display.get())}
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
                 flex items-center justify-center \
                 text-xl sm:text-2xl shadow-sm"
            )>
                {icon}
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
