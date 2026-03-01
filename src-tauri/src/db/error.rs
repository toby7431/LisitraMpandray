/// Type d'erreur interne du Repository.
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
