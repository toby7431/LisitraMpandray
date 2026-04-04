mod api_server;
mod config;
mod db;
mod export;
mod remote_client;

static MOCK_SERVER_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();
/// Port du serveur API principal — initialisé une seule fois.
static API_SERVER_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();

use config::{load_config, save_config_to_disk, AppConfig, AppMode};
use db::{
    Contribution, ContributionInput, ContributionWithMember, Member, MemberInput, MemberWithTotal,
    Repository, YearSummary,
};
use export::{build_csv_from_members, build_excel_bytes, parse_csv_to_members};
use remote_client::RemoteClient;
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use tokio::sync::RwLock;

// ─── DataSource ────────────────────────────────────────────────────────────────

/// Abstraction sur la source de données :
/// - Local  : SQLite sur ce PC (mode serveur)
/// - Remote : API HTTP sur le PC serveur (mode client)
/// - Unconfigured : premier lancement, aucune config
pub enum DataSource {
    Local(Repository),
    Remote(RemoteClient),
    Unconfigured,
}

macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            DataSource::Local(r)     => r.$method($($arg),*).await.map_err(|e| e.to_string()),
            DataSource::Remote(c)    => c.$method($($arg),*).await.map_err(|e| e.to_string()),
            DataSource::Unconfigured => Err(Self::not_configured()),
        }
    };
}

impl DataSource {
    fn not_configured() -> String {
        "non_configure".to_string()
    }

    // ── Members ───────────────────────────────────────────────────────────────

    async fn get_members(&self) -> Result<Vec<Member>, String> {
        dispatch!(self, get_members)
    }

    async fn get_members_by_type(&self, t: &str) -> Result<Vec<Member>, String> {
        dispatch!(self, get_members_by_type, t)
    }

    async fn get_members_by_type_with_total(&self, t: &str) -> Result<Vec<MemberWithTotal>, String> {
        dispatch!(self, get_members_by_type_with_total, t)
    }

    async fn get_member(&self, id: i64) -> Result<Member, String> {
        dispatch!(self, get_member, id)
    }

    async fn create_member(&self, input: MemberInput) -> Result<Member, String> {
        dispatch!(self, create_member, input)
    }

    async fn update_member(&self, id: i64, input: MemberInput) -> Result<Member, String> {
        dispatch!(self, update_member, id, input)
    }

    async fn delete_member(&self, id: i64) -> Result<(), String> {
        dispatch!(self, delete_member, id)
    }

    async fn transfer_members(&self, ids: &[i64], new_type: &str) -> Result<usize, String> {
        dispatch!(self, transfer_members, ids, new_type)
    }

    // ── Contributions ─────────────────────────────────────────────────────────

    async fn get_contributions(&self, member_id: i64) -> Result<Vec<Contribution>, String> {
        dispatch!(self, get_contributions, member_id)
    }

    async fn get_contributions_by_year(&self, year: i32) -> Result<Vec<Contribution>, String> {
        dispatch!(self, get_contributions_by_year, year)
    }

    async fn create_contribution(&self, input: ContributionInput) -> Result<Contribution, String> {
        dispatch!(self, create_contribution, input)
    }

    async fn delete_contribution(&self, id: i64) -> Result<(), String> {
        dispatch!(self, delete_contribution, id)
    }

    async fn get_contributions_by_year_with_member(
        &self,
        year: i32,
    ) -> Result<Vec<ContributionWithMember>, String> {
        dispatch!(self, get_contributions_by_year_with_member, year)
    }

    // ── Year Summaries ────────────────────────────────────────────────────────

    async fn get_year_summaries(&self) -> Result<Vec<YearSummary>, String> {
        dispatch!(self, get_year_summaries)
    }

    async fn get_year_summary(&self, year: i32) -> Result<Option<YearSummary>, String> {
        dispatch!(self, get_year_summary, year)
    }

    async fn close_year(&self, year: i32, note: Option<String>) -> Result<YearSummary, String> {
        dispatch!(self, close_year, year, note)
    }

    async fn reopen_year(&self, year: i32) -> Result<YearSummary, String> {
        dispatch!(self, reopen_year, year)
    }

    async fn check_and_close_previous_year(&self) -> Result<Option<YearSummary>, String> {
        dispatch!(self, check_and_close_previous_year)
    }

    // ── Export / Import ───────────────────────────────────────────────────────

    async fn export_members_csv(&self, member_type: &str) -> Result<String, String> {
        match self {
            DataSource::Local(r) => {
                let members = r.get_members_by_type(member_type).await.map_err(|e| e.to_string())?;
                Ok(build_csv_from_members(&members))
            }
            DataSource::Remote(c) => c.export_members_csv(member_type).await.map_err(|e| e.to_string()),
            DataSource::Unconfigured => Err(Self::not_configured()),
        }
    }

