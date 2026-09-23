//! Generic CRUD over the entity registry (SEP-017).
//!
//! Mirrors `DatabasePostgresAdapter` + `src/api/crud.py`. Identifiers always
//! come from the registry; values always travel as bound parameters.

use crate::error::AppError;
use crate::registry::{self, EntityDef};
use rusqlite::{types::Value as SqlValue, Connection};
use serde_json::{Map, Value as Json};

pub type Record = Map<String, Json>;

#[derive(serde::Serialize)]
pub struct Page {
    pub items: Vec<Record>,
    pub total: i64,
}

const DEFAULT_LIMIT: i64 = 100;
const MAX_LIMIT: i64 = 1000;

/// `%` and `_` are LIKE wildcards. Left unescaped, searching for "100%" matches
/// every row — which is what the Python version does today. Declared correction
/// (FR-006); `\` is the escape character, so it must be escaped first.
fn escape_like(term: &str) -> String {
    term.replace('\\', r"\\").replace('%', r"\%").replace('_', r"\_")
}

/// A search term counts as an id only when it is purely ASCII digits and
/// parses as i64 (030 research D1 / FR-002): `parse` alone would accept "-3"
/// and "+7", and a 30-digit number overflows — every such term degrades to
/// plain text search (FR-005). "007" normalizes to 7, as the spec assumes.
fn parse_id_term(term: &str) -> Option<i64> {
    if !term.is_empty() && term.chars().all(|c| c.is_ascii_digit()) {
        term.parse::<i64>().ok()
    } else {
        None
    }
}

fn to_json(v: SqlValue) -> Json {
    match v {
        SqlValue::Null => Json::Null,
        SqlValue::Integer(i) => Json::from(i),
        SqlValue::Real(f) => Json::from(f),
        SqlValue::Text(s) => Json::from(s),
        SqlValue::Blob(_) => Json::Null,
    }
}

/// Everything but `id` is TEXT, so non-strings are stringified rather than
/// rejected — the interface sends numbers for year fields, for instance.
fn to_sql(column: &str, v: &Json) -> SqlValue {
    match v {
        Json::Null => SqlValue::Null,
        Json::String(s) => SqlValue::Text(s.clone()),
        Json::Number(n) if column == "id" => n
            .as_i64()
            .map(SqlValue::Integer)
            .unwrap_or_else(|| SqlValue::Text(n.to_string())),
        other => SqlValue::Text(match other {
            Json::String(s) => s.clone(),
            v => v.to_string(),
        }),
    }
}

fn row_to_record(row: &rusqlite::Row, columns: &[&str]) -> rusqlite::Result<Record> {
    let mut rec = Map::with_capacity(columns.len());
    for (i, name) in columns.iter().enumerate() {
        rec.insert((*name).to_string(), to_json(row.get::<_, SqlValue>(i)?));
    }
    Ok(rec)
}

/// Keep only registry columns; silently drop anything else (FR-010).
fn sanitize<'a>(def: &EntityDef, payload: &'a Record) -> Vec<(&'static str, &'a Json)> {
    def.columns
        .iter()
        .filter_map(|c| payload.get(*c).map(|v| (*c, v)))
        .collect()
}

