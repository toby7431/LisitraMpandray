use serde::{Deserialize, Serialize};

/// Représentation d'un membre de l'église côté frontend (WASM).
/// Miroir de la struct Tauri backend — sérialisé via JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Membre {
    pub id: i64,
    pub nom: String,
    pub prenom: String,
    pub date_naissance: Option<String>,
    /// "Communiant" | "Cathekomen"
    pub type_membre: String,
    /// "Actif" | "Archive"
    pub statut: String,
    pub date_adhesion: String,
    pub telephone: Option<String>,
    pub adresse: Option<String>,
    pub notes: Option<String>,
}

/// Données d'entrée pour créer ou modifier un membre.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MembreInput {
    pub nom: String,
    pub prenom: String,
    pub date_naissance: Option<String>,
    pub type_membre: String,
    pub telephone: Option<String>,
    pub adresse: Option<String>,
    pub notes: Option<String>,
}

/// Type de membre (enum utilitaire frontend).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TypeMembre {
    Communiant,
    Cathekomen,
}

impl TypeMembre {
    pub fn as_str(self) -> &'static str {
        match self {
            TypeMembre::Communiant => "Communiant",
            TypeMembre::Cathekomen => "Cathekomen",
        }
    }
}
