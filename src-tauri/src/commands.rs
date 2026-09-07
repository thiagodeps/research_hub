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
    state.require_session()?;
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
    state.require_session()?;
    let conn = state.db()?;
    crud::get(&conn, &entity, id)
}

#[tauri::command(async)]
pub fn create_entity(
    state: State<'_, AppState>,
    entity: String,
    payload: Record,
) -> Result<Record, AppError> {
    state.require_session()?;
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
    state.require_session()?;
    let conn = state.db()?;
    crud::update(&conn, &entity, id, &payload)
}

/// Returns unit, which reaches JavaScript as `null`. That is what removes the
/// spurious error dialog on every successful delete (FR-013): the current
/// `apiFetch` parses a 204 empty body and throws before checking the status.
#[tauri::command(async)]
pub fn delete_entity(state: State<'_, AppState>, entity: String, id: i64) -> Result<(), AppError> {
    state.require_session()?;
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
    {
        let conn = state.db()?;
        crate::auth::verify_credentials(&conn, &email, &password)?;
    }
    state.open_session(&email)?;
    Ok(LoginResult {
        access_token: "local-session".into(),
        token_type: "bearer".into(),
    })
}

// ---------------------------------------------------------------- data import

/// Import the canonical archive (SEP-018).
///
/// The dialog is opened here, in Rust (decision Q1), so the frontend needs no
/// filesystem permission at all. `path` is accepted for the drag-and-drop path,
/// where the window already knows the file.
#[tauri::command(async)]
pub fn import_canonical_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::import::ImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let chosen = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Pacote canônico", &["zip"])
                .blocking_pick_file();
            // Dismissing the dialog is a no-op, not an error (FR-001).
            match picked {
                Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
                None => return Ok(None),
            }
        }
    };

    if !crate::import::looks_like_zip(&chosen) {
        return Err(AppError::Internal(
            "Apenas arquivos .zip contendo .parquet são aceitos.".into(),
        ));
    }

    let bytes = std::fs::read(&chosen)?;
    let data_dir = app_data_dir(&app)?;

    // Own connection: the import runs for seconds and must not hold the lock
    // the interface uses (FR-013).
    state.require_session()?;
    let mut conn = state.etl_connection()?;
    let summary = crate::import::import_archive(&mut conn, &bytes, &data_dir, |t| {
        let _ = app.emit("import://progress", t);
    })?;

    Ok(Some(summary))
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, AppError> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("diretório de dados indisponível: {e}")))
}

/// Export the curated base (SEP-019). Save dialog opened in Rust (Q1).
#[tauri::command(async)]
pub fn export_canonical_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<crate::export::ExportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let target = match app
        .dialog()
        .file()
        .set_file_name(crate::export::DEFAULT_FILENAME)
        .add_filter("Pacote canônico", &["zip"])
        .blocking_save_file()
    {
        Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
        None => return Ok(None),
    };

    let data_dir = app_data_dir(&app)?;
    let original = std::fs::read(data_dir.join(crate::import::ORIGINAL_ARCHIVE)).ok();

    state.require_session()?;
    let conn = state.etl_connection()?;
    let (bytes, tables, preserved) =
        crate::export::build_archive(&conn, original.as_deref(), |t| {
            let _ = app.emit("export://progress", t);
        })?;

    std::fs::write(&target, &bytes)?;

    Ok(Some(crate::export::ExportSummary {
        tables,
        preserved_entries: preserved,
        path: target.display().to_string(),
    }))
}

// ----------------------------------------------------------- special ops (020)

#[tauri::command(async)]
pub fn merge_entities(
    state: State<'_, AppState>,
    entity: String,
    source_ids: Vec<i64>,
    resolved_data: crud::Record,
) -> Result<crud::Record, AppError> {
    state.require_session()?;
    let mut conn = state.etl_connection()?;
    crate::special::merge(&mut conn, &entity, &source_ids, &resolved_data)
}

#[tauri::command(async)]
pub fn link_entities(
    state: State<'_, AppState>,
    parent_type: String,
    parent_id: i64,
    child_type: String,
    child_id: i64,
) -> Result<crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::special::link(&conn, &parent_type, parent_id, &child_type, child_id)
}

#[tauri::command(async)]
pub fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    state.close_session()
}

#[derive(serde::Serialize)]
pub struct SessionStatus {
    pub authenticated: bool,
    pub username: Option<String>,
}

#[tauri::command(async)]
pub fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let s = state.current_session()?;
    Ok(SessionStatus {
        authenticated: s.is_some(),
        username: s.map(|s| s.username),
    })
}
