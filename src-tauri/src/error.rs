//! Typed failures that cross the IPC boundary (SEP-016, FR-011).
//!
//! The frontend shim turns these into `throw new Error(message)`, preserving the
//! contract `apiFetch` has today (`throw new Error(data.detail)`), so React
//! components keep their existing error handling unchanged.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Entidade desconhecida: {0}")]
    UnknownEntity(String),

    #[error("Não encontrado")]
    NotFound,

    #[error("Coluna inválida: {0}")]
    InvalidColumn(String),

    #[error("Credenciais inválidas")]
    InvalidCredentials,

    #[error("Não autenticado")]
    Unauthenticated,

    #[error("Erro de banco de dados: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Falha de migração: {0}")]
    Migration(String),

    #[error("Erro de arquivo: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Internal(String),
}

/// Wire shape seen by JavaScript: `{ kind, message }`.
#[derive(Serialize)]
struct SerializedError {
    kind: &'static str,
    message: String,
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            Self::UnknownEntity(_) => "unknown_entity",
            Self::NotFound => "not_found",
            Self::InvalidColumn(_) => "invalid_column",
            Self::InvalidCredentials => "invalid_credentials",
            Self::Unauthenticated => "unauthenticated",
            Self::Database(_) => "database",
            Self::Migration(_) => "migration",
            Self::Io(_) => "io",
            Self::Internal(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        SerializedError {
            kind: self.kind(),
            message: self.to_string(),
        }
        .serialize(s)
    }
}

/// A poisoned mutex means another thread panicked while holding the connection.
/// Surface it as a normal error instead of cascading the panic into the window.
impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        Self::Internal("estado interno corrompido (mutex envenenado)".into())
    }
}
