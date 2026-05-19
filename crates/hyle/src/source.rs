use std::sync::OnceLock;

use serde::Deserialize;

use crate::raw::Source;
use crate::Value;

/// Raw storage field types — how a field value is encoded in a source backend.
///
/// These describe the on-disk/wire format, not the blueprint semantics.
/// Any backend (qmap, Postgres, SQLite) maps its native types to these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldStorageType {
	String,
	Int,
	Bool,
	NullableString,
	Reference,
	MultiReference,
	Inverse,
}

impl FieldStorageType {
	/// Convert from a C u32 discriminant (0–6).
	pub fn from_u32(t: u32) -> Self {
		match t {
			0 => Self::String,
			1 => Self::Int,
			2 => Self::Bool,
			3 => Self::NullableString,
			4 => Self::Reference,
			5 => Self::MultiReference,
			6 => Self::Inverse,
			_ => Self::String,
		}
	}
}

/// Metadata for a single field in a source definition.
#[derive(Debug, Clone)]
pub struct SourceFieldDef {
	pub name: String,
	pub field_type: FieldStorageType,
	pub target_source: Option<String>,
	pub inverse_name: Option<String>,
}

/// Describes a source's field layout — backend-agnostic.
///
/// Backend-specific details (handles, C FFI pointers) live in backend
/// provider crates (e.g. `hyle-source-qmap`), not here.
#[derive(Debug, Clone)]
pub struct SourceDef {
	pub id: String,
	pub fields: Vec<SourceFieldDef>,
}

/// Pluggable source loader.
///
/// Register an implementation with [`set_provider`] at startup.
/// The default (`hyle-ndc` with `qmap` feature) implementation loads
/// from ndc qmap handles.
pub trait SourceProvider: Send + Sync {
	fn load_source(&self, id: &str) -> Option<Source>;
}

static PROVIDER: OnceLock<Box<dyn SourceProvider>> = OnceLock::new();

/// Register the source provider.  Called once at module init.
/// Returns `Err(p)` if a provider is already registered.
pub fn set_provider(p: Box<dyn SourceProvider>) -> Result<(), Box<dyn SourceProvider>> {
	PROVIDER.set(p)
}

#[doc(hidden)]
pub fn get_provider() -> Option<&'static dyn SourceProvider> {
	PROVIDER.get().map(|p| p.as_ref())
}

/// Load a source by id.
pub fn load_source(id: &str) -> Option<Source> {
	get_provider()?.load_source(id)
}

/// Find a row by item id within a source.
///
/// Generic helper with no JSON assumption — works with any provider backend.
pub fn find_item(source: &Source, model_id: &str, item_id: &str) -> Option<crate::Row> {
	let model_id = model_id.strip_suffix(".items").unwrap_or(model_id);
	source.get(model_id)?.rows().into_iter().find(|row| {
		row.get("id").map_or(false, |v| matches!(v, Value::String(s) if s == item_id))
	})
}

/// Extract a field from a JSON string.
#[cfg(feature = "json")]
pub fn extract_field(json: &str, field: &str) -> Option<String> {
	serde_json::from_str::<serde_json::Value>(json)
		.ok()
		.and_then(|v| v.get(field).and_then(|v| v.as_str()).map(String::from))
}

/// Serialize a full source to a JSON string.
#[cfg(feature = "json")]
pub fn source_to_json(source: &Source) -> Option<String> {
	serde_json::to_string(source).ok()
}

/// Load a source and return it as a JSON string.
#[cfg(feature = "json")]
pub fn load_source_json(id: &str) -> Option<String> {
	let source = load_source(id)?;
	source_to_json(&source)
}

/// Load a single item and return its JSON.
#[cfg(feature = "json")]
pub fn load_item_json(id: &str, item_id: &str) -> Option<String> {
	let source = load_source(id)?;
	let model_id = id.strip_suffix(".items").unwrap_or(id);
	let row = find_item(&source, model_id, item_id)?;
	serde_json::to_string(&row).ok()
}

