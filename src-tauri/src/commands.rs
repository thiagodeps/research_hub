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

#[tauri::command(async)]
pub fn register(
    state: State<'_, AppState>,
    email: String,
    password: String,
    password_confirm: String,
) -> Result<(), AppError> {
    let conn = state.db()?;
    crate::auth::register_admin(&conn, &email, &password, &password_confirm)
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

// ------------------------------------------------------------------- SRC (033)

/// Import the SRC consolidated JSON (SEP-033, FR-006..009). Dialog opened in
/// Rust like the canonical import; own connection so the UI lock stays free.
#[tauri::command(async)]
pub fn import_src_json(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::src_domain::SrcImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let chosen = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Consolidado do SRC", &["json"])
                .blocking_pick_file();
            match picked {
                Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
                None => return Ok(None),
            }
        }
    };

    let text = std::fs::read_to_string(&chosen)?;
    let data_dir = app_data_dir(&app)?;

    state.require_session()?;
    let _ = app.emit("src-import://progress", "carregando consolidado");
    let mut conn = state.etl_connection()?;
    let summary = crate::src_domain::import(&mut conn, &text, &data_dir)?;
    let _ = app.emit("src-import://progress", "importação concluída");

    Ok(Some(summary))
}

/// Export the SRC base as a consolidated JSON (SEP-033, FR-014). Same dialog
/// discipline: save dialog in Rust, front-end never touches the filesystem.
#[tauri::command(async)]
pub fn export_src_json(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<crate::src_domain::SrcExportSummary>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let target = match app
        .dialog()
        .file()
        .set_file_name("src_consolidado.json")
        .add_filter("Consolidado do SRC", &["json"])
        .blocking_save_file()
    {
        Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
        None => return Ok(None),
    };

    state.require_session()?;
    let conn = state.etl_connection()?;
    let json = crate::src_domain::export(&conn)?;
    std::fs::write(&target, json)?;

    let counts = crate::src_domain::export_summary(&conn)?;
    Ok(Some(crate::src_domain::SrcExportSummary {
        total_acoes: counts.0,
        total_participacoes: counts.1,
        path: target.display().to_string(),
    }))
}

// SEP-033 / US3: curadoria das ações e participações. Mesma disciplina do
// CRUD genérico (validar, delegar, retornar), porém sempre nos comandos
// dedicados — o caminho do Horizon fica invariante (R6).

#[tauri::command(async)]
pub fn src_list_acoes(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
) -> Result<crate::src_domain::SrcAcaoPage, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::list_acoes(
        &conn,
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
    )
}

#[tauri::command(async)]
pub fn src_get_acao(state: State<'_, AppState>, id: i64) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::get_acao(&conn, id)
}

#[tauri::command(async)]
pub fn src_create_acao(
    state: State<'_, AppState>,
    payload: crate::crud::Record,
) -> Result<i64, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::create_acao(&conn, &payload)
}

#[tauri::command(async)]
pub fn src_update_acao(
    state: State<'_, AppState>,
    id: i64,
    payload: crate::crud::Record,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::update_acao(&conn, id, &payload)
}

#[tauri::command(async)]
pub fn src_delete_acao(
    state: State<'_, AppState>,
    id: i64,
    force: Option<bool>,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::delete_acao(&conn, id, force.unwrap_or(false))
}

#[tauri::command(async)]
pub fn src_list_participacoes(
    state: State<'_, AppState>,
    acao_id: i64,
) -> Result<Vec<crate::crud::Record>, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::list_participacoes(&conn, acao_id)
}

#[tauri::command(async)]
pub fn src_create_participacao(
    state: State<'_, AppState>,
    acao_id: i64,
    payload: crate::crud::Record,
) -> Result<i64, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::create_participacao(&conn, acao_id, &payload)
}

#[tauri::command(async)]
pub fn src_update_participacao(
    state: State<'_, AppState>,
    id: i64,
    payload: crate::crud::Record,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::update_participacao(&conn, id, &payload)
}

#[tauri::command(async)]
pub fn src_delete_participacao(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::delete_participacao(&conn, id)
}

#[tauri::command(async)]
pub fn src_get_meta(state: State<'_, AppState>) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::get_meta(&conn)
}

