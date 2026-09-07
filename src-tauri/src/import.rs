//! Canonical archive ingestion (SEP-018).

use crate::error::AppError;
use crate::parquet_io;
use crate::registry;
use rusqlite::Connection;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const ORIGINAL_ARCHIVE: &str = "original.zip";

#[derive(serde::Serialize, Clone)]
pub struct TableLoaded {
    pub table: String,
    pub rows: usize,
}

#[derive(serde::Serialize)]
pub struct ImportSummary {
    pub tables: Vec<TableLoaded>,
    pub total_rows: usize,
    pub snapshot: Option<String>,
}

/// Copy the database aside before the import destroys it (FR-011).
///
/// The import wipes every curated row by design, and the Python version offers
/// no undo — hours of manual correction can be erased by one wrong file. Uses
/// SQLite's own backup API so a live WAL is captured consistently.
pub fn snapshot(conn: &Connection, data_dir: &Path) -> Result<PathBuf, AppError> {
    let dir = data_dir.join("snapshots");
    std::fs::create_dir_all(&dir)?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = dir.join(format!("hub-{stamp}.db"));

    let mut dest = Connection::open(&path)?;
    let backup = rusqlite::backup::Backup::new(conn, &mut dest)?;
    backup.run_to_completion(200, std::time::Duration::from_millis(0), None)?;
    Ok(path)
}

fn has_any_rows(conn: &Connection) -> Result<bool, AppError> {
    for def in registry::exported() {
        let n: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", def.table), [], |r| r.get(0))?;
        if n > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Read the archive and load every canonical table it carries.
///
/// `on_table` is called as each table finishes, so the caller can emit progress
/// without this module knowing anything about Tauri.
pub fn import_archive<F>(
    conn: &mut Connection,
    archive_bytes: &[u8],
    data_dir: &Path,
    mut on_table: F,
) -> Result<ImportSummary, AppError>
where
    F: FnMut(TableLoaded),
{
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive_bytes))
        .map_err(|e| AppError::Internal(format!("arquivo .zip inválido: {e}")))?;

    // Snapshot first: after this point rows start disappearing.
    let snapshot_path = if has_any_rows(conn)? {
        Some(snapshot(conn, data_dir)?.display().to_string())
    } else {
        None
    };

    let parquet_entries: Vec<String> = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string()))
        .filter(|n| n.ends_with(".parquet"))
        .collect();

    // One transaction for the whole import: a failure halfway leaves the
    // previous state intact rather than a half-loaded base (FR-009).
    let tx = conn.transaction()?;

    for def in registry::exported() {
        tx.execute(&format!("DELETE FROM {}", def.table), [])?;
    }

    let mut loaded = Vec::new();
    let mut total = 0usize;

    for entry in parquet_entries {
        let table_name = parquet_io::table_name_from_path(&entry);
        let Some(def) = registry::ENTITIES.iter().find(|e| e.table == table_name && e.exported)
        else {
            continue; // graphs, marts and other unmanaged parquet: preserved, never parsed
        };

        let mut buf = Vec::new();
        zip.by_name(&entry)
            .map_err(|e| AppError::Internal(format!("entrada ilegível {entry}: {e}")))?
            .read_to_end(&mut buf)?;

        let table = parquet_io::read_columns(bytes::Bytes::from(buf), def.columns)?;
        if table.columns.is_empty() {
            continue;
        }

        let holes: Vec<String> = (1..=table.columns.len()).map(|i| format!("?{i}")).collect();
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            def.table,
            table.columns.join(", "),
            holes.join(", ")
        );
        {
            let mut stmt = tx.prepare(&sql)?;
            for row in &table.rows {
                stmt.execute(rusqlite::params_from_iter(row.iter()))?;
            }
        }

        let entry_loaded = TableLoaded { table: def.table.to_string(), rows: table.len() };
        total += table.len();
        on_table(entry_loaded.clone());
        loaded.push(entry_loaded);
    }

    tx.commit()?;

    // Only after a successful commit is the archive kept for export (FR-010).
    std::fs::write(data_dir.join(ORIGINAL_ARCHIVE), archive_bytes)?;

    Ok(ImportSummary { tables: loaded, total_rows: total, snapshot: snapshot_path })
}

