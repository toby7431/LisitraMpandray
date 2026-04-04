/// Commandes Tauri pour la configuration (mode serveur/client).
use js_sys::{Function, Promise, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Server,
    Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: AppMode,
    pub server_ip: String,
    pub server_port: u16,
}

// ─── Helper ───────────────────────────────────────────────────────────────────

async fn invoke_raw(cmd: &str, args: JsValue) -> Result<JsValue, String> {
    let window = web_sys::window().ok_or("Pas de window")?;
    let tauri = Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "Tauri introuvable")?;
    let core = Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "Tauri core introuvable")?;
    let invoke_fn = Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "invoke introuvable")?
        .dyn_into::<Function>()
        .map_err(|_| "invoke n'est pas une Function")?;

    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(cmd), &args)
        .map_err(|e| format!("{e:?}"))?;

    JsFuture::from(
        promise.dyn_into::<Promise>().map_err(|_| "Pas une Promise")?,
    )
    .await
    .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

async fn invoke_cmd<T: for<'de> Deserialize<'de>>(
    cmd: &str,
    args: JsValue,
) -> Result<T, String> {
    serde_wasm_bindgen::from_value(invoke_raw(cmd, args).await?)
        .map_err(|e| e.to_string())
}

fn to_js<T: Serialize>(val: &T) -> JsValue {
    serde_wasm_bindgen::to_value(val).unwrap_or(JsValue::NULL)
}

// ─── API publique ─────────────────────────────────────────────────────────────

/// Retourne la configuration actuelle (None si non configuré).
pub async fn get_config() -> Result<Option<AppConfig>, String> {
    invoke_cmd("get_config", to_js(&serde_json::json!({}))).await
}

/// Sauvegarde la configuration et initialise la source de données.
pub async fn save_config(config: &AppConfig) -> Result<(), String> {
    invoke_raw("save_config", to_js(&serde_json::json!({ "config": config }))).await.map(|_| ())
}

/// Supprime la configuration et remet l'app en état non-configuré.
pub async fn reset_config() -> Result<(), String> {
    invoke_raw("reset_config", to_js(&serde_json::json!({}))).await.map(|_| ())
}

/// Teste si le serveur à l'adresse ip:port est accessible.
pub async fn test_server_connection(ip: &str, port: u16) -> Result<bool, String> {
    invoke_cmd(
        "test_server_connection",
        to_js(&serde_json::json!({ "ip": ip, "port": port })),
    )
    .await
}

/// Démarre un serveur Axum local (SQLite en mémoire) pour simuler le mode client.
/// Retourne le port sur lequel le serveur écoute (127.0.0.1).
pub async fn start_mock_server() -> Result<u16, String> {
    invoke_cmd("start_mock_server", to_js(&serde_json::json!({}))).await
}