/// Load a single item and deserialize it into a typed struct.
/// Returns `Ok(Some(item))` on success, `Ok(None)` if not found, or `Err(msg)` on parse error.
pub fn load_typed_item<T: for<'de> Deserialize<'de>>(
	id: &str,
	item_id: &str,
) -> Result<Option<T>, String> {
	let source = match load_source(id) {
		Some(s) => s,
		None => return Ok(None),
	};
	let model_id = id.strip_suffix(".items").unwrap_or(id);
	let row = match find_item(&source, model_id, item_id) {
		Some(r) => r,
		None => return Ok(None),
	};
	deser_row(&row).map(Some).map_err(|e| format!("{e}"))
}

pub fn deser_row<'de, T: serde::Deserialize<'de>>(row: &'de crate::Row) -> Result<T, serde::de::value::Error> {
	let md = serde::de::value::MapDeserializer::new(
		row.iter().map(|(k, v)| (k.as_str(), v)),
	);
	T::deserialize(md)
}

/// Load a full source and deserialize its rows into a typed vec.
pub fn load_typed_rows<T: for<'de> Deserialize<'de>>(id: &str) -> Vec<T> {
	let source = match load_source(id) {
		Some(s) => s,
		None => return Vec::new(),
	};
	let model_id = id.strip_suffix(".items").unwrap_or(id);
	let result = match source.get(model_id) {
		Some(r) => r,
		None => return Vec::new(),
	};
	result.rows().iter()
		.filter_map(|row| deser_row(row).ok())
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ModelResult;
	use std::sync::Once;

	struct TestProvider;

	impl SourceProvider for TestProvider {
		fn load_source(&self, id: &str) -> Option<crate::raw::Source> {
			let model_id = id.strip_suffix(".items").unwrap_or(id);
			let mut source = crate::raw::Source::new();
			let mut rows = Vec::new();

			let mut row1 = crate::Row::new();
			row1.insert("id".to_string(), Value::String("a".to_owned()));
			row1.insert("name".to_string(), Value::String("Alice".to_owned()));
			rows.push(row1);

			let mut row2 = crate::Row::new();
			row2.insert("id".to_string(), Value::String("b".to_owned()));
			row2.insert("name".to_string(), Value::String("Bob".to_owned()));
			rows.push(row2);

			source.insert(model_id.to_string(), ModelResult::many(rows));
			Some(source)
		}
	}

	static INIT: Once = Once::new();

	fn init() {
		INIT.call_once(|| {
			let _ = set_provider(Box::new(TestProvider));
		});
	}

	#[test]
	fn find_item_found() {
		init();
		let source = load_source("test.items").unwrap();
		let row = find_item(&source, "test", "a");
		assert!(row.is_some());
		assert_eq!(row.unwrap().get("name").and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None }), Some("Alice"));
	}

	#[test]
	fn find_item_not_found() {
		init();
		let source = load_source("test.items").unwrap();
		let row = find_item(&source, "test", "nonexistent");
		assert!(row.is_none());
	}

	#[cfg(feature = "json")]
	#[test]
	fn extract_field_existing() {
		assert_eq!(
			extract_field(r#"{"id":"a","name":"Alice"}"#, "name"),
			Some("Alice".to_string())
		);
	}

	#[cfg(feature = "json")]
	#[test]
	fn extract_field_missing() {
		assert_eq!(extract_field(r#"{"id":"a"}"#, "missing"), None);
	}

	#[cfg(feature = "json")]
	#[test]
	fn extract_field_bad_json() {
		assert_eq!(extract_field("not json", "field"), None);
	}

	#[test]
	fn load_source_returns_source() {
		init();
		let source = load_source("test.items");
		assert!(source.is_some());
		let source = source.unwrap();
		let result = source.get("test");
		assert!(result.is_some());
		assert_eq!(result.unwrap().rows().len(), 2);
	}

	#[cfg(feature = "json")]
	#[test]
	fn source_to_json_returns_valid_json() {
		init();
		let source = load_source("test.items").unwrap();
		let json = source_to_json(&source);
		assert!(json.is_some());
		let json = json.unwrap();
		let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
		assert!(parsed.get("test").is_some());
	}

	#[cfg(feature = "json")]
	#[test]
	fn load_item_json_found() {
		init();
		let json = load_item_json("test.items", "a");
		assert!(json.is_some());
		let json = json.unwrap();
		assert!(json.contains("Alice"));
	}

	#[cfg(feature = "json")]
	#[test]
	fn load_item_json_not_found() {
		init();
		let json = load_item_json("test.items", "nonexistent");
		assert!(json.is_none());
	}

	#[test]
	fn load_typed_found() {
		init();
		#[derive(Deserialize)]
		struct Item {
			id: String,
			name: String,
		}
		let item: Result<Option<Item>, String> = load_typed_item("test.items", "a");
		let item = item.unwrap().unwrap();
		assert_eq!(item.id, "a");
		assert_eq!(item.name, "Alice");
	}

	#[test]
	fn load_typed_not_found() {
		init();
		let item: Result<Option<serde_json::Value>, String> =
			load_typed_item("test.items", "nonexistent");
		assert!(item.unwrap().is_none());
	}

	#[test]
	fn load_typed_rows_returns_all() {
		init();
		#[derive(Deserialize)]
		struct Item {
			id: String,
			name: String,
		}
		let items: Vec<Item> = load_typed_rows("test.items");
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].name, "Alice");
		assert_eq!(items[1].name, "Bob");
	}

	#[test]
	fn set_provider_behavior() {
		struct P;
		impl SourceProvider for P {
			fn load_source(&self, _: &str) -> Option<crate::raw::Source> {
				None
			}
		}
		let r1 = set_provider(Box::new(P));
		if r1.is_ok() {
			assert!(get_provider().is_some());
			assert!(set_provider(Box::new(P)).is_err());
		}
	}
}

