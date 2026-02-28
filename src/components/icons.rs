//! Icônes SVG Lucide pour l'application — stroke="currentColor", héritage Tailwind.
//! Chaque composant accepte `class` (Tailwind, défaut "w-4 h-4").
#![allow(dead_code)]
use leptos::prelude::*;

// ── Macro interne : évite la répétition du boilerplate SVG ───────────────────

macro_rules! lucide {
    ($name:ident, $body:expr) => {
        #[component]
        pub fn $name(
            #[prop(default = "w-4 h-4")] class: &'static str,
        ) -> impl IntoView {
            view! {
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class=class
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                    inner_html=$body
                />
            }
        }
    };
}

// ── Navigation ────────────────────────────────────────────────────────────────

lucide!(IconHome,
    "<path d='m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'/>\
     <polyline points='9 22 9 12 15 12 15 22'/>"
);

// Croix chrétienne — icône des Communiants.
lucide!(IconCross,
    "<path d='M11 2v7H4a1 1 0 0 0 0 2h7v11a1 1 0 0 0 2 0V11h7a1 1 0 0 0 0-2h-7V2a1 1 0 0 0-2 0Z'/>"
);

// Livre ouvert — icône des Cathécomènes.
lucide!(IconBookOpen,
    "<path d='M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z'/>\
     <path d='M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z'/>"
);

// Boîte d'archives.
lucide!(IconArchive,
    "<rect width='20' height='5' x='2' y='3' rx='1'/>\
     <path d='M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8'/>\
     <path d='M10 12h4'/>"
);

// Bâtiment église (logo navbar).
lucide!(IconChurch,
    "<path d='m18 7 4 2v11a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V9l4-2'/>\
     <path d='M14 22v-4a2 2 0 0 0-4 0v4'/>\
     <path d='M18 22V5l-6-3-6 3v17'/>\
     <path d='M12 7v5'/>\
     <path d='M10 9h4'/>"
);

// ── Thème ─────────────────────────────────────────────────────────────────────

lucide!(IconSun,
    "<circle cx='12' cy='12' r='4'/>\
     <path d='M12 2v2'/><path d='M12 20v2'/>\
     <path d='m4.93 4.93 1.41 1.41'/><path d='m17.66 17.66 1.41 1.41'/>\
     <path d='M2 12h2'/><path d='M20 12h2'/>\
     <path d='m6.34 17.66-1.41 1.41'/><path d='m19.07 4.93-1.41 1.41'/>"
);

lucide!(IconMoon,
    "<path d='M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z'/>"
);

lucide!(IconMonitor,
    "<rect width='20' height='14' x='2' y='3' rx='2'/>\
     <path d='M8 21h8'/><path d='M12 17v4'/>"
);

// ── Actions ───────────────────────────────────────────────────────────────────

lucide!(IconSearch,
    "<circle cx='11' cy='11' r='8'/>\
     <path d='m21 21-4.35-4.35'/>"
);

lucide!(IconPlus,
    "<path d='M5 12h14'/><path d='M12 5v14'/>"
);

lucide!(IconPencil,
    "<path d='M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z'/>\
     <path d='m15 5 4 4'/>"
);

lucide!(IconTrash,
    "<path d='M3 6h18'/>\
     <path d='M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6'/>\
     <path d='M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2'/>\
     <line x1='10' x2='10' y1='11' y2='17'/>\
     <line x1='14' x2='14' y1='11' y2='17'/>"
);

lucide!(IconSave,
    "<path d='M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z'/>\
     <path d='M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7'/>\
     <path d='M7 3v4a1 1 0 0 0 1 1h7'/>"
);

lucide!(IconCoins,
    "<circle cx='8' cy='8' r='6'/>\
     <path d='M18.09 10.37A6 6 0 1 1 10.34 18'/>\
     <path d='M7 6h1v4'/>\
     <path d='m16.71 13.88.7.71-2.82 2.82'/>"
);

// Transfert (flèches opposées) — bouton "Transférer vers Communiants".
lucide!(IconTransfer,
    "<path d='m16 3 4 4-4 4'/>\
     <path d='M20 7H4'/>\
     <path d='m8 21-4-4 4-4'/>\
     <path d='M4 17h16'/>"
);

lucide!(IconX,
    "<path d='M18 6 6 18'/><path d='m6 6 12 12'/>"
);

// ── Statut / Notifications ────────────────────────────────────────────────────

lucide!(IconBell,
    "<path d='M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9'/>\
     <path d='M10.3 21a1.94 1.94 0 0 0 3.4 0'/>"
);

lucide!(IconLock,
    "<rect width='18' height='11' x='3' y='11' rx='2' ry='2'/>\
     <path d='M7 11V7a5 5 0 0 1 10 0v4'/>"
);

lucide!(IconAlertTriangle,
    "<path d='m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z'/>\
     <path d='M12 9v4'/><path d='M12 17h.01'/>"
);

lucide!(IconInfo,
    "<circle cx='12' cy='12' r='10'/>\
     <path d='M12 16v-4'/><path d='M12 8h.01'/>"
);

lucide!(IconFileText,
    "<path d='M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z'/>\
     <path d='M14 2v4a2 2 0 0 0 2 2h4'/>\
     <path d='M10 9H8'/><path d='M16 13H8'/><path d='M16 17H8'/>"
);

// ── Pagination / Direction ────────────────────────────────────────────────────

lucide!(IconArrowUp,
    "<path d='m5 12 7-7 7 7'/><path d='M12 19V5'/>"
);

lucide!(IconArrowDown,
    "<path d='M12 5v14'/><path d='m19 12-7 7-7-7'/>"
);

lucide!(IconChevronLeft,
    "<path d='m15 18-6-6 6-6'/>"
);

lucide!(IconChevronRight,
    "<path d='m9 18 6-6-6-6'/>"
);

// ── Registre d'icônes par nom ─────────────────────────────────────────────────
//
// Utilisé quand l'icône est passée comme `&'static str` depuis un prop.
// Exemples : icon="cross" | icon="book" | icon="archive" | icon="home"

#[component]
pub fn PageIcon(
    name:  &'static str,
    #[prop(default = "w-5 h-5")] class: &'static str,
) -> impl IntoView {
    match name {
        "home"    => view! { <IconHome    class=class /> }.into_any(),
        "cross"   => view! { <IconCross   class=class /> }.into_any(),
        "book"    => view! { <IconBookOpen class=class /> }.into_any(),
        "archive" => view! { <IconArchive class=class /> }.into_any(),
        "church"  => view! { <IconChurch  class=class /> }.into_any(),
        _         => view! { <span class=class>{name}</span> }.into_any(),
    }
}
