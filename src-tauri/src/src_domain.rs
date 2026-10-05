//! SRC domain logic: consolidated JSON import/export and CRUD (SEP-033).
//!
//! Research decisions applied here: R2 (the consolidated file flattens
//! participations — one entry per person), R3 (raw_json + projections),
//! R4 (header counters recomputed on export), R6 (dedicated domain, the
//! generic Horizon CRUD never touches these tables).

use crate::crud::Record;
use crate::error::AppError;
use rusqlite::Connection;
use serde_json::{Map, Value as Json};
use std::path::Path;

// Label keys exactly as the SRC scraper writes them (contract C5).
pub const K_ACAO_ID: &str = "acao_id";
pub const K_PROCESSO: &str = "Processo nº";
pub const K_TITULO: &str = "Título ação";
pub const K_NATUREZA: &str = "Natureza";
pub const K_TIPO: &str = "Tipo ação";
pub const K_COORDENADOR: &str = "Coordenador(a)";
pub const K_VINCULANTE: &str = "Ação vinculante";
pub const K_CAMPUS: &str = "Campus";
pub const K_CAMPUS_LOWER: &str = "campus";
pub const K_PARTICIPACOES: &str = "participacoes";
pub const K_TOTAL_PARTICIPACOES: &str = "total_participacoes";

pub const TIPO_PUBLICO: &str = "Público-alvo";
pub const TIPO_EQUIPE: &str = "Equipe de execução";

/// What the command layer reports to the UI after an import (FR-009).
#[derive(serde::Serialize, Debug)]
pub struct SrcImportSummary {
    pub campus: Option<String>,
    pub total_acoes: usize,
    pub total_participacoes: usize,
    pub acoes_com_participacoes: usize,
    pub replaced: bool,
    pub snapshot: Option<String>,
    pub unexpected_tipos: Vec<String>,
}

/// Trimmed string content of a JSON value, or None for null/empty/non-strings.
fn label<'a>(obj: &'a Map<String, Json>, key: &str) -> Option<&'a str> {
    obj.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

struct SrcAcao {
    /// The original object, keys in file order (preserved by preserve_order).
    obj: Map<String, Json>,
    participacoes: Vec<Map<String, Json>>,
}

struct Consolidated {
    campus: Option<String>,
    acoes: Vec<SrcAcao>,
}

/// Parse + validate the whole file BEFORE anything is written (FR-007).
/// Contract C1–C4: root object with an `acoes` list; every action has a
/// non-empty `acao_id` (no duplicates); `participacoes`, when present, is a
/// list. The file is the oracle — unexpected `tipo` values come in as-is and
/// are only flagged (C6).
fn validate(root: &Json) -> Result<Consolidated, AppError> {
    let obj = root
        .as_object()
        .ok_or_else(|| AppError::Validation("a raiz do arquivo não é um objeto JSON".into()))?;
    let acoes = obj
        .get("acoes")
        .ok_or_else(|| {
            AppError::Validation(
                "arquivo sem a chave \"acoes\" — não parece um consolidado do SRC".into(),
            )
        })?
        .as_array()
        .ok_or_else(|| AppError::Validation("a chave \"acoes\" deve ser uma lista".into()))?;

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(acoes.len());
    for (i, acao) in acoes.iter().enumerate() {
        let o = acao.as_object().ok_or_else(|| {
            AppError::Validation(format!("ações[{i}] não é um objeto"))
        })?;
        let id = label(o, K_ACAO_ID).ok_or_else(|| {
            AppError::Validation(format!("ações[{i}] sem \"acao_id\" válido"))
        })?;
        if !seen.insert(id.to_string()) {
            return Err(AppError::Validation(format!(
                "acao_id duplicado no arquivo: {id}"
            )));
        }
        let participacoes = match o.get(K_PARTICIPACOES) {
            None | Some(Json::Null) => Vec::new(),
            Some(Json::Array(arr)) => {
                let mut parts = Vec::with_capacity(arr.len());
                for (j, p) in arr.iter().enumerate() {
                    parts.push(p.as_object().cloned().ok_or_else(|| {
                        AppError::Validation(format!(
                            "ações[{i}].participacoes[{j}] não é um objeto"
                        ))
                    })?);
                }
                parts
            }
            Some(_) => {
                return Err(AppError::Validation(format!(
                    "ações[{i}]: \"participacoes\" deve ser uma lista"
                )))
            }
        };
        out.push(SrcAcao { obj: o.clone(), participacoes });
    }

    let campus = obj.get("campus").and_then(|v| v.as_str()).map(str::to_string);
    Ok(Consolidated { campus, acoes: out })
}

fn has_src_rows(conn: &Connection) -> Result<bool, AppError> {
    let n: i64 =
        conn.query_row("SELECT COUNT(*) FROM src_acoes", [], |r| r.get(0))?;
    Ok(n > 0)
}

/// Insert one participação row; returns the `tipo` value as stored.
fn insert_participacao(
    tx: &rusqlite::Transaction<'_>,
    acao_row_id: i64,
    ord: usize,
    part: &Map<String, Json>,
    unexpected: &mut Vec<String>,
) -> Result<(), AppError> {
    let tipo = label(part, "tipo").unwrap_or("").to_string();
    if !tipo.is_empty() && tipo != TIPO_PUBLICO && tipo != TIPO_EQUIPE && !unexpected.contains(&tipo)
    {
        unexpected.push(tipo.clone());
    }
    let raw = Json::Object(part.clone()).to_string();
    tx.execute(
        "INSERT INTO src_participacoes (acao_row_id, ord, tipo, atividade_num, atividade_id, atividade, nome, raw_json) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            acao_row_id,
            ord as i64,
            tipo,
            label(part, "atividade_num"),
            label(part, "atividade_id"),
            label(part, "atividade"),
            label(part, "Nome"),
            raw,
        ],
    )?;
    Ok(())
}

