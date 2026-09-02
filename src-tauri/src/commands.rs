//! IPC surface (SEP-017). Thin: validate, delegate, return. No logic here.
//!
//! `#[tauri::command(async)]` runs these synchronous bodies off the main thread,
//! which is what makes the blocking rusqlite driver comfortable (AD-08).

use crate::crud::{self, Page, Record};
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command(async)]
pub fn list_entities(
    state: State<'_, AppState>,
    entity: String,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
) -> Result<Page, AppError> {
    let conn = state.db()?;
    crud::list(
        &conn,
        &entity,
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
    )
}

#[tauri::command(async)]
pub fn get_entity(state: State<'_, AppState>, entity: String, id: i64) -> Result<Record, AppError> {
    let conn = state.db()?;
    crud::get(&conn, &entity, id)
}

#[tauri::command(async)]
pub fn create_entity(
    state: State<'_, AppState>,
    entity: String,
    payload: Record,
) -> Result<Record, AppError> {
    let conn = state.db()?;
    crud::create(&conn, &entity, &payload)
}

#[tauri::command(async)]
pub fn update_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
    payload: Record,
) -> Result<Record, AppError> {
    let conn = state.db()?;
    crud::update(&conn, &entity, id, &payload)
}

/// Returns unit, which reaches JavaScript as `null`. That is what removes the
/// spurious error dialog on every successful delete (FR-013): the current
/// `apiFetch` parses a 204 empty body and throws before checking the status.
#[tauri::command(async)]
pub fn delete_entity(state: State<'_, AppState>, entity: String, id: i64) -> Result<(), AppError> {
    let conn = state.db()?;
    crud::delete(&conn, &entity, id)
}

#[derive(serde::Serialize)]
pub struct LoginResult {
    /// Kept only so the existing LoginForm keeps working unchanged; it carries
    /// no cryptographic meaning in a local app. Real sessions arrive in SEP-021.
    pub access_token: String,
    pub token_type: String,
}

#[tauri::command(async)]
pub fn login(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<LoginResult, AppError> {
    let conn = state.db()?;
    crate::auth::verify_credentials(&conn, &email, &password)?;
    Ok(LoginResult {
        access_token: "local-session".into(),
        token_type: "bearer".into(),
    })
}
