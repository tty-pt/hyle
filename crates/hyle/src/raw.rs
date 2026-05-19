use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::query::Manifest;

/// Universal value type for hyle data.
///
/// Format-agnostic — providers construct these natively.
/// JSON interop is provided via `Serialize`/`Deserialize` for consumers that
/// need it (e.g. WASM FFI, HTTP responses).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
	Null,
	Bool(bool),
	Int(i64),
	Float(f64),
	String(String),
	Array(Vec<Value>),
	Bytes(Vec<u8>),
	Map(IndexMap<String, Value>),
}

// ── From impls for ergonomic construction ─────────────────────────

impl From<i64> for Value {
	fn from(n: i64) -> Self { Value::Int(n) }
}

impl From<f64> for Value {
	fn from(f: f64) -> Self { Value::Float(f) }
}

impl From<bool> for Value {
	fn from(b: bool) -> Self { Value::Bool(b) }
}

impl From<String> for Value {
	fn from(s: String) -> Self { Value::String(s) }
}

impl From<&str> for Value {
	fn from(s: &str) -> Self { Value::String(s.to_owned()) }
}

impl From<u32> for Value {
	fn from(n: u32) -> Self { Value::Int(n as i64) }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
	fn from(arr: Vec<T>) -> Self {
		Value::Array(arr.into_iter().map(Into::into).collect())
	}
}

// ── Display ───────────────────────────────────────────────────────

impl fmt::Display for Value {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Value::Null => write!(f, ""),
			Value::Bool(b) => write!(f, "{b}"),
			Value::Int(n) => write!(f, "{n}"),
			Value::Float(n) => write!(f, "{n}"),
			Value::String(s) => write!(f, "{s}"),
			Value::Bytes(b) => {
				for byte in b {
					write!(f, "{byte:02x}")?;
				}
				Ok(())
			}
			Value::Array(arr) => {
				for (i, v) in arr.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "{v}")?;
				}
				Ok(())
			}
			Value::Map(m) => {
				write!(f, "{{")?;
				for (i, (k, v)) in m.iter().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}
					write!(f, "{k}={v}")?;
				}
				write!(f, "}}")
			}
		}
	}
}

// ── Index impls ────────────────────────────────────────────────────────

static NULL: Value = Value::Null;

impl std::ops::Index<&str> for Value {
	type Output = Value;
	fn index(&self, key: &str) -> &Value {
		match self {
			Value::Map(map) => map.get(key).unwrap_or(&NULL),
			_ => &NULL,
		}
	}
}

impl std::ops::Index<usize> for Value {
	type Output = Value;
	fn index(&self, idx: usize) -> &Value {
		match self {
			Value::Array(arr) => arr.get(idx).unwrap_or(&NULL),
			_ => &NULL,
		}
	}
}

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
/// assert_eq!(row["name"], hyle::Value::String("Alice".to_owned()));
/// ```
pub fn row_from_form(form: &IndexMap<String, String>) -> Row {
	form.iter()
		.map(|(k, v)| (k.clone(), Value::String(v.clone())))
		.collect()
}

