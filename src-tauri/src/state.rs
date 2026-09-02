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
