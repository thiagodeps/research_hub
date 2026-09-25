//! Canonical archive export (SEP-019, adapted in SEP-032).
//!
//! SQLite holds everything as text; the canonical JSON carries booleans,
//! integers, floats, nulls and structure. The original archive is the only
//! record of which column was which, so every export reads the original's
//! `{table}_canonical.json` and casts column by column (research R2 — the
//! parquet schema no longer exists to be read).
//!
//! Mirrors the old `ParquetService.export_data` output shape, with two
//! declared differences (SEP-032): only `{table}_canonical.json` is emitted —
//! never parquet — and legacy `parquet/{table}_canonical.parquet` entries from
//! the original are dropped instead of preserved (R4). A failing table aborts
//! the export instead of being silently skipped.

use crate::error::AppError;
use crate::json_io::{self, ColType};
use crate::registry;
use rusqlite::Connection;
use serde_json::{Map, Number, Value as Json};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

pub const DEFAULT_FILENAME: &str = "portal_export_canonical.zip";

#[derive(serde::Serialize, Clone)]
pub struct TableExported {
    pub table: String,
    pub rows: usize,
}

#[derive(serde::Serialize)]
pub struct ExportSummary {
    pub tables: Vec<TableExported>,
    pub preserved_entries: usize,
    pub path: String,
}

/// Text back to boolean, accepting every spelling the Python map accepts.
fn as_bool(s: &str) -> Option<bool> {
    match s {
        "1" | "True" | "true" => Some(true),
        "0" | "False" | "false" => Some(false),
        _ => None,
    }
}

/// Build one JSON value from the stored text, casting to the type the original
/// file used.
///
/// Unparseable values become null rather than aborting — this is `to_numeric`
/// with `errors='coerce'`, kept for parity. Values without a known type leave
/// as text (FR-011).
fn json_cell(raw: Option<&str>, t: Option<ColType>) -> Json {
    let Some(s) = raw else { return Json::Null };
    match t {
        Some(ColType::Bool) => as_bool(s).map(Json::Bool).unwrap_or(Json::Null),
        Some(ColType::Int) => s
            .parse::<i64>()
            .ok()
            // "12.0" must reach 12: pandas writes floats for nullable integer
            // columns and astype() truncates.
            .or_else(|| s.parse::<f64>().ok().map(|f| f as i64))
            .map(|n| Json::Number(Number::from(n)))
            .unwrap_or(Json::Null),
        Some(ColType::Float) => s
            .parse::<f64>()
            .ok()
            .filter(|f| f.is_finite())
            .and_then(Number::from_f64)
            .map(Json::Number)
            .unwrap_or(Json::Null),
        // Text and unknown types stay as text. Lists and objects (Structure)
        // become real structure so the exported JSON carries relationships
        // rather than escaped text.
        Some(ColType::Structure) => {
            serde_json::from_str(s).unwrap_or_else(|_| Json::String(s.to_string()))
        }
        _ => Json::String(s.to_string()),
    }
}

/// `json.dumps(..., ensure_ascii=False, indent=4)`.
fn to_pretty_json(records: &[Map<String, Json>]) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    serde::Serialize::serialize(&records, &mut ser)
        .map_err(|e| AppError::Internal(format!("falha ao gerar JSON: {e}")))?;
    Ok(buf)
}

struct TableData {
    /// Column order taken from the original file when present (intersection
    /// with what the database manages), otherwise the registry's.
    order: Vec<String>,
    types: HashMap<String, ColType>,
    values: HashMap<String, Vec<Option<String>>>,
    rows: usize,
}

