#![allow(dead_code)]
/// Appels aux commandes Tauri depuis le WASM frontend.
///
/// Accède à `window.__TAURI__.core.invoke` via `js_sys::Reflect` (namespacing wasm-bindgen).
/// Toutes les fonctions sont `async` et retournent `Result<T, String>`.
use js_sys::{Function, Promise, Reflect};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::models::{
    contribution::{Contribution, ContributionEditInput, ContributionInput, ContributionWithMember},
    member::{Member, MemberInput, MemberWithTotal},
    year_summary::YearSummary,
};

// ─── Helpers internes ─────────────────────────────────────────────────────────

async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, String> {
    let window = web_sys::window().ok_or("Pas de window")?;

    let tauri = Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(|_| "window.__TAURI__ introuvable — tournez-vous dans Tauri ?")?;
    let core = Reflect::get(&tauri, &JsValue::from_str("core"))
        .map_err(|_| "window.__TAURI__.core introuvable")?;
    let invoke_fn = Reflect::get(&core, &JsValue::from_str("invoke"))
        .map_err(|_| "window.__TAURI__.core.invoke introuvable")?
        .dyn_into::<Function>()
        .map_err(|_| "invoke n'est pas une Function")?;

    let promise = invoke_fn
        .call2(&core, &JsValue::from_str(cmd), &args)
        .map_err(|e| format!("Erreur invoke : {e:?}"))?;

    JsFuture::from(
        promise
            .dyn_into::<Promise>()
            .map_err(|_| "invoke n'a pas retourné une Promise")?,
    )
    .await
    .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

fn to_js<T: Serialize>(val: &T) -> JsValue {
    serde_wasm_bindgen::to_value(val).unwrap_or(JsValue::NULL)
}

/// Invoke une commande Tauri et désérialise la réponse en `T`.
async fn invoke_cmd<T: for<'de> Deserialize<'de>>(cmd: &str, args: JsValue) -> Result<T, String> {
    serde_wasm_bindgen::from_value(invoke(cmd, args).await?)
        .map_err(|e| e.to_string())
}

// ─── Member ───────────────────────────────────────────────────────────────────

pub async fn get_members() -> Result<Vec<Member>, String> {
    invoke_cmd("get_members", to_js(&serde_json::json!({}))).await
}

pub async fn get_members_by_type(member_type: &str) -> Result<Vec<Member>, String> {
    invoke_cmd("get_members_by_type", to_js(&serde_json::json!({ "memberType": member_type }))).await
}

pub async fn get_members_by_type_with_total(
    member_type: &str,
) -> Result<Vec<MemberWithTotal>, String> {
    invoke_cmd(
        "get_members_by_type_with_total",
        to_js(&serde_json::json!({ "memberType": member_type })),
    )
    .await
}

pub async fn get_member(id: i64) -> Result<Member, String> {
    invoke_cmd("get_member", to_js(&serde_json::json!({ "id": id }))).await
}

pub async fn create_member(input: &MemberInput) -> Result<Member, String> {
    invoke_cmd("create_member", to_js(&serde_json::json!({ "member": input }))).await
}

pub async fn update_member(id: i64, input: &MemberInput) -> Result<Member, String> {
    invoke_cmd(
        "update_member",
        to_js(&serde_json::json!({ "id": id, "member": input })),
    )
    .await
}

pub async fn delete_member(id: i64) -> Result<(), String> {
    invoke("delete_member", to_js(&serde_json::json!({ "id": id }))).await.map(|_| ())
}

/// Transfère une liste de membres vers un nouveau type (ex: "Communiant").
pub async fn transfer_members(ids: &[i64], new_type: &str) -> Result<usize, String> {
    invoke_cmd(
        "transfer_members",
        to_js(&serde_json::json!({ "ids": ids, "newType": new_type })),
    )
    .await
}

// ─── Contribution ─────────────────────────────────────────────────────────────

pub async fn get_contributions(member_id: i64) -> Result<Vec<Contribution>, String> {
    invoke_cmd(
        "get_contributions",
        to_js(&serde_json::json!({ "memberId": member_id })),
    )
    .await
}

pub async fn get_contributions_by_year(year: i32) -> Result<Vec<Contribution>, String> {
    invoke_cmd(
        "get_contributions_by_year",
        to_js(&serde_json::json!({ "year": year })),
    )
    .await
}

pub async fn create_contribution(input: &ContributionInput) -> Result<Contribution, String> {
    invoke_cmd(
        "create_contribution",
        to_js(&serde_json::json!({ "contribution": input })),
    )
    .await
}

pub async fn delete_contribution(id: i64) -> Result<(), String> {
    invoke("delete_contribution", to_js(&serde_json::json!({ "id": id }))).await.map(|_| ())
}

pub async fn get_contributions_by_year_with_member(
    year: i32,
) -> Result<Vec<ContributionWithMember>, String> {
    invoke_cmd(
        "get_contributions_by_year_with_member",
        to_js(&serde_json::json!({ "year": year })),
    )
    .await
}

pub async fn get_all_contributions_with_member() -> Result<Vec<ContributionWithMember>, String> {
    invoke_cmd(
        "get_all_contributions_with_member",
        to_js(&serde_json::json!({})),
    )
    .await
}

// ─── YearSummary ──────────────────────────────────────────────────────────────

pub async fn get_year_summaries() -> Result<Vec<YearSummary>, String> {
    invoke_cmd("get_year_summaries", to_js(&serde_json::json!({}))).await
}

pub async fn get_year_summary(year: i32) -> Result<Option<YearSummary>, String> {
    invoke_cmd(
        "get_year_summary",
        to_js(&serde_json::json!({ "year": year })),
    )
    .await
}

pub async fn close_year(year: i32, note: Option<String>) -> Result<YearSummary, String> {
    invoke_cmd(
        "close_year",
        to_js(&serde_json::json!({ "year": year, "note": note })),
    )
    .await
}

pub async fn reopen_year(year: i32) -> Result<YearSummary, String> {
    invoke_cmd("reopen_year", to_js(&serde_json::json!({ "year": year }))).await
}

pub async fn check_and_close_previous_year() -> Result<Option<YearSummary>, String> {
    invoke_cmd("check_and_close_previous_year", to_js(&serde_json::json!({}))).await
}

// ─── Import / Export CSV ──────────────────────────────────────────────────────

pub async fn export_members_csv(member_type: &str) -> Result<String, String> {
    invoke_cmd(
        "export_members_csv",
        to_js(&serde_json::json!({ "memberType": member_type })),
    )
    .await
}

pub async fn export_members_excel(member_type: &str) -> Result<Vec<u8>, String> {
    invoke_cmd(
        "export_members_excel",
        to_js(&serde_json::json!({ "memberType": member_type })),
    )
    .await
}

pub async fn import_members_csv(csv_content: &str, member_type: &str) -> Result<usize, String> {
    invoke_cmd(
        "import_members_csv",
        to_js(&serde_json::json!({ "csvContent": csv_content, "memberType": member_type })),
    )
    .await
}

// ─── PIN + édition contribution ───────────────────────────────────────────────

pub async fn verify_pin(pin: &str) -> Result<bool, String> {
    invoke_cmd("verify_pin", to_js(&serde_json::json!({ "pin": pin }))).await
}

pub async fn update_contribution(
    id: i64,
    input: &ContributionEditInput,
) -> Result<Contribution, String> {
    invoke_cmd(
        "update_contribution",
        to_js(&serde_json::json!({ "id": id, "input": input })),
    )
    .await
}

// ─── Fenêtre ──────────────────────────────────────────────────────────────────

pub async fn minimize_window() -> Result<(), String> {
    invoke("minimize_window", JsValue::NULL).await.map(|_| ())
}

pub async fn toggle_maximize() -> Result<(), String> {
    invoke("toggle_maximize", JsValue::NULL).await.map(|_| ())
}

pub async fn close_window() -> Result<(), String> {
    invoke("close_window", JsValue::NULL).await.map(|_| ())
}
