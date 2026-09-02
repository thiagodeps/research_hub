//! Parquet reading via arrow-rs (SEP-018).
//!
//! Values become text on the way in; the canonical types live in the original
//! archive and are restored on export (SEP-019), which is why nothing here
//! tries to be clever about them.

use crate::error::AppError;
use arrow::array::{Array, AsArray};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::collections::HashMap;

/// One parquet file read into text columns, keyed by column name.
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

/// Render one cell as the text SQLite will store.
///
/// Booleans become "True"/"False" rather than "true"/"false": that is what
/// pandas writes when it stringifies, and the export path recognizes both
/// spellings when casting back (see SEP-019).
fn cell(batch: &RecordBatch, col: usize, row: usize) -> Option<String> {
    let array = batch.column(col);
    if array.is_null(row) {
        return None;
    }
    let text = match array.data_type() {
        DataType::Boolean => {
            if array.as_boolean().value(row) { "True".to_string() } else { "False".to_string() }
        }
        DataType::Utf8 => array.as_string::<i32>().value(row).to_string(),
        DataType::LargeUtf8 => array.as_string::<i64>().value(row).to_string(),
        _ => {
            // Everything else (ints, floats, dates, nested) goes through arrow's
            // own display, which round-trips values faithfully as text.
            match arrow::util::display::array_value_to_string(array, row) {
                Ok(s) => s,
                Err(_) => return None,
            }
        }
    };
    Some(text)
}

/// Read a parquet file, keeping only `wanted` columns.
pub fn read_columns(bytes: bytes::Bytes, wanted: &[&str]) -> Result<Table, AppError> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)
        .map_err(|e| AppError::Internal(format!("parquet inválido: {e}")))?;

    let schema = builder.schema().clone();
    // Positions in the file of the columns we care about, in file order.
    let picked: Vec<(usize, String)> = schema
        .fields()
        .iter()
        .enumerate()
        .filter(|(_, f)| wanted.contains(&f.name().as_str()))
        .map(|(i, f)| (i, f.name().clone()))
        .collect();

    let columns: Vec<String> = picked.iter().map(|(_, n)| n.clone()).collect();
    let mut rows = Vec::new();

    let reader = builder
        .build()
        .map_err(|e| AppError::Internal(format!("falha ao ler parquet: {e}")))?;

    for batch in reader {
        let batch = batch.map_err(|e| AppError::Internal(format!("lote inválido: {e}")))?;
        for r in 0..batch.num_rows() {
            rows.push(picked.iter().map(|(c, _)| cell(&batch, *c, r)).collect());
        }
    }

    Ok(Table { columns, rows })
}

/// Column name to arrow type, without reading any data.
/// Used by the export path to restore original types.
pub fn schema_of(bytes: bytes::Bytes) -> Result<HashMap<String, DataType>, AppError> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)
        .map_err(|e| AppError::Internal(format!("parquet inválido: {e}")))?;
    Ok(builder
        .schema()
        .fields()
        .iter()
        .map(|f| (f.name().clone(), f.data_type().clone()))
        .collect())
}

/// Column names in the order the file declares them.
pub fn column_order(bytes: bytes::Bytes) -> Result<Vec<String>, AppError> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(bytes)
        .map_err(|e| AppError::Internal(format!("parquet inválido: {e}")))?;
    Ok(builder.schema().fields().iter().map(|f| f.name().clone()).collect())
}

/// `parquet/researchers_canonical.parquet` -> `researchers`.
pub fn table_name_from_path(path: &str) -> String {
    let base = path.rsplit('/').next().unwrap_or(path);
    let stem = base.strip_suffix(".parquet").unwrap_or(base);
    stem.strip_suffix("_canonical").unwrap_or(stem).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_table_name_from_canonical_path() {
        assert_eq!(table_name_from_path("parquet/researchers_canonical.parquet"), "researchers");
        assert_eq!(table_name_from_path("parquet/research_groups_canonical.parquet"), "research_groups");
        // Non-canonical files keep their stem and simply will not match a table.
        assert_eq!(table_name_from_path("parquet/people_relationship_graph.nodes.parquet"), "people_relationship_graph.nodes");
        assert_eq!(table_name_from_path("articles_canonical.parquet"), "articles");
    }
}
