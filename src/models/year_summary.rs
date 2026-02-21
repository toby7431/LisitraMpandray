use serde::{Deserialize, Serialize};

/// Résumé financier d'une année.
/// `total` est recalculé automatiquement à chaque modification de contribution.
/// `closed_at` est `None` quand l'année est encore ouverte.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct YearSummary {
    pub year: i32,
    /// Decimal sérialisé en chaîne, ex. "1800000.00"
    pub total:     String,
    /// ISO datetime de clôture, ex. "2025-01-10T14:30:00", ou None si ouvert
    pub closed_at: Option<String>,
    pub note:      Option<String>,
}
