/// Utilitaires partagés entre les composants frontend (WASM).
use js_sys::{Function, Promise};
use wasm_bindgen_futures::JsFuture;

/// Attendre `ms` millisecondes (non-bloquant, WASM-compatible).
pub async fn sleep_ms(ms: u32) {
    let promise = Promise::new(&mut |resolve: Function, _: Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .unwrap();
    });
    let _ = JsFuture::from(promise).await;
}

/// Formate un montant numérique (en chaîne) en "1 234 567\u{202f}Ar".
///
/// Accepte les chaînes comme "15000", "15000.50", etc.
/// Arrondit à l'entier (partie entière uniquement).
pub fn format_ariary(amount_str: &str) -> String {
    let n: i64 = amount_str.parse::<f64>().unwrap_or(0.0) as i64;
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('\u{202f}'); // espace fine insécable
        }
        result.push(c);
    }
    format!("{}\u{202f}Ar", result)
}
