/// Composant générique pour Communiants et Cathécomènes.
///
/// Orchestre la liste, les filtres, la pagination, le formulaire CRUD
/// et la modale de transfert. Délègue le rendu aux sous-composants :
/// `MemberTable`, `MemberForm`, `TransferModal`, `ContributionModal`.
use leptos::prelude::*;

use crate::{
    components::{
        contribution_modal::{ConfettiLayer, ContributionModal},
        icons::{IconAlertTriangle, IconPlus, IconSearch, IconTransfer, PageIcon},
        member_form::MemberForm,
        member_table::{MemberTable, SortCol, SortDir},
        transfer_modal::TransferModal,
    },
    models::member::MemberWithTotal,
    services::db_service,
    utils::sleep_ms,
};

const PAGE_SIZE: usize = 15;

// ─── Composant principal ──────────────────────────────────────────────────────

#[component]
pub fn MemberPage(
    member_type: &'static str,
    icon:        &'static str,
    title:       &'static str,
    subtitle:    &'static str,
    /// Classes Tailwind pour le bouton principal (ex: "bg-blue-600 hover:bg-blue-700")
    btn_class:   &'static str,
    /// Classe hover sur les lignes du tableau
    row_hover:   &'static str,
    /// Couleur des liens/boutons texte
    link_class:  &'static str,
    /// Couleur du spinner
    spin_class:  &'static str,
    /// Si `Some("Communiant")` : active la multi-sélection + bouton "Transférer"
    #[prop(optional)]
    transfer_to: Option<&'static str>,
) -> impl IntoView {

    // ── Données ────────────────────────────────────────────────────────────────
    let membres: RwSignal<Vec<MemberWithTotal>> = RwSignal::new(vec![]);
    let loading   = RwSignal::new(true);

    // ── Notification d'erreur flottante (auto-dismiss 4 s) ─────────────────────
    let notif_error: RwSignal<Option<String>> = RwSignal::new(None);
    Effect::new(move |_| {
        if notif_error.get().is_some() {
            leptos::task::spawn_local(async move {
                sleep_ms(4000).await;
                notif_error.set(None);
            });
        }
    });

    let refresh_ctr: RwSignal<u32> = RwSignal::new(0);

    Effect::new(move |_| {
        let _ = refresh_ctr.get();
        loading.set(true);
        leptos::task::spawn_local(async move {
            match db_service::get_members_by_type_with_total(member_type).await {
                Ok(liste) => membres.set(liste),
                Err(e)    => notif_error.set(Some(e)),
            }
            loading.set(false);
        });
    });

    // ── Recherche / Filtres / Tri / Pagination ─────────────────────────────────
    let recherche:    RwSignal<String>  = RwSignal::new(String::new());
    let filtre_genre: RwSignal<String>  = RwSignal::new("Tous".into());
    let sort_col:     RwSignal<SortCol> = RwSignal::new(SortCol::Nom);
    let sort_dir:     RwSignal<SortDir> = RwSignal::new(SortDir::Asc);
    let page:         RwSignal<usize>   = RwSignal::new(0);

    let selected: RwSignal<Vec<i64>> = RwSignal::new(vec![]);
    Effect::new(move |_| {
        let _ = recherche.get();
        let _ = filtre_genre.get();
        page.set(0);
        selected.set(vec![]);
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

    let all_page_selected = Memo::new(move |_| {
        let items = page_items.get();
        !items.is_empty() && items.iter().all(|m| selected.get().contains(&m.id))
    });

    // ── Transfert ──────────────────────────────────────────────────────────────
    let transferring_ids: RwSignal<Vec<i64>> = RwSignal::new(vec![]);
    let transfer_modal:   RwSignal<bool> = RwSignal::new(false);
    let transfer_loading: RwSignal<bool> = RwSignal::new(false);

    let do_transfer = Callback::new(move |()| {
        let ids = selected.get();
        if ids.is_empty() { return; }
        let target = match transfer_to { Some(t) => t, None => return };
        transfer_loading.set(true);
        transferring_ids.set(ids.clone());
        leptos::task::spawn_local(async move {
            let result = db_service::transfer_members(&ids, target).await;
            sleep_ms(400).await;
            match result {
                Ok(_) => {
                    transfer_modal.set(false);
                    selected.set(vec![]);
                    transferring_ids.set(vec![]);
                    refresh_ctr.update(|n| *n += 1);
                }
                Err(e) => {
                    notif_error.set(Some(e));
                    transferring_ids.set(vec![]);
                }
            }
            transfer_loading.set(false);
        });
    });

    // ── Formulaire membre ──────────────────────────────────────────────────────
    let modal_ouvert: RwSignal<bool>        = RwSignal::new(false);
    let edit_id:      RwSignal<Option<i64>> = RwSignal::new(None);
    let f_carte:     RwSignal<String> = RwSignal::new(String::new());
    let f_nom:       RwSignal<String> = RwSignal::new(String::new());
    let f_adresse:   RwSignal<String> = RwSignal::new(String::new());
    let f_telephone: RwSignal<String> = RwSignal::new(String::new());
    let f_travail:   RwSignal<String> = RwSignal::new(String::new());
    let f_genre:     RwSignal<String> = RwSignal::new("M".into());
    let f_loading:   RwSignal<bool>   = RwSignal::new(false);

    let reset_form = move || {
        f_carte.set(String::new());
        f_nom.set(String::new());
        f_adresse.set(String::new());
        f_telephone.set(String::new());
        f_travail.set(String::new());
        f_genre.set("M".into());
        edit_id.set(None);
    };

    // ── Modal cotisation ───────────────────────────────────────────────────────
    let contrib_open:       RwSignal<bool>   = RwSignal::new(false);
    let contrib_membre_id:  RwSignal<i64>    = RwSignal::new(0);
    let contrib_membre_nom: RwSignal<String> = RwSignal::new(String::new());
    let confetti_active:    RwSignal<bool>   = RwSignal::new(false);

    // ─── Vue ──────────────────────────────────────────────────────────────────
    view! {
        <div class="animate-fade-in space-y-4 sm:space-y-5">

            // ── Notification d'erreur flottante ────────────────────────────────
            {move || notif_error.get().map(|msg| view! {
                <div class="fixed top-5 right-5 z-[100] flex items-start gap-3 \
                            px-4 py-3 rounded-2xl shadow-2xl border \
                            bg-white dark:bg-gray-800 \
                            border-red-200 dark:border-red-700 \
                            max-w-xs w-full animate-fade-in">
                    <IconAlertTriangle class="w-5 h-5 text-red-500 dark:text-red-400 shrink-0 mt-0.5" />
                    <p class="text-sm text-red-700 dark:text-red-300 flex-1 leading-snug">{msg}</p>
                    <button
                        on:click=move |_| notif_error.set(None)
                        class="btn-ripple text-red-400 hover:text-red-600 \
                               dark:hover:text-red-200 rounded p-0.5 transition-colors"
                    >
                        "✕"
                    </button>
                </div>
            })}

            // ── En-tête ────────────────────────────────────────────────────────
            <div class="flex flex-wrap items-start sm:items-center justify-between gap-3">
                <div>
                    <h1 class="text-xl sm:text-2xl font-bold text-gray-800 dark:text-white \
                                flex items-center gap-2">
                        <PageIcon name=icon class="w-6 h-6 text-gray-600 dark:text-gray-400" />
                        {title}
                    </h1>
                    <p class="text-gray-500 dark:text-gray-400 text-xs sm:text-sm mt-0.5">
                        {subtitle}
                    </p>
                </div>
                <div class="flex items-center gap-2 flex-wrap shrink-0">
                    {move || {
                        if transfer_to.is_none() { return None; }
                        let n = selected.get().len();
                        if n == 0 { return None; }
                        Some(view! {
                            <button
                                on:click=move |_| transfer_modal.set(true)
                                class="btn-ripple px-3 py-2 text-xs sm:text-sm font-semibold text-white \
                                       bg-amber-500 hover:bg-amber-600 \
                                       rounded-xl transition-colors duration-200 \
                                       flex items-center gap-1.5 shadow-sm"
                            >
                                <IconTransfer class="w-4 h-4" />
                                {format!("Transférer ({n})")}
                            </button>
                        })
                    }}
                    <button
                        on:click=move |_| { reset_form(); modal_ouvert.set(true); }
                        class=format!("btn-ripple px-3 sm:px-4 py-2 {} text-white rounded-xl \
                                       text-xs sm:text-sm font-semibold transition-colors \
                                       duration-200 flex items-center gap-1.5 shadow-sm",
                                       btn_class)
                    >
                        <IconPlus class="w-4 h-4" />
                        " Nouveau membre"
                    </button>
                </div>
            </div>

            // ── Barre de recherche + filtres ───────────────────────────────────
            <div class="flex flex-wrap gap-2 sm:gap-3 items-center">
                <div class="relative flex-1 min-w-[180px]">
                    <span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 \
                                 pointer-events-none">
                        <IconSearch class="w-4 h-4" />
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
                {move || {
                    if transfer_to.is_none() { return None; }
                    let n = selected.get().len();
                    if n == 0 { return None; }
                    Some(view! {
                        <span class="text-xs font-semibold text-amber-600 dark:text-amber-400 \
                                     bg-amber-50 dark:bg-amber-900/30 \
                                     px-2 py-1 rounded-lg whitespace-nowrap">
                            {format!("{n} sélectionné{}", if n > 1 { "s" } else { "" })}
                        </span>
                    })
                }}
            </div>

            // ── Tableau ────────────────────────────────────────────────────────
            <MemberTable
                membres=membres
                sorted_filtered=sorted_filtered
                page=page
                total_pages=total_pages
                sort_col=sort_col
                sort_dir=sort_dir
                transfer_to=transfer_to
                selected=selected
                all_page_selected=all_page_selected
                page_items=page_items
                transferring_ids=transferring_ids
                icon=icon
                row_hover=row_hover
                link_class=link_class
                spin_class=spin_class
                loading=loading
                refresh_ctr=refresh_ctr
                notif_error=notif_error
                modal_ouvert=modal_ouvert
                edit_id=edit_id
                f_carte=f_carte
                f_nom=f_nom
                f_adresse=f_adresse
                f_telephone=f_telephone
                f_travail=f_travail
                f_genre=f_genre
                contrib_membre_id=contrib_membre_id
                contrib_membre_nom=contrib_membre_nom
                contrib_open=contrib_open
            />

            // ── Modal formulaire ───────────────────────────────────────────────
            {move || modal_ouvert.get().then(|| view! {
                <MemberForm
                    open=modal_ouvert
                    edit_id=edit_id
                    member_type=member_type
                    btn_class=btn_class
                    refresh_ctr=refresh_ctr
                    notif_error=notif_error
                    f_carte=f_carte
                    f_nom=f_nom
                    f_adresse=f_adresse
                    f_telephone=f_telephone
                    f_travail=f_travail
                    f_genre=f_genre
                    f_loading=f_loading
                />
            })}

            // ── Modal de transfert ─────────────────────────────────────────────
            {move || {
                let tt = transfer_to?;
                transfer_modal.get().then(|| view! {
                    <TransferModal
                        open=transfer_modal
                        loading=transfer_loading
                        selected=selected
                        transfer_to=tt
                        on_confirm=do_transfer
                    />
                })
            }}

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