pub fn import(
    conn: &mut Connection,
    text: &str,
    data_dir: &Path,
) -> Result<SrcImportSummary, AppError> {
    let consolidated = validate(&serde_json::from_str::<Json>(text).map_err(|e| {
        AppError::Validation(format!("o arquivo não é um JSON válido: {e}"))
    })?)?;

    // Snapshot only protects a base that had curated rows (FR-006), with the
    // src- prefix so it never reads as a Horizon snapshot (FR-015, T008).
    let replaced = has_src_rows(conn)?;
    let snapshot = if replaced {
        Some(
            crate::import::snapshot_with_prefix(conn, data_dir, "src")?
                .display()
                .to_string(),
        )
    } else {
        None
    };

    let mut unexpected: Vec<String> = Vec::new();
    let mut total_participacoes = 0usize;
    let mut acoes_com_participacoes = 0usize;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM src_participacoes", [])?;
    tx.execute("DELETE FROM src_acoes", [])?;
    tx.execute(
        "UPDATE src_meta SET campus = ?1, imported_at = ?2",
        rusqlite::params![
            consolidated.campus,
            chrono_like_stamp(),
        ],
    )?;

    for acao in &consolidated.acoes {
        let o = &acao.obj;
        tx.execute(
            "INSERT INTO src_acoes (acao_id, raw_json, processo, titulo, natureza, tipo, coordenador, acao_vinculante, campus, total_participacoes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                label(o, K_ACAO_ID),
                Json::Object(o.clone()).to_string(),
                label(o, K_PROCESSO),
                label(o, K_TITULO),
                label(o, K_NATUREZA),
                label(o, K_TIPO),
                label(o, K_COORDENADOR),
                label(o, K_VINCULANTE),
                label(o, K_CAMPUS).or_else(|| label(o, K_CAMPUS_LOWER)),
                acao.participacoes.len() as i64,
            ],
        )?;
        let acao_row_id = tx.last_insert_rowid();
        for (ord, part) in acao.participacoes.iter().enumerate() {
            insert_participacao(&tx, acao_row_id, ord, part, &mut unexpected)?;
        }
        total_participacoes += acao.participacoes.len();
        if !acao.participacoes.is_empty() {
            acoes_com_participacoes += 1;
        }
    }
    tx.commit()?;

    Ok(SrcImportSummary {
        campus: consolidated.campus,
        total_acoes: consolidated.acoes.len(),
        total_participacoes,
        acoes_com_participacoes,
        replaced,
        snapshot,
        unexpected_tipos: unexpected,
    })
}

/// UTC timestamp for the audit column, without pulling a date crate into the
/// dependency tree (same shape as the snapshot file stamp).
fn chrono_like_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{secs}")
}

/// What the command layer reports to the UI after an export (FR-014).
#[derive(serde::Serialize, Debug)]
pub struct SrcExportSummary {
    pub total_acoes: usize,
    pub total_participacoes: usize,
    pub path: String,
}

/// (total_acoes, total_participacoes) for the export summary.
pub fn export_summary(conn: &Connection) -> Result<(usize, usize), AppError> {
    let total_acoes: i64 =
        conn.query_row("SELECT COUNT(*) FROM src_acoes", [], |r| r.get(0))?;
    let total_participacoes: i64 =
        conn.query_row("SELECT COUNT(*) FROM src_participacoes", [], |r| r.get(0))?;
    Ok((total_acoes as usize, total_participacoes as usize))
}

// ---------------------------------------------------------------- CRUD (033)

/// Same shape as `crud::Page` so the front-end table stays unchanged.
#[derive(serde::Serialize, Debug)]
pub struct SrcAcaoPage {
    pub items: Vec<Record>,
    pub total: i64,
}

/// Payload field → label key inside raw_json. The two representations are
/// ALWAYS updated together (research R3): projections power search/sort,
/// raw_json is what the export emits.
const ACAO_FIELDS: &[(&str, &str)] = &[
    ("acao_id", K_ACAO_ID),
    ("processo", K_PROCESSO),
    ("titulo", K_TITULO),
    ("natureza", K_NATUREZA),
    ("tipo", K_TIPO),
    ("coordenador", K_COORDENADOR),
    ("acao_vinculante", K_VINCULANTE),
    ("campus", K_CAMPUS),
    ("resumo", "Resumo"),
];

fn acao_label_for(field: &str) -> Option<&'static str> {
    ACAO_FIELDS
        .iter()
        .find(|(f, _)| *f == field)
        .map(|(_, l)| *l)
}

/// Trimmed string content of a payload field; `None` for null/absent.
fn field_str(payload: &Record, key: &str) -> Option<String> {
    match payload.get(key) {
        None | Some(Json::Null) => None,
        Some(Json::String(s)) => Some(s.trim().to_string()),
        Some(other) => Some(other.to_string()),
    }
}

