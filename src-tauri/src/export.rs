//! Canonical archive export (SEP-019).
//!
//! The riskiest part of the migration. SQLite holds everything as text; the
//! canonical parquet holds booleans, integers and floats. The original archive
//! is the only record of which column was which, so every export reads it back
//! and casts column by column.
//!
//! Mirrors `ParquetService.export_data`, with one deliberate difference: a table
//! that fails is reported instead of being printed to stdout and skipped, which
//! today can silently drop a whole table from the delivered archive.

use crate::error::AppError;
use crate::parquet_io;
use crate::registry;
use arrow::array::{ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use rusqlite::Connection;
use serde_json::{Map, Value as Json};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;

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

/// Build one arrow column, casting text back to the type the original file used.
///
/// Unparseable values become null rather than aborting — this is `to_numeric`
/// with `errors='coerce'`, kept for parity. Note the consequence: bad data is
/// silently dropped on export, exactly as it is today.
fn column(values: &[Option<String>], target: &DataType) -> ArrayRef {
    match target {
        DataType::Boolean => Arc::new(
            values
                .iter()
                .map(|v| v.as_deref().and_then(as_bool))
                .collect::<BooleanArray>(),
        ),
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64 => Arc::new(Int64Array::from(
            values
                .iter()
                .map(|v| {
                    v.as_deref().and_then(|s| {
                        // "12.0" must reach 12: pandas writes floats for
                        // nullable integer columns and astype() truncates.
                        s.parse::<i64>().ok().or_else(|| s.parse::<f64>().ok().map(|f| f as i64))
                    })
                })
                .collect::<Vec<Option<i64>>>(),
        )),
        DataType::Float16 | DataType::Float32 | DataType::Float64 => Arc::new(Float64Array::from(
            values
                .iter()
                .map(|v| v.as_deref().and_then(|s| s.parse::<f64>().ok()))
                .collect::<Vec<Option<f64>>>(),
        )),
        // Strings, dates, timestamps and anything nested stay as text: that is
        // how they were read in, and re-deriving them would risk losing data.
        _ => Arc::new(StringArray::from(
            values.iter().map(|v| v.as_deref()).collect::<Vec<Option<&str>>>(),
        )),
    }
}

fn arrow_field_type(t: &DataType) -> DataType {
    match t {
        DataType::Boolean => DataType::Boolean,
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64
        | DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => DataType::Int64,
        DataType::Float16 | DataType::Float32 | DataType::Float64 => DataType::Float64,
        _ => DataType::Utf8,
    }
}

/// A stored string that looks like JSON becomes a real array or object, so the
/// exported JSON carries structure rather than escaped text.
fn json_value(raw: Option<&str>) -> Json {
    match raw {
        None => Json::Null,
        Some(s) => {
            let t = s.trim_start();
            if t.starts_with('[') || t.starts_with('{') {
                serde_json::from_str(s).unwrap_or_else(|_| Json::String(s.to_string()))
            } else {
                Json::String(s.to_string())
            }
        }
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
    /// Column order and types taken from the original file when it is present.
    order: Vec<String>,
    types: HashMap<String, DataType>,
    values: HashMap<String, Vec<Option<String>>>,
    rows: usize,
}

fn read_table(
    conn: &Connection,
    def: &registry::EntityDef,
    original: Option<(Vec<String>, HashMap<String, DataType>)>,
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

fn write_parquet(data: &TableData) -> Result<Vec<u8>, AppError> {
    let fields: Vec<Field> = data
        .order
        .iter()
        .map(|name| {
            let t = data.types.get(name).map(arrow_field_type).unwrap_or(DataType::Utf8);
            Field::new(name, t, true)
        })
        .collect();
    let schema = Arc::new(Schema::new(fields));

    let arrays: Vec<ArrayRef> = data
        .order
        .iter()
        .map(|name| {
            let target = data.types.get(name).cloned().unwrap_or(DataType::Utf8);
            column(&data.values[name], &target)
        })
        .collect();

    let mut out = Vec::new();
    {
        let mut writer = ArrowWriter::try_new(&mut out, schema.clone(), None)
            .map_err(|e| AppError::Internal(format!("writer parquet: {e}")))?;
        if data.rows > 0 {
            let batch = RecordBatch::try_new(schema, arrays)
                .map_err(|e| AppError::Internal(format!("lote inválido: {e}")))?;
            writer.write(&batch).map_err(|e| AppError::Internal(e.to_string()))?;
        }
        writer.close().map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(out)
}

fn write_json(data: &TableData) -> Result<Vec<u8>, AppError> {
    let mut records = Vec::with_capacity(data.rows);
    for i in 0..data.rows {
        let mut rec = Map::with_capacity(data.order.len());
        for name in &data.order {
            rec.insert(name.clone(), json_value(data.values[name][i].as_deref()));
        }
        records.push(rec);
    }
    to_pretty_json(&records)
}

/// Build the export archive.
///
/// `original` is the untouched archive kept by the import. When absent, the
/// export still runs but everything comes out as text — the degraded path the
/// Python version also has.
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
        let base = format!("{}_canonical", def.table);
        let pq_path = format!("parquet/{base}.parquet");
        let json_path = format!("{base}.json");

        // Schema of the original file for this table, if the archive has it.
        let original_schema = match original {
            Some(bytes) => {
                let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
                    .map_err(|e| AppError::Internal(format!("original.zip inválido: {e}")))?;
                // Read the bytes out before inspecting them: the entry borrows
                // the archive, and the borrow must end before `zip` is dropped.
                let raw = if let Ok(mut f) = zip.by_name(&pq_path) {
                    let mut buf = Vec::new();
                    f.read_to_end(&mut buf)?;
                    Some(buf)
                } else {
                    None
                };
                match raw {
                    Some(buf) => {
                        let b = bytes::Bytes::from(buf);
                        Some((parquet_io::column_order(b.clone())?, parquet_io::schema_of(b)?))
                    }
                    None => None,
                }
            }
            None => None,
        };

        // A failure here aborts the export instead of dropping the table
        // silently — the declared correction to bug 10.3.3.
        let data = read_table(conn, def, original_schema)
            .map_err(|e| AppError::Internal(format!("tabela {}: {e}", def.table)))?;

        generated.insert(pq_path, write_parquet(&data)?);
        generated.insert(json_path, write_json(&data)?);

        let done = TableExported { table: def.table.to_string(), rows: data.rows };
        on_table(done.clone());
        exported.push(done);
    }

    let mut out = Vec::new();
    let mut preserved = 0usize;
    {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(&mut out));

        // Every entry the export does not replace must survive untouched: 596
        // of them, including a 22.6 MB nested archive and a PDF. `raw_copy_file`
        // moves the compressed bytes across without recompressing.
        if let Some(bytes) = original {
            let mut src = zip::ZipArchive::new(std::io::Cursor::new(bytes))
                .map_err(|e| AppError::Internal(format!("original.zip inválido: {e}")))?;
            for i in 0..src.len() {
                let entry = src.by_index_raw(i).map_err(|e| AppError::Internal(e.to_string()))?;
                let name = entry.name().to_string();
                if generated.contains_key(&name) {
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

    fn typed_original() -> Vec<u8> {
        use arrow::array::{BooleanArray, Int64Array, StringArray};
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("was_student", DataType::Boolean, true),
            Field::new("classification_confidence", DataType::Float64, true),
            Field::new("research_groups", DataType::Utf8, true),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1i64])),
                Arc::new(StringArray::from(vec![Some("Original")])),
                Arc::new(BooleanArray::from(vec![Some(true)])),
                Arc::new(Float64Array::from(vec![Some(0.5)])),
                Arc::new(StringArray::from(vec![Some("[]")])),
            ],
        )
        .unwrap();

        let mut pq = Vec::new();
        {
            let mut w = ArrowWriter::try_new(&mut pq, schema, None).unwrap();
            w.write(&batch).unwrap();
            w.close().unwrap();
        }

        let mut zip_bytes = Vec::new();
        {
            let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
            let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
            z.start_file("parquet/researchers_canonical.parquet", opts).unwrap();
            z.write_all(&pq).unwrap();
            z.start_file("researchers_canonical.json", opts).unwrap();
            z.write_all(b"[]").unwrap();
            // Unmanaged entries that must survive untouched.
            z.start_file("mestrado/relatorio.html", opts).unwrap();
            z.write_all(b"<html>preservar</html>").unwrap();
            z.start_file("graphs/nodes.json", opts).unwrap();
            z.write_all(b"{\"n\":1}").unwrap();
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

    /// The core risk: text in the database must come back as the original type.
    #[test]
    fn restores_original_column_types() {
        let conn = db_with_researcher();
        let (out, _, _) = build_archive(&conn, Some(&typed_original()), |_| {}).unwrap();

        let pq = entry(&out, "parquet/researchers_canonical.parquet");
        let types = parquet_io::schema_of(bytes::Bytes::from(pq.clone())).unwrap();

        assert_eq!(types["was_student"], DataType::Boolean, "bool voltou como texto");
        assert_eq!(types["classification_confidence"], DataType::Float64);
        assert_eq!(types["id"], DataType::Int64);
        assert_eq!(types["name"], DataType::Utf8);

        // And the values survived the round trip.
        let table = parquet_io::read_columns(
            bytes::Bytes::from(pq),
            &["id", "name", "was_student", "classification_confidence"],
        )
        .unwrap();
        assert_eq!(table.rows[0][2], Some("True".into()));
        assert_eq!(table.rows[0][3], Some("0.75".into()));
    }

    /// Column order must be the original's, not the registry's.
    #[test]
    fn preserves_original_column_order_and_drops_extras() {
        let conn = db_with_researcher();
        let (out, _, _) = build_archive(&conn, Some(&typed_original()), |_| {}).unwrap();
        let pq = entry(&out, "parquet/researchers_canonical.parquet");

        let order = parquet_io::column_order(bytes::Bytes::from(pq)).unwrap();
        assert_eq!(
            order,
            vec!["id", "name", "was_student", "classification_confidence", "research_groups"],
            "ordem deve seguir o parquet original, não o registry"
        );
    }

    /// SC: the 596 untouched entries. Byte-for-byte, without recompression.
    #[test]
    fn preserves_unmanaged_entries_byte_for_byte() {
        let conn = db_with_researcher();
        let original = typed_original();
        let (out, _, preserved) = build_archive(&conn, Some(&original), |_| {}).unwrap();

        assert_eq!(preserved, 2, "html e json não gerenciados devem ser copiados");
        assert_eq!(entry(&out, "mestrado/relatorio.html"), b"<html>preservar</html>");
        assert_eq!(entry(&out, "graphs/nodes.json"), b"{\"n\":1}");
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

    /// JSON relationship columns must carry structure, not escaped text.
    #[test]
    fn json_columns_are_emitted_as_structure() {
        let conn = db_with_researcher();
        let (out, _, _) = build_archive(&conn, Some(&typed_original()), |_| {}).unwrap();
        let json: Json =
            serde_json::from_slice(&entry(&out, "researchers_canonical.json")).unwrap();

        let groups = &json[0]["research_groups"];
        assert!(groups.is_array(), "deve ser array real, não string escapada");
        assert_eq!(groups[0]["id"], 3);
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

    #[test]
    fn exports_fifteen_tables_and_skips_admins() {
        let conn = db::open_in_memory().unwrap();
        db::seed_admin(&conn).unwrap();
        let (out, tables, _) = build_archive(&conn, None, |_| {}).unwrap();

        assert_eq!(tables.len(), 15);
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
        let names: Vec<String> = (0..z.len()).map(|i| z.by_index(i).unwrap().name().to_string()).collect();
        assert_eq!(names.len(), 30, "15 parquet + 15 json");
        assert!(!names.iter().any(|n| n.contains("admins")));
        assert!(names.contains(&"parquet/research_groups_canonical.parquet".to_string()));
    }

    /// The degraded path: no original archive means everything goes out as text.
    #[test]
    fn works_without_an_original_archive() {
        let conn = db_with_researcher();
        let (out, _, preserved) = build_archive(&conn, None, |_| {}).unwrap();
        assert_eq!(preserved, 0);

        let types = parquet_io::schema_of(bytes::Bytes::from(entry(
            &out,
            "parquet/researchers_canonical.parquet",
        )))
        .unwrap();
        assert_eq!(types["was_student"], DataType::Utf8, "sem original, tudo é texto");
    }

    #[test]
    fn unparseable_numbers_become_null_not_errors() {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO researchers (id, classification_confidence) VALUES (1, 'lixo')",
            [],
        )
        .unwrap();
        let (out, _, _) = build_archive(&conn, Some(&typed_original()), |_| {}).unwrap();
        let table = parquet_io::read_columns(
            bytes::Bytes::from(entry(&out, "parquet/researchers_canonical.parquet")),
            &["classification_confidence"],
        )
        .unwrap();
        assert_eq!(table.rows[0][0], None, "valor inválido vira null, como no to_numeric coerce");
    }

    #[test]
    fn boolean_accepts_every_spelling_the_python_map_accepts() {
        for (text, want) in [("1", true), ("True", true), ("true", true), ("0", false), ("False", false)] {
            assert_eq!(as_bool(text), Some(want), "não converteu {text}");
        }
        assert_eq!(as_bool("talvez"), None);
    }

    /// The criterion the whole feature exists for: import the real 35 MB archive,
    /// export it back, and check that nothing was lost or retyped.
    #[test]
    fn real_archive_round_trip_preserves_schema_and_entries() {
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

        // 626 original entries, 30 of them replaced by this export.
        assert_eq!(preserved, 596, "as entradas não gerenciadas devem sair intactas");
        assert_eq!(tables.len(), 15);

        let mut src = zip::ZipArchive::new(std::io::Cursor::new(&original)).unwrap();
        let mut dst = zip::ZipArchive::new(std::io::Cursor::new(&out)).unwrap();
        assert_eq!(dst.len(), 626, "o pacote exportado deve ter o mesmo total de entradas");

        // The 22.6 MB nested archive must survive byte for byte, uncompressed
        // and recompressed by nobody.
        let nested_before = entry(&original, "data_snapshot.zip");
        let nested_after = entry(&out, "data_snapshot.zip");
        assert_eq!(nested_before.len(), nested_after.len());
        assert_eq!(nested_before, nested_after, "o zip aninhado foi alterado");

        // Every canonical table must come back with the original column types
        // and the original column order.
        for def in registry::exported() {
            let path = format!("parquet/{}_canonical.parquet", def.table);

            let mut before = Vec::new();
            src.by_name(&path).unwrap().read_to_end(&mut before).unwrap();
            let mut after = Vec::new();
            dst.by_name(&path).unwrap().read_to_end(&mut after).unwrap();

            let t_before = parquet_io::schema_of(bytes::Bytes::from(before.clone())).unwrap();
            let t_after = parquet_io::schema_of(bytes::Bytes::from(after.clone())).unwrap();
            let o_before = parquet_io::column_order(bytes::Bytes::from(before)).unwrap();
            let o_after = parquet_io::column_order(bytes::Bytes::from(after)).unwrap();

            // Only columns the registry manages travel; the rest are dropped by
            // the same rule pandas applies today.
            let expected_order: Vec<String> =
                o_before.iter().filter(|c| def.has_column(c)).cloned().collect();
            assert_eq!(o_after, expected_order, "{}: ordem de colunas mudou", def.table);

            for col in &o_after {
                assert_eq!(
                    arrow_field_type(&t_after[col]),
                    arrow_field_type(&t_before[col]),
                    "{}.{}: tipo não voltou ao original",
                    def.table, col
                );
            }
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_table_still_produces_valid_files() {
        let conn = db::open_in_memory().unwrap();
        let (out, tables, _) = build_archive(&conn, None, |_| {}).unwrap();
        assert!(tables.iter().all(|t| t.rows == 0));
        let json = String::from_utf8(entry(&out, "languages_canonical.json")).unwrap();
        assert_eq!(json.trim(), "[]");
        parquet_io::schema_of(bytes::Bytes::from(entry(&out, "parquet/languages_canonical.parquet")))
            .unwrap();
    }
}
