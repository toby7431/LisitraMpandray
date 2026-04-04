/// Type d'erreur interne du Repository.
#[derive(Debug)]
pub enum AppError {
    /// Erreur SQLite — loggée à la conversion, jamais exposée au frontend.
    Db,
    Validation(String),
    /// Erreur réseau (mode client HTTP).
    Network(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Db            => write!(f, "Erreur interne de la base de données."),
            AppError::Validation(s) => write!(f, "{s}"),
            AppError::Network(s)    => write!(f, "Erreur réseau : {s}"),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.message().contains("UNIQUE constraint failed") {
                return AppError::Validation(
                    "Ity laharana karatra ity dia efa misy. Mifidiana laharana hafa.".into(),
                );
            }
        }
        eprintln!("[DB Error] {e}");
        AppError::Db
    }
}
