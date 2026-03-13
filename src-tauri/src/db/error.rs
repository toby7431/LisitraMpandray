/// Type d'erreur interne du Repository.
#[derive(Debug)]
pub enum AppError {
    /**
    Erreur SQLite — loggée à la conversion, jamais exposée au frontend.
    */
    Db,
    Validation(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Db            => write!(f, "Erreur interne de la base de données."),
            AppError::Validation(s) => write!(f, "{s}"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        // Log côté Rust uniquement — le détail SQLite ne remonte jamais au frontend.
        eprintln!("[DB Error] {e}");
        AppError::Db
    }
}
