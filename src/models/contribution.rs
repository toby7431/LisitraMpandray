use serde::{Deserialize, Serialize};

/// Cotisation d'un membre.
/// `amount` est une chaîne : Decimal sérialisé en string par le backend.
/// `recorded_year` est extrait automatiquement de `payment_date` côté backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contribution {
    pub id:            i64,
    pub member_id:     i64,
    /// "YYYY-MM-DD"
    pub payment_date:  String,
    /// Période libre : "2024-01", "T1-2025", etc.
    pub period:        String,
    /// Decimal sérialisé en chaîne, ex. "15000.50"
    pub amount:        String,
    pub recorded_year: i32,
}

/// Données saisies pour enregistrer une cotisation.
/// `amount` est envoyé comme chaîne ("15000.50") et validé côté backend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ContributionInput {
    pub member_id:    i64,
    /// "YYYY-MM-DD"
    pub payment_date: String,
    pub period:       String,
    /// "15000.50"
    pub amount:       String,
}
