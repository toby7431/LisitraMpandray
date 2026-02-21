/// Repository SQLite — sqlx 0.7 + migrations embarquées.
///
/// Trois tables :
///   - members        : membres de l'église (card_number unique)
///   - contributions  : cotisations (recorded_year extrait automatiquement de payment_date)
///   - year_summaries : totaux annuels (recalculés à chaque insert/delete de contribution)
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    Row,
};
use std::str::FromStr;

// ─── Erreur interne ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    Validation(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Db(e)         => write!(f, "{e}"),
            AppError::Validation(s) => write!(f, "{s}"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Db(e)
    }
}

// ─── Modèle Member ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id:          i64,
    pub card_number: String,
    pub full_name:   String,
    pub address:     Option<String>,
    pub phone:       Option<String>,
    pub job:         Option<String>,
    pub gender:      String,      // "M" | "F"
    pub member_type: String,      // "Communiant" | "Cathekomen"
    pub created_at:  String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberInput {
    pub card_number: String,
    pub full_name:   String,
    pub address:     Option<String>,
    pub phone:       Option<String>,
    pub job:         Option<String>,
    pub gender:      String,
    pub member_type: String,
}

// ─── Modèle Contribution ──────────────────────────────────────────────────────

/// `amount` est sérialisé en chaîne pour la compatibilité JSON ↔ rust_decimal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub id:            i64,
    pub member_id:     i64,
    pub payment_date:  String,
    pub period:        String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount:        Decimal,
    pub recorded_year: i32,
}

/// `amount` reçu sous forme de chaîne depuis le frontend ("15000.50").
#[derive(Debug, Serialize, Deserialize)]
pub struct ContributionInput {
    pub member_id:    i64,
    pub payment_date: String,
    pub period:       String,
    pub amount:       String,
}

// ─── Modèle YearSummary ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearSummary {
    pub year:      i32,
    #[serde(with = "rust_decimal::serde::str")]
    pub total:     Decimal,
    pub closed_at: Option<String>,
    pub note:      Option<String>,
}

