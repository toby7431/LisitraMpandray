/// Fonctions partagées d'export/import CSV et Excel.
use rust_xlsxwriter::{Color, Format, Workbook};

use crate::db::{MemberInput, MemberWithTotal};

// ── CSV ───────────────────────────────────────────────────────────────────────

/// Échappe un champ CSV.
pub fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse une ligne CSV en tenant compte des champs entre guillemets.
pub fn parse_csv_line(line: &str) -> Vec<String> {
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

/// Construit une chaîne CSV à partir d'une liste de membres avec totaux.
pub fn build_csv_from_members(members: &[crate::db::Member]) -> String {
    let mut out = String::new();
    out.push_str("numero_carte,nom_complet,adresse,telephone,travail,genre\n");
    for m in members {
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
    out
}

/// Construit les bytes Excel à partir d'une liste de membres avec totaux.
pub fn build_excel_bytes(members: &[MemberWithTotal], sheet_name: &str) -> Result<Vec<u8>, String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(sheet_name).map_err(|e| e.to_string())?;
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

/// Parse le CSV importé en liste de MemberInput.
pub fn parse_csv_to_members(csv_content: &str, member_type: &str) -> Vec<MemberInput> {
    let mut inputs = Vec::new();
    let mut lines = csv_content.lines();

    if let Some(header) = lines.next() {
        let h = header.trim().to_lowercase();
        if !h.contains("carte") && !h.contains("nom") {
            let fields = parse_csv_line(header);
            if fields.len() >= 6 {
                inputs.push(make_member_input(&fields, member_type));
            }
        }
    }

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_line(line);
        if fields.len() < 6 {
            continue;
        }
        inputs.push(make_member_input(&fields, member_type));
    }

    inputs
}

fn make_member_input(fields: &[String], member_type: &str) -> MemberInput {
    MemberInput {
        card_number: fields[0].clone(),
        full_name:   fields[1].clone(),
        address:     if fields[2].is_empty() { None } else { Some(fields[2].clone()) },
        phone:       if fields[3].is_empty() { None } else { Some(fields[3].clone()) },
        job:         if fields[4].is_empty() { None } else { Some(fields[4].clone()) },
        gender:      fields[5].clone(),
        member_type: member_type.to_string(),
    }
}
