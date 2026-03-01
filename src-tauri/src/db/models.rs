/// Modèles de données partagés entre le Repository et les commandes Tauri.
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Member ───────────────────────────────────────────────────────────────────

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

// ─── MemberWithTotal ──────────────────────────────────────────────────────────

/// Membre avec le total de toutes ses contributions (calculé par JOIN SQL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberWithTotal {
    pub id:                  i64,
    pub card_number:         String,
    pub full_name:           String,
    pub address:             Option<String>,
    pub phone:               Option<String>,
    pub job:                 Option<String>,
    pub gender:              String,
    pub member_type:         String,
    pub created_at:          String,
    /// Total en Ariary, arrondi à l'entier (ex: "15000")
    pub total_contributions: String,
}

// ─── Contribution ─────────────────────────────────────────────────────────────

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

// ─── ContributionWithMember ───────────────────────────────────────────────────

/// Cotisation avec le nom complet du membre (JOIN SQL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionWithMember {
    pub id:            i64,
    pub member_id:     i64,
    pub member_name:   String,
    pub payment_date:  String,
    pub period:        String,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount:        Decimal,
    pub recorded_year: i32,
}

// ─── YearSummary ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearSummary {
    pub year:      i32,
    #[serde(with = "rust_decimal::serde::str")]
    pub total:     Decimal,
    pub closed_at: Option<String>,
    pub note:      Option<String>,
}
