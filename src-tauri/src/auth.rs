//! Credential verification (SEP-017 partial; session gate lands in SEP-021).
//!
//! Pulled forward deliberately: without it the login screen cannot be passed and
//! no CRUD work is reachable. Only the check is here — commands are still
//! ungated, matching today's system where no endpoint validates the token.

use crate::error::AppError;
use rusqlite::Connection;

/// Verify against the `admins` table. Unifies the two divergent Python paths,
/// which look up by `email` in memory and by `username` in SQL.
pub fn verify_credentials(conn: &Connection, username: &str, password: &str) -> Result<(), AppError> {
    let normalized = username.trim().to_lowercase();
    let hash: Option<String> = conn
        .query_row(
            "SELECT hashed_password FROM admins WHERE LOWER(username) = ?1",
            [&normalized],
            |r| r.get(0),
        )
        .ok();

    // Compare even when the user is unknown, so a wrong username and a wrong
    // password take the same time and cannot be told apart by timing.
    let stored = hash.unwrap_or_else(|| {
        "$2b$12$0000000000000000000000000000000000000000000000000000".to_string()
    });

    match bcrypt::verify(password, &stored) {
        Ok(true) => Ok(()),
        _ => Err(AppError::InvalidCredentials),
    }
}

/// Validate email format simply without heavy dependencies.
fn is_valid_email(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.len() < 2 {
        return false;
    }
    domain_parts.iter().all(|part| !part.is_empty())
}

/// Register a new admin account with email and password, hashing the password
/// securely with bcrypt before persisting to SQLite.
pub fn register_admin(
    conn: &Connection,
    email: &str,
    password: &str,
    password_confirm: &str,
) -> Result<(), AppError> {
    let normalized_email = email.trim().to_lowercase();

    if !is_valid_email(&normalized_email) {
        return Err(AppError::Validation("Formato de email inválido.".into()));
    }

    if password != password_confirm {
        return Err(AppError::Validation(
            "A confirmação de senha não coincide com a senha digitada.".into(),
        ));
    }

    if password.trim().is_empty() || password.len() < 8 {
        return Err(AppError::Validation(
            "A senha deve conter no mínimo 8 caracteres.".into(),
        ));
    }

    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM admins WHERE LOWER(username) = ?1",
        [&normalized_email],
        |r| r.get(0),
    )?;

    if existing > 0 {
        return Err(AppError::Validation(
            "Este email já está cadastrado.".into(),
        ));
    }

    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("falha ao gerar hash: {e}")))?;

    conn.execute(
        "INSERT INTO admins (username, hashed_password) VALUES (?1, ?2)",
        [&normalized_email, &hashed],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn conn() -> Connection {
        let c = db::open_in_memory().unwrap();
        db::seed_admin(&c).unwrap();
        c
    }

    #[test]
    fn accepts_the_seeded_admin() {
        assert!(verify_credentials(&conn(), db::DEFAULT_ADMIN_USERNAME, "admin123").is_ok());
    }

    #[test]
    fn accepts_the_seeded_admin_case_insensitively() {
        assert!(verify_credentials(&conn(), "ADMIN@ADMIN.COM", "admin123").is_ok());
    }

    #[test]
    fn rejects_wrong_password() {
        assert!(matches!(
            verify_credentials(&conn(), db::DEFAULT_ADMIN_USERNAME, "errada"),
            Err(AppError::InvalidCredentials)
        ));
    }

    #[test]
    fn rejects_unknown_user_without_leaking_which_field_failed() {
        assert!(matches!(
            verify_credentials(&conn(), "ninguem@exemplo.com", "admin123"),
            Err(AppError::InvalidCredentials)
        ));
    }

    #[test]
    fn register_creates_account_with_bcrypt_hash() {
        let c = conn();
        let res = register_admin(&c, "novo.professor@ifes.edu.br", "SenhaSegura123", "SenhaSegura123");
        assert!(res.is_ok());

        let hash: String = c
            .query_row(
                "SELECT hashed_password FROM admins WHERE username = 'novo.professor@ifes.edu.br'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert!(hash.starts_with("$2b$12$"), "o hash deve ser gerado por bcrypt");
        assert!(bcrypt::verify("SenhaSegura123", &hash).unwrap());
        assert!(!bcrypt::verify("SenhaErrada", &hash).unwrap());
    }

    #[test]
    fn register_normalizes_email_to_lowercase_and_trims() {
        let c = conn();
        let res = register_admin(&c, "  Espacos@IFES.edu.br  ", "SenhaSegura123", "SenhaSegura123");
        assert!(res.is_ok());

        let count: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM admins WHERE username = 'espacos@ifes.edu.br'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn register_rejects_duplicate_email() {
        let c = conn();
        let res = register_admin(&c, db::DEFAULT_ADMIN_USERNAME, "SenhaSegura123", "SenhaSegura123");
        assert!(matches!(res, Err(AppError::Validation(msg)) if msg.contains("já está cadastrado")));

        // Duplicate with uppercase
        let res_upper = register_admin(&c, "ADMIN@ADMIN.COM", "SenhaSegura123", "SenhaSegura123");
        assert!(matches!(res_upper, Err(AppError::Validation(msg)) if msg.contains("já está cadastrado")));
    }

    #[test]
    fn register_rejects_password_mismatch() {
        let c = conn();
        let res = register_admin(&c, "outro@ifes.edu.br", "SenhaSegura123", "SenhaDiferente123");
        assert!(matches!(res, Err(AppError::Validation(msg)) if msg.contains("não coincide")));
    }

    #[test]
    fn register_rejects_short_password() {
        let c = conn();
        let res = register_admin(&c, "outro@ifes.edu.br", "curta", "curta");
        assert!(matches!(res, Err(AppError::Validation(msg)) if msg.contains("mínimo 8 caracteres")));
    }

    #[test]
    fn register_rejects_empty_or_whitespace_password() {
        let c = conn();
        let res = register_admin(&c, "outro@ifes.edu.br", "        ", "        ");
        assert!(matches!(res, Err(AppError::Validation(msg)) if msg.contains("mínimo 8 caracteres")));
    }

    #[test]
    fn register_rejects_invalid_email_format() {
        let c = conn();
        let res1 = register_admin(&c, "sem-arroba", "SenhaSegura123", "SenhaSegura123");
        assert!(matches!(res1, Err(AppError::Validation(msg)) if msg.contains("Formato de email inválido")));

        let res2 = register_admin(&c, "sem-dominio@", "SenhaSegura123", "SenhaSegura123");
        assert!(matches!(res2, Err(AppError::Validation(msg)) if msg.contains("Formato de email inválido")));

        let res3 = register_admin(&c, "@sem-local.com", "SenhaSegura123", "SenhaSegura123");
        assert!(matches!(res3, Err(AppError::Validation(msg)) if msg.contains("Formato de email inválido")));
    }

    #[test]
    fn newly_registered_account_can_login() {
        let c = conn();
        register_admin(&c, "prof.doutor@ifes.edu.br", "MinhaSenha@2026", "MinhaSenha@2026").unwrap();
        assert!(verify_credentials(&c, "prof.doutor@ifes.edu.br", "MinhaSenha@2026").is_ok());
        assert!(verify_credentials(&c, "PROF.DOUTOR@IFES.EDU.BR", "MinhaSenha@2026").is_ok());
    }
}