pub fn list(
    conn: &Connection,
    route: &str,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&str>,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<Page, AppError> {
    let def = registry::by_route(route)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);

    // Trimmed before classification (030 research D6); empty after trim means
    // no filter at all, as before.
    let term = search.map(str::trim).filter(|t| !t.is_empty());
    // Numeric terms additionally match the exact id (FR-002/FR-003).
    let id = term.and_then(parse_id_term);
    // The LIKE pattern is byte-for-byte the old one (FR-005): %term% with
    // wildcards escaped as literal text.
    let like = term.map(|t| format!("%{}%", escape_like(t)));

    // WHERE shapes per the 030 data-model grammar (T1–T4). Explicit `?N`
    // numbering lets the relevance ORDER BY reuse `?1` (research D2/D3).
    let (where_sql, bind): (String, Vec<SqlValue>) = match (id, &like, def.search_column) {
        // Numeric in an entity with a searchable column: id OR text (FR-004).
        (Some(id_val), Some(pattern), Some(col)) => (
            format!(" WHERE (id = ?1 OR {col} LIKE ?2 ESCAPE '\\')"),
            vec![SqlValue::Integer(id_val), SqlValue::Text(pattern.clone())],
        ),
        // Numeric in an entity without one: the id is all there is (FR-006).
        (Some(id_val), _, None) => (
            " WHERE id = ?1".to_string(),
            vec![SqlValue::Integer(id_val)],
        ),
        // Text-only: exactly today's filter, unchanged (FR-005).
        (None, Some(pattern), Some(col)) => (
            format!(" WHERE {col} LIKE ?1 ESCAPE '\\'"),
            vec![SqlValue::Text(pattern.clone())],
        ),
        // No usable term (also covers text terms on no-column entities,
        // which stay ignored — FR-007 parity).
        _ => (String::new(), Vec::new()),
    };
    // Unknown sort columns are ignored, not rejected — parity with Python.
    // Relevance (when the term is numeric) leads; the user's sort applies to
    // the remaining rows, nulls last in both directions.
    let mut order_clauses: Vec<String> = Vec::new();
    if id.is_some() {
        order_clauses.push("(id = ?1) DESC".to_string());
    }
    if let Some(col) = sort.filter(|s| def.has_column(s)) {
        let expr = EntityDef::sort_expr(col);
        let dir = if order == Some("desc") { "DESC" } else { "ASC" };
        // `expr IS NULL` first puts nulls last in both directions.
        order_clauses.push(format!("({expr} IS NULL), {expr} {dir}"));
    }
    let order_sql = if order_clauses.is_empty() {
        String::new()
    } else {
        format!(" ORDER BY {}", order_clauses.join(", "))
    };

    let table = def.table;
    let total: i64 = if bind.is_empty() {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?
    } else {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM {table}{where_sql}"),
            rusqlite::params_from_iter(bind.iter()),
            |r| r.get(0),
        )?
    };

    let cols = def.columns.join(", ");
    let sql = format!("SELECT {cols} FROM {table}{where_sql}{order_sql} LIMIT ? OFFSET ?");
    let mut stmt = conn.prepare(&sql)?;

    // Bare `?` in LIMIT/OFFSET take the next numbers after the highest
    // explicit one, so the values simply follow the WHERE bindings.
    let mut params: Vec<SqlValue> = bind.clone();
    params.push(SqlValue::Integer(limit));
    params.push(SqlValue::Integer(offset));
    let items = stmt
        .query_map(rusqlite::params_from_iter(params), |r| {
            row_to_record(r, def.columns)
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Page { items, total })
}

pub fn get(conn: &Connection, route: &str, id: i64) -> Result<Record, AppError> {
    let def = registry::by_route(route)?;
    let sql = format!(
        "SELECT {} FROM {} WHERE id = ?1",
        def.columns.join(", "),
        def.table
    );
    conn.query_row(&sql, [id], |r| row_to_record(r, def.columns))
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => AppError::NotFound,
            other => other.into(),
        })
}