    async fn export_members_excel(&self, member_type: &str) -> Result<Vec<u8>, String> {
        match self {
            DataSource::Local(r) => {
                let members = r.get_members_by_type_with_total(member_type).await.map_err(|e| e.to_string())?;
                build_excel_bytes(&members, member_type)
            }
            DataSource::Remote(c) => c.export_members_excel(member_type).await.map_err(|e| e.to_string()),
            DataSource::Unconfigured => Err(Self::not_configured()),
        }
    }

    async fn import_members_csv(&self, csv_content: String, member_type: &str) -> Result<usize, String> {
        match self {
            DataSource::Local(r) => {
                let inputs = parse_csv_to_members(&csv_content, member_type);
                r.import_members(inputs).await.map_err(|e| e.to_string())
            }
            DataSource::Remote(c) => c.import_members_csv(csv_content, member_type).await.map_err(|e| e.to_string()),
            DataSource::Unconfigured => Err(Self::not_configured()),
        }
    }
}

// ─── AppState ──────────────────────────────────────────────────────────────────

pub struct AppState {
    pub app_data_dir: PathBuf,
    pub source: Arc<RwLock<DataSource>>,
}

// ─── Initialisation de la source ──────────────────────────────────────────────

async fn init_source(app_data_dir: &PathBuf, cfg: &AppConfig) -> Result<DataSource, String> {
    match &cfg.mode {
        AppMode::Server => {
            let db_path = app_data_dir
                .join("fjkm.db")
                .to_str()
                .ok_or("Chemin DB invalide")?
                .to_owned();
            let repo = Repository::new(&db_path).await.map_err(|e| e.to_string())?;
            let port = cfg.server_port;
            // Ne démarrer le serveur qu'une seule fois (idempotent)
            if API_SERVER_PORT.get().is_none() {
                let repo_clone = repo.clone();
                std::thread::spawn(move || {
                    tokio::runtime::Runtime::new()
                        .expect("Runtime Axum")
                        .block_on(api_server::start_server(repo_clone, port));
                });
                let _ = API_SERVER_PORT.set(port);
            }
            Ok(DataSource::Local(repo))
        }
        AppMode::Client => Ok(DataSource::Remote(RemoteClient::new(cfg.server_url()))),
    }
}

// ─── Commandes config ──────────────────────────────────────────────────────────

#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<Option<AppConfig>, String> {
    Ok(load_config(&state.app_data_dir))
}

#[tauri::command]
async fn save_config(
    state: tauri::State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    save_config_to_disk(&state.app_data_dir, &config)?;
    let new_source = init_source(&state.app_data_dir, &config).await?;
    *state.source.write().await = new_source;
    Ok(())
}

#[tauri::command]
async fn reset_config(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let config_file = config::config_path(&state.app_data_dir);
    if config_file.exists() {
        std::fs::remove_file(&config_file).map_err(|e| e.to_string())?;
    }
    *state.source.write().await = DataSource::Unconfigured;
    Ok(())
}

/// Démarre un serveur Axum local (SQLite :memory:) pour tester le mode client.
/// Idempotent : si déjà démarré, retourne le port existant.
#[tauri::command]
async fn start_mock_server() -> Result<u16, String> {
    if let Some(&port) = MOCK_SERVER_PORT.get() {
        return Ok(port);
    }
    let port = 7655u16;
    let repo = Repository::new(":memory:").await.map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        tokio::runtime::Runtime::new()
            .expect("Runtime mock")
            .block_on(api_server::start_server(repo, port));
    });
    // Attendre que le serveur soit prêt
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    let _ = MOCK_SERVER_PORT.set(port);
    Ok(port)
}

#[tauri::command]
async fn test_server_connection(ip: String, port: u16) -> Result<bool, String> {
    let url = format!("http://{ip}:{port}/api/health");
    let client = reqwest::Client::new();
    match client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(e)   => Err(e.to_string()),
    }
}

// ─── Commandes Member ──────────────────────────────────────────────────────────

#[tauri::command]
async fn get_members(state: tauri::State<'_, AppState>) -> Result<Vec<Member>, String> {
    state.source.read().await.get_members().await
}

#[tauri::command]
async fn get_members_by_type(
    state: tauri::State<'_, AppState>,
    member_type: String,
) -> Result<Vec<Member>, String> {
    state.source.read().await.get_members_by_type(&member_type).await
}

#[tauri::command]
async fn get_members_by_type_with_total(
    state: tauri::State<'_, AppState>,
    member_type: String,
) -> Result<Vec<MemberWithTotal>, String> {
    state.source.read().await.get_members_by_type_with_total(&member_type).await
}

#[tauri::command]
async fn get_member(state: tauri::State<'_, AppState>, id: i64) -> Result<Member, String> {
    state.source.read().await.get_member(id).await
}

#[tauri::command]
async fn create_member(
    state: tauri::State<'_, AppState>,
    member: MemberInput,
) -> Result<Member, String> {
    state.source.read().await.create_member(member).await
}

#[tauri::command]
async fn update_member(
    state: tauri::State<'_, AppState>,
    id: i64,
    member: MemberInput,
) -> Result<Member, String> {
    state.source.read().await.update_member(id, member).await
}

