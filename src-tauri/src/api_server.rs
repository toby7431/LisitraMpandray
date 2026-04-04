/// Serveur Axum — exposé par le PC serveur sur le réseau local.
///
/// Le PC client appelle ces endpoints via RemoteClient (reqwest).
/// Toutes les routes reflètent exactement les commandes Tauri.
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

use crate::db::Repository;
use crate::export::{
    build_csv_from_members, build_excel_bytes, parse_csv_to_members,
};

type Repo = Arc<Repository>;
type ApiErr = (StatusCode, String);

fn e500(e: impl std::fmt::Display) -> ApiErr {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

// ── Lancement ─────────────────────────────────────────────────────────────────

pub async fn start_server(repo: Repository, port: u16) {
    let repo = Arc::new(repo);

    let app = Router::new()
        // Santé
        .route("/api/health", get(health))
        // Members
        .route("/api/members", get(get_members).post(create_member))
        .route("/api/members/by-type/:member_type", get(get_members_by_type))
        .route("/api/members/by-type/:member_type/totals", get(get_members_by_type_with_total))
        .route("/api/members/:id", get(get_member).put(update_member).delete(delete_member_route))
        .route("/api/transfer-members", post(transfer_members))
        // Contributions
        .route("/api/contributions", post(create_contribution))
        .route("/api/contributions/by-member/:member_id", get(get_contributions_by_member))
        .route("/api/contributions/by-year/:year/with-member", get(get_contributions_by_year_with_member))
        .route("/api/contributions/all/with-member", get(get_all_contributions_with_member))
        .route("/api/contributions/by-year/:year", get(get_contributions_by_year))
        .route("/api/contributions/:id", delete(delete_contribution_route).put(update_contribution_route))
        // PIN
        .route("/api/verify-pin", post(verify_pin_route))
        // Year summaries
        .route("/api/year-summaries", get(get_year_summaries))
        .route("/api/year-summaries/:year", get(get_year_summary))
        .route("/api/year-summaries/:year/close", post(close_year))
        .route("/api/year-summaries/:year/reopen", post(reopen_year))
        .route("/api/year/check-close", post(check_and_close_previous_year))
        // Export / Import
        .route("/api/export/csv/:member_type", get(export_csv))
        .route("/api/export/excel/:member_type", get(export_excel))
        .route("/api/import/csv/:member_type", post(import_csv))
        .layer(CorsLayer::permissive())
        .with_state(repo);

    let addr = format!("0.0.0.0:{port}");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[API Server] Impossible de démarrer sur {addr}: {e}");
            return;
        }
    };

    eprintln!("[API Server] Démarré sur le port {port}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("[API Server] Erreur: {e}");
    }
}

// ── Health ────────────────────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

// ── Members ───────────────────────────────────────────────────────────────────

async fn get_members(State(repo): State<Repo>) -> Result<impl IntoResponse, ApiErr> {
    repo.get_members().await.map(Json).map_err(e500)
}

async fn get_member(
    State(repo): State<Repo>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_member(id).await.map(Json).map_err(e500)
}

async fn get_members_by_type(
    State(repo): State<Repo>,
    Path(member_type): Path<String>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_members_by_type(&member_type).await.map(Json).map_err(e500)
}

async fn get_members_by_type_with_total(
    State(repo): State<Repo>,
    Path(member_type): Path<String>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_members_by_type_with_total(&member_type).await.map(Json).map_err(e500)
}

async fn create_member(
    State(repo): State<Repo>,
    Json(input): Json<crate::db::MemberInput>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.create_member(input).await.map(Json).map_err(e500)
}

async fn update_member(
    State(repo): State<Repo>,
    Path(id): Path<i64>,
    Json(input): Json<crate::db::MemberInput>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.update_member(id, input).await.map(Json).map_err(e500)
}

async fn delete_member_route(
    State(repo): State<Repo>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.delete_member(id).await.map(|_| StatusCode::NO_CONTENT).map_err(e500)
}

#[derive(Deserialize)]
struct TransferBody {
    ids: Vec<i64>,
    new_type: String,
}

