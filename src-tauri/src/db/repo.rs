/// Repository SQLite — sqlx 0.7 + migrations embarquées.
///
/// Trois tables :
///   - members        : membres de l'église (card_number unique)
///   - contributions  : cotisations (recorded_year extrait automatiquement de payment_date)
///   - year_summaries : totaux annuels (recalculés à chaque insert/delete de contribution)
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    Row,
};
use std::str::FromStr;

use super::{
    error::AppError,
    models::{
        Contribution, ContributionInput, ContributionWithMember,
        Member, MemberInput, MemberWithTotal, YearSummary,
    },
};

pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    /// Ouvre (ou crée) la base SQLite, active les FK, puis exécute les migrations.
    pub async fn new(db_path: &str) -> Result<Self, AppError> {
        // `filename()` prend un chemin OS (backslashes Windows OK, espaces OK).
        // `from_str("sqlite://:memory:")` est conservé pour les tests en mémoire.
        let base = if db_path == ":memory:" {
            SqliteConnectOptions::from_str("sqlite://:memory:").map_err(AppError::Db)?
        } else {
            SqliteConnectOptions::new().filename(db_path)
        };
        let options = base
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

    /// Formate un Decimal en chaîne lisible "1 234 567 Ariary" (partie entière seulement).
    fn format_ariary_note(total: &Decimal) -> String {
        let n = total.to_string();
        let integer_part = n.split('.').next().unwrap_or("0");
        let len = integer_part.len();
        let mut result = String::new();
        for (i, c) in integer_part.chars().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                result.push(' ');
            }
            result.push(c);
        }
        format!("{} Ariary", result)
    }

    /// Variante transactionnelle de `refresh_year_total` — exécutée dans une tx ouverte.
    /// Garantit que SELECT contributions + UPSERT year_summaries sont atomiques.
    ///
    /// `tx` est `&mut Transaction<'_, Sqlite>` ; pour obtenir `&mut SqliteConnection`
    /// (seul type implémentant `Executor`), on double-déréférence : `&mut **tx`.
    async fn refresh_year_total_tx(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        year: i32,
    ) -> Result<(), AppError> {
        let rows = sqlx::query(
            "SELECT amount FROM contributions WHERE recorded_year = ?",
        )
        .bind(year)
        .fetch_all(&mut **tx)
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
        .execute(&mut **tx)
        .await?;

        Ok(())
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

    pub async fn get_members_by_type_with_total(
        &self,
        member_type: &str,
    ) -> Result<Vec<MemberWithTotal>, AppError> {
        let rows = sqlx::query(
            "SELECT m.id, m.card_number, m.full_name, m.address, m.phone, m.job,
                    m.gender, m.member_type, m.created_at,
                    COALESCE(SUM(CAST(c.amount AS REAL)), 0.0) AS total_contributions
             FROM members m
             LEFT JOIN contributions c ON c.member_id = m.id
             WHERE m.member_type = ?
             GROUP BY m.id
             ORDER BY m.full_name ASC",
        )
        .bind(member_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| {
                let total: f64 = r.get("total_contributions");
                MemberWithTotal {
                    id:                  r.get("id"),
                    card_number:         r.get("card_number"),
                    full_name:           r.get("full_name"),
                    address:             r.get("address"),
                    phone:               r.get("phone"),
                    job:                 r.get("job"),
                    gender:              r.get("gender"),
                    member_type:         r.get("member_type"),
                    created_at:          r.get("created_at"),
                    total_contributions: format!("{:.0}", total),
                }
            })
            .collect())
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

    /// Transfère plusieurs membres vers un nouveau type (ex: "Cathekomen" → "Communiant").
    /// Les contributions restent liées à leurs IDs — aucune perte de données.
    pub async fn transfer_members(
        &self,
        ids: &[i64],
        new_type: &str,
    ) -> Result<usize, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        if new_type != "Communiant" && new_type != "Cathekomen" {
            return Err(AppError::Validation(
                format!("Type de membre invalide : '{new_type}'. Valeurs acceptées : 'Communiant', 'Cathekomen'."),
            ));
        }
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "UPDATE members SET member_type = ? WHERE id IN ({})",
            placeholders
        );
        let mut q = sqlx::query(&sql).bind(new_type);
        for id in ids {
            q = q.bind(*id);
        }
        let result = q.execute(&self.pool).await?;
        Ok(result.rows_affected() as usize)
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

    /// Cotisations d'une année avec le nom du membre (JOIN).
    /// Triées par date ASC (la plus ancienne en tête) — cohérent avec l'affichage archives.
    pub async fn get_contributions_by_year_with_member(
        &self,
        year: i32,
    ) -> Result<Vec<ContributionWithMember>, AppError> {
        let rows = sqlx::query(
            "SELECT c.id, c.member_id, m.full_name AS member_name,
                    c.payment_date, c.period, c.amount, c.recorded_year
             FROM contributions c
             JOIN members m ON m.id = c.member_id
             WHERE c.recorded_year = ?
             ORDER BY c.payment_date ASC",
        )
        .bind(year)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| {
                let amount_str: String = r.get("amount");
                ContributionWithMember {
                    id:            r.get("id"),
                    member_id:     r.get("member_id"),
                    member_name:   r.get("member_name"),
                    payment_date:  r.get("payment_date"),
                    period:        r.get("period"),
                    amount:        Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO),
                    recorded_year: r.get("recorded_year"),
                }
            })
            .collect())
    }

    /// Vérifie si l'année précédente est déjà clôturée.
    /// Si non → calcule le total, génère une note et clôture automatiquement.
    /// Retourne `Some(YearSummary)` si une clôture vient d'être effectuée, `None` sinon.
    pub async fn check_and_close_previous_year(&self) -> Result<Option<YearSummary>, AppError> {
        let prev_year = chrono::Utc::now().year() - 1;

        // Déjà clôturé → rien à faire
        if let Some(existing) = self.get_year_summary(prev_year).await? {
            if existing.closed_at.is_some() {
                return Ok(None);
            }
        }

        // S'assurer que le résumé existe (même à 0) + recalculer le total
        self.refresh_year_total(prev_year).await?;

        let total = self
            .get_year_summary(prev_year)
            .await?
            .map(|s| s.total)
            .unwrap_or(Decimal::ZERO);

        let note = format!(
            "CONTRIBUTIONS de l'année {} / TOTAL : {}",
            prev_year,
            Self::format_ariary_note(&total)
        );

        let closed = self.close_year(prev_year, Some(note)).await?;
        Ok(Some(closed))
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

        // Transaction : INSERT + refresh_year_total sont atomiques.
        let mut tx = self.pool.begin().await?;

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
        .fetch_one(&mut *tx)
        .await?;

        Self::refresh_year_total_tx(&mut tx, recorded_year).await?;

        tx.commit().await?;

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
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query("SELECT recorded_year FROM contributions WHERE id = ?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let year: i32 = row.get("recorded_year");

        sqlx::query("DELETE FROM contributions WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        Self::refresh_year_total_tx(&mut tx, year).await?;

        tx.commit().await?;

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
    /// Tout est atomique : refresh_year_total + UPDATE closed_at + lecture finale.
    pub async fn close_year(
        &self,
        year: i32,
        note: Option<String>,
    ) -> Result<YearSummary, AppError> {
        let mut tx = self.pool.begin().await?;

        Self::refresh_year_total_tx(&mut tx, year).await?;

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        sqlx::query(
            "UPDATE year_summaries SET closed_at = ?, note = ? WHERE year = ?",
        )
        .bind(&now)
        .bind(&note)
        .bind(year)
        .execute(&mut *tx)
        .await?;

        // Lire l'état final dans la même transaction
        let row = sqlx::query(
            "SELECT year, total, closed_at, note FROM year_summaries WHERE year = ?",
        )
        .bind(year)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        row.as_ref()
            .map(Self::map_year_summary)
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Crée une DB SQLite en mémoire avec migrations appliquées.
    async fn make_repo() -> Repository {
        Repository::new(":memory:").await.expect("DB en mémoire")
    }

    fn member_input(card: &str, name: &str, mtype: &str) -> MemberInput {
        MemberInput {
            card_number: card.into(),
            full_name:   name.into(),
            address:     None,
            phone:       None,
            job:         None,
            gender:      "M".into(),
            member_type: mtype.into(),
        }
    }

    fn contribution_input(member_id: i64, date: &str, period: &str, amount: &str) -> ContributionInput {
        ContributionInput {
            member_id,
            payment_date: date.into(),
            period:       period.into(),
            amount:       amount.into(),
        }
    }

    // ── Membres ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_member_ok() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Jean Dupont", "Communiant")).await.unwrap();
        assert_eq!(m.card_number, "C001");
        assert_eq!(m.full_name, "Jean Dupont");
        assert_eq!(m.member_type, "Communiant");
        assert!(m.id > 0);
    }

    #[tokio::test]
    async fn test_create_member_carte_vide() {
        let repo = make_repo().await;
        let err = repo.create_member(member_input("", "Jean", "Communiant")).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_member_nom_vide() {
        let repo = make_repo().await;
        let err = repo.create_member(member_input("C001", "  ", "Communiant")).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_member_carte_duplicate() {
        let repo = make_repo().await;
        repo.create_member(member_input("C001", "Jean", "Communiant")).await.unwrap();
        let err = repo.create_member(member_input("C001", "Pierre", "Communiant")).await.unwrap_err();
        assert!(matches!(err, AppError::Db(_)));
    }

    #[tokio::test]
    async fn test_get_members_vide() {
        let repo = make_repo().await;
        let list = repo.get_members().await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_get_members() {
        let repo = make_repo().await;
        repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_member(member_input("C002", "Bob", "Cathekomen")).await.unwrap();
        let list = repo.get_members().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_get_members_by_type() {
        let repo = make_repo().await;
        repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_member(member_input("C002", "Bob", "Cathekomen")).await.unwrap();
        repo.create_member(member_input("C003", "Carol", "Communiant")).await.unwrap();

        let comm = repo.get_members_by_type("Communiant").await.unwrap();
        assert_eq!(comm.len(), 2);

        let cath = repo.get_members_by_type("Cathekomen").await.unwrap();
        assert_eq!(cath.len(), 1);
    }

    #[tokio::test]
    async fn test_update_member() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let updated = repo.update_member(m.id, member_input("C001-U", "Alice Martin", "Communiant")).await.unwrap();
        assert_eq!(updated.card_number, "C001-U");
        assert_eq!(updated.full_name, "Alice Martin");
    }

    #[tokio::test]
    async fn test_delete_member_cascade() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2024-03-01", "2024", "5000")).await.unwrap();
        repo.delete_member(m.id).await.unwrap();
        let list = repo.get_members().await.unwrap();
        assert!(list.is_empty());
        let contribs = repo.get_contributions(m.id).await.unwrap();
        assert!(contribs.is_empty());
    }

    #[tokio::test]
    async fn test_transfer_members() {
        let repo = make_repo().await;
        let m1 = repo.create_member(member_input("C001", "Alice", "Cathekomen")).await.unwrap();
        let m2 = repo.create_member(member_input("C002", "Bob", "Cathekomen")).await.unwrap();
        let n = repo.transfer_members(&[m1.id, m2.id], "Communiant").await.unwrap();
        assert_eq!(n, 2);
        let comm = repo.get_members_by_type("Communiant").await.unwrap();
        assert_eq!(comm.len(), 2);
        let cath = repo.get_members_by_type("Cathekomen").await.unwrap();
        assert!(cath.is_empty());
    }

    #[tokio::test]
    async fn test_transfer_ids_vides() {
        let repo = make_repo().await;
        let n = repo.transfer_members(&[], "Communiant").await.unwrap();
        assert_eq!(n, 0);
    }

    // ── Total contributions membre ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_total_contributions_zero() {
        let repo = make_repo().await;
        repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let list = repo.get_members_by_type_with_total("Communiant").await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].total_contributions, "0");
    }

    #[tokio::test]
    async fn test_total_contributions_somme() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2024-01-15", "2024", "10000")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2024-06-01", "2024", "5000.50")).await.unwrap();
        let list = repo.get_members_by_type_with_total("Communiant").await.unwrap();
        let total: f64 = list[0].total_contributions.parse().unwrap();
        assert!((total - 15000.0).abs() < 2.0);
    }

    // ── Contributions ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_create_contribution_ok() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let c = repo.create_contribution(contribution_input(m.id, "2024-03-15", "2024", "12000")).await.unwrap();
        assert_eq!(c.member_id, m.id);
        assert_eq!(c.period, "2024");
        assert_eq!(c.recorded_year, 2024);
        assert_eq!(c.amount.to_string(), "12000");
    }

    #[tokio::test]
    async fn test_create_contribution_montant_invalide() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let err = repo.create_contribution(contribution_input(m.id, "2024-03-15", "2024", "abc")).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_contribution_montant_negatif() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let err = repo.create_contribution(contribution_input(m.id, "2024-03-15", "2024", "-500")).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_create_contribution_date_invalide() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let err = repo.create_contribution(contribution_input(m.id, "15-03-2024", "2024", "1000")).await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[tokio::test]
    async fn test_delete_contribution_recalcule_total() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        let c1 = repo.create_contribution(contribution_input(m.id, "2024-01-01", "2024", "10000")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2024-06-01", "2024", "5000")).await.unwrap();

        let s = repo.get_year_summary(2024).await.unwrap().unwrap();
        assert_eq!(s.total, Decimal::from_str("15000").unwrap());

        repo.delete_contribution(c1.id).await.unwrap();
        let s2 = repo.get_year_summary(2024).await.unwrap().unwrap();
        assert_eq!(s2.total, Decimal::from_str("5000").unwrap());
    }

    #[tokio::test]
    async fn test_get_contributions_by_year_with_member() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice Rakoto", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2024-04-10", "2024", "8000")).await.unwrap();
        let list = repo.get_contributions_by_year_with_member(2024).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].member_name, "Alice Rakoto");
        assert_eq!(list[0].recorded_year, 2024);
    }

    // ── Résumés annuels ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_year_summary_auto_cree() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2023-05-01", "2023", "20000")).await.unwrap();
        let s = repo.get_year_summary(2023).await.unwrap().unwrap();
        assert_eq!(s.year, 2023);
        assert_eq!(s.total, Decimal::from_str("20000").unwrap());
        assert!(s.closed_at.is_none());
    }

    #[tokio::test]
    async fn test_close_and_reopen_year() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2022-01-01", "2022", "50000")).await.unwrap();

        let closed = repo.close_year(2022, Some("Test note".into())).await.unwrap();
        assert!(closed.closed_at.is_some());
        assert_eq!(closed.note.as_deref(), Some("Test note"));

        let reopened = repo.reopen_year(2022).await.unwrap();
        assert!(reopened.closed_at.is_none());
        assert!(reopened.note.is_none());
    }

    #[tokio::test]
    async fn test_close_year_sans_contributions() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2021-01-01", "2021", "0")).await.unwrap();
        let closed = repo.close_year(2021, None).await.unwrap();
        assert!(closed.closed_at.is_some());
    }

    #[tokio::test]
    async fn test_get_year_summaries_ordre_desc() {
        let repo = make_repo().await;
        let m = repo.create_member(member_input("C001", "Alice", "Communiant")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2021-01-01", "2021", "1000")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2023-01-01", "2023", "2000")).await.unwrap();
        repo.create_contribution(contribution_input(m.id, "2022-01-01", "2022", "3000")).await.unwrap();
        let list = repo.get_year_summaries().await.unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].year, 2023);
        assert_eq!(list[1].year, 2022);
        assert_eq!(list[2].year, 2021);
    }

    #[tokio::test]
    async fn test_format_ariary_note() {
        let d = Decimal::from_str("1234567").unwrap();
        assert_eq!(Repository::format_ariary_note(&d), "1 234 567 Ariary");
        let z = Decimal::ZERO;
        assert_eq!(Repository::format_ariary_note(&z), "0 Ariary");
    }
}