pub fn create(conn: &Connection, route: &str, payload: &Record) -> Result<Record, AppError> {
    let def = registry::by_route(route)?;
    let fields = sanitize(def, payload);

    // An explicit id must survive (parquet import relies on it, FR-011), but an
    // empty one from the "new record" form must not become a literal NULL id.
    let fields: Vec<_> = fields
        .into_iter()
        .filter(|(c, v)| !(*c == "id" && (v.is_null() || v == &&Json::from(""))))
        .collect();

    if fields.is_empty() {
        conn.execute(&format!("INSERT INTO {} DEFAULT VALUES", def.table), [])?;
    } else {
        let names: Vec<&str> = fields.iter().map(|(c, _)| *c).collect();
        let holes: Vec<String> = (1..=fields.len()).map(|i| format!("?{i}")).collect();
        let values: Vec<SqlValue> = fields.iter().map(|(c, v)| to_sql(c, v)).collect();
        conn.execute(
            &format!(
                "INSERT INTO {} ({}) VALUES ({})",
                def.table,
                names.join(", "),
                holes.join(", ")
            ),
            rusqlite::params_from_iter(values),
        )?;
    }

    get(conn, route, conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    route: &str,
    id: i64,
    payload: &Record,
) -> Result<Record, AppError> {
    let def = registry::by_route(route)?;
    // Confirm existence first so a no-op update still reports not-found.
    get(conn, route, id)?;

    // The path id wins over any id in the body (parity with FastAPI).
    let fields: Vec<_> = sanitize(def, payload)
        .into_iter()
        .filter(|(c, _)| *c != "id")
        .collect();

    if !fields.is_empty() {
        let assignments: Vec<String> = fields
            .iter()
            .enumerate()
            .map(|(i, (c, _))| format!("{c} = ?{}", i + 1))
            .collect();
        let mut values: Vec<SqlValue> = fields.iter().map(|(c, v)| to_sql(c, v)).collect();
        values.push(SqlValue::Integer(id));
        conn.execute(
            &format!(
                "UPDATE {} SET {} WHERE id = ?{}",
                def.table,
                assignments.join(", "),
                values.len()
            ),
            rusqlite::params_from_iter(values),
        )?;
    }

    get(conn, route, id)
}

pub fn delete(conn: &Connection, route: &str, id: i64) -> Result<(), AppError> {
    let def = registry::by_route(route)?;
    let affected = conn.execute(
        &format!("DELETE FROM {} WHERE id = ?1", def.table),
        [id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use serde_json::json;

    fn rec(v: Json) -> Record {
        v.as_object().unwrap().clone()
    }

    fn seeded() -> Connection {
        let conn = db::open_in_memory().unwrap();
        for (i, name) in ["Serra", "Vitoria", "Cariacica", "Alegre"].iter().enumerate() {
            conn.execute(
                "INSERT INTO campuses (id, name, short_name) VALUES (?1, ?2, ?3)",
                rusqlite::params![i as i64 + 1, name, name.to_lowercase()],
            )
            .unwrap();
        }
        conn
    }

    #[test]
    fn lists_with_total_independent_of_page() {
        let conn = seeded();
        let page = list(&conn, "campuses", Some(2), Some(0), None, None, None).unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.total, 4, "total must count all matches, not the page");
    }

    #[test]
    fn paginates_with_offset() {
        let conn = seeded();
        let p1 = list(&conn, "campuses", Some(2), Some(0), None, Some("id"), None).unwrap();
        let p2 = list(&conn, "campuses", Some(2), Some(2), None, Some("id"), None).unwrap();
        assert_eq!(p1.items[0]["name"], json!("Serra"));
        assert_eq!(p2.items[0]["name"], json!("Cariacica"));
    }

    #[test]
    fn clamps_limit_and_offset() {
        let conn = seeded();
        assert_eq!(
            list(&conn, "campuses", Some(99_999), Some(-5), None, None, None)
                .unwrap()
                .items
                .len(),
            4
        );
        assert_eq!(
            list(&conn, "campuses", Some(0), None, None, None, None).unwrap().items.len(),
            1
        );
    }

    #[test]
    fn searches_case_insensitively() {
        let conn = seeded();
        let page = list(&conn, "campuses", None, None, Some("serra"), None, None).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0]["name"], json!("Serra"));
    }

    /// SC-006 / FR-006: the declared correction. Unescaped, this returns every row.
    #[test]
    fn search_treats_wildcards_as_literal_text() {
        let conn = seeded();
        conn.execute("INSERT INTO campuses (name) VALUES ('Meta 100% batida')", [])
            .unwrap();

        let page = list(&conn, "campuses", None, None, Some("100%"), None, None).unwrap();
        assert_eq!(page.total, 1, "'%' must be literal, not a wildcard");

        let underscore = list(&conn, "campuses", None, None, Some("_"), None, None).unwrap();
        assert_eq!(underscore.total, 0, "'_' must be literal, not any-char");
    }

    /// FR-007: proficiencies has no name/title/username column.
    #[test]
    fn search_is_ignored_for_entities_without_search_column() {
        let conn = db::open_in_memory().unwrap();
        conn.execute("INSERT INTO proficiencies (reading) VALUES ('boa')", []).unwrap();
        let page = list(&conn, "proficiencies", None, None, Some("qualquer"), None, None).unwrap();
        assert_eq!(page.total, 1, "search must be ignored, not applied or failed");
    }

    /// US1/SC-003: a purely numeric term matches the exact id, and that record
    /// leads the list. '3' occurs in no campus name, so only the id matches.
    #[test]
    fn search_by_exact_id_returns_record_first() {
        let conn = seeded();
        let page = list(&conn, "campuses", None, None, Some("3"), None, None).unwrap();
        assert_eq!(page.total, 1, "the id match is exact, not a substring");
        assert_eq!(page.items[0]["id"], json!(3));
        assert_eq!(page.items[0]["name"], json!("Cariacica"));
    }

    /// Research D1: "003" is digits-only and parses to 3 — leading zeros
    /// normalize through the i64 parse.
    #[test]
    fn search_by_id_with_leading_zeros_matches_same_record() {
        let conn = seeded();
        let page = list(&conn, "campuses", None, None, Some("003"), None, None).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.items[0]["id"], json!(3));
    }

    /// US1 scenario 3: an id with no record yields an empty page, not an error.
    #[test]
    fn search_by_missing_id_returns_empty_page_without_error() {
        let conn = seeded();
        let page = list(&conn, "campuses", None, None, Some("9999999"), None, None).unwrap();
        assert!(page.items.is_empty());
        assert_eq!(page.total, 0);
    }

    /// US1 scenario 2 + research D6: a whitespace-only paste must not become a
    /// LIKE filter; empty-after-trim means no filter at all.
    #[test]
    fn empty_or_whitespace_search_lists_everything() {
        let conn = seeded();
        let none = list(&conn, "campuses", None, None, None, None, None).unwrap();
        assert_eq!(none.total, 4);
        let empty = list(&conn, "campuses", None, None, Some(""), None, None).unwrap();
        assert_eq!(empty.total, 4);
        let blanks = list(&conn, "campuses", None, None, Some("   "), None, None).unwrap();
        assert_eq!(blanks.total, 4, "whitespace-only must list everything");
    }

    /// FR-004: numeric terms are additive — the exact-id match AND the textual
    /// matches land in the same list, with the id row first (SC-003).
    #[test]
    fn numeric_search_returns_union_of_id_and_text_matches() {
        let conn = seeded();
        conn.execute("INSERT INTO campuses (id, name) VALUES (7, 'Campus Sete')", []).unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (5, 'Bloco 7')", []).unwrap();

        let page = list(&conn, "campuses", None, None, Some("7"), None, None).unwrap();
        assert_eq!(page.total, 2, "id 7 plus the name containing '7'");
        assert_eq!(page.items[0]["id"], json!(7), "the exact id match leads");
        assert!(page.items.iter().any(|r| r["name"] == json!("Bloco 7")));
    }

    /// FR-002 / data-model T4: anything that is not pure digits stays
    /// text-only — mixed terms, decimals, negatives and i64 overflow never
    /// match an id.
    #[test]
    fn non_numeric_terms_use_text_search_only() {
        let conn = seeded();
        conn.execute("INSERT INTO campuses (id, name) VALUES (7, 'Bloco abc123')", []).unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (8, 'Meta 12.5')", []).unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (9, 'Débito -3')", []).unwrap();

        let mixed = list(&conn, "campuses", None, None, Some("abc123"), None, None).unwrap();
        assert_eq!(mixed.total, 1, "mixed terms never match an id");
        assert_eq!(mixed.items[0]["name"], json!("Bloco abc123"));

        let decimal = list(&conn, "campuses", None, None, Some("12.5"), None, None).unwrap();
        assert_eq!(decimal.total, 1);
        assert_eq!(decimal.items[0]["name"], json!("Meta 12.5"));

        let negative = list(&conn, "campuses", None, None, Some("-3"), None, None).unwrap();
        assert_eq!(negative.total, 1);
        assert_eq!(negative.items[0]["name"], json!("Débito -3"));

        let overflow = list(&conn, "campuses", None, None, Some("999999999999999999999999999999"), None, None).unwrap();
        assert_eq!(overflow.total, 0, "overflow degrades to text, which matches nothing here");
    }

    /// Research D3: the id row leads even when the user chose a sort; the
    /// remaining rows follow that sort (asc and desc).
    #[test]
    fn user_sort_still_applies_after_id_relevance() {
        let conn = seeded();
        conn.execute("INSERT INTO campuses (id, name) VALUES (7, 'Campus Sete')", []).unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (5, 'Bloco 7')", []).unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (6, 'Sala 7')", []).unwrap();

        let asc = list(&conn, "campuses", None, None, Some("7"), Some("name"), Some("asc")).unwrap();
        assert_eq!(asc.items[0]["id"], json!(7), "id match still leads");
        let asc_names: Vec<&str> = asc.items[1..].iter().filter_map(|r| r["name"].as_str()).collect();
        assert_eq!(asc_names, vec!["Bloco 7", "Sala 7"], "rest follows the user's sort");

        let desc = list(&conn, "campuses", None, None, Some("7"), Some("name"), Some("desc")).unwrap();
        assert_eq!(desc.items[0]["id"], json!(7));
        let desc_names: Vec<&str> = desc.items[1..].iter().filter_map(|r| r["name"].as_str()).collect();
        assert_eq!(desc_names, vec!["Sala 7", "Bloco 7"]);
    }

    /// FR-006 / research D4: entities without a searchable text column gain
    /// id search; text-only terms remain fully ignored, so the existing
    /// `search_is_ignored_for_entities_without_search_column` keeps passing.
    #[test]
    fn numeric_search_filters_by_id_for_entities_without_search_column() {
        let conn = db::open_in_memory().unwrap();
        conn.execute("INSERT INTO proficiencies (reading) VALUES ('boa')", []).unwrap();
        let target = conn.last_insert_rowid();
        conn.execute("INSERT INTO proficiencies (reading) VALUES ('otima')", []).unwrap();

        let by_id = list(&conn, "proficiencies", None, None, Some(&target.to_string()), None, None).unwrap();
        assert_eq!(by_id.total, 1, "the id filter applies even without a text column");
        assert_eq!(by_id.items[0]["id"], json!(target));

        let by_text = list(&conn, "proficiencies", None, None, Some("qualquer"), None, None).unwrap();
        assert_eq!(by_text.total, 2, "textual terms stay ignored here");
    }

    /// SC-004 / US-3 scenario 2: the same rule everywhere in the registry —
    /// every exported entity returns exactly its own record when searched by
    /// its id.
    #[test]
    fn numeric_search_works_uniformly_across_all_entities() {
        let conn = db::open_in_memory().unwrap();
        for def in registry::exported() {
            let created = create(&conn, def.route, &rec(json!({})))
                .unwrap_or_else(|e| panic!("{}: create falhou: {e}", def.route));
            let id = created["id"].as_i64().unwrap();

            let page = list(&conn, def.route, None, None, Some(&id.to_string()), None, None)
                .unwrap_or_else(|e| panic!("{}: list by id falhou: {e}", def.route));
            assert_eq!(page.total, 1, "{}: id search returns exactly one record", def.route);
            assert_eq!(page.items[0]["id"], json!(id), "{}: the record itself comes back", def.route);
        }
    }

    #[test]
    fn sorts_both_directions_with_nulls_last() {
        let conn = seeded();
        conn.execute("INSERT INTO campuses (id, name) VALUES (9, NULL)", []).unwrap();

        let asc = list(&conn, "campuses", None, None, None, Some("name"), Some("asc")).unwrap();
        let desc = list(&conn, "campuses", None, None, None, Some("name"), Some("desc")).unwrap();

        assert_eq!(asc.items[0]["name"], json!("Alegre"));
        assert_eq!(asc.items.last().unwrap()["name"], Json::Null);
        assert_eq!(desc.items[0]["name"], json!("Vitoria"));
        assert_eq!(
            desc.items.last().unwrap()["name"],
            Json::Null,
            "nulls stay last when descending too"
        );
    }

    /// FR-005: JSON relationship columns order by link count.
    #[test]
    fn json_columns_sort_by_number_of_links() {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO researchers (name, research_groups) VALUES ('Tres', ?1)",
            [r#"[{"id":1},{"id":2},{"id":3}]"#],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO researchers (name, research_groups) VALUES ('Um', ?1)",
            [r#"[{"id":9}]"#],
        )
        .unwrap();

        let page = list(
            &conn, "researchers", None, None, None, Some("research_groups"), Some("desc"),
        )
        .unwrap();
        assert_eq!(
            page.items[0]["name"],
            json!("Tres"),
            "most links first, not lexicographic order"
        );
    }

    #[test]
    fn unknown_sort_column_is_ignored() {
        let conn = seeded();
        let page = list(&conn, "campuses", None, None, None, Some("nao_existe; DROP TABLE campuses"), None);
        assert_eq!(page.unwrap().total, 4);
    }

    #[test]
    fn create_read_update_delete_roundtrip() {
        let conn = seeded();

        let created = create(&conn, "campuses", &rec(json!({"name": "Novo", "campus": "x"}))).unwrap();
        let id = created["id"].as_i64().unwrap();
        assert_eq!(created["name"], json!("Novo"));

        let updated = update(&conn, "campuses", id, &rec(json!({"name": "Renomeado"}))).unwrap();
        assert_eq!(updated["name"], json!("Renomeado"));
        assert_eq!(updated["campus"], json!("x"), "unmentioned columns must survive");

        delete(&conn, "campuses", id).unwrap();
        assert!(matches!(get(&conn, "campuses", id), Err(AppError::NotFound)));
    }

    #[test]
    fn create_ignores_unknown_keys_and_empty_id() {
        let conn = seeded();
        let created = create(
            &conn,
            "campuses",
            &rec(json!({"id": "", "name": "Auto", "hacker": "DROP TABLE"})),
        )
        .unwrap();
        assert!(created["id"].as_i64().unwrap() > 0, "empty id must be assigned");
        assert_eq!(created["name"], json!("Auto"));
    }

    /// FR-011: the parquet import supplies its own ids.
    #[test]
    fn create_honors_explicit_id() {
        let conn = db::open_in_memory().unwrap();
        let created = create(&conn, "campuses", &rec(json!({"id": 777, "name": "Fixo"}))).unwrap();
        assert_eq!(created["id"], json!(777));
    }

    #[test]
    fn update_ignores_body_id_and_uses_path_id() {
        let conn = seeded();
        let updated = update(&conn, "campuses", 1, &rec(json!({"id": 999, "name": "Mudou"}))).unwrap();
        assert_eq!(updated["id"], json!(1));
        assert!(get(&conn, "campuses", 999).is_err());
    }

    #[test]
    fn missing_records_report_not_found() {
        let conn = seeded();
        assert!(matches!(get(&conn, "campuses", 12345), Err(AppError::NotFound)));
        assert!(matches!(delete(&conn, "campuses", 12345), Err(AppError::NotFound)));
        assert!(matches!(
            update(&conn, "campuses", 12345, &rec(json!({"name": "x"}))),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn unknown_entity_never_reaches_sql() {
        let conn = seeded();
        assert!(matches!(
            list(&conn, "campuses; DROP TABLE campuses", None, None, None, None, None),
            Err(AppError::UnknownEntity(_))
        ));
        let still_there: i64 = conn.query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0)).unwrap();
        assert_eq!(still_there, 4);
    }

    /// SC-001: every entity must support the full cycle, not just the easy ones.
    #[test]
    fn all_fifteen_entities_support_full_crud() {
        let conn = db::open_in_memory().unwrap();
        for def in registry::exported() {
            let created = create(&conn, def.route, &rec(json!({})))
                .unwrap_or_else(|e| panic!("{}: create falhou: {e}", def.route));
            let id = created["id"].as_i64().unwrap();

            get(&conn, def.route, id).unwrap_or_else(|e| panic!("{}: get falhou: {e}", def.route));
            list(&conn, def.route, None, None, None, None, None)
                .unwrap_or_else(|e| panic!("{}: list falhou: {e}", def.route));
            update(&conn, def.route, id, &rec(json!({"campus": "Serra"})))
                .unwrap_or_else(|e| panic!("{}: update falhou: {e}", def.route));
            delete(&conn, def.route, id)
                .unwrap_or_else(|e| panic!("{}: delete falhou: {e}", def.route));
        }
    }

    /// FR-015: SQL NULL must reach the interface as JSON null, since the JSON
    /// rendering from feature 013 branches on it.
    #[test]
    fn null_columns_serialize_as_json_null() {
        let conn = seeded();
        let r = get(&conn, "campuses", 1).unwrap();
        assert_eq!(r["description"], Json::Null);
    }
}