#[tauri::command]
async fn delete_member(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    state.source.read().await.delete_member(id).await
}

// ─── Commandes Contribution ────────────────────────────────────────────────────

#[tauri::command]
async fn get_contributions(
    state: tauri::State<'_, AppState>,
    member_id: i64,
) -> Result<Vec<Contribution>, String> {
    state.source.read().await.get_contributions(member_id).await
}

#[tauri::command]
async fn get_contributions_by_year(
    state: tauri::State<'_, AppState>,
    year: i32,
) -> Result<Vec<Contribution>, String> {
    state.source.read().await.get_contributions_by_year(year).await
}

#[tauri::command]
async fn create_contribution(
    state: tauri::State<'_, AppState>,
    contribution: ContributionInput,
) -> Result<Contribution, String> {
    state.source.read().await.create_contribution(contribution).await
}

#[tauri::command]
async fn delete_contribution(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    state.source.read().await.delete_contribution(id).await
}

// ─── Commandes YearSummary ────────────────────────────────────────────────────

#[tauri::command]
async fn get_year_summaries(state: tauri::State<'_, AppState>) -> Result<Vec<YearSummary>, String> {
    state.source.read().await.get_year_summaries().await
}

#[tauri::command]
async fn get_year_summary(
    state: tauri::State<'_, AppState>,
    year: i32,
) -> Result<Option<YearSummary>, String> {
    state.source.read().await.get_year_summary(year).await
}

#[tauri::command]
async fn close_year(
    state: tauri::State<'_, AppState>,
    year: i32,
    note: Option<String>,
) -> Result<YearSummary, String> {
    state.source.read().await.close_year(year, note).await
}

#[tauri::command]
async fn reopen_year(
    state: tauri::State<'_, AppState>,
    year: i32,
) -> Result<YearSummary, String> {
    state.source.read().await.reopen_year(year).await
}

#[tauri::command]
async fn transfer_members(
    state: tauri::State<'_, AppState>,
    ids: Vec<i64>,
    new_type: String,
) -> Result<usize, String> {
    state.source.read().await.transfer_members(&ids, &new_type).await
}

// ─── Commandes Archives ────────────────────────────────────────────────────────

#[tauri::command]
async fn get_contributions_by_year_with_member(
    state: tauri::State<'_, AppState>,
    year: i32,
) -> Result<Vec<ContributionWithMember>, String> {
    state
        .source
        .read()
        .await
        .get_contributions_by_year_with_member(year)
        .await
}

#[tauri::command]
async fn check_and_close_previous_year(
    state: tauri::State<'_, AppState>,
) -> Result<Option<YearSummary>, String> {
    state.source.read().await.check_and_close_previous_year().await
}

// ─── Commandes Import / Export ─────────────────────────────────────────────────

#[tauri::command]
async fn export_members_csv(
    state: tauri::State<'_, AppState>,
    member_type: String,
) -> Result<String, String> {
    state.source.read().await.export_members_csv(&member_type).await
}

#[tauri::command]
async fn export_members_excel(
    state: tauri::State<'_, AppState>,
    member_type: String,
) -> Result<Vec<u8>, String> {
    state.source.read().await.export_members_excel(&member_type).await
}

#[tauri::command]
async fn import_members_csv(
    state: tauri::State<'_, AppState>,
    csv_content: String,
    member_type: String,
) -> Result<usize, String> {
    state
        .source
        .read()
        .await
        .import_members_csv(csv_content, &member_type)
        .await
}

// ─── Commandes fenêtre ─────────────────────────────────────────────────────────

#[tauri::command]
async fn minimize_window(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Fenêtre introuvable".to_string())?
        .minimize()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_maximize(app: tauri::AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "Fenêtre introuvable".to_string())?;
    if win.is_maximized().map_err(|e| e.to_string())? {
        win.unmaximize().map_err(|e| e.to_string())
    } else {
        win.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "Fenêtre introuvable".to_string())?
        .close()
        .map_err(|e| e.to_string())
}

// ─── Point d'entrée ────────────────────────────────────────────────────────────

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Impossible d'obtenir app_data_dir");
            std::fs::create_dir_all(&app_dir).expect("Impossible de créer app_data_dir");

            let config = load_config(&app_dir);

            let source = match config {
                None => DataSource::Unconfigured,
                Some(cfg) => {
                    let rt = tokio::runtime::Runtime::new()
                        .expect("Impossible de créer le runtime Tokio");
                    match rt.block_on(init_source(&app_dir, &cfg)) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[Setup] Erreur init source: {e} — démarrage sans config");
                            DataSource::Unconfigured
                        }
                    }
                }
            };

            app.manage(AppState {
                app_data_dir: app_dir,
                source: Arc::new(RwLock::new(source)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            get_config,
            save_config,
            reset_config,
            test_server_connection,
            start_mock_server,
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
            // Import / Export
            export_members_csv,
            export_members_excel,
            import_members_csv,
            // Fenêtre
            minimize_window,
            toggle_maximize,
            close_window,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de Tauri");
}
