/// Composant générique pour Communiants et Cathécomènes.
///
/// Tableau complet : N° carte, Nom, Adresse, Téléphone, Travail, Genre, Total contributions.
/// Recherche live, tri par colonne, filtre genre, pagination, formulaire CRUD modal.
use leptos::prelude::*;

use crate::{
    components::{
        contribution_modal::{ConfettiLayer, ContributionModal},
        phone_input::PhoneInput,
    },
    models::member::{MemberInput, MemberWithTotal},
    services::db_service,
};

const PAGE_SIZE: usize = 15;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn format_ariary(total_str: &str) -> String {
    let n: i64 = total_str.parse::<f64>().unwrap_or(0.0) as i64;
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('\u{202f}');
        }
        result.push(c);
    }
    format!("{}\u{202f}Ar", result)
}

fn non_empty(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() { None } else { Some(t) }
}

// ─── Tri ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum SortCol { Carte, Nom, Adresse, Telephone, Travail, Genre, Total }

#[derive(Clone, Copy, PartialEq)]
enum SortDir { Asc, Desc }

impl SortDir {
    fn toggle(self) -> Self {
        match self { Self::Asc => Self::Desc, Self::Desc => Self::Asc }
    }
    fn arrow(self) -> &'static str {
        match self { Self::Asc => " ↑", Self::Desc => " ↓" }
    }
}

// ─── Composant principal ──────────────────────────────────────────────────────