// ── C source registry FFI ─────────────────────────────────────────────────────

#[cfg(all(feature = "csource", not(target_arch = "wasm32")))]
pub mod csource {
	use std::ffi::{CStr, CString, c_char, c_int, c_void};
	use crate::{Row, Value};
	use indexmap::IndexMap;

	// ---- C types ------------------------------------------------------------

	#[repr(C)]
	struct HyleRowSet {
		row_hd:    u32,
		fields_hd: u32,
	}

	#[repr(C)]
	struct HyleQuery {
		sort_field:    *const c_char,
		sort_asc:      bool,
		page:          u32,
		per_page:      u32,
		q:             *const c_char,
		filters:       *mut HyleFieldFilter,
		filter_count:  u32,
		include:       *const *const c_char,
		include_count: u32,
	}

	#[repr(C)]
	struct HyleFieldFilter {
		field: *const c_char,
		value: *const c_char,
	}

	#[repr(C)]
	struct HyleSourceRow {
		id:           *const c_char,
		field_names:  *const *const c_char,
		field_values: *const *const c_char,
		field_count:  usize,
	}

	/// Mirror of `hyle_field_t` in C.  All pointer fields may be null.
	#[repr(C)]
	struct HyleField {
		name:          *const c_char,
		field_type:    u32,
		writable:      c_int,
		target_source: *const c_char,
		inverse_name:  *const c_char,
		required:      c_int,
		min:           i64,
		max:           i64,
		min_length:    usize,
		max_length:    usize,
		pattern:       *const c_char,
	}

	unsafe extern "C" {
		fn hyle_source_register(
			source_id:   *const c_char,
			fields:      *const HyleField,
			field_count: usize,
			record_id:   u32,
			flags:       u32,
			user:        *mut c_void,
		) -> u32;
		fn hyle_source_put(
			source_id: *const c_char,
			row_id:    *const c_char,
			names:     *const *const c_char,
			values:    *const *const c_char,
			count:     usize,
		) -> c_int;
		fn hyle_source_del(source_id: *const c_char, row_id: *const c_char);
		fn hyle_source_query(
			source_id: *const c_char,
			query:     *const HyleQuery,
			out:       *mut HyleRowSet,
			total_out: *mut usize,
		) -> c_int;
		fn hyle_row_set_free(rs: *mut HyleRowSet);
		fn hyle_row_set_to_rows(
			rs:        *const HyleRowSet,
			rows_out:  *mut *mut HyleSourceRow,
			count_out: *mut usize,
		) -> c_int;
		fn hyle_source_rows_free(rows: *mut HyleSourceRow, count: usize);
	}

	// ---- field type ---------------------------------------------------------

	/// Field type discriminants matching `hyle_field_type_t` in C.
	#[derive(Clone, Copy, Debug)]
	#[repr(u32)]
	pub enum FieldType {
		String         = 0,
		Int            = 1,
		Bool           = 2,
		NullableString = 3,
		Reference      = 4,
		MultiReference = 5,
		Inverse        = 6,
	}