fn read_table(
    conn: &Connection,
    def: &registry::EntityDef,
    original: Option<(Vec<String>, HashMap<String, ColType>)>,
) -> Result<TableData, AppError> {
    let cols: Vec<&str> = def.columns.to_vec();
    let sql = format!("SELECT {} FROM {}", cols.join(", "), def.table);
    let mut stmt = conn.prepare(&sql)?;

    let mut values: HashMap<String, Vec<Option<String>>> =
        cols.iter().map(|c| ((*c).to_string(), Vec::new())).collect();
    let mut rows = 0usize;

    let mut q = stmt.query([])?;
    while let Some(row) = q.next()? {
        for (i, name) in cols.iter().enumerate() {
            // Everything is TEXT except id; read both as an optional string.
            let v: Option<String> = match row.get_ref(i)? {
                rusqlite::types::ValueRef::Null => None,
                rusqlite::types::ValueRef::Integer(n) => Some(n.to_string()),
                rusqlite::types::ValueRef::Real(f) => Some(f.to_string()),
                rusqlite::types::ValueRef::Text(t) => {
                    Some(String::from_utf8_lossy(t).into_owned())
                }
                rusqlite::types::ValueRef::Blob(_) => None,
            };
            values.get_mut(*name).unwrap().push(v);
        }
        rows += 1;
    }

    // Original column order, intersected with what we actually have — the exact
    // `[c for c in df_orig.columns if c in df.columns]` rule. Columns the
    // database has but the original does not are dropped, as they are today.
    let (order, types) = match original {
        Some((orig_order, orig_types)) => (
            orig_order.into_iter().filter(|c| values.contains_key(c)).collect(),
            orig_types,
        ),
        // No original archive: everything goes out as text, the degraded path.
        None => (cols.iter().map(|c| (*c).to_string()).collect(), HashMap::new()),
    };

    Ok(TableData { order, types, values, rows })
}

fn write_json(data: &TableData) -> Result<Vec<u8>, AppError> {
    let mut records = Vec::with_capacity(data.rows);
    for i in 0..data.rows {
        let mut rec = Map::with_capacity(data.order.len());
        for name in &data.order {
            rec.insert(name.clone(), json_cell(data.values[name][i].as_deref(), data.types.get(name).copied()));
        }
        records.push(rec);
    }
    to_pretty_json(&records)
}