#[component]
pub fn MemberPage(
    member_type: &'static str,
    icon:        &'static str,
    title:       &'static str,
    subtitle:    &'static str,
    /// Classes Tailwind pour le bouton principal (ex: "bg-blue-600 hover:bg-blue-700")
    btn_class:   &'static str,
    /// Classe hover sur les lignes du tableau (ex: "hover:bg-blue-50/50 ...")
    row_hover:   &'static str,
    /// Couleur des liens/boutons texte (ex: "text-blue-600 dark:text-blue-400")
    link_class:  &'static str,
    /// Couleur du spinner (ex: "border-blue-500")
    spin_class:  &'static str,
) -> impl IntoView {

    // ── Données ────────────────────────────────────────────────────────────────
    let membres: RwSignal<Vec<MemberWithTotal>> = RwSignal::new(vec![]);
    let loading   = RwSignal::new(true);
    let erreur: RwSignal<Option<String>> = RwSignal::new(None);

    // Déclencheur de rechargement (incrémenter pour rafraîchir)
    let refresh_ctr: RwSignal<u32> = RwSignal::new(0);

    Effect::new(move |_| {
        let _ = refresh_ctr.get();
        loading.set(true);
        erreur.set(None);
        leptos::task::spawn_local(async move {
            match db_service::get_members_by_type_with_total(member_type).await {
                Ok(liste) => membres.set(liste),
                Err(e)    => erreur.set(Some(e)),
            }
            loading.set(false);
        });
    });

    // ── Recherche / Filtres / Tri / Pagination ─────────────────────────────────
    let recherche:    RwSignal<String> = RwSignal::new(String::new());
    let filtre_genre: RwSignal<String> = RwSignal::new("Tous".into());
    let sort_col:     RwSignal<SortCol> = RwSignal::new(SortCol::Nom);
    let sort_dir:     RwSignal<SortDir> = RwSignal::new(SortDir::Asc);
    let page:         RwSignal<usize>  = RwSignal::new(0);

    // Reset page quand la recherche ou le filtre change
    Effect::new(move |_| {
        let _ = recherche.get();
        let _ = filtre_genre.get();
        page.set(0);
    });

    let sorted_filtered = Memo::new(move |_| {
        let q     = recherche.get().to_lowercase();
        let genre = filtre_genre.get();
        let col   = sort_col.get();
        let dir   = sort_dir.get();

        let mut list: Vec<MemberWithTotal> = membres
            .get()
            .into_iter()
            .filter(|m| {
                (genre == "Tous" || m.gender == genre)
                    && (q.is_empty()
                        || m.full_name.to_lowercase().contains(&q)
                        || m.card_number.to_lowercase().contains(&q)
                        || m.address.as_deref().unwrap_or("").to_lowercase().contains(&q)
                        || m.phone.as_deref().unwrap_or("").to_lowercase().contains(&q)
                        || m.job.as_deref().unwrap_or("").to_lowercase().contains(&q))
            })
            .collect();

        list.sort_by(|a, b| {
            use std::cmp::Ordering;
            let ord: Ordering = match col {
                SortCol::Carte     => a.card_number.cmp(&b.card_number),
                SortCol::Nom       => a.full_name.cmp(&b.full_name),
                SortCol::Adresse   => a.address.as_deref().unwrap_or("").cmp(b.address.as_deref().unwrap_or("")),
                SortCol::Telephone => a.phone.as_deref().unwrap_or("").cmp(b.phone.as_deref().unwrap_or("")),
                SortCol::Travail   => a.job.as_deref().unwrap_or("").cmp(b.job.as_deref().unwrap_or("")),
                SortCol::Genre     => a.gender.cmp(&b.gender),
                SortCol::Total     => {
                    let ta: i64 = a.total_contributions.parse().unwrap_or(0);
                    let tb: i64 = b.total_contributions.parse().unwrap_or(0);
                    ta.cmp(&tb)
                }
            };
            if dir == SortDir::Desc { ord.reverse() } else { ord }
        });
        list
    });

    let total_pages = Memo::new(move |_| {
        ((sorted_filtered.get().len() + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    });

    let page_items = Memo::new(move |_| {
        sorted_filtered
            .get()
            .into_iter()
            .skip(page.get() * PAGE_SIZE)
            .take(PAGE_SIZE)
            .collect::<Vec<_>>()
    });

    // ── Modal / Formulaire ─────────────────────────────────────────────────────
    let modal_ouvert: RwSignal<bool>        = RwSignal::new(false);
    let edit_id:      RwSignal<Option<i64>> = RwSignal::new(None);

    let f_carte:     RwSignal<String> = RwSignal::new(String::new());
    let f_nom:       RwSignal<String> = RwSignal::new(String::new());
    let f_adresse:   RwSignal<String> = RwSignal::new(String::new());
    let f_telephone: RwSignal<String> = RwSignal::new(String::new());
    let f_travail:   RwSignal<String> = RwSignal::new(String::new());
    let f_genre:     RwSignal<String> = RwSignal::new("M".into());
    let f_erreur:    RwSignal<Option<String>> = RwSignal::new(None);
    let f_loading:   RwSignal<bool>   = RwSignal::new(false);

    // ── Modal Cotisation ───────────────────────────────────────────────────────
    let contrib_open:      RwSignal<bool>   = RwSignal::new(false);
    let contrib_membre_id: RwSignal<i64>    = RwSignal::new(0);
    let contrib_membre_nom: RwSignal<String> = RwSignal::new(String::new());
    let confetti_active:   RwSignal<bool>   = RwSignal::new(false);

    let reset_form = move || {
        f_carte.set(String::new());
        f_nom.set(String::new());
        f_adresse.set(String::new());
        f_telephone.set(String::new());
        f_travail.set(String::new());
        f_genre.set("M".into());
        f_erreur.set(None);
        edit_id.set(None);
    };

    let soumettre = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // Le PhoneInput laisse "+261 " si vide → traiter comme None
        let phone_val = f_telephone.get();
        let phone = if phone_val.trim() == "+261" || phone_val.trim().len() <= 5 {
            None
        } else {
            Some(phone_val.trim().to_string())
        };

        let input = MemberInput {
            card_number: f_carte.get().trim().to_string(),
            full_name:   f_nom.get().trim().to_string(),
            address:     non_empty(f_adresse.get()),
            phone,
            job:         non_empty(f_travail.get()),
            gender:      f_genre.get(),
            member_type: member_type.to_string(),
        };
        f_loading.set(true);
        f_erreur.set(None);
        let eid = edit_id.get();
        leptos::task::spawn_local(async move {
            let res = if let Some(id) = eid {
                db_service::update_member(id, &input).await.map(|_| ())
            } else {
                db_service::create_member(&input).await.map(|_| ())
            };
            match res {
                Ok(_) => {
                    modal_ouvert.set(false);
                    refresh_ctr.update(|n| *n += 1);
                }
                Err(e) => f_erreur.set(Some(e)),
            }
            f_loading.set(false);
        });
    };

    // ─── Vue ──────────────────────────────────────────────────────────────────
    view! {
        <div class="animate-fade-in space-y-4 sm:space-y-5">

            // ── En-tête ────────────────────────────────────────────────────────
            <div class="flex flex-wrap items-start sm:items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl sm:text-2xl font-bold text-gray-800 dark:text-white \
                                flex items-center gap-2">
                        {icon} " " {title}
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 text-xs sm:text-sm mt-0.5">
                        {subtitle}
                    </p>
                </div>
                <button
                    on:click=move |_| { reset_form(); modal_ouvert.set(true); }
                    class=format!("px-3 sm:px-4 py-2 {} text-white rounded-xl \
                                   text-xs sm:text-sm font-semibold transition-colors \
                                   duration-200 flex items-center gap-1.5 shrink-0 \
                                   shadow-sm", btn_class)
                >
                    "➕ Nouveau membre"
                </button>
            </div>

            // ── Barre de recherche + filtres ───────────────────────────────────
            <div class="flex flex-wrap gap-2 sm:gap-3 items-center">
                <div class="relative flex-1 min-w-[180px]">
                    <span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 \
                                 select-none text-sm pointer-events-none">
                        "🔍"
                    </span>
                    <input
                        type="text"
                        placeholder="Rechercher…"
                        class="w-full pl-9 pr-3 py-2 text-sm \
                               bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                               border border-gray-200 dark:border-gray-600 \
                               rounded-xl text-gray-800 dark:text-white \
                               placeholder-gray-400 dark:placeholder-gray-500 \
                               focus:outline-none focus:ring-2 focus:ring-blue-400 transition"
                        prop:value=move || recherche.get()
                        on:input=move |ev| recherche.set(event_target_value(&ev))
                    />
                </div>
                <select
                    class="px-3 py-2 text-sm \
                           bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                           border border-gray-200 dark:border-gray-600 \
                           rounded-xl text-gray-800 dark:text-white \
                           focus:outline-none focus:ring-2 focus:ring-blue-400 transition"
                    prop:value=move || filtre_genre.get()
                    on:change=move |ev| filtre_genre.set(event_target_value(&ev))
                >
                    <option value="Tous">"Tous"</option>
                    <option value="M">"Hommes"</option>
                    <option value="F">"Femmes"</option>
                </select>
                <span class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                    {move || {
                        let n = sorted_filtered.get().len();
                        format!("{n} membre{}", if n > 1 { "s" } else { "" })
                    }}
                </span>
            </div>

            // ── Bannière d'erreur ──────────────────────────────────────────────
            {move || erreur.get().map(|e| view! {
                <div class="p-3 bg-red-50 dark:bg-red-900/30 \
                            border border-red-200 dark:border-red-700 \
                            rounded-xl text-red-700 dark:text-red-300 text-sm">
                    "⚠️ " {e}
                </div>
            })}

            // ── Contenu principal ──────────────────────────────────────────────
            {move || {
                if loading.get() {
                    view! {
                        <div class="flex justify-center py-16">
                            <div class=format!(
                                "w-8 h-8 border-4 {} border-t-transparent \
                                 rounded-full animate-spin", spin_class
                            ) />
                        </div>
                    }.into_any()

                } else if membres.get().is_empty() {
                    view! {
                        <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                    rounded-2xl border border-gray-100 dark:border-gray-700 \
                                    text-center py-16 text-gray-400 dark:text-gray-500">
                            <div class="text-5xl mb-3">{icon}</div>
                            <p class="text-base font-medium">"Aucun membre enregistré"</p>
                            <p class="text-xs mt-1">
                                "Cliquez sur « Nouveau membre » pour commencer."
                            </p>
                        </div>
                    }.into_any()

                } else {
                    view! {
                        <div class="space-y-3">
                            // ── Tableau ──────────────────────────────────────────
                            <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                        rounded-2xl border border-gray-100 dark:border-gray-700 \
                                        overflow-hidden shadow-sm">
                                <div class="overflow-x-auto">
                                    <table class="w-full text-sm min-w-[700px]">
                                        <thead>
                                            <tr class="bg-gray-50/80 dark:bg-gray-900/50 \
                                                       border-b border-gray-100 dark:border-gray-700 \
                                                       text-gray-600 dark:text-gray-400 font-semibold">
                                                <Th label="N° Carte" col=SortCol::Carte sort_col=sort_col sort_dir=sort_dir />
                                                <Th label="Nom complet" col=SortCol::Nom sort_col=sort_col sort_dir=sort_dir />
                                                <Th label="Adresse" col=SortCol::Adresse sort_col=sort_col sort_dir=sort_dir />
                                                <Th label="Téléphone" col=SortCol::Telephone sort_col=sort_col sort_dir=sort_dir />
                                                <Th label="Travail" col=SortCol::Travail sort_col=sort_col sort_dir=sort_dir />
                                                <Th label="Genre" col=SortCol::Genre sort_col=sort_col sort_dir=sort_dir />
                                                <Th label="Total cotisations" col=SortCol::Total sort_col=sort_col sort_dir=sort_dir />
                                                <th class="px-3 py-3 text-right pr-4">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || page_items.get()
                                                key=|m| m.id
                                                children=move |m: MemberWithTotal| {
                                                    let m_edit = m.clone();
                                                    let mid    = m.id;
                                                    let total  = format_ariary(&m.total_contributions);
                                                    let genre_label = if m.gender == "M" { "♂ Homme" } else { "♀ Femme" };

                                                    view! {
                                                        <tr class=format!(
                                                            "border-b border-gray-50 dark:border-gray-700/50 \
                                                             {} transition-colors duration-100", row_hover
                                                        )>
                                                            <td class="px-3 py-2.5 font-mono text-xs \
                                                                       text-gray-500 dark:text-gray-400 whitespace-nowrap">
                                                                {m.card_number.clone()}
                                                            </td>
                                                            <td class="px-3 py-2.5 font-semibold \
                                                                       text-gray-800 dark:text-white whitespace-nowrap">
                                                                {m.full_name.clone()}
                                                            </td>
                                                            <td class="px-3 py-2.5 text-gray-600 \
                                                                       dark:text-gray-300 max-w-[140px] truncate">
                                                                {m.address.clone().unwrap_or_else(|| "—".into())}
                                                            </td>
                                                            <td class="px-3 py-2.5 text-gray-600 \
                                                                       dark:text-gray-300 whitespace-nowrap">
                                                                {m.phone.clone().unwrap_or_else(|| "—".into())}
                                                            </td>
                                                            <td class="px-3 py-2.5 text-gray-600 \
                                                                       dark:text-gray-300 max-w-[120px] truncate">
                                                                {m.job.clone().unwrap_or_else(|| "—".into())}
                                                            </td>
                                                            <td class="px-3 py-2.5 text-gray-600 \
                                                                       dark:text-gray-300 whitespace-nowrap">
                                                                {genre_label}
                                                            </td>
                                                            <td class="px-3 py-2.5 font-mono font-semibold \
                                                                       text-gray-800 dark:text-white whitespace-nowrap">
                                                                {total}
                                                            </td>
                                                            <td class="px-3 py-2.5 pr-4 text-right whitespace-nowrap">
                                                                <button
                                                                    title="Cotisation"
                                                                    class="mr-2 text-xs text-amber-500 dark:text-amber-400 \
                                                                           hover:underline font-medium"
                                                                    on:click=move |_| {
                                                                        contrib_membre_id.set(mid);
                                                                        contrib_membre_nom.set(m.full_name.clone());
                                                                        contrib_open.set(true);
                                                                    }
                                                                >
                                                                    "💰"
                                                                </button>
                                                                <button
                                                                    title="Modifier"
                                                                    class=format!("mr-2 text-xs {} \
                                                                                   hover:underline font-medium", link_class)
                                                                    on:click=move |_| {
                                                                        edit_id.set(Some(m_edit.id));
                                                                        f_carte.set(m_edit.card_number.clone());
                                                                        f_nom.set(m_edit.full_name.clone());
                                                                        f_adresse.set(m_edit.address.clone().unwrap_or_default());
                                                                        f_telephone.set(m_edit.phone.clone().unwrap_or_default());
                                                                        f_travail.set(m_edit.job.clone().unwrap_or_default());
                                                                        f_genre.set(m_edit.gender.clone());
                                                                        f_erreur.set(None);
                                                                        modal_ouvert.set(true);
                                                                    }
                                                                >
                                                                    "✏️"
                                                                </button>
                                                                <button
                                                                    title="Supprimer"
                                                                    class="text-xs text-red-500 dark:text-red-400 \
                                                                           hover:underline font-medium"
                                                                    on:click=move |_| {
                                                                        let ok = web_sys::window()
                                                                            .and_then(|w| {
                                                                                w.confirm_with_message(
                                                                                    "Supprimer ce membre ? Cette action est irréversible.",
                                                                                ).ok()
                                                                            })
                                                                            .unwrap_or(false);
                                                                        if ok {
                                                                            leptos::task::spawn_local(async move {
                                                                                match db_service::delete_member(mid).await {
                                                                                    Ok(_)  => refresh_ctr.update(|n| *n += 1),
                                                                                    Err(e) => erreur.set(Some(e)),
                                                                                }
                                                                            });
                                                                        }
                                                                    }
                                                                >
                                                                    "🗑️"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
                                </div>
                            </div>

                            // ── Pagination ───────────────────────────────────────
                            <div class="flex items-center justify-between flex-wrap gap-2 px-1">
                                <span class="text-xs text-gray-500 dark:text-gray-400">
                                    {move || {
                                        let total = sorted_filtered.get().len();
                                        let p     = page.get();
                                        let from  = (p * PAGE_SIZE + 1).min(total);
                                        let to    = ((p + 1) * PAGE_SIZE).min(total);
                                        format!("{from}–{to} sur {total}")
                                    }}
                                </span>
                                <div class="flex items-center gap-1">
                                    <button
                                        disabled=move || page.get() == 0
                                        on:click=move |_| page.update(|p| *p = p.saturating_sub(1))
                                        class="px-3 py-1.5 text-xs rounded-lg \
                                               bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                               border border-gray-200 dark:border-gray-600 \
                                               text-gray-700 dark:text-gray-300 \
                                               disabled:opacity-40 disabled:cursor-not-allowed \
                                               hover:bg-gray-50 dark:hover:bg-gray-700 transition"
                                    >
                                        "← Préc."
                                    </button>
                                    <span class="px-3 py-1.5 text-xs font-medium \
                                                 text-gray-700 dark:text-gray-300">
                                        {move || format!("{} / {}", page.get() + 1, total_pages.get())}
                                    </span>
                                    <button
                                        disabled=move || page.get() + 1 >= total_pages.get()
                                        on:click=move |_| page.update(|p| *p += 1)
                                        class="px-3 py-1.5 text-xs rounded-lg \
                                               bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                               border border-gray-200 dark:border-gray-600 \
                                               text-gray-700 dark:text-gray-300 \
                                               disabled:opacity-40 disabled:cursor-not-allowed \
                                               hover:bg-gray-50 dark:hover:bg-gray-700 transition"
                                    >
                                        "Suiv. →"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            // ── Modal formulaire ───────────────────────────────────────────────
            {move || modal_ouvert.get().then(|| {
                let is_edit    = edit_id.get().is_some();
                let modal_title = if is_edit { "Modifier le membre" } else { "Nouveau membre" };

                view! {
                    // Overlay
                    <div
                        class="fixed inset-0 z-50 flex items-center justify-center p-4 \
                               bg-black/40 dark:bg-black/60 backdrop-blur-sm"
                        on:click=move |ev| {
                            // Ferme si clic sur l'overlay (pas sur le panneau)
                            if ev.target() == ev.current_target() {
                                modal_ouvert.set(false);
                            }
                        }
                    >
                        // Panneau
                        <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl \
                                    w-full max-w-lg max-h-[90vh] overflow-y-auto \
                                    border border-gray-100 dark:border-gray-700">

                            // En-tête modal
                            <div class="flex items-center justify-between px-6 pt-5 pb-4 \
                                        border-b border-gray-100 dark:border-gray-700">
                                <h2 class="text-base font-bold text-gray-800 dark:text-white">
                                    {modal_title}
                                </h2>
                                <button
                                    on:click=move |_| modal_ouvert.set(false)
                                    class="text-gray-400 hover:text-gray-600 \
                                           dark:hover:text-gray-200 text-xl leading-none \
                                           transition-colors"
                                >
                                    "✕"
                                </button>
                            </div>

                            // Formulaire
                            <form on:submit=soumettre class="px-6 py-5 space-y-4">

                                // N° carte + Genre (côte à côte)
                                <div class="grid grid-cols-2 gap-3">
                                    <div>
                                        <label class=LABEL>"N° carte *"</label>
                                        <input
                                            type="text" required
                                            placeholder="ex : C-0042"
                                            class=INPUT
                                            prop:value=move || f_carte.get()
                                            on:input=move |ev| f_carte.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class=LABEL>"Genre *"</label>
                                        <select
                                            class=INPUT
                                            prop:value=move || f_genre.get()
                                            on:change=move |ev| f_genre.set(event_target_value(&ev))
                                        >
                                            <option value="M">"Masculin"</option>
                                            <option value="F">"Féminin"</option>
                                        </select>
                                    </div>
                                </div>

                                // Nom complet
                                <div>
                                    <label class=LABEL>"Nom complet *"</label>
                                    <input
                                        type="text" required
                                        placeholder="Prénom Nom"
                                        class=INPUT
                                        prop:value=move || f_nom.get()
                                        on:input=move |ev| f_nom.set(event_target_value(&ev))
                                    />
                                </div>

                                // Adresse
                                <div>
                                    <label class=LABEL>"Adresse"</label>
                                    <input
                                        type="text"
                                        placeholder="Quartier, ville…"
                                        class=INPUT
                                        prop:value=move || f_adresse.get()
                                        on:input=move |ev| f_adresse.set(event_target_value(&ev))
                                    />
                                </div>

                                // Téléphone
                                <div>
                                    <label class=LABEL>"Téléphone"</label>
                                    <PhoneInput value=f_telephone class=INPUT />
                                </div>

                                // Travail
                                <div>
                                    <label class=LABEL>"Travail / Emploi"</label>
                                    <input
                                        type="text"
                                        placeholder="Enseignant, Commerçant…"
                                        class=INPUT
                                        prop:value=move || f_travail.get()
                                        on:input=move |ev| f_travail.set(event_target_value(&ev))
                                    />
                                </div>

                                // Erreur formulaire
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
                                        on:click=move |_| modal_ouvert.set(false)
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
                                        class=format!("px-4 py-2 text-sm font-semibold \
                                                       text-white {} rounded-xl \
                                                       disabled:opacity-60 disabled:cursor-wait \
                                                       transition-colors shadow-sm", btn_class)
                                    >
                                        {move || if f_loading.get() { "Enregistrement…" } else { "Enregistrer" }}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                }
            })}

            // ── Modal cotisation ───────────────────────────────────────────────
            {move || {
                if !contrib_open.get() { return None; }
                let mid  = contrib_membre_id.get();
                let mnom = contrib_membre_nom.get();
                Some(view! {
                    <ContributionModal
                        membre_id=mid
                        membre_nom=mnom
                        open=contrib_open
                        refresh_ctr=refresh_ctr
                        confetti_active=confetti_active
                    />
                })
            }}

            // ── Couche confetti ────────────────────────────────────────────────
            <ConfettiLayer active=confetti_active />

        </div>
    }
}

// ─── Constantes de style formulaire ──────────────────────────────────────────

const LABEL: &str = "block text-xs font-semibold text-gray-600 dark:text-gray-400 mb-1";
const INPUT: &str = "w-full px-3 py-2 text-sm \
                     bg-gray-50 dark:bg-gray-700/60 \
                     border border-gray-200 dark:border-gray-600 \
                     rounded-xl text-gray-800 dark:text-white \
                     placeholder-gray-400 dark:placeholder-gray-500 \
                     focus:outline-none focus:ring-2 focus:ring-blue-400 transition";

// ─── Composant en-tête de colonne triable ─────────────────────────────────────

#[component]
fn Th(
    label:    &'static str,
    col:      SortCol,
    sort_col: RwSignal<SortCol>,
    sort_dir: RwSignal<SortDir>,
) -> impl IntoView {
    view! {
        <th
            class="px-3 py-3 text-left cursor-pointer select-none \
                   hover:text-gray-800 dark:hover:text-white transition-colors \
                   whitespace-nowrap"
            on:click=move |_| {
                if sort_col.get() == col {
                    sort_dir.update(|d| *d = d.toggle());
                } else {
                    sort_col.set(col);
                    sort_dir.set(SortDir::Asc);
                }
            }
        >
            {label}
            {move || if sort_col.get() == col { sort_dir.get().arrow() } else { "" }}
        </th>
    }
}