async fn transfer_members(
    State(repo): State<Repo>,
    Json(body): Json<TransferBody>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.transfer_members(&body.ids, &body.new_type)
        .await
        .map(Json)
        .map_err(e500)
}

// ── Contributions ─────────────────────────────────────────────────────────────

async fn get_contributions_by_member(
    State(repo): State<Repo>,
    Path(member_id): Path<i64>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_contributions(member_id).await.map(Json).map_err(e500)
}

async fn get_contributions_by_year(
    State(repo): State<Repo>,
    Path(year): Path<i32>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_contributions_by_year(year).await.map(Json).map_err(e500)
}

async fn create_contribution(
    State(repo): State<Repo>,
    Json(input): Json<crate::db::ContributionInput>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.create_contribution(input).await.map(Json).map_err(e500)
}

async fn delete_contribution_route(
    State(repo): State<Repo>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.delete_contribution(id).await.map(|_| StatusCode::NO_CONTENT).map_err(e500)
}

async fn get_contributions_by_year_with_member(
    State(repo): State<Repo>,
    Path(year): Path<i32>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_contributions_by_year_with_member(year)
        .await
        .map(Json)
        .map_err(e500)
}

async fn get_all_contributions_with_member(
    State(repo): State<Repo>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_all_contributions_with_member()
        .await
        .map(Json)
        .map_err(e500)
}

// ── Year Summaries ────────────────────────────────────────────────────────────

async fn get_year_summaries(State(repo): State<Repo>) -> Result<impl IntoResponse, ApiErr> {
    repo.get_year_summaries().await.map(Json).map_err(e500)
}

async fn get_year_summary(
    State(repo): State<Repo>,
    Path(year): Path<i32>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.get_year_summary(year).await.map(Json).map_err(e500)
}

#[derive(Deserialize)]
struct CloseYearBody {
    note: Option<String>,
}

async fn close_year(
    State(repo): State<Repo>,
    Path(year): Path<i32>,
    Json(body): Json<CloseYearBody>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.close_year(year, body.note).await.map(Json).map_err(e500)
}

async fn reopen_year(
    State(repo): State<Repo>,
    Path(year): Path<i32>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.reopen_year(year).await.map(Json).map_err(e500)
}

async fn check_and_close_previous_year(
    State(repo): State<Repo>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.check_and_close_previous_year().await.map(Json).map_err(e500)
}

// ── PIN ───────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VerifyPinBody {
    pin: String,
}

async fn verify_pin_route(
    State(repo): State<Repo>,
    Json(body): Json<VerifyPinBody>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.verify_pin(&body.pin).await.map(Json).map_err(e500)
}

async fn update_contribution_route(
    State(repo): State<Repo>,
    Path(id): Path<i64>,
    Json(input): Json<crate::db::ContributionEditInput>,
) -> Result<impl IntoResponse, ApiErr> {
    repo.update_contribution(id, input)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

// ── Export / Import ───────────────────────────────────────────────────────────

async fn export_csv(
    State(repo): State<Repo>,
    Path(member_type): Path<String>,
) -> Result<impl IntoResponse, ApiErr> {
    let members = repo.get_members_by_type(&member_type).await.map_err(e500)?;
    let csv = build_csv_from_members(&members);
    // Retourner comme JSON string pour que le client puisse désérialiser facilement
    Ok(Json(csv))
}

async fn export_excel(
    State(repo): State<Repo>,
    Path(member_type): Path<String>,
) -> Result<impl IntoResponse, ApiErr> {
    let members = repo.get_members_by_type_with_total(&member_type).await.map_err(e500)?;
    let bytes = build_excel_bytes(&members, &member_type).map_err(|e| e500(e))?;
    Ok((
        StatusCode::OK,
        [(
            "Content-Type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )],
        bytes,
    ))
}

#[derive(Deserialize)]
struct ImportCsvBody {
    content: String,
}

async fn import_csv(
    State(repo): State<Repo>,
    Path(member_type): Path<String>,
    Json(body): Json<ImportCsvBody>,
) -> Result<impl IntoResponse, ApiErr> {
    let inputs = parse_csv_to_members(&body.content, &member_type);
    repo.import_members(inputs).await.map(Json).map_err(e500)
}
