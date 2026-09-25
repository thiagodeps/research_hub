//! Canonical JSON table reading (SEP-032).
//!
//! The DataLake replaced the parquet pair with plain `{table}_canonical.json`
//! files. Values become text on the way in — same contract as `parquet_io` —
//! with the spellings the export path already understands: booleans as
//! `"True"/"False"`, numbers in their own decimal form, `null` as SQL NULL,
//! and lists/objects as compact JSON text.

use crate::error::AppError;
use serde_json::{Map, Value as Json};
use std::collections::HashMap;

/// One canonical JSON file read into text columns, keyed by column name.
pub struct Table {
    pub columns: Vec<String>,
    /// `rows[i][j]` is the value of `columns[j]` in row `i`. `None` is SQL NULL.
    pub rows: Vec<Vec<Option<String>>>,
}

impl Table {
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}

/// Render one JSON value as the text SQLite will store (research R5).
fn cell(value: &Json) -> Option<String> {
    match value {
        Json::Null => None,
        Json::Bool(true) => Some("True".to_string()),
        Json::Bool(false) => Some("False".to_string()),
        // `to_string` on a Number preserves its own form: 23 -> "23", 23.0 -> "23.0".
        Json::Number(n) => Some(n.to_string()),
        Json::String(s) => Some(s.clone()),
        // Relationship arrays and nested objects travel as compact JSON text.
        Json::Array(_) | Json::Object(_) => Some(value.to_string()),
    }
}

/// Read a canonical JSON file, keeping only `wanted` columns.
///
/// Rows may carry extra fields (upstream evolution) — ignored. Managed fields
/// absent from a row read as NULL. Column order follows the file's key order.
pub fn read_columns(bytes: &[u8], wanted: &[&str]) -> Result<Table, AppError> {
    let records: Vec<Json> = serde_json::from_slice(bytes)
        .map_err(|e| AppError::Internal(format!("JSON canônico inválido: {e}")))?;

    let objects: Vec<&Map<String, Json>> = records
        .iter()
        .map(|r| match r {
            Json::Object(map) => Ok(map),
            other => Err(AppError::Internal(format!(
                "JSON canônico inválido: registro não é um objeto ({other})"
            ))),
        })
        .collect::<Result<_, _>>()?;

    // Column order: keys in first-appearance order across rows, filtered by
    // `wanted` — the JSON analogue of parquet's schema order.
    let mut columns: Vec<String> = Vec::new();
    for map in &objects {
        for key in map.keys() {
            if wanted.contains(&key.as_str()) && !columns.contains(key) {
                columns.push(key.clone());
            }
        }
    }

    let rows: Vec<Vec<Option<String>>> = objects
        .iter()
        .map(|map| {
            columns
                .iter()
                .map(|c| map.get(c).and_then(cell))
                .collect()
        })
        .collect();

    Ok(Table { columns, rows })
}

/// Column type as declared by the first non-null value of the original file.
///
/// JSON has no schema, so the export path infers one per column — the JSON
/// analogue of reading the parquet schema (research R2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColType {
    Bool,
    Int,
    Float,
    Text,
    Structure,
}

/// Infer per-column types from a canonical JSON file.
pub fn infer_column_types(bytes: &[u8]) -> Result<HashMap<String, ColType>, AppError> {
    let records: Vec<Json> = serde_json::from_slice(bytes)
        .map_err(|e| AppError::Internal(format!("JSON canônico inválido: {e}")))?;

    let mut types: HashMap<String, ColType> = HashMap::new();
    for record in &records {
        let Json::Object(map) = record else {
            return Err(AppError::Internal(
                "JSON canônico inválido: registro não é um objeto".to_string(),
            ));
        };
        for (key, value) in map {
            if types.contains_key(key) {
                continue; // first non-null wins, like pandas inference
            }
            if value.is_null() {
                continue;
            }
            types.insert(
                key.clone(),
                match value {
                    Json::Bool(_) => ColType::Bool,
                    Json::Number(n) if n.is_i64() || n.is_u64() => ColType::Int,
                    Json::Number(_) => ColType::Float,
                    Json::String(_) => ColType::Text,
                    Json::Array(_) | Json::Object(_) => ColType::Structure,
                    Json::Null => unreachable!("filtrado acima"),
                },
            );
        }
    }
    Ok(types)
}