	/// Description of a single field for source registration.
	pub struct FieldDef {
		pub name: &'static str,
		pub kind: FieldType,
	}

	// ---- storage held alive for process lifetime ----------------------------

	/// Strings and HyleField arrays that must outlive the C registration call.
	/// Leaked intentionally — source registrations are permanent.
	struct Registration {
		_names:  Vec<CString>,
		_fields: Vec<HyleField>,
	}

	fn build_registration(id_cs: &CString, fields: &[FieldDef]) -> Registration {
		let names: Vec<CString> = fields
			.iter()
			.map(|f| CString::new(f.name).expect("field name must not contain nul"))
			.collect();
		let hyle_fields: Vec<HyleField> = fields
			.iter()
			.zip(names.iter())
			.map(|(f, name_cs)| HyleField {
				name:          name_cs.as_ptr(),
				field_type:    f.kind as u32,
				writable:      0,
				target_source: std::ptr::null(),
				inverse_name:  std::ptr::null(),
				required:      0,
				min:           0,
				max:           0,
				min_length:    0,
				max_length:    0,
				pattern:       std::ptr::null(),
			})
			.collect();
		unsafe {
			hyle_source_register(
				id_cs.as_ptr(),
				hyle_fields.as_ptr(),
				hyle_fields.len(),
				0,
				0,
				std::ptr::null_mut(),
			);
		}
		Registration { _names: names, _fields: hyle_fields }
	}

	/// Register a source with libhyle. `fields_hd` returned by C is discarded
	/// (Rust doesn't need it directly).  Leaks the registration storage.
	pub fn register_source(source_id: &'static str, fields: &[FieldDef]) {
		let id_cs = CString::new(source_id).expect("source_id must not contain nul");
		let reg = build_registration(&id_cs, fields);
		Box::leak(Box::new((id_cs, reg)));
	}

	// ---- put / del ----------------------------------------------------------

	fn value_to_cstring(v: &Value) -> CString {
		let s = match v {
			Value::String(s)  => s.clone(),
			Value::Int(n)     => n.to_string(),
			Value::Bool(b)    => (if *b { "true" } else { "false" }).to_owned(),
			Value::Float(f)   => f.to_string(),
			Value::Array(arr) => arr
				.iter()
				.map(|v| match v {
					Value::String(s) => s.clone(),
					other => format!("{other:?}"),
				})
				.collect::<Vec<_>>()
				.join("\n"),
			Value::Bytes(_) | Value::Map(_) | Value::Null => String::new(),
		};
		CString::new(s).unwrap_or_default()
	}

	/// Add or update a row.  `row` is a `Row` (IndexMap<String, Value>).
	/// The row must contain an `"id"` field used as the row key.
	pub fn source_put(source_id: &str, row: &Row) {
		let id_val = match row.get("id") {
			Some(Value::String(s)) => s.clone(),
			Some(Value::Int(n))    => n.to_string(),
			_                      => return,
		};
		let id_cs  = match CString::new(source_id) { Ok(c) => c, Err(_) => return };
		let row_cs = match CString::new(id_val)    { Ok(c) => c, Err(_) => return };

		let names:  Vec<CString> = row.keys()
			.map(|k| CString::new(k.as_str()).unwrap_or_default())
			.collect();
		let values: Vec<CString> = row.values()
			.map(value_to_cstring)
			.collect();
		let name_ptrs:  Vec<*const c_char> = names.iter().map(|s| s.as_ptr()).collect();
		let value_ptrs: Vec<*const c_char> = values.iter().map(|s| s.as_ptr()).collect();

		unsafe {
			hyle_source_put(
				id_cs.as_ptr(),
				row_cs.as_ptr(),
				name_ptrs.as_ptr(),
				value_ptrs.as_ptr(),
				row.len(),
			);
		}
	}

	/// Remove a row from a registered source.
	pub fn source_del(source_id: &str, row_id: &str) {
		let id_cs  = match CString::new(source_id) { Ok(c) => c, Err(_) => return };
		let row_cs = match CString::new(row_id)     { Ok(c) => c, Err(_) => return };
		unsafe { hyle_source_del(id_cs.as_ptr(), row_cs.as_ptr()) };
	}

	// ---- query / find -------------------------------------------------------

