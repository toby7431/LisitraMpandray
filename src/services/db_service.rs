/// Appels aux commandes Tauri depuis le WASM frontend.
///
/// Accède à `window.__TAURI__.core.invoke` via `js_sys::Reflect` (namespacing wasm-bindgen).
/// Toutes les fonctions sont `async` et retournent `Result<T, String>`.
use js_sys::{Function, Promise, Reflect};
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::models::{
    contribution::{Contribution, ContributionInput, ContributionWithMember},
    member::{Member, MemberInput, MemberWithTotal},
    year_summary::YearSummary,
};

// ─── Helper interne ───────────────────────────────────────────────────────────

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

// ─── Member ───────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn get_members() -> Result<Vec<Member>, String> {
    let res = invoke("get_members", to_js(&serde_json::json!({}))).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn get_members_by_type(member_type: &str) -> Result<Vec<Member>, String> {
    let res = invoke(
        "get_members_by_type",
        to_js(&serde_json::json!({ "memberType": member_type })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn get_members_by_type_with_total(
    member_type: &str,
) -> Result<Vec<MemberWithTotal>, String> {
    let res = invoke(
        "get_members_by_type_with_total",
        to_js(&serde_json::json!({ "memberType": member_type })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub async fn get_member(id: i64) -> Result<Member, String> {
    let res = invoke("get_member", to_js(&serde_json::json!({ "id": id }))).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn create_member(input: &MemberInput) -> Result<Member, String> {
    let res = invoke("create_member", to_js(&serde_json::json!({ "member": input }))).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn update_member(id: i64, input: &MemberInput) -> Result<Member, String> {
    let res = invoke(
        "update_member",
        to_js(&serde_json::json!({ "id": id, "member": input })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn delete_member(id: i64) -> Result<(), String> {
    invoke("delete_member", to_js(&serde_json::json!({ "id": id }))).await?;
    Ok(())
}

/// Transfère une liste de membres vers un nouveau type (ex: "Communiant").
pub async fn transfer_members(ids: &[i64], new_type: &str) -> Result<usize, String> {
    let res = invoke(
        "transfer_members",
        to_js(&serde_json::json!({ "ids": ids, "newType": new_type })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

// ─── Contribution ─────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn get_contributions(member_id: i64) -> Result<Vec<Contribution>, String> {
    let res = invoke(
        "get_contributions",
        to_js(&serde_json::json!({ "memberId": member_id })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub async fn get_contributions_by_year(year: i32) -> Result<Vec<Contribution>, String> {
    let res = invoke(
        "get_contributions_by_year",
        to_js(&serde_json::json!({ "year": year })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn create_contribution(input: &ContributionInput) -> Result<Contribution, String> {
    let res = invoke(
        "create_contribution",
        to_js(&serde_json::json!({ "contribution": input })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub async fn delete_contribution(id: i64) -> Result<(), String> {
    invoke(
        "delete_contribution",
        to_js(&serde_json::json!({ "id": id })),
    )
    .await?;
    Ok(())
}

pub async fn get_contributions_by_year_with_member(
    year: i32,
) -> Result<Vec<ContributionWithMember>, String> {
    let res = invoke(
        "get_contributions_by_year_with_member",
        to_js(&serde_json::json!({ "year": year })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

// ─── YearSummary ──────────────────────────────────────────────────────────────

pub async fn get_year_summaries() -> Result<Vec<YearSummary>, String> {
    let res = invoke("get_year_summaries", to_js(&serde_json::json!({}))).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn get_year_summary(year: i32) -> Result<Option<YearSummary>, String> {
    let res = invoke(
        "get_year_summary",
        to_js(&serde_json::json!({ "year": year })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub async fn close_year(year: i32, note: Option<String>) -> Result<YearSummary, String> {
    let res = invoke(
        "close_year",
        to_js(&serde_json::json!({ "year": year, "note": note })),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub async fn reopen_year(year: i32) -> Result<YearSummary, String> {
    let res = invoke("reopen_year", to_js(&serde_json::json!({ "year": year }))).await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
}

pub async fn check_and_close_previous_year() -> Result<Option<YearSummary>, String> {
    let res = invoke(
        "check_and_close_previous_year",
        to_js(&serde_json::json!({})),
    )
    .await?;
    serde_wasm_bindgen::from_value(res).map_err(|e| e.to_string())
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
