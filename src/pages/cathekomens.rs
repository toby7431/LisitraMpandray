use leptos::prelude::*;
use crate::components::member_page::MemberPage;

#[component]
pub fn Cathekomens() -> impl IntoView {
    view! {
        <MemberPage
            member_type="Cathekomen"
            icon="book"
            title="Cathécomènes"
            subtitle="Membres en cours de formation catéchétique"
            btn_class="bg-emerald-600 hover:bg-emerald-700"
            row_hover="hover:bg-emerald-50/50 dark:hover:bg-emerald-900/10"
            link_class="text-emerald-600 dark:text-emerald-400"
            spin_class="border-emerald-500"
            transfer_to="Communiant"
        />
    }
}
