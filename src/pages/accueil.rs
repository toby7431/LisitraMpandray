use js_sys::{Date, Math};
use leptos::prelude::*;

use crate::components::icons::PageIcon;
use crate::services::db_service;
use crate::utils::{format_ariary, sleep_ms};

// ─── Versets bibliques — sélection aléatoire à chaque ouverture ──────────────

const VERSES: &[(&str, &str)] = &[
    ("Jaona 3:16",
     "Fa toy izany no nitiavan'Andriamanitra izao tontolo izao: nomeny \
      ny Zanani-lahy tokana, mba tsy ho very izay rehetra mino Azy, \
      fa hanana fiainana mandrakizay."),
    ("Filipiana 4:13",
     "Izay rehetra vitako amin'ny mampahery ahy."),
    ("Salamo 23:1",
     "Jehovah no Mpiandry ahy; Tsy hanan-java-mahory aho."),
    ("Romana 8:28",
     "Ary fantatray fa ny zavatra rehetra dia miara-miasa hahasoa \
      izay tia an'Andriamanitra."),
    ("Josoa 1:9",
     "Mahereza sy matanjaha; aza matahotra, ary aza mivadi-po; \
      fa Jehovah Andriamanitrao no momba anao na aiza na aiza alehanao."),
    ("Matio 11:28",
     "Mankanesa amiko, ianareo rehetra izay miasa fatratra sy mavesatra entana, \
      dia hampasoavy anareo Aho."),
    ("Ohabolana 3:5-6",
     "Matokia an'i Jehovah amin'ny fonao rehetra, ary aza miankina \
      amin'ny fahalalanao; ekeo Izy amin'ny alalanao rehetra, \
      dia Izy no hamaivana ny làlanao."),
    ("Isaia 40:31",
     "Fa izay miandry an'i Jehovah no hananany hery vaovao; \
      Hanidina toy ny fanihin'ny voromahery izy."),
    ("Salamo 46:2",
     "Andriamanitra no fialofantsika sy heritsika, \
      Mpamonjy mora azo amin'ny fahoriana."),
    ("1 Korintiana 13:13",
     "Fa ankehitriny dia mitoetra ireo telo ireo: ny finoana sy ny fanantenana \
      ary ny fitiavana; fa ny fitiavana no lehibe indrindra amin'ireo."),
];

// ─── Helpers async ────────────────────────────────────────────────────────────

const ANIM_STEPS: i64 = 35;

async fn animate_count(signal: RwSignal<i64>, target: i64) {
    if target <= 0 {
        signal.set(0);
        return;
    }
    for i in 1..=ANIM_STEPS {
        signal.set(target * i / ANIM_STEPS);
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
    let communiants_display: RwSignal<i64> = RwSignal::new(0);
    let cathekumens_display: RwSignal<i64> = RwSignal::new(0);
    let contributions_display: RwSignal<i64> = RwSignal::new(0);

    // Chargement + animation au montage
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(list) = db_service::get_members_by_type("Communiant").await {
                animate_count(communiants_display, list.len() as i64).await;
            }
            if let Ok(list) = db_service::get_members_by_type("Cathekomen").await {
                animate_count(cathekumens_display, list.len() as i64).await;
            }
            if let Ok(Some(summary)) = db_service::get_year_summary(current_year).await {
                if let Ok(total) = summary.total.parse::<f64>() {
                    animate_count(contributions_display, total as i64).await;
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
                    "✦ Andininy androany ✦"
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
            <section class="grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-2 gap-4 \
                            max-w-2xl mx-auto w-full px-4">

                <StatCard
                    icon="cross"
                    title="Mpandray"
                    subtitle=""
                    color_class="from-blue-500 to-indigo-600"
                    count=communiants_display
                />

                <StatCard
                    icon="book"
                    title="Katekomena"
                    subtitle=""
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
                            {format!("Adidy {}", current_year)}
                        </p>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                            "Fitambarana adidy amin'ity taona ity"
                        </p>
                    </div>
                    <p class="text-2xl sm:text-3xl font-bold font-mono \
                               text-gray-800 dark:text-white shrink-0">
                        {move || format_ariary(&contributions_display.get().to_string())}
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
    count: RwSignal<i64>,
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
