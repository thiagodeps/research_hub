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
    let hash: Option<String> = conn
        .query_row(
            "SELECT hashed_password FROM admins WHERE username = ?1",
            [username],
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
}
