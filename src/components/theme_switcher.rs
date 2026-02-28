use leptos::prelude::*;

use crate::app::{Theme, ThemeCtx};
use crate::components::icons::{IconMoon, IconMonitor, IconSun};

#[component]
pub fn ThemeSwitcher() -> impl IntoView {
    let ctx = use_context::<ThemeCtx>().expect("ThemeCtx manquant");

    let cycle = move |_| {
        ctx.theme.update(|t| {
            *t = match *t {
                Theme::Light  => Theme::Dark,
                Theme::Dark   => Theme::System,
                Theme::System => Theme::Light,
            };
        });
    };

    view! {
        <button
            on:click=cycle
            title="Changer le thème (Lumineux → Sombre → Système)"
            class="btn-ripple theme-icon-btn flex items-center gap-1.5 px-3 py-1.5 rounded-lg \
                   bg-white/60 dark:bg-gray-700/60 backdrop-blur \
                   border border-gray-200 dark:border-gray-600 \
                   text-gray-700 dark:text-gray-200 \
                   hover:bg-white dark:hover:bg-gray-700 \
                   text-sm font-medium select-none"
        >
            {move || match ctx.theme.get() {
                Theme::Light  => view! { <IconSun     class="w-4 h-4" /> }.into_any(),
                Theme::Dark   => view! { <IconMoon    class="w-4 h-4" /> }.into_any(),
                Theme::System => view! { <IconMonitor class="w-4 h-4" /> }.into_any(),
            }}
            <span class="hidden sm:inline">
                {move || ctx.theme.get().label()}
            </span>
        </button>
    }
}