/// Column names in first-appearance order across all records.
///
/// Used by the export path to intersect the original column set with what the
/// database manages.
pub fn column_order(bytes: &[u8]) -> Result<Vec<String>, AppError> {
    let records: Vec<Json> = serde_json::from_slice(bytes)
        .map_err(|e| AppError::Internal(format!("JSON canônico inválido: {e}")))?;
    let mut order: Vec<String> = Vec::new();
    for record in &records {
        let Json::Object(map) = record else {
            return Err(AppError::Internal(
                "JSON canônico inválido: registro não é um objeto".to_string(),
            ));
        };
        for key in map.keys() {
            if !order.contains(key) {
                order.push(key.clone());
            }
        }
    }
    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(canon: &[u8], wanted: &[&str]) -> Result<Table, AppError> {
        read_columns(canon, wanted)
    }

    /// Value of `column` in `row`, found by name (key order in JSON files is
    /// alphabetical, so positional asserts would be brittle).
    fn at<'a>(table: &'a Table, row: usize, column: &str) -> &'a Option<String> {
        let i = table.columns.iter().position(|c| c == column).unwrap();
        &table.rows[row][i]
    }

    /// T002: values render as the text SQLite stores, with export-compatible spellings.
    #[test]
    fn renders_values_as_storage_text() {
        let canon = br#"[
            {"id": 1, "name": "Serra", "active": true, "score": 23, "weight": 23.0,
             "links": [{"id": 3}], "extra": "ignorar"}
        ]"#;
        let table = read(
            canon,
            &["id", "name", "active", "score", "weight", "links", "extra"],
        )
        .unwrap();

        assert_eq!(table.len(), 1);
        assert_eq!(at(&table, 0, "id"), &Some("1".into()), "inteiro vira texto");
        assert_eq!(at(&table, 0, "name"), &Some("Serra".into()));
        assert_eq!(at(&table, 0, "active"), &Some("True".into()), "booleano usa True/False");
        assert_eq!(at(&table, 0, "score"), &Some("23".into()));
        assert_eq!(at(&table, 0, "weight"), &Some("23.0".into()), "decimal preserva a forma");
        assert_eq!(
            at(&table, 0, "links"),
            &Some(r#"[{"id":3}]"#.into()),
            "estrutura vira JSON compacto"
        );
        assert_eq!(at(&table, 0, "extra"), &Some("ignorar".into()));
    }

    /// T002: only wanted columns are picked; extra and absent fields behave.
    #[test]
    fn filters_wanted_columns_and_tolerates_missing() {
        let canon = br#"[
            {"id": 1, "name": "A", "campus": {"id": 1, "name": "A"}},
            {"id": 2}
        ]"#;
        let table = read(canon, &["id", "name", "parent_id"]).unwrap();

        // Columns are the keys the file actually carries (parquet semantics):
        // "parent_id" is wanted but never appears, so it is not a column here.
        assert_eq!(table.columns, vec!["id", "name"]);
        assert_eq!(at(&table, 0, "id"), &Some("1".into()));
        assert_eq!(at(&table, 0, "name"), &Some("A".into()));
        assert_eq!(at(&table, 1, "name"), &None, "campo ausente na linha é NULL");
        // The nested "campus" field was never requested, so it vanished.
        assert!(!table.columns.iter().any(|c| c == "campus"));
    }

    /// T002: empty file is a valid empty table; malformed JSON errors.
    #[test]
    fn empty_table_is_valid_and_malformed_errors() {
        let table = read(b"[]", &["id"]).unwrap();
        assert_eq!(table.len(), 0);
        assert!(table.columns.is_empty());

        assert!(read(b"{nao e json", &["id"]).is_err());
        assert!(read(br#"[1, 2]"#, &["id"]).is_err(), "registro não-objeto erro");
    }

    /// T012 (US2, FR-008): the legacy nested campus object is discarded, never loaded.
    #[test]
    fn discards_nested_campus_field() {
        let canon = br#"[
            {"id": 6, "name": "Serra", "description": null, "short_name": null,
             "organization_id": 1, "parent_id": null,
             "campus": {"id": 6, "name": "Serra"}}
        ]"#;
        let table = read(
            canon,
            &["id", "name", "description", "short_name", "organization_id", "parent_id"],
        )
        .unwrap();

        assert_eq!(table.columns.len(), 6);
        assert!(!table.columns.iter().any(|c| c == "campus"));
        assert_eq!(at(&table, 0, "name"), &Some("Serra".into()));
        assert_eq!(at(&table, 0, "parent_id"), &None);
    }

    /// T019 seed (US3, R2): per-column type from the first non-null value.
    #[test]
    fn infers_column_types_from_first_non_null() {
        let canon = br#"[
            {"id": 1, "name": null, "active": true, "score": 23, "weight": 0.5,
             "links": [{"id": 3}], "org": "IFES"},
            {"id": 2, "name": "Serra", "score": 24.0}
        ]"#;
        let types = infer_column_types(canon).unwrap();

        assert_eq!(types["id"], ColType::Int);
        assert_eq!(types["name"], ColType::Text, "null inicial não define tipo");
        assert_eq!(types["active"], ColType::Bool);
        assert_eq!(types["score"], ColType::Int, "primeiro não-null vence");
        assert_eq!(types["weight"], ColType::Float);
        assert_eq!(types["links"], ColType::Structure);
        assert_eq!(types["org"], ColType::Text);
        assert!(!types.contains_key("desconhecida"));
    }

    /// T019 seed: all-null columns stay untyped; malformed errors.
    #[test]
    fn all_null_columns_stay_untyped() {
        let types = infer_column_types(br#"[{"id": 1, "name": null}]"#).unwrap();
        assert_eq!(types.len(), 1);
        assert!(infer_column_types(b"quebra").is_err());
    }
}