#[tauri::command(async)]
pub fn src_update_meta(
    state: State<'_, AppState>,
    campus: Option<String>,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::update_meta(&conn, campus.as_deref())
}

// ----------------------------------------------------------- special ops (020)

// ------------------------------------------------------- GitHub sync (034)
//
// Toda a conversa com github.com acontece no núcleo (Princípio VI); aqui só
// orquestramos: sessão, config local (sync_domain), cliente HTTP
// (sync_github) e os imports/exports já existentes — o ritual de segurança
// do import manual é o MESMO, só muda a origem dos bytes (FR-003).

#[tauri::command(async)]
pub fn github_get_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::sync_domain::SyncConfigView, AppError> {
    state.require_session()?;
    let file = crate::sync_domain::load_config(&app_data_dir(&app)?)?;
    crate::sync_domain::config_view(&project, &file)
}

#[tauri::command(async)]
pub async fn github_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    url: String,
) -> Result<serde_json::Value, AppError> {
    use tauri::Emitter;
    state.require_session()?;

    let source = crate::sync_domain::normalize_url(&url)?;
    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    let mut conn = state.etl_connection()?;

    let result = crate::sync_github::download_and_import(
        &mut conn,
        &data_dir,
        &project,
        &source,
        &client,
        &|p| {
            let _ = app.emit("sync://progress", p);
        },
    )
    .await;

    let _ = app.emit(
        "sync://progress",
        crate::sync_domain::SyncProgress {
            operation: "download",
            project: project.clone(),
            phase: if result.is_ok() { "done" } else { "failed" },
            bytes_done: 0,
            bytes_total: None,
        },
    );
    result
}

#[tauri::command(async)]
pub fn github_set_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    repo: String,
    branch: String,
    path: String,
) -> Result<(), AppError> {
    state.require_session()?;
    crate::sync_domain::set_project_config(&app_data_dir(&app)?, &project, &repo, &branch, &path)?;
    Ok(())
}

#[tauri::command(async)]
pub async fn github_check_destination(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::sync_domain::CheckDestinationView, AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    crate::sync_github::check_project_destination(&data_dir, &project, &client).await
}

#[tauri::command(async)]
pub async fn github_upload(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    confirm_overwrite: Option<bool>,
) -> Result<crate::sync_github::UploadResult, AppError> {
    use tauri::Emitter;
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    let bytes = {
        let conn = state.etl_connection()?;
        let original = std::fs::read(data_dir.join(crate::import::ORIGINAL_ARCHIVE)).ok();
        let (b, _tables, _preserved) =
            crate::export::build_archive(&conn, original.as_deref(), |_| {})?;
        b
    };

    let result = crate::sync_github::upload_horizon_export(
        bytes,
        &data_dir,
        &project,
        confirm_overwrite.unwrap_or(false),
        &client,
        &|p| {
            let _ = app.emit("sync://progress", p);
        },
    )
    .await;

    let _ = app.emit(
        "sync://progress",
        crate::sync_domain::SyncProgress {
            operation: "upload",
            project: project.clone(),
            phase: if result.is_ok() { "done" } else { "failed" },
            bytes_done: 0,
            bytes_total: None,
        },
    );

    result
}

#[tauri::command(async)]
pub fn github_save_token(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    token: String,
) -> Result<(), AppError> {
    state.require_session()?;
    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::SyncConfig("Token de acesso não pode ficar vazio.".into()));
    }
    let data_dir = app_data_dir(&app)?;
    let mut file = crate::sync_domain::load_config(&data_dir)?;
    file.token = Some(token.to_string());
    crate::sync_domain::save_config(&data_dir, &file)?;
    Ok(())
}

#[tauri::command(async)]
pub async fn github_test_token(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::sync_github::TokenInfo, AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let file = crate::sync_domain::load_config(&data_dir)?;
    let token = file.token.ok_or_else(|| {
        AppError::SyncConfig("Nenhum token configurado.".into())
    })?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        Some(token),
    );
    client.test_token().await
}

#[tauri::command(async)]
pub fn github_remove_token(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let mut file = crate::sync_domain::load_config(&data_dir)?;
    file.token = None;
    crate::sync_domain::save_config(&data_dir, &file)?;
    Ok(())
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
