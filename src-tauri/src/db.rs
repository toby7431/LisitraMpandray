use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

// ─── Modèles ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membre {
    pub id: i64,
    pub nom: String,
    pub prenom: String,
    pub date_naissance: Option<String>,
    pub type_membre: String, // "Communiant" | "Cathekomen"
    pub statut: String,      // "Actif" | "Archive"
    pub date_adhesion: String,
    pub telephone: Option<String>,
    pub adresse: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MembreInput {
    pub nom: String,
    pub prenom: String,
    pub date_naissance: Option<String>,
    pub type_membre: String,
    pub telephone: Option<String>,
    pub adresse: Option<String>,
    pub notes: Option<String>,
}

// ─── État partagé ───────────────────────────────────────────────────────────

pub struct DbState {
    pool: SqlitePool,
}

impl DbState {
    pub async fn new(db_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(db_url).await?;
        let state = DbState { pool };
        state.init_tables().await?;
        Ok(state)
    }

    async fn init_tables(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS membres (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                nom             TEXT NOT NULL,
                prenom          TEXT NOT NULL,
                date_naissance  TEXT,
                type_membre     TEXT NOT NULL DEFAULT 'Communiant',
                statut          TEXT NOT NULL DEFAULT 'Actif',
                date_adhesion   TEXT NOT NULL,
                telephone       TEXT,
                adresse         TEXT,
                notes           TEXT,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── CRUD ────────────────────────────────────────────────────────────────

    pub async fn get_membres(&self) -> Result<Vec<Membre>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, nom, prenom, date_naissance, type_membre, statut, \
             date_adhesion, telephone, adresse, notes \
             FROM membres ORDER BY nom ASC, prenom ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| Membre {
                id: r.get("id"),
                nom: r.get("nom"),
                prenom: r.get("prenom"),
                date_naissance: r.get("date_naissance"),
                type_membre: r.get("type_membre"),
                statut: r.get("statut"),
                date_adhesion: r.get("date_adhesion"),
                telephone: r.get("telephone"),
                adresse: r.get("adresse"),
                notes: r.get("notes"),
            })
            .collect())
    }

    pub async fn add_membre(&self, input: MembreInput) -> Result<Membre, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%d").to_string();
        let row = sqlx::query(
            r#"
            INSERT INTO membres
                (nom, prenom, date_naissance, type_membre, statut,
                 date_adhesion, telephone, adresse, notes, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'Actif', ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&input.nom)
        .bind(&input.prenom)
        .bind(&input.date_naissance)
        .bind(&input.type_membre)
        .bind(&now)
        .bind(&input.telephone)
        .bind(&input.adresse)
        .bind(&input.notes)
        .bind(&now)
        .bind(&now)
        .fetch_one(&self.pool)
        .await?;

        Ok(Membre {
            id: row.get("id"),
            nom: input.nom,
            prenom: input.prenom,
            date_naissance: input.date_naissance,
            type_membre: input.type_membre,
            statut: "Actif".into(),
            date_adhesion: now,
            telephone: input.telephone,
            adresse: input.adresse,
            notes: input.notes,
        })
    }

    pub async fn update_membre(
        &self,
        id: i64,
        input: MembreInput,
    ) -> Result<Membre, sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        sqlx::query(
            r#"
            UPDATE membres
            SET nom=?, prenom=?, date_naissance=?, type_membre=?,
                telephone=?, adresse=?, notes=?, updated_at=?
            WHERE id=?
            "#,
        )
        .bind(&input.nom)
        .bind(&input.prenom)
        .bind(&input.date_naissance)
        .bind(&input.type_membre)
        .bind(&input.telephone)
        .bind(&input.adresse)
        .bind(&input.notes)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query(
            "SELECT id, nom, prenom, date_naissance, type_membre, statut, \
             date_adhesion, telephone, adresse, notes FROM membres WHERE id=?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Membre {
            id: row.get("id"),
            nom: row.get("nom"),
            prenom: row.get("prenom"),
            date_naissance: row.get("date_naissance"),
            type_membre: row.get("type_membre"),
            statut: row.get("statut"),
            date_adhesion: row.get("date_adhesion"),
            telephone: row.get("telephone"),
            adresse: row.get("adresse"),
            notes: row.get("notes"),
        })
    }

    pub async fn archive_membre(&self, id: i64) -> Result<(), sqlx::Error> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        sqlx::query("UPDATE membres SET statut='Archive', updated_at=? WHERE id=?")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_membre(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM membres WHERE id=?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
