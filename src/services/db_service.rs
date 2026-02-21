/// Appels aux commandes Tauri depuis le WASM frontend.
///
/// On accède à `window.__TAURI__.core.invoke` via `js_sys::Reflect` pour éviter
/// les problèmes de namespacing wasm-bindgen avec les objets imbriqués.
use js_sys::{Function, Promise, Reflect};
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::models::membre::{Membre, MembreInput};

// ─── Helpers internes ────────────────────────────────────────────────────────

/// Appelle `window.__TAURI__.core.invoke(cmd, args)` et retourne le JsValue résultat.
async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, String> {
    let window = web_sys::window().ok_or("Pas de window")?;

    let tauri = Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "window.__TAURI__ introuvable — l'app tourne-t-elle dans Tauri ?")?;
    let core = Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "window.__TAURI__.core introuvable")?;
    let invoke_fn = Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "window.__TAURI__.core.invoke introuvable")?
        .dyn_into::<Function>()
        .map_err(|_| "invoke n'est pas une Function")?;

    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(cmd), &args)
        .map_err(|e| format!("Erreur invoke : {:?}", e))?;

    let result = JsFuture::from(
        promise
            .dyn_into::<Promise>()
            .map_err(|_| "invoke n'a pas retourné une Promise")?,
    )
    .await
    .map_err(|e| {
        e.as_string()
            .unwrap_or_else(|| format!("{:?}", e))
    })?;

    Ok(result)
}

fn to_js<T: Serialize>(val: &T) -> JsValue {
    serde_wasm_bindgen::to_value(val).unwrap_or(JsValue::NULL)
}

// ─── API publique ────────────────────────────────────────────────────────────

pub async fn get_membres() -> Result<Vec<Membre>, String> {
    let res = invoke("get_membres", to_js(&serde_json::json!({}))).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn add_membre(input: &MembreInput) -> Result<Membre, String> {
    let args = to_js(&serde_json::json!({ "membre": input }));
    let res = invoke("add_membre", args).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn update_membre(id: i64, input: &MembreInput) -> Result<Membre, String> {
    let args = to_js(&serde_json::json!({ "id": id, "membre": input }));
    let res = invoke("update_membre", args).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn archive_membre(id: i64) -> Result<(), String> {
    let args = to_js(&serde_json::json!({ "id": id }));
    invoke("archive_membre", args).await?;
    Ok(())
}

pub async fn delete_membre(id: i64) -> Result<(), String> {
    let args = to_js(&serde_json::json!({ "id": id }));
    invoke("delete_membre", args).await?;
    Ok(())
}
