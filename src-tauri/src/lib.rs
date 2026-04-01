mod db;
use db::{
    Contribution, ContributionInput, ContributionWithMember, Member, MemberInput, MemberWithTotal,
    Repository, YearSummary,
};
use rust_xlsxwriter::{Color, Format, Workbook};
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

// ─── Commandes Import / Export CSV ───────────────────────────────────────────

/// Échappe un champ CSV : ajoute des guillemets si le champ contient une virgule,
/// un guillemet ou un saut de ligne. Les guillemets internes sont doublés.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse une ligne CSV en tenant compte des champs entre guillemets.
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(field.trim().to_string());
            field = String::new();
        } else {
            field.push(ch);
        }
    }
    fields.push(field.trim().to_string());
    fields
}

#[tauri::command]
async fn export_members_csv(
    state: tauri::State<'_, Repository>,
    member_type: String,
) -> Result<String, String> {
    let members = state
        .get_members_by_type(&member_type)
        .await
        .map_err(|e| e.to_string())?;

    let mut out = String::new();
    out.push_str("numero_carte,nom_complet,adresse,telephone,travail,genre\n");
    for m in &members {
        out.push_str(&csv_escape(&m.card_number));
        out.push(',');
        out.push_str(&csv_escape(&m.full_name));
        out.push(',');
        out.push_str(&csv_escape(m.address.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_escape(m.phone.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_escape(m.job.as_deref().unwrap_or("")));
        out.push(',');
        out.push_str(&csv_escape(&m.gender));
        out.push('\n');
    }
    Ok(out)
}

#[tauri::command]
async fn export_members_excel(
    state: tauri::State<'_, Repository>,
    member_type: String,
) -> Result<Vec<u8>, String> {
    let members = state
        .get_members_by_type_with_total(&member_type)
        .await
        .map_err(|e| e.to_string())?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(&member_type).map_err(|e| e.to_string())?;
    worksheet.set_freeze_panes(1, 0).map_err(|e| e.to_string())?;

    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White);

    let headers = [
        "N° Carte", "Nom Complet", "Adresse",
        "Téléphone", "Travail", "Genre", "Total Cotisations",
    ];
    for (col, &h) in headers.iter().enumerate() {
        worksheet
            .write_with_format(0, col as u16, h, &header_fmt)
            .map_err(|e| e.to_string())?;
    }

    for (row, m) in members.iter().enumerate() {
        let r = (row + 1) as u32;
        worksheet.write(r, 0, m.card_number.as_str()).map_err(|e| e.to_string())?;
        worksheet.write(r, 1, m.full_name.as_str()).map_err(|e| e.to_string())?;
        worksheet.write(r, 2, m.address.as_deref().unwrap_or("")).map_err(|e| e.to_string())?;
        worksheet.write(r, 3, m.phone.as_deref().unwrap_or("")).map_err(|e| e.to_string())?;
        worksheet.write(r, 4, m.job.as_deref().unwrap_or("")).map_err(|e| e.to_string())?;
        worksheet.write(r, 5, m.gender.as_str()).map_err(|e| e.to_string())?;
        worksheet.write(r, 6, m.total_contributions.as_str()).map_err(|e| e.to_string())?;
    }

    worksheet.autofit();

    workbook.save_to_buffer().map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_members_csv(
    state: tauri::State<'_, Repository>,
    csv_content: String,
    member_type: String,
) -> Result<usize, String> {
    let mut inputs = Vec::new();
    let mut lines = csv_content.lines();

    // Sauter l'en-tête
    if let Some(header) = lines.next() {
        let h = header.trim().to_lowercase();
        // Vérification souple : la première ligne doit ressembler à un en-tête
        if !h.contains("carte") && !h.contains("nom") {
            // Pas d'en-tête reconnu — on réintègre la ligne comme donnée
            let fields = parse_csv_line(header);
            if fields.len() >= 6 {
                inputs.push(MemberInput {
                    card_number: fields[0].clone(),
                    full_name:   fields[1].clone(),
                    address:     if fields[2].is_empty() { None } else { Some(fields[2].clone()) },
                    phone:       if fields[3].is_empty() { None } else { Some(fields[3].clone()) },
                    job:         if fields[4].is_empty() { None } else { Some(fields[4].clone()) },
                    gender:      fields[5].clone(),
                    member_type: member_type.clone(),
                });
            }
        }
    }

    for line in lines {
        let line = line.trim();
        if line.is_empty() { continue; }
        let fields = parse_csv_line(line);
        if fields.len() < 6 { continue; }
        inputs.push(MemberInput {
            card_number: fields[0].clone(),
            full_name:   fields[1].clone(),
            address:     if fields[2].is_empty() { None } else { Some(fields[2].clone()) },
            phone:       if fields[3].is_empty() { None } else { Some(fields[3].clone()) },
            job:         if fields[4].is_empty() { None } else { Some(fields[4].clone()) },
            gender:      fields[5].clone(),
            member_type: member_type.clone(),
        });
    }

    state
        .import_members(inputs)
        .await
        .map_err(|e| e.to_string())
}

// ─── Commandes fenêtre ────────────────────────────────────────────────────────

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
