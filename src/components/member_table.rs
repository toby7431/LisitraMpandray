/// Tableau des membres avec tri par colonne.
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::{
    components::icons::{
        IconChevronLeft, IconChevronRight, IconCoins, IconPencil, IconSearch,
        IconTrash, PageIcon,
    },
    models::member::MemberWithTotal,
    services::db_service,
    utils::format_ariary,
};

const PAGE_SIZE: usize = 15;

// ─── Tri ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum SortCol { Carte, Nom, Adresse, Telephone, Travail, Genre, Total }

#[derive(Clone, Copy, PartialEq)]
pub enum SortDir { Asc, Desc }

impl SortDir {
    pub fn toggle(self) -> Self {
        match self { Self::Asc => Self::Desc, Self::Desc => Self::Asc }
    }
    pub fn arrow(self) -> &'static str {
        match self { Self::Asc => " ↑", Self::Desc => " ↓" }
    }
}

// ─── Helper interne ───────────────────────────────────────────────────────────

fn checked_from_event(ev: web_sys::Event) -> bool {
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|el| el.checked())
        .unwrap_or(false)
}

// ─── Composant Th ─────────────────────────────────────────────────────────────

#[component]
pub fn Th(
    label:       &'static str,
    col:         SortCol,
    sort_col:    RwSignal<SortCol>,
    sort_dir:    RwSignal<SortDir>,
    #[prop(optional)]
    extra_class: &'static str,
) -> impl IntoView {
    view! {
        <th
            class=format!("px-3 py-3 text-left cursor-pointer select-none \
                           hover:text-gray-800 dark:hover:text-white transition-colors \
                           whitespace-nowrap {extra_class}")
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

// ─── Composant MemberTable ────────────────────────────────────────────────────

#[component]
pub fn MemberTable(
    // ── Données et pagination ────────────────────────────────────────────────
    membres:          RwSignal<Vec<MemberWithTotal>>,
    sorted_filtered:  Memo<Vec<MemberWithTotal>>,
    page:             RwSignal<usize>,
    total_pages:      Memo<usize>,
    // ── Tri ──────────────────────────────────────────────────────────────────
    sort_col:         RwSignal<SortCol>,
    sort_dir:         RwSignal<SortDir>,
    // ── Sélection / transfert ─────────────────────────────────────────────
    transfer_to:      Option<&'static str>,
    selected:         RwSignal<Vec<i64>>,
    all_page_selected: Memo<bool>,
    page_items:       Memo<Vec<MemberWithTotal>>,
    transferring_ids: RwSignal<Vec<i64>>,
    // ── Style paramétrable ────────────────────────────────────────────────
    icon:             &'static str,
    row_hover:        &'static str,
    link_class:       &'static str,
    spin_class:       &'static str,
    // ── Signaux partagés ──────────────────────────────────────────────────
    loading:          RwSignal<bool>,
    refresh_ctr:      RwSignal<u32>,
    notif_error:      RwSignal<Option<String>>,
    // ── Ouverture modale édition ──────────────────────────────────────────
    modal_ouvert:     RwSignal<bool>,
    edit_id:          RwSignal<Option<i64>>,
    f_carte:          RwSignal<String>,
    f_nom:            RwSignal<String>,
    f_adresse:        RwSignal<String>,
    f_telephone:      RwSignal<String>,
    f_travail:        RwSignal<String>,
    f_genre:          RwSignal<String>,
    // ── Ouverture modale cotisation ───────────────────────────────────────
    contrib_membre_id:  RwSignal<i64>,
    contrib_membre_nom: RwSignal<String>,
    contrib_open:       RwSignal<bool>,
) -> impl IntoView {
    view! {
        {move || {
            if loading.get() {
                return view! {
                    <div class="flex justify-center py-16">
                        <div class=format!(
                            "w-8 h-8 border-4 {} border-t-transparent \
                             rounded-full animate-spin", spin_class
                        ) />
                    </div>
                }.into_any();
            }

            if membres.get().is_empty() {
                return view! {
                    <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                rounded-2xl border border-gray-100 dark:border-gray-700 \
                                text-center py-16 text-gray-400 dark:text-gray-500">
                        <div class="flex justify-center mb-3">
                            <PageIcon name=icon class="w-12 h-12 text-gray-300 dark:text-gray-600" />
                        </div>
                        <p class="text-base font-medium">"Aucun membre enregistré"</p>
                        <p class="text-xs mt-1">
                            "Cliquez sur « Nouveau membre » pour commencer."
                        </p>
                    </div>
                }.into_any();
            }

            if sorted_filtered.get().is_empty() {
                return view! {
                    <div class="bg-white/60 dark:bg-gray-800/60 backdrop-blur \
                                rounded-2xl border border-gray-100 dark:border-gray-700 \
                                text-center py-16 text-gray-400 dark:text-gray-500">
                        <div class="flex justify-center mb-3">
                            <IconSearch class="w-12 h-12 text-gray-300 dark:text-gray-600" />
                        </div>
                        <p class="text-base font-medium">"Aucun résultat"</p>
                        <p class="text-xs mt-1">"Essayez d'autres termes de recherche."</p>
                    </div>
                }.into_any();
            }

            view! {
                <div class="space-y-3">
                    // ── Tableau ───────────────────────────────────────────────
                    <div class="bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                rounded-2xl border border-gray-100 dark:border-gray-700 \
                                overflow-hidden shadow-sm">
                        <div class="overflow-x-auto">
                            <table class="w-full text-sm">
                                <thead>
                                    <tr class="bg-gray-50/80 dark:bg-gray-900/50 \
                                               border-b border-gray-100 dark:border-gray-700 \
                                               text-gray-600 dark:text-gray-400 font-semibold">
                                        {transfer_to.map(|_| view! {
                                            <th class="pl-4 pr-2 py-3 w-10">
                                                <input
                                                    type="checkbox"
                                                    class="custom-check"
                                                    title="Tout sélectionner"
                                                    prop:checked=move || all_page_selected.get()
                                                    on:change=move |ev: web_sys::Event| {
                                                        let checked = checked_from_event(ev);
                                                        let items   = page_items.get();
                                                        selected.update(|s| {
                                                            if checked {
                                                                for m in &items {
                                                                    if !s.contains(&m.id) {
                                                                        s.push(m.id);
                                                                    }
                                                                }
                                                            } else {
                                                                let ids: Vec<i64> = items.iter().map(|m| m.id).collect();
                                                                s.retain(|id| !ids.contains(id));
                                                            }
                                                        });
                                                    }
                                                />
                                            </th>
                                        })}
                                        <Th label="N° Carte"     col=SortCol::Carte     sort_col=sort_col sort_dir=sort_dir extra_class="hidden sm:table-cell" />
                                        <Th label="Nom complet"  col=SortCol::Nom       sort_col=sort_col sort_dir=sort_dir />
                                        <Th label="Adresse"      col=SortCol::Adresse   sort_col=sort_col sort_dir=sort_dir extra_class="hidden md:table-cell" />
                                        <Th label="Téléphone"    col=SortCol::Telephone sort_col=sort_col sort_dir=sort_dir extra_class="hidden lg:table-cell" />
                                        <Th label="Travail"      col=SortCol::Travail   sort_col=sort_col sort_dir=sort_dir extra_class="hidden md:table-cell" />
                                        <Th label="Genre"        col=SortCol::Genre     sort_col=sort_col sort_dir=sort_dir extra_class="hidden sm:table-cell" />
                                        <Th label="Total cotis." col=SortCol::Total     sort_col=sort_col sort_dir=sort_dir />
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
                                                <tr class=move || {
                                                    let sliding = transferring_ids.get().contains(&mid);
                                                    format!(
                                                        "tr-hover border-b border-gray-50 \
                                                         dark:border-gray-700/50 \
                                                         {} transition-colors duration-150{}",
                                                        row_hover,
                                                        if sliding { " row-sliding-out" } else { "" }
                                                    )
                                                }>
                                                    {transfer_to.map(|_| view! {
                                                        <td class="pl-4 pr-2 py-2.5">
                                                            <input
                                                                type="checkbox"
                                                                class="custom-check"
                                                                prop:checked=move || selected.get().contains(&mid)
                                                                on:change=move |ev: web_sys::Event| {
                                                                    let checked = checked_from_event(ev);
                                                                    selected.update(|s| {
                                                                        if checked {
                                                                            if !s.contains(&mid) { s.push(mid); }
                                                                        } else {
                                                                            s.retain(|&id| id != mid);
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                        </td>
                                                    })}
                                                    <td class="hidden sm:table-cell px-3 py-2.5 \
                                                               font-mono text-xs \
                                                               text-gray-500 dark:text-gray-400 \
                                                               whitespace-nowrap">
                                                        {m.card_number.clone()}
                                                    </td>
                                                    <td class="px-3 py-2.5 font-semibold \
                                                               text-gray-800 dark:text-white \
                                                               whitespace-nowrap">
                                                        {m.full_name.clone()}
                                                    </td>
                                                    <td class="hidden md:table-cell px-3 py-2.5 \
                                                               text-gray-600 dark:text-gray-300 \
                                                               max-w-[140px] truncate">
                                                        {m.address.clone().unwrap_or_else(|| "—".into())}
                                                    </td>
                                                    <td class="hidden lg:table-cell px-3 py-2.5 \
                                                               text-gray-600 dark:text-gray-300 \
                                                               whitespace-nowrap">
                                                        {m.phone.clone().unwrap_or_else(|| "—".into())}
                                                    </td>
                                                    <td class="hidden md:table-cell px-3 py-2.5 \
                                                               text-gray-600 dark:text-gray-300 \
                                                               max-w-[120px] truncate">
                                                        {m.job.clone().unwrap_or_else(|| "—".into())}
                                                    </td>
                                                    <td class="hidden sm:table-cell px-3 py-2.5 \
                                                               text-gray-600 dark:text-gray-300 \
                                                               whitespace-nowrap">
                                                        {genre_label}
                                                    </td>
                                                    <td class="px-3 py-2.5 font-mono font-semibold \
                                                               text-gray-800 dark:text-white \
                                                               whitespace-nowrap">
                                                        {total}
                                                    </td>
                                                    <td class="px-3 py-2.5 pr-4 text-right whitespace-nowrap">
                                                        <button
                                                            title="Cotisation"
                                                            class="btn-ripple mr-2 text-xs text-amber-500 \
                                                                   dark:text-amber-400 rounded \
                                                                   hover:scale-125 transition-transform \
                                                                   duration-150 font-medium"
                                                            on:click=move |_| {
                                                                contrib_membre_id.set(mid);
                                                                contrib_membre_nom.set(m.full_name.clone());
                                                                contrib_open.set(true);
                                                            }
                                                        >
                                                            <IconCoins class="w-4 h-4" />
                                                        </button>
                                                        <button
                                                            title="Modifier"
                                                            class=format!("btn-ripple mr-2 text-xs {} \
                                                                           rounded hover:scale-125 \
                                                                           transition-transform duration-150 \
                                                                           font-medium", link_class)
                                                            on:click=move |_| {
                                                                edit_id.set(Some(m_edit.id));
                                                                f_carte.set(m_edit.card_number.clone());
                                                                f_nom.set(m_edit.full_name.clone());
                                                                f_adresse.set(m_edit.address.clone().unwrap_or_default());
                                                                f_telephone.set(m_edit.phone.clone().unwrap_or_default());
                                                                f_travail.set(m_edit.job.clone().unwrap_or_default());
                                                                f_genre.set(m_edit.gender.clone());
                                                                modal_ouvert.set(true);
                                                            }
                                                        >
                                                            <IconPencil class="w-4 h-4" />
                                                        </button>
                                                        <button
                                                            title="Supprimer"
                                                            class="btn-ripple text-xs text-red-500 \
                                                                   dark:text-red-400 rounded \
                                                                   hover:scale-125 transition-transform \
                                                                   duration-150 font-medium"
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
                                                                            Err(e) => notif_error.set(Some(e)),
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        >
                                                            <IconTrash class="w-4 h-4" />
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

                    // ── Pagination (masquée si une seule page) ────────────────
                    {move || (total_pages.get() > 1).then(|| view! {
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
                                    class="btn-ripple px-3 py-1.5 text-xs rounded-lg \
                                           bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                           border border-gray-200 dark:border-gray-600 \
                                           text-gray-700 dark:text-gray-300 \
                                           disabled:opacity-40 disabled:cursor-not-allowed \
                                           hover:bg-gray-50 dark:hover:bg-gray-700 transition"
                                >
                                    <span class="flex items-center gap-1">
                                        <IconChevronLeft class="w-3.5 h-3.5" />
                                        "Préc."
                                    </span>
                                </button>
                                <span class="px-3 py-1.5 text-xs font-medium \
                                             text-gray-700 dark:text-gray-300">
                                    {move || format!("{} / {}", page.get() + 1, total_pages.get())}
                                </span>
                                <button
                                    disabled=move || page.get() + 1 >= total_pages.get()
                                    on:click=move |_| page.update(|p| *p += 1)
                                    class="btn-ripple px-3 py-1.5 text-xs rounded-lg \
                                           bg-white/70 dark:bg-gray-800/70 backdrop-blur \
                                           border border-gray-200 dark:border-gray-600 \
                                           text-gray-700 dark:text-gray-300 \
                                           disabled:opacity-40 disabled:cursor-not-allowed \
                                           hover:bg-gray-50 dark:hover:bg-gray-700 transition"
                                >
                                    <span class="flex items-center gap-1">
                                        "Suiv."
                                        <IconChevronRight class="w-3.5 h-3.5" />
                                    </span>
                                </button>
                            </div>
                        </div>
                    })}
                </div>
            }.into_any()
        }}
    }
}
