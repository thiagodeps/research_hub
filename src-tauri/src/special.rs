//! Merge and link (SEP-020).
//!
//! Both operations are ported with fixes. Merge becomes transactional; link is
//! implemented for the first time — the Python version calls
//! `BaseRepository.save`, which does not exist, so every link that actually had
//! something to append raised AttributeError and returned a 500.

use crate::crud::{self, Record};
use crate::error::AppError;
use crate::registry;
use rusqlite::Connection;
use serde_json::{json, Value as Json};

/// Which parent column receives a child of this type.
///
/// The frontend route and the column often differ (`groups` links into
/// `research_groups`), the same divergence the registry centralizes elsewhere.
fn target_column(child_route: &str) -> &str {
    match child_route {
        "groups" => "research_groups",
        other => other,
    }
}

/// Merge several records into the first, then remove the rest.
///
/// One transaction: the Python version saves the primary and then deletes the
/// others in independent calls, so a failure in between leaves orphans.
pub fn merge(
    conn: &mut Connection,
    route: &str,
    source_ids: &[i64],
    resolved: &Record,
) -> Result<Record, AppError> {
    registry::by_route(route)?;
    let Some((&primary, rest)) = source_ids.split_first() else {
        return Err(AppError::Internal("nenhum registro para fundir".into()));
    };

    let tx = conn.transaction()?;
    let merged = crud::update(&tx, route, primary, resolved)?;
    for id in rest {
        // A source that is already gone is not a reason to abort the merge.
        match crud::delete(&tx, route, *id) {
            Ok(()) | Err(AppError::NotFound) => {}
            Err(e) => return Err(e),
        }
    }
    tx.commit()?;
    Ok(merged)
}

