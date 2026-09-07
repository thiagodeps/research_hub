//! Managed state handed to Tauri commands (SEP-016, FR-012).

use crate::error::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// Authenticated admin. Populated in SEP-021; the slot exists here so commands
/// can already be written against a real gate.
#[derive(Clone, Debug)]
pub struct Session {
    pub username: String,
}

pub struct AppState {
    /// Interactive connection, used by CRUD commands. Short-lived locks only.
    db: Mutex<Connection>,
    /// Kept so the ETL can open its own connection instead of holding this lock.
    pub db_path: PathBuf,
    pub session: Mutex<Option<Session>>,
}

impl AppState {
    pub fn new(db: Connection, db_path: PathBuf) -> Self {
        Self { db: Mutex::new(db), db_path, session: Mutex::new(None) }
    }

    pub fn db(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        Ok(self.db.lock()?)
    }

    /// Own connection for long operations, so imports never block the UI.
    pub fn etl_connection(&self) -> Result<Connection, AppError> {
        crate::db::open(&self.db_path)
    }
}

impl AppState {
    /// Reject the call unless a session is open (SEP-021).
    ///
    /// This makes authentication real for the first time: today the JWT is
    /// created at login and never validated by any endpoint, so any caller can
    /// read every table without credentials.
    pub fn require_session(&self) -> Result<Session, AppError> {
        self.session.lock()?.clone().ok_or(AppError::Unauthenticated)
    }

    pub fn open_session(&self, username: &str) -> Result<(), AppError> {
        *self.session.lock()? = Some(Session { username: username.to_string() });
        Ok(())
    }

    pub fn close_session(&self) -> Result<(), AppError> {
        *self.session.lock()? = None;
        Ok(())
    }

    pub fn current_session(&self) -> Result<Option<Session>, AppError> {
        Ok(self.session.lock()?.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn state() -> AppState {
        AppState::new(db::open_in_memory().unwrap(), PathBuf::from(":memory:"))
    }

    #[test]
    fn commands_are_refused_until_login() {
        let s = state();
        assert!(matches!(s.require_session(), Err(AppError::Unauthenticated)));

        s.open_session("admin@admin.com").unwrap();
        assert_eq!(s.require_session().unwrap().username, "admin@admin.com");

        s.close_session().unwrap();
        assert!(matches!(s.require_session(), Err(AppError::Unauthenticated)));
    }

    #[test]
    fn session_status_reflects_login_state() {
        let s = state();
        assert!(s.current_session().unwrap().is_none());
        s.open_session("admin@admin.com").unwrap();
        assert!(s.current_session().unwrap().is_some());
    }
}
