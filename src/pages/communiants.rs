use leptos::prelude::*;
use crate::components::member_page::MemberPage;

#[component]
pub fn Communiants() -> impl IntoView {
    view! {
        <MemberPage
            member_type="Communiant"
            icon="cross"
            title="Communiants"
            subtitle="Membres communiants actifs de l'église"
            btn_class="bg-blue-600 hover:bg-blue-700"
            row_hover="hover:bg-blue-50/50 dark:hover:bg-blue-900/10"
            link_class="text-blue-600 dark:text-blue-400"
            spin_class="border-blue-500"
        />
    }
}