// ─── Repository ───────────────────────────────────────────────────────────────

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    /// Ouvre (ou crée) la base SQLite, active les FK, puis exécute les migrations.
    pub async fn new(db_path: &str) -> Result<Self, AppError> {
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{db_path}"))
            .map_err(AppError::Db)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePool::connect_with(options).await?;

        // Migrations embarquées (src-tauri/migrations/)
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| AppError::Db(sqlx::Error::from(e)))?;

        Ok(Repository { pool })
    }

    // ── Helpers privés ────────────────────────────────────────────────────────

    fn map_member(r: &sqlx::sqlite::SqliteRow) -> Member {
        Member {
            id:          r.get("id"),
            card_number: r.get("card_number"),
            full_name:   r.get("full_name"),
            address:     r.get("address"),
            phone:       r.get("phone"),
            job:         r.get("job"),
            gender:      r.get("gender"),
            member_type: r.get("member_type"),
            created_at:  r.get("created_at"),
        }
    }

    fn map_contribution(r: &sqlx::sqlite::SqliteRow) -> Contribution {
        let amount_str: String = r.get("amount");
        Contribution {
            id:            r.get("id"),
            member_id:     r.get("member_id"),
            payment_date:  r.get("payment_date"),
            period:        r.get("period"),
            amount:        Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO),
            recorded_year: r.get("recorded_year"),
        }
    }

    fn map_year_summary(r: &sqlx::sqlite::SqliteRow) -> YearSummary {
        let total_str: String = r.get("total");
        YearSummary {
            year:      r.get("year"),
            total:     Decimal::from_str(&total_str).unwrap_or(Decimal::ZERO),
            closed_at: r.get("closed_at"),
            note:      r.get("note"),
        }
    }

    /// Recalcule le total d'une année depuis les contributions, puis fait un UPSERT.
    async fn refresh_year_total(&self, year: i32) -> Result<(), AppError> {
        let rows = sqlx::query(
            "SELECT amount FROM contributions WHERE recorded_year = ?",
        )
        .bind(year)
        .fetch_all(&self.pool)
        .await?;

        let total: Decimal = rows
            .iter()
            .filter_map(|r| {
                let s: String = r.get("amount");
                Decimal::from_str(&s).ok()
            })
            .fold(Decimal::ZERO, |acc, d| acc + d);

        sqlx::query(
            "INSERT INTO year_summaries (year, total)
             VALUES (?, ?)
             ON CONFLICT(year) DO UPDATE SET total = excluded.total",
        )
        .bind(year)
        .bind(total.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ── Member CRUD ───────────────────────────────────────────────────────────

    pub async fn get_members(&self) -> Result<Vec<Member>, AppError> {
        let rows = sqlx::query(
            "SELECT id, card_number, full_name, address, phone, job,
                    gender, member_type, created_at
             FROM members
             ORDER BY full_name ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(Self::map_member).collect())
    }

    pub async fn get_members_by_type(&self, member_type: &str) -> Result<Vec<Member>, AppError> {
        let rows = sqlx::query(
            "SELECT id, card_number, full_name, address, phone, job,
                    gender, member_type, created_at
             FROM members
             WHERE member_type = ?
             ORDER BY full_name ASC",
        )
        .bind(member_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(Self::map_member).collect())
    }

    pub async fn get_member(&self, id: i64) -> Result<Member, AppError> {
        let row = sqlx::query(
            "SELECT id, card_number, full_name, address, phone, job,
                    gender, member_type, created_at
             FROM members
             WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_member(&row))
    }

    pub async fn create_member(&self, input: MemberInput) -> Result<Member, AppError> {
        if input.card_number.trim().is_empty() {
            return Err(AppError::Validation("Le numéro de carte est requis.".into()));
        }
        if input.full_name.trim().is_empty() {
            return Err(AppError::Validation("Le nom complet est requis.".into()));
        }

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        let row = sqlx::query(
            "INSERT INTO members
                 (card_number, full_name, address, phone, job, gender, member_type, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(&input.card_number)
        .bind(&input.full_name)
        .bind(&input.address)
        .bind(&input.phone)
        .bind(&input.job)
        .bind(&input.gender)
        .bind(&input.member_type)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Member {
            id:          row.get("id"),
            card_number: input.card_number,
            full_name:   input.full_name,
            address:     input.address,
            phone:       input.phone,
            job:         input.job,
            gender:      input.gender,
            member_type: input.member_type,
            created_at:  now,
        })
    }

    pub async fn update_member(&self, id: i64, input: MemberInput) -> Result<Member, AppError> {
        if input.card_number.trim().is_empty() {
            return Err(AppError::Validation("Le numéro de carte est requis.".into()));
        }
        if input.full_name.trim().is_empty() {
            return Err(AppError::Validation("Le nom complet est requis.".into()));
        }

        sqlx::query(
            "UPDATE members
             SET card_number = ?, full_name = ?, address = ?, phone = ?,
                 job = ?, gender = ?, member_type = ?
             WHERE id = ?",
        )
        .bind(&input.card_number)
        .bind(&input.full_name)
        .bind(&input.address)
        .bind(&input.phone)
        .bind(&input.job)
        .bind(&input.gender)
        .bind(&input.member_type)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.get_member(id).await
    }

    pub async fn delete_member(&self, id: i64) -> Result<(), AppError> {
        // Les contributions liées sont supprimées en cascade (FK ON DELETE CASCADE)
        sqlx::query("DELETE FROM members WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // ── Contribution CRUD ─────────────────────────────────────────────────────

    pub async fn get_contributions(&self, member_id: i64) -> Result<Vec<Contribution>, AppError> {
        let rows = sqlx::query(
            "SELECT id, member_id, payment_date, period, amount, recorded_year
             FROM contributions
             WHERE member_id = ?
             ORDER BY payment_date DESC",
        )
        .bind(member_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(Self::map_contribution).collect())
    }

    pub async fn get_contributions_by_year(
        &self,
        year: i32,
    ) -> Result<Vec<Contribution>, AppError> {
        let rows = sqlx::query(
            "SELECT id, member_id, payment_date, period, amount, recorded_year
             FROM contributions
             WHERE recorded_year = ?
             ORDER BY payment_date DESC",
        )
        .bind(year)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(Self::map_contribution).collect())
    }

    pub async fn create_contribution(
        &self,
        input: ContributionInput,
    ) -> Result<Contribution, AppError> {
        // Valider et parser le montant
        let amount = Decimal::from_str(input.amount.trim())
            .map_err(|_| AppError::Validation(
                format!("Montant invalide : '{}'. Utilisez le format '15000.50'.", input.amount),
            ))?;

        if amount < Decimal::ZERO {
            return Err(AppError::Validation("Le montant ne peut pas être négatif.".into()));
        }

        // Extraire l'année — recorded_year est automatique
        let recorded_year = NaiveDate::parse_from_str(&input.payment_date, "%Y-%m-%d")
            .map(|d| d.year())
            .map_err(|_| AppError::Validation(
                format!(
                    "Date de paiement invalide : '{}'. Format attendu : YYYY-MM-DD.",
                    input.payment_date
                ),
            ))?;

        let row = sqlx::query(
            "INSERT INTO contributions (member_id, payment_date, period, amount, recorded_year)
             VALUES (?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(input.member_id)
        .bind(&input.payment_date)
        .bind(&input.period)
        .bind(amount.to_string())
        .bind(recorded_year)
        .fetch_one(&self.pool)
        .await?;

        // Mise à jour automatique du résumé annuel
        self.refresh_year_total(recorded_year).await?;

        Ok(Contribution {
            id:            row.get("id"),
            member_id:     input.member_id,
            payment_date:  input.payment_date,
            period:        input.period,
            amount,
            recorded_year,
        })
    }

    pub async fn delete_contribution(&self, id: i64) -> Result<(), AppError> {
        // Récupérer l'année avant suppression pour mettre à jour le total
        let row = sqlx::query("SELECT recorded_year FROM contributions WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        let year: i32 = row.get("recorded_year");

        sqlx::query("DELETE FROM contributions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        // Recalcul du total annuel
        self.refresh_year_total(year).await?;

        Ok(())
    }

    // ── YearSummary ───────────────────────────────────────────────────────────

    pub async fn get_year_summaries(&self) -> Result<Vec<YearSummary>, AppError> {
        let rows = sqlx::query(
            "SELECT year, total, closed_at, note
             FROM year_summaries
             ORDER BY year DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(Self::map_year_summary).collect())
    }

    pub async fn get_year_summary(&self, year: i32) -> Result<Option<YearSummary>, AppError> {
        let row = sqlx::query(
            "SELECT year, total, closed_at, note FROM year_summaries WHERE year = ?",
        )
        .bind(year)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.as_ref().map(Self::map_year_summary))
    }

    /// Clôture une année : enregistre closed_at + note.
    /// Crée le résumé s'il n'existe pas encore.
    pub async fn close_year(
        &self,
        year: i32,
        note: Option<String>,
    ) -> Result<YearSummary, AppError> {
        self.refresh_year_total(year).await?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        sqlx::query(
            "UPDATE year_summaries SET closed_at = ?, note = ? WHERE year = ?",
        )
        .bind(&now)
        .bind(&note)
        .bind(year)
        .execute(&self.pool)
        .await?;

        self.get_year_summary(year)
            .await?
            .ok_or_else(|| AppError::Validation(format!("Résumé pour {year} introuvable.")))
    }

    /// Réouvre une année clôturée (supprime closed_at + note).
    pub async fn reopen_year(&self, year: i32) -> Result<YearSummary, AppError> {
        sqlx::query(
            "UPDATE year_summaries SET closed_at = NULL, note = NULL WHERE year = ?",
        )
        .bind(year)
        .execute(&self.pool)
        .await?;

        self.get_year_summary(year)
            .await?
            .ok_or_else(|| AppError::Validation(format!("Résumé pour {year} introuvable.")))
    }
}