/// Same conversion discipline as crud.rs: everything is TEXT, so non-strings
/// are stringified rather than rejected (the interface sends numbers).
fn to_text(v: &Json) -> String {
    match v {
        Json::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn sync_total(conn: &Connection, acao_row_id: i64) -> Result<(), AppError> {
    conn.execute(
        "UPDATE src_acoes SET total_participacoes = \
         (SELECT COUNT(*) FROM src_participacoes WHERE acao_row_id = ?1) WHERE id = ?1",
        [acao_row_id],
    )?;
    Ok(())
}

pub fn list_acoes(
    conn: &Connection,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&str>,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<SrcAcaoPage, AppError> {
    const DEFAULT_LIMIT: i64 = 100;
    const MAX_LIMIT: i64 = 1000;
    let def = crate::registry::by_src_route("src_acoes")?;

    // Whitelist the sort column: fail typed instead of interpolating SQL.
    let sort = match sort {
        None | Some("") => "id",
        Some(c) => {
            if def.columns.contains(&c) && c != "raw_json" {
                c
            } else {
                return Err(AppError::InvalidColumn(c.to_string()));
            }
        }
    };
    let dir = match order {
        Some("desc") => "DESC",
        _ => "ASC",
    };

    let mut where_clause = String::new();
    let mut params: Vec<rusqlite::types::Value> = Vec::new();
    if let Some(term) = search.map(str::trim).filter(|t| !t.is_empty()) {
        where_clause = "WHERE titulo LIKE ?1 ESCAPE '\\'".into();
        params.push(rusqlite::types::Value::Text(format!(
            "%{}%",
            crate::crud::escape_like(term)
        )));
    }

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM src_acoes {where_clause}"),
        rusqlite::params_from_iter(params.iter()),
        |r| r.get(0),
    )?;

    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);
    let mut sql = format!(
        "SELECT {} FROM src_acoes {where_clause} ORDER BY {sort} {dir}, id ASC LIMIT {limit} OFFSET {offset}",
        def.columns.iter().filter(|c| **c != "raw_json").map(|c| c.to_string()).collect::<Vec<_>>().join(", ")
    );
    sql.push_str(""); // keep the format expression tied to the statement above

    let mut stmt = conn.prepare(&sql)?;
    let cols: Vec<&str> = def.columns.iter().filter(|c| **c != "raw_json").copied().collect();
    let rows = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |r| {
            let mut record = Record::new();
            for (i, col) in cols.iter().enumerate() {
                let v: rusqlite::types::Value = r.get(i)?;
                record.insert(
                    col.to_string(),
                    match v {
                        rusqlite::types::Value::Null => Json::Null,
                        rusqlite::types::Value::Text(s) => Json::from(s),
                        rusqlite::types::Value::Integer(i) => Json::from(i),
                        rusqlite::types::Value::Real(f) => Json::from(f),
                        rusqlite::types::Value::Blob(_) => Json::Null,
                    },
                );
            }
            Ok(record)
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(SrcAcaoPage { items: rows, total })
}

pub fn get_acao(conn: &Connection, id: i64) -> Result<Record, AppError> {
    let def = crate::registry::by_src_route("src_acoes")?;
    let cols: Vec<&str> = def.columns.iter().filter(|c| **c != "raw_json").copied().collect();
    let mut stmt = conn.prepare(&format!(
        "SELECT {}, raw_json FROM src_acoes WHERE id = ?1",
        cols.join(", ")
    ))?;
    let mut rows = stmt.query([id])?;
    let row = rows
        .next()?
        .ok_or(AppError::NotFound)?;

    let mut record = Record::new();
    for (i, col) in cols.iter().enumerate() {
        let v: rusqlite::types::Value = row.get(i)?;
        record.insert(
            col.to_string(),
            match v {
                rusqlite::types::Value::Null => Json::Null,
                rusqlite::types::Value::Text(s) => Json::from(s),
                rusqlite::types::Value::Integer(i) => Json::from(i),
                rusqlite::types::Value::Real(f) => Json::from(f),
                rusqlite::types::Value::Blob(_) => Json::Null,
            },
        );
    }
    // The long text lives only inside raw_json — hand it over as `resumo`.
    let raw: String = row.get(cols.len())?;
    if let Ok(obj) = serde_json::from_str::<Json>(&raw) {
        if let Some(resumo) = obj.get("Resumo").and_then(|v| v.as_str()) {
            record.insert("resumo".into(), Json::from(resumo));
        }
    }
    Ok(record)
}

pub fn create_acao(conn: &Connection, payload: &Record) -> Result<i64, AppError> {
    let acao_id = field_str(payload, "acao_id").filter(|s| !s.is_empty()).ok_or_else(|| {
        AppError::Validation("acao_id é obrigatório e não pode ficar em branco".into())
    })?;

    let dup: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM src_acoes WHERE acao_id = ?1)",
            [&acao_id],
            |r| r.get(0),
        )?;
    if dup {
        return Err(AppError::Validation(format!(
            "já existe uma ação com acao_id {acao_id} (duplicado)"
        )));
    }

    // raw_json in the file's own shape: canonical label keys, in order (C5).
    let mut raw = Map::new();
    raw.insert(K_ACAO_ID.into(), Json::from(acao_id.as_str()));
    for (field, label) in ACAO_FIELDS.iter().skip(1) {
        if let Some(value) = field_str(payload, field).filter(|s| !s.is_empty()) {
            raw.insert(label.to_string(), Json::from(value));
        }
    }

    conn.execute(
        "INSERT INTO src_acoes (acao_id, raw_json, processo, titulo, natureza, tipo, coordenador, acao_vinculante, campus) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            acao_id,
            Json::Object(raw).to_string(),
            field_str(payload, "processo").filter(|s| !s.is_empty()),
            field_str(payload, "titulo").filter(|s| !s.is_empty()),
            field_str(payload, "natureza").filter(|s| !s.is_empty()),
            field_str(payload, "tipo").filter(|s| !s.is_empty()),
            field_str(payload, "coordenador").filter(|s| !s.is_empty()),
            field_str(payload, "acao_vinculante").filter(|s| !s.is_empty()),
            field_str(payload, "campus").filter(|s| !s.is_empty()),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_acao(conn: &Connection, id: i64, payload: &Record) -> Result<(), AppError> {
    let current_raw: String = conn
        .query_row("SELECT raw_json FROM src_acoes WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| AppError::NotFound)?;
    let current_id: String = conn
        .query_row("SELECT acao_id FROM src_acoes WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| AppError::NotFound)?;

    let mut raw: Map<String, Json> = serde_json::from_str(&current_raw)?;
    let mut sets: Vec<(&'static str, String)> = Vec::new();

    for (field, value) in payload {
        if field == "id" {
            continue;
        }
        if field == "total_participacoes" {
            // Derived counter (R4): recomputed from the rows, never hand-set —
            // but the editor form carries it in the payload, so skip silently.
            continue;
        }
        let label = acao_label_for(field)
            .ok_or_else(|| AppError::Validation(format!("campo desconhecido: {field}")))?;

        if field == "acao_id" {
            let wanted = to_text(value).trim().to_string();
            if wanted != current_id {
                return Err(AppError::Validation(
                    "o identificador acao_id não pode ser alterado (referência do pipeline SRC)".into(),
                ));
            }
            continue;
        }

        match value {
            Json::Null => {
                // Clearing: the key leaves the raw object, the column goes NULL.
                raw.shift_remove(label);
                sets.push((label, String::new())); // marker; column set to NULL below
            }
            v => {
                let text = to_text(v);
                raw.insert(label.to_string(), Json::from(text.trim()));
                sets.push((label, text.trim().to_string()));
            }
        }
    }

    conn.execute(
        "UPDATE src_acoes SET raw_json = ?1 WHERE id = ?2",
        rusqlite::params![Json::Object(raw).to_string(), id],
    )?;
    for (label, text) in sets {
        // Column names come from the fixed field→label table, never the user.
        let column = ACAO_FIELDS
            .iter()
            .find(|(_, l)| *l == label)
            .map(|(f, _)| *f)
            .unwrap_or("resumo"); // resumo has no projection column
        if column == "resumo" {
            continue;
        }
        let value = if text.is_empty() {
            // Either cleared (Null marker) or genuinely empty — both end NULL,
            // matching how the import stores absent fields.
            None
        } else {
            Some(text.as_str())
        };
        conn.execute(
            &format!("UPDATE src_acoes SET {column} = ?1 WHERE id = ?2"),
            rusqlite::params![value, id],
        )?;
    }
    Ok(())
}

pub fn delete_acao(conn: &Connection, id: i64, force: bool) -> Result<(), AppError> {
    let (processo, titulo): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT processo, titulo FROM src_acoes WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| AppError::NotFound)?;

    // Children reference the parent textually (Assumptions: by processo or
    // by título). Find them before anything is destroyed (FR-013).
    let mut stmt = conn.prepare(
        "SELECT id, acao_id, acao_vinculante, raw_json FROM src_acoes \
         WHERE id != ?1 AND acao_vinculante IS NOT NULL \
         AND (acao_vinculante = ?2 OR (?3 IS NOT NULL AND acao_vinculante = ?3))",
    )?;
    let filhas: Vec<(i64, String, String, String)> = stmt
        .query_map(
            rusqlite::params![id, processo.clone().unwrap_or_default(), titulo],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    if !filhas.is_empty() && !force {
        let ids: Vec<&str> = filhas.iter().map(|(_, acao_id, ..)| acao_id.as_str()).collect();
        return Err(AppError::Conflict(format!(
            "existem {} ação(ões) vinculada(s) a esta (acao_id: {}). Desvincule-as primeiro ou confirme a exclusão forçada.",
            filhas.len(),
            ids.join(", ")
        )));
    }

    // Forced: detach the children explicitly — in the column AND in raw_json,
    // so the export never carries a pointer to a deleted parent.
    for (fid, _, vinculo, raw) in &filhas {
        if vinculo != &processo.clone().unwrap_or_default() && titulo.as_deref() != Some(vinculo.as_str()) {
            continue;
        }
        let mut obj: Map<String, Json> = serde_json::from_str(raw)?;
        if let Some(v) = obj.get_mut(K_VINCULANTE) {
            if v.as_str().map(str::trim).unwrap_or("") == vinculo.trim() {
                *v = Json::from("");
            }
        }
        conn.execute(
            "UPDATE src_acoes SET acao_vinculante = NULL, raw_json = ?1 WHERE id = ?2",
            rusqlite::params![Json::Object(obj).to_string(), fid],
        )?;
    }

    conn.execute("DELETE FROM src_acoes WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list_participacoes(conn: &Connection, acao_id: i64) -> Result<Vec<Record>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, ord, tipo, atividade_num, atividade_id, atividade, nome, raw_json \
         FROM src_participacoes WHERE acao_row_id = ?1 ORDER BY ord",
    )?;
    let rows = stmt
        .query_map([acao_id], |r| {
            let mut rec = Record::new();
            rec.insert("id".into(), Json::from(r.get::<_, i64>(0)?));
            rec.insert("ord".into(), Json::from(r.get::<_, i64>(1)?));
            rec.insert("tipo".into(), Json::from(r.get::<_, String>(2)?));
            for (i, key) in [(3usize, "atividade_num"), (4, "atividade_id"), (5, "atividade"), (6, "nome")] {
                let v: Option<String> = r.get(i)?;
                rec.insert(key.into(), v.map(Json::from).unwrap_or(Json::Null));
            }
            // The person's fields live under the file's own label keys (C5):
            // merge them in so the editor shows Nome/CPF/E-mail/Função as-is.
            let raw: String = r.get(7)?;
            if let Ok(obj) = serde_json::from_str::<Record>(&raw) {
                for (k, v) in obj {
                    rec.entry(k).or_insert(v);
                }
            }
            Ok(rec)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn create_participacao(
    conn: &Connection,
    acao_id: i64,
    payload: &Record,
) -> Result<i64, AppError> {
    let exists: bool = conn
        .query_row("SELECT EXISTS(SELECT 1 FROM src_acoes WHERE id = ?1)", [acao_id], |r| r.get(0))?;
    if !exists {
        return Err(AppError::NotFound);
    }

    let tipo = field_str(payload, "tipo")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            AppError::Validation("tipo é obrigatório (Público-alvo ou Equipe de execução)".into())
        })?;

    // raw_json in the file's own order: activity context first, then tipo,
    // then the person's label keys exactly as the payload carries them (C5).
    let mut raw = Map::new();
    for key in ["atividade_num", "atividade_id", "atividade"] {
        if let Some(v) = field_str(payload, key).filter(|s| !s.is_empty()) {
            raw.insert(key.into(), Json::from(v));
        }
    }
    raw.insert("tipo".into(), Json::from(tipo.as_str()));
    for (k, v) in payload {
        if k == "id" || k == "tipo" || k == "atividade_num" || k == "atividade_id" || k == "atividade" {
            continue;
        }
        raw.insert(k.clone(), v.clone());
    }

    let next_ord: i64 = conn.query_row(
        "SELECT COALESCE(MAX(ord), 0) + 1 FROM src_participacoes WHERE acao_row_id = ?1",
        [acao_id],
        |r| r.get(0),
    )?;

    conn.execute(
        "INSERT INTO src_participacoes (acao_row_id, ord, tipo, atividade_num, atividade_id, atividade, nome, raw_json) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            acao_id,
            next_ord,
            tipo,
            field_str(payload, "atividade_num").filter(|s| !s.is_empty()),
            field_str(payload, "atividade_id").filter(|s| !s.is_empty()),
            field_str(payload, "atividade").filter(|s| !s.is_empty()),
            field_str(payload, "Nome").filter(|s| !s.is_empty()),
            Json::Object(raw).to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid();
    sync_total(conn, acao_id)?;
    Ok(id)
}

pub fn update_participacao(conn: &Connection, id: i64, payload: &Record) -> Result<(), AppError> {
    let (acao_row_id, current_raw): (i64, String) = conn
        .query_row(
            "SELECT acao_row_id, raw_json FROM src_participacoes WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| AppError::NotFound)?;

    let mut raw: Map<String, Json> = serde_json::from_str(&current_raw)?;
    for (key, value) in payload {
        if key == "id" || key == "ord" || key == "acao_row_id" {
            continue;
        }
        if key == "tipo" {
            let tipo = to_text(value);
            if tipo.trim().is_empty() {
                return Err(AppError::Validation(
                    "tipo é obrigatório (Público-alvo ou Equipe de execução)".into(),
                ));
            }
            raw.insert("tipo".into(), Json::from(tipo.trim()));
            conn.execute(
                "UPDATE src_participacoes SET tipo = ?1 WHERE id = ?2",
                rusqlite::params![tipo.trim(), id],
            )?;
            continue;
        }
        if matches!(key.as_str(), "atividade_num" | "atividade_id" | "atividade" | "Nome") {
            match value {
                Json::Null => {
                    raw.shift_remove(key);
                }
                v => {
                    let text = to_text(v);
                    raw.insert(key.clone(), Json::from(text.trim()));
                }
            }
        } else {
            // Free-form label key: goes into raw_json as-is (C5), no projection.
            raw.insert(key.clone(), value.clone());
        }
    }

    conn.execute(
        "UPDATE src_participacoes SET raw_json = ?1 WHERE id = ?2",
        rusqlite::params![Json::Object(raw.clone()).to_string(), id],
    )?;
    // Projections that come straight from the raw keys:
    conn.execute(
        "UPDATE src_participacoes SET \
         atividade_num = ?1, atividade_id = ?2, atividade = ?3, nome = ?4 WHERE id = ?5",
        rusqlite::params![
            raw.get("atividade_num").and_then(|v| v.as_str()),
            raw.get("atividade_id").and_then(|v| v.as_str()),
            raw.get("atividade").and_then(|v| v.as_str()),
            raw.get("Nome").and_then(|v| v.as_str()),
            id,
        ],
    )?;
    sync_total(conn, acao_row_id)?;
    Ok(())
}

pub fn delete_participacao(conn: &Connection, id: i64) -> Result<(), AppError> {
    let (acao_row_id, ord): (i64, i64) = conn
        .query_row(
            "SELECT acao_row_id, ord FROM src_participacoes WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| AppError::NotFound)?;

    conn.execute("DELETE FROM src_participacoes WHERE id = ?1", [id])?;
    // ord stays contiguous (data-model invariant): everything after shifts up.
    conn.execute(
        "UPDATE src_participacoes SET ord = ord - 1 WHERE acao_row_id = ?1 AND ord > ?2",
        rusqlite::params![acao_row_id, ord],
    )?;
    sync_total(conn, acao_row_id)?;
    Ok(())
}

pub fn get_meta(conn: &Connection) -> Result<Record, AppError> {
    let mut record = Record::new();
    let (campus, imported_at): (Option<String>, String) = conn
        .query_row(
            "SELECT campus, imported_at FROM src_meta LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
    record.insert("campus".into(), campus.map(Json::from).unwrap_or(Json::Null));
    record.insert("imported_at".into(), Json::from(imported_at));
    Ok(record)
}

pub fn update_meta(conn: &Connection, campus: Option<&str>) -> Result<(), AppError> {
    conn.execute(
        "UPDATE src_meta SET campus = ?1",
        [campus.filter(|c| !c.trim().is_empty())],
    )?;
    Ok(())
}

pub fn export(conn: &Connection) -> Result<String, AppError> {
    let campus: Option<String> = conn
        .query_row("SELECT campus FROM src_meta LIMIT 1", [], |r| r.get(0))?;

    // Root keys in the same order consolidar.py writes them (contract C9).
    let mut root = Map::new();
    root.insert(
        "campus".into(),
        campus.map(Json::String).unwrap_or(Json::Null),
    );

    let mut acoes_out: Vec<Json> = Vec::new();
    let mut acoes_com_participacoes = 0usize;
    let mut total_publico = 0usize;
    let mut total_equipe = 0usize;
    let mut atividade_contextos = std::collections::HashSet::new();

    let mut stmt = conn.prepare("SELECT id, raw_json FROM src_acoes ORDER BY id")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let acao_row_id: i64 = row.get(0)?;
        let raw: String = row.get(1)?;
        let mut obj: Map<String, Json> = serde_json::from_str(&raw)?;

        let mut parts_out: Vec<Json> = Vec::new();
        let mut p_stmt =
            conn.prepare("SELECT raw_json, tipo, atividade_num, atividade_id, atividade FROM src_participacoes WHERE acao_row_id = ?1 ORDER BY ord")?;
        let mut p_rows = p_stmt.query(rusqlite::params![acao_row_id])?;
        while let Some(p) = p_rows.next()? {
            let praw: String = p.get(0)?;
            let pobj: Map<String, Json> = serde_json::from_str(&praw)?;
            let tipo = label(&pobj, "tipo").unwrap_or("").to_string();
            match tipo.as_str() {
                TIPO_PUBLICO => total_publico += 1,
                TIPO_EQUIPE => total_equipe += 1,
                _ => {}
            }
            // Atividade de origem: contexto distinto (atividade_num + id + nome).
            atividade_contextos.insert((
                p.get::<_, Option<String>>(2)?,
                p.get::<_, Option<String>>(3)?,
                p.get::<_, Option<String>>(4)?,
            ));
            parts_out.push(Json::Object(pobj));
        }
        let total_parts = parts_out.len();
        if total_parts > 0 {
            acoes_com_participacoes += 1;
        }
        obj.insert(K_PARTICIPACOES.into(), Json::from(parts_out));
        obj.insert(K_TOTAL_PARTICIPACOES.into(), Json::from(total_parts));
        acoes_out.push(Json::Object(obj));
    }

    // Contadores derivados (C8) — em ordem, como no arquivo original.
    root.insert("total_acoes".into(), Json::from(acoes_out.len()));
    root.insert(
        "acoes_com_participacoes".into(),
        Json::from(acoes_com_participacoes),
    );
    root.insert(
        "total_atividades".into(),
        Json::from(atividade_contextos.len()),
    );
    root.insert("total_publico_alvo".into(), Json::from(total_publico));
    root.insert("total_equipe".into(), Json::from(total_equipe));
    root.insert("acoes".into(), Json::from(acoes_out));

    Ok(serde_json::to_string_pretty(&Json::Object(root))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path as StdPath;

    /// Fixture T001: 3 ações (uma com participações dos dois tipos, uma sem,
    /// uma com chave de rótulo extra e campo ausente) + contadores coerentes.
    fn consolidated_fixture() -> String {
        let path = StdPath::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/src_consolidado_exemplo.json");
        std::fs::read_to_string(path).unwrap()
    }

    mod tempdir {
        use std::path::{Path, PathBuf};
        pub struct Dir(PathBuf);
        impl Dir {
            pub fn new() -> Self {
                let n = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let p = std::env::temp_dir().join(format!("rh-src-test-{n}"));
                std::fs::create_dir_all(&p).unwrap();
                Dir(p)
            }
            pub fn path(&self) -> &Path {
                &self.0
            }
        }
        impl Drop for Dir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    use tempdir::Dir;

    fn fixture() -> (Connection, Dir) {
        let dir = Dir::new();
        let conn = crate::db::open(&dir.path().join("hub.db")).unwrap();
        crate::db::migrate(&conn).unwrap();
        crate::db::seed_admin(&conn).unwrap();
        (conn, dir)
    }

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    /// Uma ação curada à mão, para provar que falhas de importação não
    /// destroem a base (FR-007).
    fn seed_curated(conn: &Connection) {
        conn.execute(
            "INSERT INTO src_acoes (acao_id, raw_json, titulo) VALUES ('1001', '{}', 'Curado à mão')",
            [],
        )
        .unwrap();
    }

    fn consolidated_with_acoes(acoes: &str) -> String {
        format!(
            r#"{{"campus":"Serra","total_acoes":1,"acoes_com_participacoes":0,"total_atividades":0,"total_publico_alvo":0,"total_equipe":0,"acoes":{acoes}}}"#
        )
    }

    // ---------- T018: validação (FR-007) ----------

    #[test]
    fn import_loads_exact_counts() {
        let (mut conn, dir) = fixture();
        let s = import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        assert_eq!(s.total_acoes, 3);
        assert_eq!(s.total_participacoes, 3);
        assert_eq!(s.acoes_com_participacoes, 1);
        assert_eq!(count(&conn, "src_acoes"), 3);
        assert_eq!(count(&conn, "src_participacoes"), 3);
        // Ação 1 tem 3 participações; ações 2 e 3 têm 0.
        let t1: i64 = conn
            .query_row(
                "SELECT total_participacoes FROM src_acoes WHERE acao_id = '1001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(t1, 3);
    }

    #[test]
    fn import_rejects_non_json_and_keeps_base() {
        let (mut conn, dir) = fixture();
        seed_curated(&conn);
        let err = import(&mut conn, "isto não é json", dir.path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");
        assert_eq!(count(&conn, "src_acoes"), 1, "base intacta após falha");
    }

    #[test]
    fn import_rejects_root_without_acoes() {
        let (mut conn, dir) = fixture();
        seed_curated(&conn);
        let err = import(&mut conn, r#"{"campus":"Serra"}"#, dir.path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");
        assert_eq!(count(&conn, "src_acoes"), 1, "base intacta após falha");
    }

    #[test]
    fn import_rejects_acao_without_id() {
        let (mut conn, dir) = fixture();
        seed_curated(&conn);
        let body = consolidated_with_acoes(r#"[{"Processo nº": "1/2025"}]"#);
        let err = import(&mut conn, &body, dir.path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");
        assert_eq!(count(&conn, "src_acoes"), 1, "base intacta após falha");
    }

    #[test]
    fn import_rejects_duplicate_acao_id() {
        let (mut conn, dir) = fixture();
        seed_curated(&conn);
        let body = consolidated_with_acoes(
            r#"[{"acao_id":"2001"},{"acao_id":"2001"}]"#,
        );
        let err = import(&mut conn, &body, dir.path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");
        assert_eq!(count(&conn, "src_acoes"), 1, "base intacta após falha");
    }

    #[test]
    fn import_rejects_participacoes_not_list() {
        let (mut conn, dir) = fixture();
        seed_curated(&conn);
        let body =
            consolidated_with_acoes(r#"[{"acao_id":"2001","participacoes":"x"}]"#);
        let err = import(&mut conn, &body, dir.path()).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");
        assert_eq!(count(&conn, "src_acoes"), 1, "base intacta após falha");
    }

    #[test]
    fn import_accepts_empty_base_and_reports_it() {
        let (mut conn, dir) = fixture();
        let s = import(
            &mut conn,
            r#"{"campus":"Serra","total_acoes":0,"acoes":[]}"#,
            dir.path(),
        )
        .unwrap();
        assert_eq!(s.total_acoes, 0);
        assert_eq!(count(&conn, "src_acoes"), 0);
        assert!(!s.replaced);
    }

    // ---------- T019: round-trip (SC-003, contrato C9) ----------

    #[test]
    fn export_of_untouched_import_reproduces_the_file_object_by_object() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();

        let out = export(&conn).unwrap();
        let exported: Json = serde_json::to_value(serde_json::from_str::<Json>(&out).unwrap()).unwrap();
        let original: Json = serde_json::from_str(&consolidated_fixture()).unwrap();

        // Igualdade objeto a objeto, incluindo a ORDEM das chaves: se os dois
        // parseamentos preservam a ordem (preserve_order) e a remontagem
        // respeita raw_json, as serializações compactas são idênticas.
        assert_eq!(
            serde_json::to_string(&exported).unwrap(),
            serde_json::to_string(&original).unwrap(),
            "export de base intacta deve reproduzir o arquivo campo a campo"
        );
    }

    // ---------- T020: snapshot / substituição (FR-006, FR-015) ----------

    #[test]
    fn first_import_has_no_snapshot() {
        let (mut conn, dir) = fixture();
        let s = import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        assert!(s.snapshot.is_none(), "base vazia: nada a proteger");
        assert!(!s.replaced);
    }

    #[test]
    fn src_import_snapshots_before_replacing() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        // Curadoria entre importações:
        conn.execute(
            "UPDATE src_acoes SET titulo = 'Curado à mão' WHERE acao_id = '1001'",
            [],
        )
        .unwrap();

        let s = import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let snap = s.snapshot.expect("importação sobre base com curadoria cria snapshot");
        assert!(
            Path::new(&snap)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("src-"),
            "snapshot do SRC usa o prefixo src- (FR-015): {snap}"
        );

        let restored = Connection::open(&snap).unwrap();
        let titulo: String = restored
            .query_row(
                "SELECT titulo FROM src_acoes WHERE acao_id = '1001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(titulo, "Curado à mão", "o snapshot contém o estado anterior");
    }

    #[test]
    fn src_import_replaces_never_merges() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();

        let menor = consolidated_with_acoes(r#"[{"acao_id":"2001","Título ação":"Outra"}]"#);
        import(&mut conn, &menor, dir.path()).unwrap();

        assert_eq!(count(&conn, "src_acoes"), 1, "substituição, nunca mescla");
        let sobrou: String = conn
            .query_row("SELECT acao_id FROM src_acoes", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sobrou, "2001");
    }

    #[test]
    fn admins_survive_the_src_import() {
        let (mut conn, dir) = fixture();
        let before: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let after: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();
        assert_eq!(before, after);
        assert_eq!(count(&conn, "researchers"), 0, "Horizon intocado");
    }

    // ---------- T029: CRUD de ações (FR-010/FR-011) ----------

    fn rec(pairs: &[(&str, Json)]) -> Record {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    fn acao_row_id(conn: &Connection, acao_id: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM src_acoes WHERE acao_id = ?1",
            [acao_id],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn create_acao_builds_raw_json_with_canonical_label_keys() {
        let (conn, _dir) = fixture();
        let id = create_acao(
            &conn,
            &rec(&[
                ("acao_id", Json::from("3001")),
                ("processo", Json::from("0123.777/2025")),
                ("titulo", Json::from("Ação Criada na Curadoria")),
                ("natureza", Json::from("Extensão")),
                ("resumo", Json::from("Resumo com acentuação: ação.")),
            ]),
        )
        .unwrap();

        let raw: String = conn
            .query_row("SELECT raw_json FROM src_acoes WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        let obj: Json = raw.parse().unwrap();
        assert_eq!(obj["acao_id"], "3001", "identificador é chave canônica");
        assert_eq!(obj["Processo nº"], "0123.777/2025", "chave de rótulo do arquivo");
        assert_eq!(obj["Título ação"], "Ação Criada na Curadoria");
        assert_eq!(obj["Resumo"], "Resumo com acentuação: ação.");
        assert_eq!(obj["Natureza"], "Extensão");
    }

    #[test]
    fn create_acao_requires_unique_non_empty_acao_id() {
        let (conn, _dir) = fixture();
        let payload = rec(&[("acao_id", Json::from("3001")), ("titulo", Json::from("A"))]);
        create_acao(&conn, &payload).unwrap();

        let dup = create_acao(&conn, &payload).unwrap_err();
        assert!(
            matches!(dup, AppError::Validation(ref m) if m.to_lowercase().contains("duplic")),
            "erro: {dup:?}"
        );

        let vazio = create_acao(&conn, &rec(&[("acao_id", Json::from("  "))])).unwrap_err();
        assert!(matches!(vazio, AppError::Validation(_)), "erro: {vazio:?}");

        let ausente = create_acao(&conn, &rec(&[("titulo", Json::from("A"))])).unwrap_err();
        assert!(matches!(ausente, AppError::Validation(_)), "erro: {ausente:?}");
    }

    #[test]
    fn update_acao_syncs_projection_and_raw_json() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1001");

        update_acao(
            &conn,
            id,
            &rec(&[("titulo", Json::from("Título Editado à Mão"))]),
        )
        .unwrap();

        let (titulo, raw): (String, String) = conn
            .query_row(
                "SELECT titulo, raw_json FROM src_acoes WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(titulo, "Título Editado à Mão", "projeção atualizada");
        let obj: Json = raw.parse().unwrap();
        assert_eq!(obj["Título ação"], "Título Editado à Mão", "raw_json sincronizado");
        assert_eq!(obj["Natureza"], "Extensão", "campos não tocados intactos");
    }

    #[test]
    fn update_acao_never_changes_the_identifier() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1001");

        let err = update_acao(
            &conn,
            id,
            &rec(&[("acao_id", Json::from("9999"))]),
        )
        .unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");

        let ainda: String = conn
            .query_row("SELECT acao_id FROM src_acoes WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(ainda, "1001", "identificador preservado (Assumptions)");
    }

    #[test]
    fn update_acao_rejects_unknown_fields() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1001");
        let err = update_acao(&conn, id, &rec(&[("coluna_inventada", Json::from("x"))])).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");

        // O contador é derivado das linhas (R4): o editor o carrega no payload
        // do formulário e ele deve ser ignorado silenciosamente, nunca rejeitado.
        update_acao(
            &conn,
            id,
            &rec(&[
                ("total_participacoes", Json::from(99)),
                ("titulo", Json::from("Título Válido")),
            ]),
        )
        .unwrap();
        let (titulo, total): (String, i64) = conn
            .query_row(
                "SELECT titulo, total_participacoes FROM src_acoes WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(titulo, "Título Válido");
        assert_eq!(total, 3, "contador derivado segue as linhas, não o payload");
    }

    #[test]
    fn list_acoes_searches_sorts_and_paginates() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();

        // Busca por título (search_column), acento e maiúsculas à parte.
        let page = list_acoes(&conn, None, None, Some("robótica"), None, None).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0]["acao_id"], Json::from("1002"));

        // Ordenação por título asc.
        let page = list_acoes(&conn, None, None, None, Some("titulo"), Some("asc")).unwrap();
        assert_eq!(page.items[0]["titulo"], Json::from("Alfabetização Digital no Campo"));

        // Paginação.
        let page = list_acoes(&conn, Some(2), Some(1), None, None, None).unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 2);

        // Coluna de ordenação inválida → erro tipado, não SQL injetado.
        let err = list_acoes(&conn, None, None, None, Some("raw_json; DROP TABLE"), None).unwrap_err();
        assert!(matches!(err, AppError::InvalidColumn(_)), "erro: {err:?}");
    }

    #[test]
    fn get_acao_returns_projections_and_resumo() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1001");

        let row = get_acao(&conn, id).unwrap();
        assert_eq!(row["titulo"], Json::from("Alfabetização Digital no Campo"));
        assert_eq!(row["processo"], Json::from("0123.456/2025"));
        assert_eq!(
            row["resumo"],
            Json::from("Projeto de extensão com acentuação: ação, coração, educação."),
            "resumo extraído do raw_json"
        );
        assert!(row.get("raw_json").is_none(), "raw_json não atravessa a fronteira");
    }

    #[test]
    fn deleting_acao_referenced_as_vinculante_conflicts_until_forced() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let pai = acao_row_id(&conn, "1001"); // ação 3 vincula pelo processo dela

        let err = delete_acao(&conn, pai, false).unwrap_err();
        assert!(
            matches!(err, AppError::Conflict(ref m) if m.contains("1003")),
            "conflito deve listar as filhas: {err:?}"
        );

        delete_acao(&conn, pai, true).unwrap();
        let pai_existe: i64 = conn
            .query_row("SELECT COUNT(*) FROM src_acoes WHERE id = ?1", [pai], |r| r.get(0))
            .unwrap();
        assert_eq!(pai_existe, 0);

        let vinculo: Option<String> = conn
            .query_row(
                "SELECT acao_vinculante FROM src_acoes WHERE acao_id = '1003'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(vinculo, None, "filha desvinculada explicitamente, sem órfã silenciosa");

        // E o raw_json da filha também perde o vínculo (o export sai consistente).
        let raw: String = conn
            .query_row("SELECT raw_json FROM src_acoes WHERE acao_id = '1003'", [], |r| r.get(0))
            .unwrap();
        let obj: Json = raw.parse().unwrap();
        assert!(obj.get(K_VINCULANTE).map(|v| v.as_str() == Some("")).unwrap_or(true) || obj.get(K_VINCULANTE).is_none(),
            "raw_json da filha sem o vínculo removido: {raw}");
    }

    #[test]
    fn delete_acao_without_children_just_deletes() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        delete_acao(&conn, acao_row_id(&conn, "1002"), false).unwrap();
        assert_eq!(count(&conn, "src_acoes"), 2);
    }

    // ---------- T030: CRUD de participações (FR-012) ----------

    #[test]
    fn create_participacao_appends_with_canonical_keys_and_updates_count() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1002"); // ação sem participações

        let first = create_participacao(
            &conn,
            id,
            &rec(&[
                ("tipo", Json::from(TIPO_PUBLICO)),
                ("atividade_num", Json::from("1")),
                ("atividade", Json::from("Turma única")),
                ("Nome", Json::from("Participante Novo")),
                ("CPF", Json::from("000.000.000-00")),
            ]),
        )
        .unwrap();

        let raw: String = conn
            .query_row(
                "SELECT raw_json, (SELECT total_participacoes FROM src_acoes WHERE id = ?2) \
                 FROM src_participacoes WHERE id = ?1",
                rusqlite::params![first, id],
                |r| r.get(0),
            )
            .unwrap();
        let obj: Json = raw.parse().unwrap();
        assert_eq!(obj["tipo"], TIPO_PUBLICO);
        assert_eq!(obj["Nome"], "Participante Novo");
        assert_eq!(obj["CPF"], "000.000.000-00");

        let (ord, total): (i64, i64) = conn
            .query_row(
                "SELECT ord, (SELECT total_participacoes FROM src_acoes WHERE id = ?2) \
                 FROM src_participacoes WHERE id = ?1",
                rusqlite::params![first, id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(ord, 1, "primeira participação da ação: ord = 1");
        assert_eq!(total, 1, "total_participacoes da ação sincronizado");

        create_participacao(&conn, id, &rec(&[("tipo", Json::from(TIPO_EQUIPE))])).unwrap();
        let total: i64 = conn
            .query_row("SELECT total_participacoes FROM src_acoes WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2);
    }

    #[test]
    fn create_participacao_requires_tipo() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1002");
        let err = create_participacao(&conn, id, &rec(&[("Nome", Json::from("X"))])).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)), "erro: {err:?}");
    }

    #[test]
    fn update_participacao_merges_into_raw_json_preserving_unknown_keys() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();

        // A primeira participação do arquivo (Público-alvo) tem CPF/E-mail/Situação.
        let pid: i64 = conn
            .query_row(
                "SELECT p.id FROM src_participacoes p \
                 JOIN src_acoes a ON a.id = p.acao_row_id \
                 WHERE a.acao_id = '1001' ORDER BY p.ord LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();

        update_participacao(
            &conn,
            pid,
            &rec(&[
                ("Nome", Json::from("Nome Editado")),
                ("Situação", Json::from("APROVADO")),
            ]),
        )
        .unwrap();

        let raw: String = conn
            .query_row("SELECT raw_json FROM src_participacoes WHERE id = ?1", [pid], |r| r.get(0))
            .unwrap();
        let obj: Json = raw.parse().unwrap();
        assert_eq!(obj["Nome"], "Nome Editado", "campo editado");
        assert_eq!(obj["Situação"], "APROVADO", "campo adicionado");
        assert_eq!(obj["CPF"], "123.456.789-00", "chave de rótulo não tocada preservada (C5)");
        assert_eq!(obj["tipo"], TIPO_PUBLICO);

        let nome: String = conn
            .query_row("SELECT nome FROM src_participacoes WHERE id = ?1", [pid], |r| r.get(0))
            .unwrap();
        assert_eq!(nome, "Nome Editado", "projeção sincronizada");
    }

    #[test]
    fn delete_participacao_keeps_ord_contiguous_and_count_synced() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1002");

        let a = create_participacao(&conn, id, &rec(&[("tipo", Json::from(TIPO_PUBLICO))])).unwrap();
        let b = create_participacao(&conn, id, &rec(&[("tipo", Json::from(TIPO_PUBLICO))])).unwrap();
        let c = create_participacao(&conn, id, &rec(&[("tipo", Json::from(TIPO_EQUIPE))])).unwrap();

        delete_participacao(&conn, b).unwrap();

        let ords: Vec<i64> = {
            let mut stmt = conn
                .prepare("SELECT ord FROM src_participacoes WHERE acao_row_id = ?1 ORDER BY ord")
                .unwrap();
            stmt.query_map([id], |r| r.get(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        assert_eq!(ords, vec![1, 2], "ord permanece contígua após exclusão");
        let total: i64 = conn
            .query_row("SELECT total_participacoes FROM src_acoes WHERE id = ?1", [id], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 2);
        let _ = (a, c);
    }

    #[test]
    fn meta_roundtrip_feeds_the_export() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();

        let meta = get_meta(&conn).unwrap();
        assert_eq!(meta["campus"], Json::from("Serra"));

        update_meta(&conn, Some("Vitória")).unwrap();
        let out = export(&conn).unwrap();
        let obj: Json = out.parse().unwrap();
        assert_eq!(obj["campus"], "Vitória", "campus do meta alimenta o export");
    }

    #[test]
    fn list_participacoes_exposes_person_label_keys() {
        let (mut conn, dir) = fixture();
        import(&mut conn, &consolidated_fixture(), dir.path()).unwrap();
        let id = acao_row_id(&conn, "1001");

        let rows = list_participacoes(&conn, id).unwrap();
        let primeira = &rows[0];
        // O editor precisa dos campos da pessoa com os rótulos do arquivo (C5).
        assert_eq!(primeira["Nome"], Json::from("João Pedro Alves"));
        assert_eq!(primeira["CPF"], Json::from("123.456.789-00"));
        assert_eq!(primeira["E-mail"], Json::from("joao@example.com"));
        assert_eq!(primeira["tipo"], Json::from(TIPO_PUBLICO));
        // Projeções continuam presentes para a ordenação/agrupamento da UI.
        assert!(primeira.get("ord").is_some());
        assert!(primeira.get("id").is_some());
    }

    // ---------- T041: desempenho (SC-006) ----------

    #[test]
    fn import_large_consolidado_under_30s_and_paginates() {
        let path = StdPath::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/src_consolidado_grande.json");
        let Ok(text) = std::fs::read_to_string(&path) else {
            // A fixture nasce do script descartável (generate_large_consolidado.py)
            // e não é versionada; em clones frescos a verificação roda após gerá-la.
            eprintln!("fixture grande ausente; gere com generate_large_consolidado.py");
            return;
        };

        let (mut conn, dir) = fixture();
        let start = std::time::Instant::now();
        let summary = import(&mut conn, &text, dir.path()).unwrap();
        let elapsed = start.elapsed();
        assert_eq!(summary.total_acoes, 2000);
        assert_eq!(summary.total_participacoes, 10_000);
        assert!(
            elapsed.as_secs() < 30,
            "import levou {elapsed:?} (SC-006 pede < 30 s)"
        );

        // A listagem pagina sem carregar tudo (SC-006).
        let page = list_acoes(&conn, Some(50), Some(0), None, None, None).unwrap();
        assert_eq!(page.items.len(), 50);
        assert_eq!(page.total, 2000);

        // E o export continua coerente com as linhas (C8) na base grande.
        let out = export(&conn).unwrap();
        let obj: Json = out.parse().unwrap();
        assert_eq!(obj["total_acoes"], 2000);
        assert_eq!(obj["total_publico_alvo"], 5000);
        assert_eq!(obj["total_equipe"], 5000);
    }
}
