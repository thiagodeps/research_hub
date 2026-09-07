//! SQLite bootstrap: file location, pragmas, migrations, admin seed (SEP-016).

use crate::error::AppError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Ordered migrations. Index + 1 is the `user_version` reached by applying it.
/// Append only — never edit a shipped entry.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_init.sql")];

pub const DEFAULT_ADMIN_USERNAME: &str = "admin@admin.com";
const DEFAULT_ADMIN_PASSWORD: &str = "admin123";

/// `hub.db` inside the OS app-data directory, creating it if absent (FR-001).
pub fn database_path(app_data_dir: &Path) -> Result<PathBuf, AppError> {
    std::fs::create_dir_all(app_data_dir)?;
    Ok(app_data_dir.join("hub.db"))
}

/// Open a connection with the pragmas every connection needs.
///
/// The ETL path (SEP-018/019) opens its own connection through this same
/// function, so long imports never hold the interactive lock (FR-012).
pub fn open(path: &Path) -> Result<Connection, AppError> {
    let conn = Connection::open(path)?;
    configure(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection, AppError> {
    let conn = Connection::open_in_memory()?;
    // WAL is meaningless in memory; the rest still applies.
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<(), AppError> {
    // WAL lets the interactive reader and the ETL writer proceed together (AD-08).
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    Ok(())
}

/// Apply pending migrations, tracked by `PRAGMA user_version` (FR-003).
/// Idempotent: re-running on an up-to-date database is a no-op.
pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let current = usize::try_from(current)
        .map_err(|_| AppError::Migration(format!("user_version inválido: {current}")))?;

    if current > MIGRATIONS.len() {
        return Err(AppError::Migration(format!(
            "banco na versão {current}, mais nova que as {} migrações conhecidas — \
             provavelmente gravado por uma versão posterior do aplicativo",
            MIGRATIONS.len()
        )));
    }

    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        conn.execute_batch(sql)?;
        conn.pragma_update(None, "user_version", i as i64 + 1)?;
    }
    Ok(())
}

/// Create the default admin only when none exists (FR-009).
///
/// Unified on `username` + `hashed_password`: today `orm.py` and
/// `scripts/seed.py` disagree on both column names, and the seed script cannot
/// run at all as a result (bug 10.3.4).
pub fn seed_admin(conn: &Connection) -> Result<(), AppError> {
    let existing: i64 = conn.query_row("SELECT COUNT(*) FROM admins", [], |r| r.get(0))?;
    if existing > 0 {
        return Ok(());
    }
    let hash = bcrypt::hash(DEFAULT_ADMIN_PASSWORD, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("falha ao gerar hash: {e}")))?;
    conn.execute(
        "INSERT INTO admins (username, hashed_password) VALUES (?1, ?2)",
        (DEFAULT_ADMIN_USERNAME, hash),
    )?;
    Ok(())
}

/// Open, migrate and seed in one step. Called from Tauri's `setup`.
pub fn initialize(app_data_dir: &Path) -> Result<Connection, AppError> {
    let conn = open(&database_path(app_data_dir)?)?;
    migrate(&conn)?;
    seed_admin(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    fn table_columns(conn: &Connection, table: &str) -> Vec<String> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .expect("table_info");
        let cols = stmt
            .query_map([], |r| r.get::<_, String>(1))
            .expect("query")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect");
        cols
    }

    /// SC-001: the schema is verified by querying it, not by reading the file.
    #[test]
    fn migration_creates_sixteen_tables() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' \
                 AND name NOT LIKE 'sqlite_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 16, "esperado 16 tabelas (15 exportadas + admins)");
    }

    /// FR-005 / Q5: the orphan entity must not come along.
    #[test]
    fn universities_table_was_dropped() {
        let conn = open_in_memory().unwrap();
        let found: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name='universities'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(found, 0);
    }

    /// SC-002: registry and schema must agree in BOTH directions. A column added
    /// to one and forgotten in the other fails here rather than at runtime.
    #[test]
    fn registry_and_schema_agree() {
        let conn = open_in_memory().unwrap();
        for def in registry::ENTITIES {
            let actual = table_columns(&conn, def.table);
            assert!(!actual.is_empty(), "tabela ausente: {}", def.table);

            for declared in def.columns {
                assert!(
                    actual.iter().any(|c| c == declared),
                    "{}: registry declara '{}', ausente no schema",
                    def.table,
                    declared
                );
            }
            for existing in &actual {
                assert!(
                    def.has_column(existing),
                    "{}: schema tem '{}', ausente no registry",
                    def.table,
                    existing
                );
            }
        }
    }

    #[test]
    fn search_column_exists_when_declared() {
        let conn = open_in_memory().unwrap();
        for def in registry::ENTITIES {
            if let Some(col) = def.search_column {
                assert!(
                    def.has_column(col),
                    "{}: coluna de busca '{}' não existe",
                    def.table,
                    col
                );
                assert!(table_columns(&conn, def.table).iter().any(|c| c == col));
            }
        }
    }

    /// FR-003: re-running migrations must be a no-op, not an error.
    #[test]
    fn migrations_are_idempotent() {
        let conn = open_in_memory().unwrap();
        let before: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        migrate(&conn).expect("segunda migração deve ser no-op");
        let after: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(before, after);
        assert_eq!(after, MIGRATIONS.len() as i64);
    }

    /// SC-003: seeding twice leaves one admin, with the original hash.
    #[test]
    fn admin_seed_is_idempotent_and_never_overwrites() {
        let conn = open_in_memory().unwrap();
        seed_admin(&conn).unwrap();
        let first: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();

        seed_admin(&conn).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM admins", [], |r| r.get(0)).unwrap();
        let second: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(first, second, "o hash existente não pode ser regravado");
    }

    #[test]
    fn seeded_admin_password_verifies() {
        let conn = open_in_memory().unwrap();
        seed_admin(&conn).unwrap();
        let hash: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();
        assert!(bcrypt::verify(DEFAULT_ADMIN_PASSWORD, &hash).unwrap());
        assert!(!bcrypt::verify("senha errada", &hash).unwrap());
    }

    /// FR-004: explicit ids from the parquet import must be honored (risk 10.4).
    #[test]
    fn explicit_id_is_preserved_on_insert() {
        let conn = open_in_memory().unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (4242, 'Serra')", [])
            .unwrap();
        let name: String = conn
            .query_row("SELECT name FROM campuses WHERE id = 4242", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Serra");
    }

    /// FR-007 / SC-004.
    #[test]
    fn routes_resolve_and_unknown_route_is_typed() {
        assert_eq!(registry::by_route("groups").unwrap().table, "research_groups");
        for def in registry::ENTITIES {
            assert_eq!(registry::by_route(def.route).unwrap().table, def.table);
        }
        assert!(matches!(
            registry::by_route("../../etc/passwd"),
            Err(AppError::UnknownEntity(_))
        ));
    }

    /// FR-008: JSON relationship columns order by link count, not by text.
    #[test]
    fn json_columns_sort_by_length() {
        use registry::EntityDef;
        assert_eq!(EntityDef::sort_expr("research_groups"), "length(research_groups)");
        assert_eq!(EntityDef::sort_expr("name"), "name");
    }

    #[test]
    fn exported_entities_exclude_admins() {
        let exported: Vec<&str> = registry::exported().map(|e| e.route).collect();
        assert_eq!(exported.len(), 15);
        assert!(!exported.contains(&"admins"));
    }
}
