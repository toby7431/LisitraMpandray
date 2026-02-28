mod db;
use db::{
    Contribution, ContributionInput, ContributionWithMember, Member, MemberInput, MemberWithTotal,
    Repository, YearSummary,
};
use tauri::Manager;

// ─── Commandes Member ─────────────────────────────────────────────────────────

#[tauri::command]
async fn get_members(
    state: tauri::State<'_, Repository>,
) -> Result<Vec<Member>, String> {
    state.get_members().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_members_by_type(
    state: tauri::State<'_, Repository>,
    member_type: String,
) -> Result<Vec<Member>, String> {
    state.get_members_by_type(&member_type).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_members_by_type_with_total(
    state: tauri::State<'_, Repository>,
    member_type: String,
) -> Result<Vec<MemberWithTotal>, String> {
    state
        .get_members_by_type_with_total(&member_type)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_member(
    state: tauri::State<'_, Repository>,
    id: i64,
) -> Result<Member, String> {
    state.get_member(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_member(
    state: tauri::State<'_, Repository>,
    member: MemberInput,
) -> Result<Member, String> {
    state.create_member(member).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_member(
    state: tauri::State<'_, Repository>,
    id: i64,
    member: MemberInput,
) -> Result<Member, String> {
    state.update_member(id, member).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_member(
    state: tauri::State<'_, Repository>,
    id: i64,
) -> Result<(), String> {
    state.delete_member(id).await.map_err(|e| e.to_string())
}

// ─── Commandes Contribution ───────────────────────────────────────────────────

#[tauri::command]
async fn get_contributions(
    state: tauri::State<'_, Repository>,
    member_id: i64,
) -> Result<Vec<Contribution>, String> {
    state.get_contributions(member_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_contributions_by_year(
    state: tauri::State<'_, Repository>,
    year: i32,
) -> Result<Vec<Contribution>, String> {
    state.get_contributions_by_year(year).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_contribution(
    state: tauri::State<'_, Repository>,
    contribution: ContributionInput,
) -> Result<Contribution, String> {
    state.create_contribution(contribution).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_contribution(
    state: tauri::State<'_, Repository>,
    id: i64,
) -> Result<(), String> {
    state.delete_contribution(id).await.map_err(|e| e.to_string())
}

// ─── Commandes YearSummary ────────────────────────────────────────────────────

#[tauri::command]
async fn get_year_summaries(
    state: tauri::State<'_, Repository>,
) -> Result<Vec<YearSummary>, String> {
    state.get_year_summaries().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_year_summary(
    state: tauri::State<'_, Repository>,
    year: i32,
) -> Result<Option<YearSummary>, String> {
    state.get_year_summary(year).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn close_year(
    state: tauri::State<'_, Repository>,
    year: i32,
    note: Option<String>,
) -> Result<YearSummary, String> {
    state.close_year(year, note).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn reopen_year(
    state: tauri::State<'_, Repository>,
    year: i32,
) -> Result<YearSummary, String> {
    state.reopen_year(year).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn transfer_members(
    state: tauri::State<'_, Repository>,
    ids: Vec<i64>,
    new_type: String,
) -> Result<usize, String> {
    state.transfer_members(&ids, &new_type).await.map_err(|e| e.to_string())
}

// ─── Commandes Archives ───────────────────────────────────────────────────────

#[tauri::command]
async fn get_contributions_by_year_with_member(
    state: tauri::State<'_, Repository>,
    year: i32,
) -> Result<Vec<ContributionWithMember>, String> {
    state
        .get_contributions_by_year_with_member(year)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_and_close_previous_year(
    state: tauri::State<'_, Repository>,
) -> Result<Option<YearSummary>, String> {
    state
        .check_and_close_previous_year()
        .await
        .map_err(|e| e.to_string())
}

// ─── Point d'entrée ───────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Impossible d'obtenir app_data_dir");
            std::fs::create_dir_all(&app_dir)
                .expect("Impossible de créer app_data_dir");

            let db_path = app_dir
                .join("eglise.db")
                .to_str()
                .expect("Chemin DB invalide (non-UTF8)")
                .to_owned();

            let rt = tokio::runtime::Runtime::new()
                .expect("Impossible de créer le runtime Tokio");
            let repo = rt
                .block_on(Repository::new(&db_path))
                .expect("Impossible d'initialiser la base SQLite");

            app.manage(repo);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Member
            get_members,
            get_members_by_type,
            get_members_by_type_with_total,
            get_member,
            create_member,
            update_member,
            delete_member,
            // Contribution
            get_contributions,
            get_contributions_by_year,
            create_contribution,
            delete_contribution,
            // YearSummary
            get_year_summaries,
            get_year_summary,
            close_year,
            reopen_year,
            // Transfer
            transfer_members,
            // Archives
            get_contributions_by_year_with_member,
            check_and_close_previous_year,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de Tauri");
}