	struct QueryBridge {
		_sort_field:   Option<CString>,
		_q:            Option<CString>,
		_filter_names: Vec<CString>,
		_filter_vals:  Vec<CString>,
		filters:       Vec<HyleFieldFilter>,
	}

	fn build_query_bridge(query: &crate::Query) -> (QueryBridge, HyleQuery) {
		let sort_field_cs = query.sort.as_ref()
			.map(|s| CString::new(s.field.as_str()).unwrap_or_default());
		let q_cs: Option<CString> = None;

		let mut filter_names: Vec<CString> = Vec::new();
		let mut filter_vals:  Vec<CString> = Vec::new();

		for (field, val) in &query.where_ {
			let val_str = match val {
				Value::String(s) if !s.is_empty() => s.clone(),
				Value::Bool(b) => (if *b { "true" } else { "false" }).to_owned(),
				Value::Int(n)  => n.to_string(),
				_ => continue,
			};
			filter_names.push(CString::new(field.as_str()).unwrap_or_default());
			filter_vals.push(CString::new(val_str).unwrap_or_default());
		}

		let filters: Vec<HyleFieldFilter> = filter_names.iter().zip(filter_vals.iter())
			.map(|(n, v)| HyleFieldFilter {
				field: n.as_ptr(),
				value: v.as_ptr(),
			})
			.collect();

		let cq = HyleQuery {
			sort_field:   sort_field_cs.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
			sort_asc:     query.sort.as_ref().map(|s| s.ascending).unwrap_or(true),
			page:         query.page.unwrap_or(0) as u32,
			per_page:     query.per_page.unwrap_or(0) as u32,
			q:            q_cs.as_ref().map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
			filters:      if filters.is_empty() {
				std::ptr::null_mut()
			} else {
				filters.as_ptr() as *mut HyleFieldFilter
			},
			filter_count: filters.len() as u32,
			include:      std::ptr::null(),
			include_count: 0,
		};

		let bridge = QueryBridge {
			_sort_field:   sort_field_cs,
			_q:            q_cs,
			_filter_names: filter_names,
			_filter_vals:  filter_vals,
			filters,
		};
		(bridge, cq)
	}

	fn row_set_to_rust(rs: &HyleRowSet) -> Vec<Row> {
		let mut rows_ptr: *mut HyleSourceRow = std::ptr::null_mut();
		let mut count: usize = 0;
		let rc = unsafe { hyle_row_set_to_rows(rs, &mut rows_ptr, &mut count) };
		if rc != 0 || rows_ptr.is_null() {
			return Vec::new();
		}
		let mut out = Vec::with_capacity(count);
		for i in 0..count {
			let r = unsafe { &*rows_ptr.add(i) };
			let id = unsafe { CStr::from_ptr(r.id) }.to_str().unwrap_or("").to_owned();
			let mut row: Row = IndexMap::new();
			row.insert("id".to_owned(), Value::String(id));
			for j in 0..r.field_count {
				let name = unsafe {
					CStr::from_ptr(*r.field_names.add(j))
						.to_str().unwrap_or("").to_owned()
				};
				let val_str = unsafe {
					CStr::from_ptr(*r.field_values.add(j))
						.to_str().unwrap_or("").to_owned()
				};
				if name != "id" {
					row.insert(name, Value::String(val_str));
				}
			}
			out.push(row);
		}
		unsafe { hyle_source_rows_free(rows_ptr, count) };
		out
	}

	/// Query a registered source with filter/sort/paginate.
	/// Returns `(rows_on_page, total_matching)`.
	pub fn source_query(
		source_id: &str,
		query: &crate::Query,
	) -> Result<(Vec<Row>, usize), String> {
		let id_cs = CString::new(source_id).map_err(|e| e.to_string())?;
		let (bridge, cq) = build_query_bridge(query);
		let _ = &bridge;

		let mut out = HyleRowSet { row_hd: 0, fields_hd: 0 };
		let mut total: usize = 0;

		unsafe {
			let rc = hyle_source_query(id_cs.as_ptr(), &cq, &mut out, &mut total);
			if rc != 0 {
				return Err(format!("hyle_source_query returned {rc}"));
			}
		}

		let rows = row_set_to_rust(&out);
		unsafe { hyle_row_set_free(&mut out) };
		Ok((rows, total))
	}
}