/// Build the export archive.
///
/// `original` is the untouched archive kept by the import. When absent, the
/// export still runs but everything comes out as text — the degraded path the
/// Python version also had.
pub fn build_archive<F>(
    conn: &Connection,
    original: Option<&[u8]>,
    mut on_table: F,
) -> Result<(Vec<u8>, Vec<TableExported>, usize), AppError>
where
    F: FnMut(TableExported),
{
    let mut generated: HashMap<String, Vec<u8>> = HashMap::new();
    let mut exported = Vec::new();

    for def in registry::exported() {
        let json_path = format!("{}_canonical.json", def.table);

        // Column types of the original JSON file for this table, if present.
        // Research R2: the JSON itself is the schema source now.
        let original_schema = match original {
            Some(bytes) => {
                let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
                    .map_err(|e| AppError::Internal(format!("original.zip inválido: {e}")))?;
                // Read the bytes out before inspecting them: the entry borrows
                // the archive, and the borrow must end before `zip` is dropped.
                let raw = if let Ok(mut f) = zip.by_name(&json_path) {
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    Some(buf)
                } else {
                    None
                };
                match raw {
                    Some(buf) => Some((
                        json_io::column_order(&buf)?,
                        json_io::infer_column_types(&buf)?,
                    )),
                    None => None,
                }
            }
            None => None,
        };

        // A failure here aborts the export instead of dropping the table
        // silently (FR-013).
        let data = read_table(conn, def, original_schema)
            .map_err(|e| AppError::Internal(format!("tabela {}: {e}", def.table)))?;

        generated.insert(json_path, write_json(&data)?);

        let done = TableExported { table: def.table.to_string(), rows: data.rows };
        on_table(done.clone());
        exported.push(done);
    }

    // Legacy residue: parquet files of managed tables are neither regenerated
    // nor preserved (research R4) — the declared SEP-032 format change.
    let dropped: HashSet<String> = registry::exported()
        .map(|def| format!("parquet/{}_canonical.parquet", def.table))
        .collect();

    let mut out = Vec::new();
    let mut preserved = 0usize;
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut out));

        // Every entry the export does not replace — and did not drop — must
        // survive untouched: graphs, trackings, marts, nested archives. The
        // `raw_copy_file` path moves compressed bytes without recompressing.
        if let Some(bytes) = original {
            let mut src = zip::ZipArchive::new(std::io::Cursor::new(bytes))
                .map_err(|e| AppError::Internal(format!("original.zip inválido: {e}")))?;
            for i in 0..src.len() {
                let entry = src.by_index_raw(i).map_err(|e| AppError::Internal(e.to_string()))?;
                let name = entry.name().to_string();
                if generated.contains_key(&name) || dropped.contains(&name) {
                    continue;
                }
                writer
                    .raw_copy_file(entry)
                    .map_err(|e| AppError::Internal(format!("cópia de {name}: {e}")))?;
                preserved += 1;
            }
        }

        let opts: zip::write::FileOptions<()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        // Sorted so the archive is reproducible run to run.
        let mut names: Vec<&String> = generated.keys().collect();
        names.sort();
        for name in names {
            writer
                .start_file(name, opts)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            writer.write_all(&generated[name])?;
        }
        writer.finish().map_err(|e| AppError::Internal(e.to_string()))?;
    }

    Ok((out, exported, preserved))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    /// A typed original in the new JSON-only format, plus unmanaged entries
    /// and one legacy parquet residue that must be dropped.
    fn typed_json_original() -> Vec<u8> {
        let researchers: &[u8] =
            br#"[{"id": 1, "name": "Original", "was_student": true, "classification_confidence": 0.5, "research_groups": [{"id": 3, "name": "GP"}]}]"#;
        let mut zip_bytes = Vec::new();
        {
            let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
            let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
            z.start_file("researchers_canonical.json", opts).unwrap();
            z.write_all(researchers).unwrap();
            // Unmanaged entries that must survive untouched.
            z.start_file("mestrado/relatorio.html", opts).unwrap();
            z.write_all(b"<html>preservar</html>").unwrap();
            z.start_file("graphs/nodes.json", opts).unwrap();
            z.write_all(b"{\"n\":1}").unwrap();
            // Legacy residue of the old format: dropped, never preserved.
            z.start_file("parquet/researchers_canonical.parquet", opts).unwrap();
            z.write_all(b"residuo do formato antigo").unwrap();
            z.finish().unwrap();
        }
        zip_bytes
    }

    fn db_with_researcher() -> Connection {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO researchers (id, name, was_student, classification_confidence, research_groups)
             VALUES (1, 'Editado', 'True', '0.75', '[{\"id\":3,\"name\":\"GP\"}]')",
            [],
        )
        .unwrap();
        conn
    }

    fn entry(zip_bytes: &[u8], name: &str) -> Vec<u8> {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).unwrap();
        let mut f = z.by_name(name).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf
    }

    /// T017 / FR-011: the original's JSON is the schema source. Text stored in
    /// the database comes back with the original types.
    #[test]
    fn restores_original_types_from_the_original_json() {
        let conn = db_with_researcher();
        let (out, _, _) = build_archive(&conn, Some(&typed_json_original()), |_| {}).unwrap();
        let json: Json = serde_json::from_slice(&entry(&out, "researchers_canonical.json")).unwrap();

        assert_eq!(json[0]["was_student"], Json::Bool(true), "bool voltou como texto");
        assert_eq!(json[0]["id"], 1, "inteiro voltou como texto");
        assert_eq!(json[0]["classification_confidence"], 0.75, "decimal voltou como texto");
        assert!(json[0]["name"].is_string());

        let groups = &json[0]["research_groups"];
        assert!(groups.is_array(), "vínculo deve ser estrutura real, não texto escapado");
        assert_eq!(groups[0]["id"], 3);
    }

    /// T016 / FR-009: the export emits only the 15 canonical JSONs.
    #[test]
    fn exports_only_canonical_json_files() {
        let conn = db::open_in_memory().unwrap();
        db::seed_admin(&conn).unwrap();
        let (out, tables, _) = build_archive(&conn, None, |_| {}).unwrap();

        assert_eq!(tables.len(), 15);
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
        let names: Vec<String> =
            (0..z.len()).map(|i| z.by_index(i).unwrap().name().to_string()).collect();
        assert_eq!(names.len(), 15, "somente os 15 JSONs canônicos");
        assert!(names.iter().all(|n| n.ends_with("_canonical.json")));
        assert!(!names.iter().any(|n| n.starts_with("parquet/")), "parquet nunca sai");
        assert!(!names.iter().any(|n| n.contains("admins")));
    }

    /// T018 / R4: legacy managed parquet entries are dropped; unmanaged
    /// entries survive byte for byte.
    #[test]
    fn legacy_parquet_entries_are_dropped_not_preserved() {
        let conn = db_with_researcher();
        let original = typed_json_original();
        let (out, _, preserved) = build_archive(&conn, Some(&original), |_| {}).unwrap();

        assert_eq!(preserved, 2, "apenas html e json não gerenciados devem ser copiados");
        assert_eq!(entry(&out, "mestrado/relatorio.html"), b"<html>preservar</html>");
        assert_eq!(entry(&out, "graphs/nodes.json"), b"{\"n\":1}");
        assert!(
            zip::ZipArchive::new(std::io::Cursor::new(&out))
                .unwrap()
                .by_name("parquet/researchers_canonical.parquet")
                .is_err(),
            "o parquet legado da tabela gerenciada não pode reaparecer"
        );
    }

    #[test]
    fn json_is_indented_four_spaces_and_unescaped() {
        let conn = db::open_in_memory().unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Ciências Agrárias')", [])
            .unwrap();

        let (out, _, _) = build_archive(&conn, None, |_| {}).unwrap();
        let json = String::from_utf8(entry(&out, "campuses_canonical.json")).unwrap();

        assert!(json.contains("\n        \"id\""), "indentação deve ser de 4 espaços");
        assert!(json.contains("Ciências Agrárias"), "UTF-8 não pode ser escapado");
        assert!(!json.contains("\\u00ea"));
    }

    #[test]
    fn null_is_emitted_as_json_null_not_nan() {
        let conn = db::open_in_memory().unwrap();
        conn.execute("INSERT INTO campuses (id, name) VALUES (1, NULL)", []).unwrap();

        let (out, _, _) = build_archive(&conn, None, |_| {}).unwrap();
        let text = String::from_utf8(entry(&out, "campuses_canonical.json")).unwrap();
        assert!(text.contains("\"name\": null"));
        assert!(!text.contains("NaN"));
    }

    /// The degraded path: no original archive means everything goes out as
    /// text — including booleans and relationship JSON (FR-011).
    #[test]
    fn works_without_an_original_archive() {
        let conn = db_with_researcher();
        let (out, _, preserved) = build_archive(&conn, None, |_| {}).unwrap();
        assert_eq!(preserved, 0);

        let json: Json = serde_json::from_slice(&entry(&out, "researchers_canonical.json")).unwrap();
        assert_eq!(json[0]["was_student"], Json::String("True".into()), "sem original, tudo é texto");
        assert_eq!(json[0]["id"], Json::String("1".into()));
        assert!(
            json[0]["research_groups"].is_string(),
            "vínculo sem tipo conhecido sai como texto"
        );
    }

    #[test]
    fn unparseable_numbers_become_null_not_errors() {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO researchers (id, classification_confidence) VALUES (1, 'lixo')",
            [],
        )
        .unwrap();
        let (out, _, _) = build_archive(&conn, Some(&typed_json_original()), |_| {}).unwrap();
        let json: Json = serde_json::from_slice(&entry(&out, "researchers_canonical.json")).unwrap();
        assert!(json[0]["classification_confidence"].is_null(), "valor inválido vira null, como no to_numeric coerce");
    }

    #[test]
    fn boolean_accepts_every_spelling_the_python_map_accepts() {
        for (text, want) in [("1", true), ("True", true), ("true", true), ("0", false), ("False", false)] {
            assert_eq!(as_bool(text), Some(want), "não converteu {text}");
        }
        assert_eq!(as_bool("talvez"), None);
    }

    /// The criterion the whole feature exists for: import the real JSON-only
    /// archive, export it back, and check that nothing was lost or retyped.
    #[test]
    fn real_archive_round_trip_preserves_types_and_entries() {
        let archive_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap().join("exports_canonical.zip");
        let original = std::fs::read(&archive_path).expect("fixture ausente");

        // Import into a scratch database, then export straight back out.
        let dir = std::env::temp_dir().join(format!(
            "rh-rt-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let mut conn = db::open(&dir.join("hub.db")).unwrap();
        db::migrate(&conn).unwrap();
        crate::import::import_archive(&mut conn, &original, &dir, |_| {}).unwrap();

        let (out, tables, preserved) = build_archive(&conn, Some(&original), |_| {}).unwrap();

        // 395 original entries: 15 canonical JSONs replaced by this export,
        // the rest preserved (no parquet exists in the new format to drop).
        assert_eq!(preserved, 380, "as entradas não gerenciadas devem sair intactas");
        assert_eq!(tables.len(), 15);

        let mut src = zip::ZipArchive::new(std::io::Cursor::new(&original)).unwrap();
        let mut dst = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
        assert_eq!(dst.len(), 395, "o pacote exportado deve ter o mesmo total de entradas");
        assert!(
            (0..dst.len()).all(|i| !dst.by_index(i).unwrap().name().starts_with("parquet/")),
            "nenhum parquet pode existir no export"
        );

        // Unmanaged tracking files must survive byte for byte, uncompressed
        // and recompressed by nobody.
        let mut before = Vec::new();
        src.by_name("advisorships_tracking.json").unwrap().read_to_end(&mut before).unwrap();
        let mut after = Vec::new();
        dst.by_name("advisorships_tracking.json").unwrap().read_to_end(&mut after).unwrap();
        assert_eq!(before, after, "entrada não gerenciada foi alterada");

        // Every canonical table must come back with the original's types.
        for def in registry::exported() {
            let path = format!("{}_canonical.json", def.table);

            let mut before = Vec::new();
            src.by_name(&path).unwrap().read_to_end(&mut before).unwrap();
            let mut after = Vec::new();
            dst.by_name(&path).unwrap().read_to_end(&mut after).unwrap();

            let t_before = json_io::infer_column_types(&before)
                .unwrap_or_else(|e| panic!("{}: original ilegível: {e}", def.table));
            let t_after = json_io::infer_column_types(&after)
                .unwrap_or_else(|e| panic!("{}: export ilegível: {e}", def.table));

            for (col, want) in &t_before {
                match t_after.get(col) {
                    Some(got) => assert_eq!(got, want, "{}#{col}: tipo não voltou ao original", def.table),
                    None => panic!("{}#{col}: coluna do original sumiu no export", def.table),
                }
            }
        }

        // And the campus data itself survived the round trip.
        let campuses: Json = serde_json::from_slice(&entry(&out, "campuses_canonical.json")).unwrap();
        assert_eq!(campuses.as_array().unwrap().len(), 23);
        assert_eq!(campuses[0]["name"], "Vila Velha");
        assert!(campuses[0]["campus"].is_null(), "nenhum campus aninhado pode voltar");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_table_still_produces_valid_files() {
        let conn = db::open_in_memory().unwrap();
        let (out, tables, _) = build_archive(&conn, None, |_| {}).unwrap();
        assert!(tables.iter().all(|t| t.rows == 0));
        let json = String::from_utf8(entry(&out, "languages_canonical.json")).unwrap();
        assert_eq!(json.trim(), "[]");
    }
}
