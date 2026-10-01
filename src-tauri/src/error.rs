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
    Validation(String),

    /// SEP-033 (FR-013): a request would leave dangling references — e.g.
    /// deleting a ação still referenced as "Ação vinculante" — and the caller
    /// must decide (message lists the children).
    #[error("{0}")]
    Conflict(String),

    /// SEP-034: falha de rede (DNS, timeout, conexão recusada). A mensagem é
    /// acionável e nunca contém o token (FR-006/FR-007).
    #[error("{0}")]
    Network(String),

    /// SEP-034: o GitHub respondeu um erro. `status == 404` serializa como
    /// kind `not_found` (contrato contracts/ipc-commands.md); os demais como
    /// `github_api`. `hint` é o que o curador deve fazer a seguir.
    #[error("{hint}")]
    GithubApi { status: u16, hint: String },

    /// SEP-034: arquivo acima do limite desta entrega (Q3: 100 MB).
    #[error("Arquivo acima do limite de {limit_bytes} bytes aceito nesta versão.")]
    TooLarge { limit_bytes: usize },

    /// SEP-034: configuração de sincronização ausente/inválida.
    #[error("{0}")]
    SyncConfig(String),

    /// SEP-034 (FR-005): recusa de política — hoje, o envio do consolidado SRC.
    #[error("{0}")]
    SyncPolicy(String),

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
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::UnknownEntity(_) => "unknown_entity",
            Self::NotFound => "not_found",
            Self::InvalidColumn(_) => "invalid_column",
            Self::InvalidCredentials => "invalid_credentials",
            Self::Unauthenticated => "unauthenticated",
            Self::Database(_) => "database",
            Self::Migration(_) => "migration",
            Self::Io(_) => "io",
            Self::Validation(_) => "validation",
            Self::Conflict(_) => "conflict",
            Self::Network(_) => "network",
            // 404 do GitHub é "não encontrado" para o curador (branch/arquivo);
            // os demais status ficam como github_api com a dica acionável.
            Self::GithubApi { status, .. } if *status == 404 => "not_found",
            Self::GithubApi { .. } => "github_api",
            Self::TooLarge { .. } => "too_large",
            Self::SyncConfig(_) => "sync_config",
            Self::SyncPolicy(_) => "sync_policy",
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

#[cfg(test)]
mod tests {
    use super::*;

    /// SEP-034 / T003: os kinds novos atravessam o IPC como `{ kind, message }`
    /// e 404 do GitHub aparece para o curador como `not_found`.
    #[test]
    fn kinds_de_sincronizacao_batem_com_o_contrato() {
        assert_eq!(AppError::Network("sem rede".into()).kind(), "network");
        assert_eq!(
            AppError::GithubApi { status: 500, hint: "boom".into() }.kind(),
            "github_api"
        );
        assert_eq!(
            AppError::GithubApi { status: 404, hint: "branch inexistente".into() }.kind(),
            "not_found"
        );
        assert_eq!(AppError::TooLarge { limit_bytes: 1 }.kind(), "too_large");
        assert_eq!(AppError::SyncConfig("sem destino".into()).kind(), "sync_config");
        assert_eq!(AppError::SyncPolicy("src não sobe".into()).kind(), "sync_policy");
    }

    /// FR-007: mensagens dizem o que houve E o que fazer. FR-006: nunca o token.
    #[test]
    fn mensagens_sao_acionaveis() {
        let e = AppError::GithubApi {
            status: 403,
            hint: "sem permissão de escrita; nada foi gravado".into(),
        };
        assert!(e.to_string().contains("nada foi gravado"));

        let e = AppError::TooLarge { limit_bytes: 100 * 1024 * 1024 };
        assert!(e.to_string().contains("limite"));
    }
}
