use leptos::prelude::*;

#[component]
pub fn Accueil() -> impl IntoView {
    view! {
        <div class="animate-fade-in space-y-10">

            // ── En-tête ───────────────────────────────────────────────────
            <section class="text-center pt-8 pb-4">
                <div class="text-6xl mb-4">"⛪"</div>
                <h1 class="text-3xl sm:text-4xl font-bold text-gray-800 dark:text-white mb-3">
                    "Bienvenue dans Église Gestion"
                </h1>
                <p class="text-gray-500 dark:text-gray-400 text-lg max-w-xl mx-auto">
                    "Gestion des membres, communiants et cathécomènes — Madagascar"
                </p>
            </section>

            // ── Cartes statistiques ───────────────────────────────────────
            <section class="grid grid-cols-1 sm:grid-cols-3 gap-5 max-w-3xl mx-auto">
                <StatCard
                    icon="✝️"
                    title="Communiants"
                    subtitle="Membres actifs"
                    color_class="from-blue-500 to-indigo-600"
                />
                <StatCard
                    icon="📖"
                    title="Cathécomènes"
                    subtitle="En formation"
                    color_class="from-emerald-500 to-teal-600"
                />
                <StatCard
                    icon="📦"
                    title="Archives"
                    subtitle="Membres archivés"
                    color_class="from-gray-400 to-slate-600"
                />
            </section>

            // ── Actions rapides ───────────────────────────────────────────
            <section class="max-w-3xl mx-auto">
                <h2 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-4">
                    "Actions rapides"
                </h2>
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    <QuickAction
                        href="/communiants"
                        icon="➕"
                        label="Ajouter un communiant"
                        desc="Enregistrer un nouveau membre communiant"
                    />
                    <QuickAction
                        href="/cathekomens"
                        icon="📝"
                        label="Ajouter un cathécomène"
                        desc="Enregistrer un nouveau cathécomène"
                    />
                </div>
            </section>

        </div>
    }
}

#[component]
fn StatCard(
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    color_class: &'static str,
) -> impl IntoView {
    view! {
        <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                    rounded-2xl p-6 shadow-sm \
                    border border-gray-100 dark:border-gray-700 \
                    flex flex-col items-center gap-3 \
                    hover:shadow-md transition-shadow duration-200">
            <div class=format!("w-14 h-14 rounded-xl bg-gradient-to-br {color_class} \
                                flex items-center justify-center text-2xl shadow-sm")>
                {icon}
            </div>
            <div class="text-center">
                <p class="font-semibold text-gray-800 dark:text-white">{title}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{subtitle}</p>
            </div>
            <p class="text-3xl font-bold text-gray-800 dark:text-white">"—"</p>
        </div>
    }
}

#[component]
fn QuickAction(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    desc: &'static str,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center gap-4 p-4 \
                   bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                   rounded-xl border border-gray-100 dark:border-gray-700 \
                   hover:border-blue-300 dark:hover:border-blue-600 \
                   hover:shadow-sm transition-all duration-200 group"
        >
            <span class="text-2xl">{icon}</span>
            <div>
                <p class="font-medium text-gray-800 dark:text-white \
                           group-hover:text-blue-600 dark:group-hover:text-blue-400 \
                           transition-colors duration-200">
                    {label}
                </p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{desc}</p>
            </div>
        </a>
    }
}
