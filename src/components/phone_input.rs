/// Champ téléphone Madagascar contrôlé.
///
/// - Démarre toujours par "+261 " (impossible à supprimer)
/// - Format automatique : +261 34 123 45  (7 chiffres, groupes 2-3-2)
/// - N'accepte que des chiffres ; espaces gérés automatiquement
use leptos::prelude::*;

// ─── Formatage ────────────────────────────────────────────────────────────────

/// Formate 0-7 chiffres abonnés en "+261 XX XXX XX".
pub fn fmt_phone(digits: &str) -> String {
    let d: Vec<char> = digits.chars().collect();
    let len = d.len();
    let mut r = "+261 ".to_string();
    if len > 0 { r.extend(&d[..2.min(len)]); }
    if len > 2 { r.push(' '); r.extend(&d[2..5.min(len)]); }
    if len > 5 { r.push(' '); r.extend(&d[5..7.min(len)]); }
    r
}

/// Extrait les 7 chiffres abonnés depuis une chaîne quelconque.
fn extract_digits(raw: &str) -> String {
    let all: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    let sub = if all.starts_with("261") { &all[3..] } else { &all };
    sub.chars().take(7).collect()
}

// ─── Composant ────────────────────────────────────────────────────────────────

#[component]
pub fn PhoneInput(
    value: RwSignal<String>,
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let node: NodeRef<leptos::html::Input> = NodeRef::new();

    // Synchronise le DOM quand la valeur change depuis l'extérieur
    Effect::new(move |_| {
        let v = value.get();
        if let Some(el) = node.get() {
            el.set_value(&v);
        }
    });

    // ── Saisie ────────────────────────────────────────────────────────────────
    // on:input — type inféré par Leptos (web_sys::Event)
    let on_input = move |_| {
        let el = match node.get() { Some(e) => e, None => return };
        let digits = extract_digits(&el.value());
        let formatted = fmt_phone(&digits);
        el.set_value(&formatted);
        value.set(formatted.clone());
        let pos = formatted.len() as u32;
        let _ = el.set_selection_range(pos, pos);
    };

    // ── Protection du préfixe (Backspace / Delete) ────────────────────────────
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let el = match node.get() { Some(e) => e, None => return };
        let cursor  = el.selection_start().ok().flatten().unwrap_or(0);
        let sel_end = el.selection_end().ok().flatten().unwrap_or(0);
        let key     = ev.key();
        if (key == "Backspace" || key == "Delete") && cursor <= 5 && sel_end <= 5 {
            ev.prevent_default();
        }
    };

    // ── Focus : injecte le préfixe si vide, curseur en fin ───────────────────
    let on_focus = move |_| {
        let el = match node.get() { Some(e) => e, None => return };
        let v = value.get_untracked();
        if v.len() < 5 {
            let pre = "+261 ".to_string();
            value.set(pre.clone());
            el.set_value(&pre);
        }
        let len = el.value().len() as u32;
        let _ = el.set_selection_range(len, len);
    };

    // ── Clic : empêche de placer le curseur avant le préfixe ─────────────────
    let on_click = move |_| {
        let el = match node.get() { Some(e) => e, None => return };
        let cur = el.selection_start().ok().flatten().unwrap_or(0);
        if cur < 5 {
            let end = 5_u32.min(el.value().len() as u32);
            let _ = el.set_selection_range(end, end);
        }
    };

    view! {
        <input
            type="tel"
            node_ref=node
            placeholder="+261 34 123 45"
            class=class
            on:input=on_input
            on:keydown=on_keydown
            on:focus=on_focus
            on:click=on_click
        />
    }
}
