//! Canonical archive ingestion (SEP-018, adapted in SEP-032).
//!
//! The primary source is `{table}_canonical.json` at the archive root; the
//! legacy `parquet/{table}_canonical.parquet` remains a fallback for old
//! packages, and the JSON wins when both exist (research R1).

use crate::error::AppError;
use crate::json_io;
use crate::parquet_io;
use crate::registry;
use rusqlite::Connection;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const ORIGINAL_ARCHIVE: &str = "original.zip";

#[derive(serde::Serialize, Clone, Debug)]
pub struct TableLoaded {
    pub table: String,
    pub rows: usize,
}

#[derive(serde::Serialize, Debug)]
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
/// For each managed table the source is `{table}_canonical.json` at the root,
/// falling back to the legacy `parquet/{table}_canonical.parquet`; when both
/// exist the JSON wins (SEP-032, FR-006). A managed table with neither file
/// aborts the import before anything destructive happens (FR-014).
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

    // Resolve the source of every managed table up front (research R1/R7).
    let names: Vec<String> = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_string()))
        .collect();
    let has = |n: &str| names.iter().any(|name| name == n);

    enum Source {
        Json(String),
        Parquet(String),
    }

    let mut missing: Vec<&'static str> = Vec::new();
    let mut sources: Vec<(&'static registry::EntityDef, Source)> = Vec::new();
    for def in registry::exported() {
        let base = format!("{}_canonical", def.table);
        let json_path = format!("{base}.json");
        let parquet_path = format!("parquet/{base}.parquet");
        if has(&json_path) {
            sources.push((def, Source::Json(json_path)));
        } else if has(&parquet_path) {
            sources.push((def, Source::Parquet(parquet_path)));
        } else {
            missing.push(def.table);
        }
    }
    if !missing.is_empty() {
        return Err(AppError::Validation(format!(
            "pacote sem o arquivo canônico das tabelas: {}",
            missing.join(", ")
        )));
    }

    // Snapshot only after the package proved complete, and always before rows
    // start disappearing (FR-011).
    let snapshot_path = if has_any_rows(conn)? {
        Some(snapshot(conn, data_dir)?.display().to_string())
    } else {
        None
    };

    // One transaction for the whole import: a failure halfway leaves the
    // previous state intact rather than a half-loaded base (FR-009).
    let tx = conn.transaction()?;

    for def in registry::exported() {
        tx.execute(&format!("DELETE FROM {}", def.table), [])?;
    }

    let mut loaded = Vec::new();
    let mut total = 0usize;

    for (def, source) in sources {
        let entry = match &source {
            Source::Json(path) | Source::Parquet(path) => path,
        };
        let mut buf = Vec::new();
        zip.by_name(entry)
            .map_err(|e| AppError::Internal(format!("entrada ilegível {entry}: {e}")))?
            .read_to_end(&mut buf)?;

        let table = match &source {
            Source::Json(_) => json_io::read_columns(&buf, def.columns)?,
            Source::Parquet(_) => {
                let pq = parquet_io::read_columns(bytes::Bytes::from(buf), def.columns)?;
                // Both readers produce the same shape; unify on the JSON one.
                json_io::Table { columns: pq.columns, rows: pq.rows }
            }
        };
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

    /// Build a ZIP from raw name/bytes pairs (JSON fixtures and unmanaged entries).
    fn zip_from(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
        let mut zip_bytes = Vec::new();
        {
            let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
            let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
            for (name, bytes) in entries {
                z.start_file(*name, opts).unwrap();
                z.write_all(bytes).unwrap();
            }
            z.finish().unwrap();
        }
        zip_bytes
    }

    /// A `{table}_canonical.json` body with id/name rows.
    fn campuses_json(rows: &[(i64, &str)]) -> Vec<u8> {
        let items: Vec<String> = rows
            .iter()
            .map(|(id, name)| format!(r#"{{"id": {id}, "name": "{name}"}}"#))
            .collect();
        format!("[{}]", items.join(", ")).into_bytes()
    }

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

        // Complete package: the 14 tables this fixture does not exercise go as
        // empty JSON lists so the FR-014 completeness check passes.
        zip_from(&complete_package(vec![
            ("parquet/campuses_canonical.parquet", parquet_bytes),
            ("people_relationship_graph.nodes.json", b"[]".to_vec()),
        ]))
    }

    /// FR-014 requires every managed table's file to be present. Fixtures add
    /// the tables they do not exercise as empty JSON lists (empty tables load
    /// as nothing and emit no progress, so old expectations keep holding).
    fn complete_package(mut entries: Vec<(&'static str, Vec<u8>)>) -> Vec<(&'static str, Vec<u8>)> {
        /// Leak a tiny fixture name so it can sit in the entries vec as 'static.
        fn json_name(table: &str) -> &'static str {
            Box::leak(format!("{table}_canonical.json").into_boxed_str())
        }

        let missing: Vec<&'static str> = registry::exported()
            .filter(|def| {
                let json = format!("{}_canonical.json", def.table);
                let parquet = format!("parquet/{}_canonical.parquet", def.table);
                !entries.iter().any(|(n, _)| *n == json || *n == parquet)
            })
            .map(|def| def.table)
            .collect();
        for table in missing {
            entries.push((json_name(table), b"[]".to_vec()));
        }
        entries
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

    /// SC-001 (shape), SEP-032 T004: JSON-only archives land in the right table.
    #[test]
    fn loads_canonical_json_tables() {
        let (mut conn, dir) = fixture();
        let zip = zip_from(&complete_package(vec![
            ("campuses_canonical.json", campuses_json(&[(77, "Serra"), (2, "Vitoria")])),
            (
                "languages_canonical.json",
                br#"[{"id": 5, "name": "Portugues"}]"#.to_vec(),
            ),
            // Unmanaged entry, to prove it is skipped rather than parsed.
            ("people_relationship_graph.json", b"[]".to_vec()),
        ]));

        let mut seen = Vec::new();
        let summary = import_archive(&mut conn, &zip, dir.path(), |t| {
            seen.push((t.table, t.rows))
        })
        .unwrap();

        assert_eq!(summary.total_rows, 3);
        assert_eq!(summary.tables.len(), 2);
        assert_eq!(seen, vec![("campuses".to_string(), 2), ("languages".to_string(), 1)]);

        let name: String = conn
            .query_row("SELECT name FROM campuses WHERE id = 77", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "Serra", "ids explícitos devem sobreviver");
    }

    /// FR-014 (T005, clarificação Q1): a package missing a managed table's
    /// file aborts with a clear error and leaves the base untouched.
    #[test]
    fn aborts_when_managed_table_file_missing() {
        let (mut conn, dir) = fixture();
        conn.execute("INSERT INTO campuses (name) VALUES ('Intacto')", []).unwrap();

        let zip = zip_from(&[("campuses_canonical.json", campuses_json(&[(1, "Serra")]))]);
        let err = import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap_err();

        assert!(
            err.to_string().contains("languages"),
            "o erro deve nomear a tabela faltante: {err}"
        );

        let name: String = conn.query_row("SELECT name FROM campuses", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "Intacto", "a base deve permanecer intacta");
        assert!(
            !dir.path().join("snapshots").exists(),
            "nenhum snapshot para uma importação que falhou na validação"
        );
        assert!(
            !dir.path().join(ORIGINAL_ARCHIVE).exists(),
            "o original não pode ser substituído por um pacote incompleto"
        );
    }

    /// FR-006 (T006): legacy packages keep working, and JSON wins when both
    /// formats carry the same table.
    #[test]
    fn legacy_packages_still_load_with_json_precedence() {
        let (mut conn, dir) = fixture();
        // Legacy pair: the parquet says "Antigo", the JSON says "Novo".
        let zip = zip_from(&complete_package(vec![
            ("parquet/campuses_canonical.parquet", {
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
                        Arc::new(Int64Array::from(vec![1i64])),
                        Arc::new(StringArray::from(vec![Some("Antigo")])),
                    ],
                )
                .unwrap();
                let mut pq = Vec::new();
                {
                    let mut w = ArrowWriter::try_new(&mut pq, schema, None).unwrap();
                    w.write(&batch).unwrap();
                    w.close().unwrap();
                }
                pq
            }),
            ("campuses_canonical.json", campuses_json(&[(1, "Novo")])),
            // A table that only exists as JSON in this hypothetical package.
            ("languages_canonical.json", br#"[{"id": 5, "name": "Portugues"}]"#.to_vec()),
        ]));

        import_archive(&mut conn, &zip, dir.path(), |_| {}).unwrap();

        let name: String = conn.query_row("SELECT name FROM campuses", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "Novo", "quando JSON e parquet existem, o JSON vence");
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM languages", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1, "tabela apenas-JSON carrega mesmo em pacote com parquet");
    }

    /// SC-001 (counts), SEP-032 T007: the reference archive is now JSON-only;
    /// every managed table must load with the row count the file carries.
    #[test]
    fn reference_archive_matches_row_counts() {
        let archive = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .join("exports_canonical.zip");
        if !archive.exists() {
            panic!("fixture ausente: {}", archive.display());
        }

        let expected: &[(&str, i64)] = &[
            ("researchers", 9694), ("students", 6282), ("articles", 2133),
            ("initiatives", 4044), ("knowledge_areas", 1552),
            ("professional_activities", 2099), ("research_productions", 975),
            ("research_groups", 353), ("proficiencies", 220), ("advisorships", 188),
            ("organizations", 139), ("awards", 52), ("campuses", 23),
            ("fellowships", 19), ("languages", 8),
        ];

        let (mut conn, dir) = fixture();
        let bytes = std::fs::read(&archive).unwrap();
        let summary = import_archive(&mut conn, &bytes, dir.path(), |_| {}).unwrap();

        for (table, want) in expected {
            let got: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(got, *want, "{table}: importado {got}, o arquivo tem {want}");
        }

        assert_eq!(summary.tables.len(), 15, "as 15 tabelas canonicas devem carregar");
        assert_eq!(summary.total_rows, expected.iter().map(|(_, n)| *n as usize).sum::<usize>());
    }

    /// SC-001 (shape): declared rows land in the right table.

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
