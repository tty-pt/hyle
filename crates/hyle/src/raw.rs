use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::query::Manifest;

pub type Value = JsonValue;
pub type Row = IndexMap<String, Value>;
pub type Source = IndexMap<String, ModelResult>;

// ── Row constructors ──────────────────────────────────────────────────────────

/// Convert a URL-decoded form submission (`IndexMap<String, String>`) into a
/// `Row` (`IndexMap<String, Value>`).
///
/// Each string value becomes a `Value::String`.  This is the correct
/// representation for data arriving via an HTML `<form method="post">` or the
/// equivalent JSON-encoded form from a JS mutation.
///
/// # Example
/// ```
/// use indexmap::IndexMap;
/// let mut form = IndexMap::new();
/// form.insert("name".to_owned(), "Alice".to_owned());
/// let row = hyle::row_from_form(&form);
/// assert_eq!(row["name"], serde_json::json!("Alice"));
/// ```
pub fn row_from_form(form: &IndexMap<String, String>) -> Row {
    form.iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect()
}

/// Extract a `Row` from a `serde_json::Value` that is expected to be an object.
///
/// Returns an empty `Row` when `value` is not an object.  This matches the
/// behaviour needed when deserialising a JSON body sent by the JS mutation path
/// (where the body is already a `Value`).
///
/// # Example
/// ```
/// let body = serde_json::json!({ "name": "Alice", "active": true });
/// let row = hyle::row_from_value(&body);
/// assert_eq!(row["name"], serde_json::json!("Alice"));
/// assert_eq!(row["active"], serde_json::json!(true));
/// ```
pub fn row_from_value(value: &Value) -> Row {
    match value.as_object() {
        Some(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        None => IndexMap::new(),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResult {
    pub(crate) result: ModelRows,
    pub total: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum ModelRows {
    One(Row),
    Many(Vec<Row>),
}

impl ModelRows {
    pub(crate) fn rows(&self) -> Vec<Row> {
        match self {
            Self::One(row) => vec![row.clone()],
            Self::Many(rows) => rows.clone(),
        }
    }
}

impl ModelResult {
    pub fn one(row: Row) -> Self {
        Self {
            result: ModelRows::One(row),
            total: 1,
        }
    }

    pub fn many(rows: Vec<Row>) -> Self {
        let total = rows.len();
        Self {
            result: ModelRows::Many(rows),
            total,
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        self.result.rows()
    }
}

/// Normalise an outcome's rows into a flat `Vec<Row>`.
#[cfg_attr(not(feature = "wasm"), allow(dead_code))]
pub(crate) fn rows_from_outcome(outcome: &Outcome) -> Vec<Row> {
    outcome.rows.rows()
}

/// Returns `true` when the outcome represents a single record — either because
/// the query used `method: "one"` or because the source returned a single row
/// (`ModelRows::One`).
///
/// Use this to decide whether to treat `rows[0]` as *the* record rather than a
/// list entry.
pub fn is_single(manifest: &Manifest, outcome: &Outcome) -> bool {
    manifest.method.as_deref() == Some("one") || matches!(outcome.rows, ModelRows::One(_))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(hidden)]
pub struct Outcome {
    pub(crate) rows: ModelRows,
    pub total: usize,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub lookups: IndexMap<String, IndexMap<String, Row>>,
}

impl Outcome {
    /// Normalise this outcome's rows into a flat `Vec<Row>`.
    pub fn rows(&self) -> Vec<Row> {
        self.rows.rows()
    }

    /// Construct an empty outcome (no rows, no lookups). Useful for tests and stubs.
    pub fn empty() -> Self {
        Self {
            rows: ModelRows::Many(vec![]),
            total: 0,
            lookups: IndexMap::new(),
        }
    }
}

/// Convert a JSON value to a lookup key string (used for reference resolution).
///
/// Returns `Some` for `String` and `Number` values; `None` for all others.
pub(crate) fn value_to_lookup_key(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn row_from_form_converts_strings() {
        let mut form = IndexMap::new();
        form.insert("name".to_owned(), "Alice".to_owned());
        form.insert("active".to_owned(), "true".to_owned());
        let row = row_from_form(&form);
        assert_eq!(row["name"], json!("Alice"));
        assert_eq!(row["active"], json!("true")); // stays as string
    }

    #[test]
    fn row_from_form_empty() {
        let row = row_from_form(&IndexMap::new());
        assert!(row.is_empty());
    }

    #[test]
    fn row_from_value_object() {
        let body = json!({ "name": "Alice", "active": true });
        let row = row_from_value(&body);
        assert_eq!(row["name"], json!("Alice"));
        assert_eq!(row["active"], json!(true)); // preserves original type
    }

    #[test]
    fn row_from_value_non_object_returns_empty() {
        assert!(row_from_value(&json!(null)).is_empty());
        assert!(row_from_value(&json!("string")).is_empty());
        assert!(row_from_value(&json!([1, 2])).is_empty());
    }

    #[test]
    fn rows_from_outcome_normalises_single_row() {
        let row: Row = IndexMap::from([("id".to_owned(), json!(1))]);
        let mr = ModelResult::one(row.clone());
        let outcome = Outcome {
            rows: mr.result,
            total: 1,
            lookups: IndexMap::new(),
        };
        let rows = rows_from_outcome(&outcome);
        assert_eq!(rows, vec![row]);
    }
}
