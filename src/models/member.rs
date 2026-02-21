use serde::{Deserialize, Serialize};

/// Membre de l'église — miroir du modèle backend Tauri.
/// `amount` et `total` sont des chaînes : le backend sérialise `Decimal` en string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Member {
    pub id:          i64,
    pub card_number: String,
    pub full_name:   String,
    pub address:     Option<String>,
    pub phone:       Option<String>,
    pub job:         Option<String>,
    /// "M" | "F"
    pub gender:      String,
    /// "Communiant" | "Cathekomen"
    pub member_type: String,
    pub created_at:  String,
}

/// Membre avec total des contributions (retourné par `get_members_by_type_with_total`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub total_contributions: String,
}

/// Données saisies pour créer ou modifier un membre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MemberInput {
    pub card_number: String,
    pub full_name:   String,
    pub address:     Option<String>,
    pub phone:       Option<String>,
    pub job:         Option<String>,
    pub gender:      String,
    pub member_type: String,
}