pub fn looks_like_zip(path: &Path) -> bool {
    path.extension().map(|e| e.eq_ignore_ascii_case("zip")).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::io::Write;

    /// Build a minimal archive with one canonical parquet, written through
    /// arrow so the bytes are real parquet rather than a fixture.
    fn archive_with_campuses(rows: &[(i64, &str)]) -> Vec<u8> {
        use arrow::array::{Int64Array, StringArray};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use parquet::arrow::ArrowWriter;
        use std::sync::Arc;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(rows.iter().map(|(i, _)| *i).collect::<Vec<_>>())),
                Arc::new(StringArray::from(rows.iter().map(|(_, n)| *n).collect::<Vec<_>>())),
            ],
        )
        .unwrap();

        let mut parquet_bytes = Vec::new();
        {
            let mut w = ArrowWriter::try_new(&mut parquet_bytes, schema, None).unwrap();
            w.write(&batch).unwrap();
            w.close().unwrap();
        }

        let mut zip_bytes = Vec::new();
        {
            let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
            let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
            z.start_file("parquet/campuses_canonical.parquet", opts).unwrap();
            z.write_all(&parquet_bytes).unwrap();
            // An unmanaged entry, to prove it is skipped rather than parsed.
            z.start_file("people_relationship_graph.nodes.json", opts).unwrap();
            z.write_all(b"[]").unwrap();
            z.finish().unwrap();
        }
        zip_bytes
    }

    fn fixture() -> (Connection, tempdir::Dir) {
        let dir = tempdir::Dir::new();
        let mut conn = db::open(&dir.path().join("hub.db")).unwrap();
        db::migrate(&conn).unwrap();
        db::seed_admin(&conn).unwrap();
        let _ = &mut conn;
        (conn, dir)
    }

    /// SC-001 (shape): declared rows land in the right table.
    #[test]
    fn loads_canonical_tables() {
        let (mut conn, dir) = fixture();
        let zip = archive_with_campuses(&[(1, "Serra"), (2, "Vitoria")]);

        let summary = import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap();

        assert_eq!(summary.total_rows, 2);
        assert_eq!(summary.tables.len(), 1);
        assert_eq!(summary.tables[0].table, "campuses");

        let n: i64 = conn.query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 2);
    }

    /// FR-007: relationship arrays reference these ids, so they must survive.
    #[test]
    fn preserves_explicit_ids() {
        let (mut conn, dir) = fixture();
        let zip = archive_with_campuses(&[(77, "Serra")]);
        import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap();

        let name: String = conn
            .query_row("SELECT name FROM campuses WHERE id = 77", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Serra");
    }

    /// FR-008 / SC-005: the archive replaces the base rather than merging.
    #[test]
    fn replaces_previous_rows_and_is_repeatable() {
        let (mut conn, dir) = fixture();
        conn.execute("INSERT INTO campuses (name) VALUES ('Antigo')", []).unwrap();

        let zip = archive_with_campuses(&[(1, "Serra")]);
        import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap();
        import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap();

        let names: Vec<String> = conn
            .prepare("SELECT name FROM campuses")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(names, vec!["Serra"], "importar duas vezes não pode duplicar");
    }

    /// SC-002: wiping curated data must not log the admin out of their own app.
    #[test]
    fn admins_survive_the_import() {
        let (mut conn, dir) = fixture();
        let before: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();

        import_archive(&mut conn, &archive_with_campuses(&[(1, "Serra")]), dir.path(), |_| {}).unwrap();

        let after: String = conn
            .query_row("SELECT hashed_password FROM admins", [], |r| r.get(0))
            .unwrap();
        assert_eq!(before, after);
    }

    /// SC-004 / FR-011: the only feature that can destroy user work must leave a way back.
    #[test]
    fn snapshots_before_wiping_existing_data() {
        let (mut conn, dir) = fixture();
        conn.execute("INSERT INTO campuses (name) VALUES ('Curado a mao')", []).unwrap();

        let summary =
            import_archive(&mut conn, &archive_with_campuses(&[(1, "Serra")]), dir.path(), |_| {}).unwrap();

        let snap = summary.snapshot.expect("deve haver snapshot quando havia dado");
        let restored = Connection::open(&snap).unwrap();
        let name: String = restored
            .query_row("SELECT name FROM campuses", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Curado a mao", "o snapshot deve conter o estado anterior");
    }

    #[test]
    fn no_snapshot_when_there_was_nothing_to_lose() {
        let (mut conn, dir) = fixture();
        let summary =
            import_archive(&mut conn, &archive_with_campuses(&[(1, "Serra")]), dir.path(), |_| {}).unwrap();
        assert!(summary.snapshot.is_none());
    }

    /// FR-010: the export step rebuilds the archive from this copy.
    #[test]
    fn keeps_an_untouched_copy_of_the_archive() {
        let (mut conn, dir) = fixture();
        let zip = archive_with_campuses(&[(1, "Serra")]);
        import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap();

        let kept = std::fs::read(dir.path().join(ORIGINAL_ARCHIVE)).unwrap();
        assert_eq!(kept, zip, "a cópia deve ser byte a byte idêntica");
    }

    /// SC-003 / FR-009: a bad archive must not leave a half-loaded base.
    #[test]
    fn invalid_archive_leaves_database_untouched() {
        let (mut conn, dir) = fixture();
        conn.execute("INSERT INTO campuses (name) VALUES ('Intacto')", []).unwrap();

        let err = import_archive(&mut conn, b"isto nao e um zip", dir.path(), |_| {});
        assert!(err.is_err());

        let name: String = conn.query_row("SELECT name FROM campuses", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "Intacto");
    }

    /// FR-012: progress must be reported per table, not only at the end.
    #[test]
    fn reports_progress_per_table() {
        let (mut conn, dir) = fixture();
        let mut seen = Vec::new();
        import_archive(&mut conn, &archive_with_campuses(&[(1, "A"), (2, "B")]), dir.path(), |t| {
            seen.push((t.table, t.rows))
        })
        .unwrap();
        assert_eq!(seen, vec![("campuses".to_string(), 2)]);
    }

    #[test]
    fn rejects_non_zip_by_extension() {
        assert!(looks_like_zip(Path::new("/tmp/exports.zip")));
        assert!(looks_like_zip(Path::new("/tmp/exports.ZIP")));
        assert!(!looks_like_zip(Path::new("/tmp/exports.parquet")));
    }

    /// SC-001: the criterion that matters. Imports the real archive and compares
    /// every table against the counts the Python importer produces for the same
    /// file (measured with pandas before the migration started).
    #[test]
    fn reference_archive_matches_python_row_counts() {
        let archive = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("exports_canonical.zip");
        if !archive.exists() {
            panic!("fixture ausente: {}", archive.display());
        }

        let expected: &[(&str, i64)] = &[
            ("researchers", 4225), ("initiatives", 4122), ("students", 2625),
            ("professional_activities", 2041), ("articles", 2027),
            ("research_productions", 951), ("knowledge_areas", 415),
            ("research_groups", 347), ("proficiencies", 209), ("advisorships", 183),
            ("organizations", 123), ("awards", 51), ("campuses", 23),
            ("fellowships", 19), ("languages", 8),
        ];

        let (mut conn, dir) = fixture();
        let bytes = std::fs::read(&archive).unwrap();
        let summary = import_archive(&mut conn, &bytes, dir.path(), |_| {}).unwrap();

        for (table, want) in expected {
            let got: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(got, *want, "{table}: importado {got}, Python produz {want}");
        }

        assert_eq!(summary.tables.len(), 15, "as 15 tabelas canonicas devem carregar");
        assert_eq!(summary.total_rows, expected.iter().map(|(_, n)| *n as usize).sum::<usize>());
    }

    /// Minimal scratch directory helper; avoids a dependency for four tests.
    mod tempdir {
        use std::path::{Path, PathBuf};
        pub struct Dir(PathBuf);
        impl Dir {
            pub fn new() -> Self {
                let n = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let p = std::env::temp_dir().join(format!("rh-test-{n}"));
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
}