/// Extract a `Row` from a [`Value`] that is expected to be a map.
///
/// Returns an empty `Row` when `value` is not a map.  This matches the
/// behaviour needed when deserialising a JSON body sent by the JS mutation path.
///
/// # Example
/// ```
/// use indexmap::IndexMap;
/// let mut map = IndexMap::new();
/// map.insert("name".to_owned(), hyle::Value::String("Alice".to_owned()));
/// map.insert("active".to_owned(), hyle::Value::Bool(true));
/// let value = hyle::Value::Map(map);
/// let row = hyle::row_from_value(&value);
/// assert_eq!(row["name"], hyle::Value::String("Alice".to_owned()));
/// assert_eq!(row["active"], hyle::Value::Bool(true));
/// ```
pub fn row_from_value(value: &Value) -> Row {
	match value {
		Value::Map(map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
		_ => IndexMap::new(),
	}
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResult {
	pub result: ModelRows,
	pub total: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ModelRows {
	One(Row),
	Many(Vec<Row>),
}

impl ModelRows {
	pub fn rows(&self) -> Vec<Row> {
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
pub fn rows_from_outcome(outcome: &Outcome) -> Vec<Row> {
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
	pub rows: ModelRows,
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

/// Convert a value to a lookup key string (used for reference resolution).
///
/// Returns `Some` for `String`, `Int`, and `Float` values; `None` for all others.
pub(crate) fn value_to_lookup_key(value: &Value) -> Option<String> {
	match value {
		Value::String(s) => Some(s.clone()),
		Value::Int(n) => Some(n.to_string()),
		Value::Float(n) => Some(n.to_string()),
		_ => None,
	}
}

// ── RowItem ────────────────────────────────────────────────────────────

/// A single parsed row from an index file (one tab/space-separated line).
///
/// Used as the intermediate carrier between ndc's flat text index format and
/// a hyle `Source`.  Convert to a `Row` with `RowItem::into_row`.
#[derive(Clone, Debug)]
pub struct RowItem {
	pub id: String,
	pub title: String,
	pub extra: Vec<(String, String)>,
}

impl RowItem {
	/// Convert this `RowItem` into a hyle `Row` suitable for a `Source`.
	pub fn into_row(&self) -> Row {
		let mut row = Row::new();
		row.insert("id".to_owned(), Value::String(self.id.clone()));
		row.insert("title".to_owned(), Value::String(self.title.clone()));
		for (k, v) in &self.extra {
			row.insert(k.clone(), Value::String(v.clone()));
		}
		row
	}
}

/// Parse tab-separated index data into `RowItem` vecs.
///
/// Lines without a tab are split on the first space instead.
pub fn parse_index_items(body: &str, extra_keys: &[&str]) -> Vec<RowItem> {
	body.lines()
		.filter_map(|line| {
			let line = line.trim_end_matches('\r').trim();
			if line.is_empty() {
				return None;
			}
			if line.contains('\t') {
				let mut parts = line.splitn(2 + extra_keys.len(), '\t');
				let id = parts.next()?.to_string();
				let title = parts.next().unwrap_or("").to_string();
				let title = if title.is_empty() { id.clone() } else { title };
				let extra = extra_keys
					.iter()
					.zip(parts)
					.map(|(k, v)| (k.to_string(), v.to_string()))
					.collect();
				Some(RowItem { id, title, extra })
			} else {
				let idx = line.find(' ')?;
				let id = line[..idx].to_string();
				let title = line[idx + 1..].to_string();
				let title = if title.is_empty() { id.clone() } else { title };
				Some(RowItem { id, title, extra: vec![] })
			}
		})
		.collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
	use super::*;

	fn v_string(s: &str) -> Value {
		Value::String(s.to_owned())
	}

	#[test]
	fn row_from_form_converts_strings() {
		let mut form = IndexMap::new();
		form.insert("name".to_owned(), "Alice".to_owned());
		form.insert("active".to_owned(), "true".to_owned());
		let row = row_from_form(&form);
		assert_eq!(row["name"], v_string("Alice"));
		assert_eq!(row["active"], v_string("true")); // stays as string
	}

	#[test]
	fn row_from_form_empty() {
		let row = row_from_form(&IndexMap::new());
		assert!(row.is_empty());
	}

	#[test]
	fn row_from_value_object() {
		let mut map = IndexMap::new();
		map.insert("name".to_owned(), v_string("Alice"));
		map.insert("active".to_owned(), Value::Bool(true));
		let value = Value::Map(map);
		let row = row_from_value(&value);
		assert_eq!(row["name"], v_string("Alice"));
		assert_eq!(row["active"], Value::Bool(true)); // preserves original type
	}

	#[test]
	fn row_from_value_non_object_returns_empty() {
		assert!(row_from_value(&Value::Null).is_empty());
		assert!(row_from_value(&v_string("string")).is_empty());
		assert!(row_from_value(&Value::Array(vec![Value::Int(1), Value::Int(2)])).is_empty());
	}

	#[test]
	fn rows_from_outcome_normalises_single_row() {
		let row: Row = IndexMap::from([("id".to_owned(), Value::Int(1))]);
		let mr = ModelResult::one(row.clone());
		let outcome = Outcome {
			rows: mr.result,
			total: 1,
			lookups: IndexMap::new(),
		};
		let rows = rows_from_outcome(&outcome);
		assert_eq!(rows, vec![row]);
	}

	#[test]
	fn value_display_string() {
		assert_eq!(format!("{}", v_string("hello")), "hello");
	}

	#[test]
	fn value_display_int() {
		assert_eq!(format!("{}", Value::Int(42)), "42");
	}

	#[test]
	fn value_display_float() {
		assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
	}

	#[test]
	fn value_display_bool() {
		assert_eq!(format!("{}", Value::Bool(true)), "true");
		assert_eq!(format!("{}", Value::Bool(false)), "false");
	}

	#[test]
	fn value_display_null() {
		assert_eq!(format!("{}", Value::Null), "");
	}

	#[test]
	fn value_display_array() {
		let arr = Value::Array(vec![v_string("a"), v_string("b")]);
		assert_eq!(format!("{arr}"), "a, b");
	}

	#[test]
	fn value_to_lookup_key_string() {
		assert_eq!(
			value_to_lookup_key(&v_string("abc")),
			Some("abc".to_owned())
		);
	}

	#[test]
	fn value_to_lookup_key_int() {
		assert_eq!(value_to_lookup_key(&Value::Int(42)), Some("42".to_owned()));
	}

	#[test]
	fn value_to_lookup_key_null() {
		assert_eq!(value_to_lookup_key(&Value::Null), None);
	}
}
