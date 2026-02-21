mod db;
use db::{DbState, MembreInput};
use tauri::Manager;

// ─── Commandes Tauri ────────────────────────────────────────────────────────

#[tauri::command]
async fn get_membres(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<db::Membre>, String> {
    state.get_membres().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_membre(
    state: tauri::State<'_, DbState>,
    membre: MembreInput,
) -> Result<db::Membre, String> {
    state.add_membre(membre).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_membre(
    state: tauri::State<'_, DbState>,
    id: i64,
    membre: MembreInput,
) -> Result<db::Membre, String> {
    state.update_membre(id, membre).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn archive_membre(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.archive_membre(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_membre(
    state: tauri::State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    state.delete_membre(id).await.map_err(|e| e.to_string())
}

// ─── Point d'entrée ─────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Répertoire de données de l'application
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Impossible d'obtenir le répertoire app_data");
            std::fs::create_dir_all(&app_dir)
                .expect("Impossible de créer le répertoire app_data");

            let db_path = app_dir.join("eglise.db");
            let db_url = format!(
                "sqlite://{}?mode=rwc",
                db_path.to_str().expect("Chemin DB invalide")
            );

            // Initialiser la base de données (synchrone au démarrage)
            let rt = tokio::runtime::Runtime::new()
                .expect("Impossible de créer le runtime Tokio");
            let db_state = rt
                .block_on(DbState::new(&db_url))
                .expect("Impossible d'initialiser la base de données SQLite");

            app.manage(db_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_membres,
            add_membre,
            update_membre,
            archive_membre,
            delete_membre,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de l'application Tauri");
}