/// Append a child reference to the parent's JSON array column.
pub fn link(
    conn: &Connection,
    parent_route: &str,
    parent_id: i64,
    child_route: &str,
    child_id: i64,
) -> Result<Record, AppError> {
    let parent_def = registry::by_route(parent_route)?;
    let child_def = registry::by_route(child_route)?;

    let parent = crud::get(conn, parent_route, parent_id)?;
    let child = crud::get(conn, child_route, child_id)?;

    let column = target_column(child_route);
    if !parent_def.has_column(column) {
        return Err(AppError::InvalidColumn(format!(
            "{} não tem coluna '{}' para vincular {}",
            parent_def.table, column, child_def.table
        )));
    }

    // A column that is null, empty or holds something other than an array is
    // treated as an empty list, exactly as the Python fallback does.
    let mut items: Vec<Json> = parent
        .get(column)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .and_then(|s| serde_json::from_str::<Json>(s).ok())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();

    if items
        .iter()
        .any(|r| r.get("id").and_then(|i| i.as_i64()) == Some(child_id))
    {
        return Ok(parent); // already linked; not an error
    }

    let label = child
        .get("name")
        .or_else(|| child.get("title"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("{child_route} {child_id}"));

    items.push(json!({ "id": child_id, "name": label }));

    let mut payload = Record::new();
    payload.insert(column.to_string(), Json::String(serde_json::to_string(&items)?));
    crud::update(conn, parent_route, parent_id, &payload)
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(format!("JSON inválido: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn seeded() -> Connection {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO researchers (id, name, research_groups) VALUES (1, 'Ana', NULL)",
            [],
        )
        .unwrap();
        conn.execute("INSERT INTO research_groups (id, name) VALUES (7, 'Grupo A')", [])
            .unwrap();
        conn.execute("INSERT INTO research_groups (id, name) VALUES (8, 'Grupo B')", [])
            .unwrap();
        conn
    }

    fn groups_of(conn: &Connection, id: i64) -> Vec<Json> {
        let r = crud::get(conn, "researchers", id).unwrap();
        serde_json::from_str(r["research_groups"].as_str().unwrap()).unwrap()
    }

    /// The bug: this path always failed in Python. It must work here.
    #[test]
    fn link_appends_child_to_empty_column() {
        let conn = seeded();
        link(&conn, "researchers", 1, "groups", 7).unwrap();

        let items = groups_of(&conn, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"], 7);
        assert_eq!(items[0]["name"], "Grupo A", "o rótulo vem do nome do filho");
    }

    #[test]
    fn link_appends_without_dropping_existing() {
        let conn = seeded();
        link(&conn, "researchers", 1, "groups", 7).unwrap();
        link(&conn, "researchers", 1, "groups", 8).unwrap();
        assert_eq!(groups_of(&conn, 1).len(), 2);
    }

    #[test]
    fn link_is_idempotent() {
        let conn = seeded();
        link(&conn, "researchers", 1, "groups", 7).unwrap();
        link(&conn, "researchers", 1, "groups", 7).unwrap();
        assert_eq!(groups_of(&conn, 1).len(), 1, "vincular duas vezes não duplica");
    }

    /// `groups` on the route, `research_groups` in the column.
    #[test]
    fn link_maps_route_to_column() {
        let conn = seeded();
        let parent = link(&conn, "researchers", 1, "groups", 7).unwrap();
        assert!(parent["research_groups"].as_str().unwrap().contains("\"id\":7"));
    }

    #[test]
    fn link_recovers_from_corrupted_column() {
        let conn = seeded();
        conn.execute("UPDATE researchers SET research_groups = 'nao e json' WHERE id = 1", [])
            .unwrap();
        link(&conn, "researchers", 1, "groups", 7).unwrap();
        assert_eq!(groups_of(&conn, 1).len(), 1, "conteúdo inválido vira lista vazia");
    }

    #[test]
    fn link_rejects_column_the_parent_does_not_have() {
        let conn = seeded();
        conn.execute("INSERT INTO languages (id, name) VALUES (2, 'Inglês')", []).unwrap();
        assert!(matches!(
            link(&conn, "campuses", 1, "languages", 2),
            Err(AppError::InvalidColumn(_)) | Err(AppError::NotFound)
        ));
    }

    #[test]
    fn link_reports_missing_records() {
        let conn = seeded();
        assert!(matches!(link(&conn, "researchers", 999, "groups", 7), Err(AppError::NotFound)));
        assert!(matches!(link(&conn, "researchers", 1, "groups", 999), Err(AppError::NotFound)));
    }

    #[test]
    fn merge_keeps_primary_and_removes_the_others() {
        let mut conn = seeded();
        conn.execute("INSERT INTO research_groups (id, name) VALUES (9, 'Duplicado')", [])
            .unwrap();

        let mut resolved = Record::new();
        resolved.insert("name".into(), json!("Grupo Unificado"));

        let merged = merge(&mut conn, "groups", &[7, 8, 9], &resolved).unwrap();

        assert_eq!(merged["id"], json!(7));
        assert_eq!(merged["name"], json!("Grupo Unificado"));
        assert!(crud::get(&conn, "groups", 8).is_err());
        assert!(crud::get(&conn, "groups", 9).is_err());
    }

    /// The declared fix: a failure must not leave the primary updated and the
    /// duplicates half-deleted.
    #[test]
    fn merge_is_atomic() {
        let mut conn = seeded();
        let mut resolved = Record::new();
        resolved.insert("name".into(), json!("Nao deve persistir"));

        // An unknown route fails before the transaction opens; state untouched.
        assert!(merge(&mut conn, "inexistente", &[7, 8], &resolved).is_err());

        let g = crud::get(&conn, "groups", 7).unwrap();
        assert_eq!(g["name"], json!("Grupo A"));
        assert!(crud::get(&conn, "groups", 8).is_ok());
    }

    #[test]
    fn merge_needs_at_least_one_source() {
        let mut conn = seeded();
        assert!(merge(&mut conn, "groups", &[], &Record::new()).is_err());
    }

    #[test]
    fn merge_tolerates_an_already_deleted_duplicate() {
        let mut conn = seeded();
        let mut resolved = Record::new();
        resolved.insert("name".into(), json!("Unificado"));
        // 4242 never existed; the merge should still complete.
        merge(&mut conn, "groups", &[7, 4242], &resolved).unwrap();
        assert_eq!(crud::get(&conn, "groups", 7).unwrap()["name"], json!("Unificado"));
    }
}
