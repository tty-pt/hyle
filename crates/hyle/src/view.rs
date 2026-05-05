use std::cmp::Ordering;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::blueprint::Blueprint;
use crate::error::{Error, HyleResult};
use crate::field::{Field, FieldType, Primitive};
use crate::query::Manifest;
use crate::raw::{Outcome, Row, Value, value_to_lookup_key};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub key: String,
    pub field: Field,
    pub label: String,
}

pub(crate) fn derive_columns(blueprint: &Blueprint, manifest: &Manifest) -> HyleResult<Vec<Column>> {
    let model = blueprint
        .models
        .get(&manifest.base)
        .ok_or_else(|| Error::UnknownModel(manifest.base.clone()))?;

    manifest
        .fields
        .iter()
        .map(|field_name| {
            let field = model
                .fields
                .get(field_name)
                .ok_or_else(|| Error::UnknownField {
                    model: manifest.base.clone(),
                    field: field_name.clone(),
                })?;

            Ok(Column {
                key: field_name.clone(),
                field: field.clone(),
                label: field.label.clone(),
            })
        })
        .collect()
}

/// Derive the 2-D filter layout from `manifest.filter_fields`.
///
/// Each inner `Vec<Column>` corresponds to one row of filter inputs as
#[cfg_attr(not(feature = "wasm"), allow(dead_code))]
pub(crate) fn derive_filter_layout(
    blueprint: &Blueprint,
    manifest: &Manifest,
) -> HyleResult<Vec<Vec<Column>>> {
    let model = blueprint
        .models
        .get(&manifest.base)
        .ok_or_else(|| Error::UnknownModel(manifest.base.clone()))?;

    let layout = manifest
        .filter_fields
        .iter()
        .map(|line| {
            line.iter()
                .filter_map(|field_name| {
                    let field = model.fields.get(field_name)?;
                    Some(Column {
                        key: field_name.clone(),
                        field: field.clone(),
                        label: field.label.clone(),
                    })
                })
                .collect()
        })
        .collect();

    Ok(layout)
}

pub fn filter_rows(rows: &[Row], filters: &IndexMap<String, Value>) -> Vec<Row> {
    rows.iter()
        .filter(|row| {
            filters.iter().all(|(key, filter)| {
                if filter_is_empty(filter) {
                    return true;
                }

                let row_val = row.get(key);

                // Array filter (from JS multi-select): every selected id must
                // appear in the row's array field.
                if let Some(Value::Array(filter_arr)) = Some(filter) {
                    if !filter_arr.is_empty() {
                        let filter_ids: Vec<String> = filter_arr.iter().map(value_to_filter_text).collect();
                        if let Some(Value::Array(row_arr)) = row_val {
                            let row_ids: Vec<String> = row_arr.iter().map(value_to_filter_text).collect();
                            return filter_ids.iter().all(|fid| row_ids.iter().any(|rid| rid.contains(fid.as_str())));
                        }
                        // Array filter against a scalar — fall through to substring match
                        let row_value = row_val.map(value_to_filter_text).unwrap_or_default();
                        return filter_ids.iter().all(|fid| row_value.contains(fid.as_str()));
                    }
                    return true;
                }

                // When the filter value is a comma-joined string (produced by
                // multi-select checkboxes or repeated URL params) AND the row
                // stores an array, check that every selected id appears in the
                // array.  A single value (no comma) falls through to the normal
                // substring match against the scalar or JSON-serialised row value.
                if let Some(Value::String(filter_str)) = Some(filter) {
                    let parts: Vec<&str> = filter_str.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
                    if parts.len() > 1 {
                        if let Some(Value::Array(arr)) = row_val {
                            let arr_ids: Vec<String> = arr.iter().map(value_to_filter_text).collect();
                            return parts.iter().all(|p| arr_ids.iter().any(|id| id.contains(&p.to_lowercase())));
                        }
                    }
                }

                let row_value = row_val.map(value_to_filter_text).unwrap_or_default();
                let filter_value = value_to_filter_text(filter);
                row_value.contains(&filter_value)
            })
        })
        .cloned()
        .collect()
}

pub fn display_value(
    blueprint: &Blueprint,
    outcome: &Outcome,
    model_name: &str,
    field_name: &str,
    value: &Value,
) -> String {
    if value.is_null() {
        return String::new();
    }

    let Some(model) = blueprint.models.get(model_name) else {
        return value_to_display_text(value);
    };

    let Some(field) = model.fields.get(field_name) else {
        return value_to_display_text(value);
    };

    display_value_for_type(blueprint, outcome, model_name, &field.field_type, value)
}

fn display_value_for_type(
    blueprint: &Blueprint,
    outcome: &Outcome,
    model_name: &str,
    field_type: &FieldType,
    value: &Value,
) -> String {
    match field_type {
        FieldType::Reference { reference } => {
            let lookup_key = value_to_lookup_key(value);
            if let Some(related) = lookup_key.and_then(|key| {
                outcome
                    .lookups
                    .get(&reference.entity)
                    .and_then(|lookup| lookup.get(&key))
            }) {
                if let Some(display) = related.get(&reference.display_field) {
                    return value_to_display_text(display);
                }
            }
            value_to_display_text(value)
        }

        FieldType::Primitive { primitive } => match primitive {
            Primitive::Boolean => {
                if let Some(b) = value.as_bool() {
                    return if b { "Yes" } else { "No" }.to_owned();
                }
                value_to_display_text(value)
            }
            _ => value_to_display_text(value),
        },

        FieldType::Array { item } => {
            if let Some(arr) = value.as_array() {
                arr.iter()
                    .map(|v| display_value_for_type(blueprint, outcome, model_name, item, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                value_to_display_text(value)
            }
        }

        FieldType::Shape { fields } => {
            if let Some(obj) = value.as_object() {
                fields
                    .iter()
                    .filter_map(|(key, shape_field)| {
                        let sub_val = obj.get(key)?;
                        if sub_val.is_null() {
                            return None;
                        }
                        let displayed = display_value_for_type(
                            blueprint,
                            outcome,
                            model_name,
                            &shape_field.field_type,
                            sub_val,
                        );
                        Some(format!("{}: {}", shape_field.label, displayed))
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            } else {
                value_to_display_text(value)
            }
        }
    }
}

/// Fallback display helper for when the blueprint is not available at the call
/// site (e.g. a table cell renderer that only holds an [`Outcome`]).
///
/// Resolves references by scanning all lookup tables in the outcome and
/// picking the first non-`"id"` string field of the matched row.  Falls back
/// to primitive formatting (`"Yes"/"No"` for booleans, `""` for nulls).
///
/// When the blueprint **is** available prefer [`display_value`], which uses
/// explicit field-type information for accurate rendering (arrays, shapes,
/// typed references).
pub fn display_value_from_outcome(outcome: &Outcome, _key: &str, val: &Value) -> String {
    if let Value::String(s) = val {
        for lookup in outcome.lookups.values() {
            if let Some(ref_row) = lookup.get(s.as_str()) {
                for (k, v) in ref_row {
                    if k != "id" {
                        if let Value::String(label) = v {
                            return label.clone();
                        }
                    }
                }
            }
        }
        return s.clone();
    }
    match val {
        Value::Bool(b) => if *b { "Yes" } else { "No" }.to_owned(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        // Value::String is handled above by the early-return if-let branch.
        Value::Array(_) | Value::Object(_) | Value::String(_) => serde_json::to_string(val).unwrap_or_default(),
    }
}

fn filter_is_empty(value: &Value) -> bool {
    value.is_null()
        || value.as_str().is_some_and(str::is_empty)
        || value.as_array().is_some_and(Vec::is_empty)
}

fn value_to_filter_text(value: &Value) -> String {
    value_to_display_text(value).to_lowercase()
}

fn value_to_display_text(value: &Value) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        _ => value.to_string(),
    }
}

/// Apply id-filter, filter_rows, sort, and pagination to `rows` according to `manifest`.
///
/// Sort is "natural": if both values parse as `f64`, they are compared numerically;
/// otherwise they are compared as byte-order strings.
pub fn apply_view(rows: Vec<Row>, manifest: &Manifest) -> Vec<Row> {
    // a) id filter
    let rows: Vec<Row> = if let Some(id) = &manifest.id {
        let id_str = value_to_filter_text(id);
        rows.into_iter()
            .filter(|row| {
                row.get("id")
                    .map(value_to_filter_text)
                    .as_deref()
                    == Some(id_str.as_str())
            })
            .collect()
    } else {
        rows
    };

    // b) filter_rows
    let rows = if manifest.filter.is_empty() {
        rows
    } else {
        filter_rows(&rows, &manifest.filter)
    };

    // c) sort
    let mut rows = rows;
    if let Some(sort) = &manifest.sort {
        rows.sort_by(|a, b| {
            let av = a.get(&sort.field).map(value_to_display_text).unwrap_or_default();
            let bv = b.get(&sort.field).map(value_to_display_text).unwrap_or_default();
            let ord = match (av.parse::<f64>(), bv.parse::<f64>()) {
                (Ok(an), Ok(bn)) => an.partial_cmp(&bn).unwrap_or(Ordering::Equal),
                _ => av.cmp(&bv),
            };
            if sort.ascending { ord } else { ord.reverse() }
        });
    }

    // d) paginate
    if let Some(per_page) = manifest.per_page {
        let page = manifest.page.unwrap_or(1).max(1);
        let start = (page - 1) * per_page;
        rows.into_iter().skip(start).take(per_page).collect()
    } else {
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{Manifest, Sort};
    use indexmap::IndexMap;
    use serde_json::json;

    fn make_manifest() -> Manifest {
        Manifest {
            base: "users".into(),
            id: None,
            fields: vec![],
            filter: IndexMap::new(),
            lookups: vec![],
            inlines: vec![],
            page: None,
            per_page: None,
            sort: None,
            method: None,
            filter_fields: vec![],
        }
    }

    fn row(id: i64, name: &str, age: i64) -> Row {
        let mut m = Row::new();
        m.insert("id".into(), json!(id));
        m.insert("name".into(), json!(name));
        m.insert("age".into(), json!(age));
        m
    }

    fn rows() -> Vec<Row> {
        vec![
            row(1, "Alice", 30),
            row(2, "Bob", 25),
            row(3, "Charlie", 35),
        ]
    }

    // --- id filter ---

    #[test]
    fn id_filter_matches_one() {
        let manifest = Manifest { id: Some(json!(2)), ..make_manifest() };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], json!("Bob"));
    }

    #[test]
    fn id_filter_no_match() {
        let manifest = Manifest { id: Some(json!(99)), ..make_manifest() };
        let result = apply_view(rows(), &manifest);
        assert!(result.is_empty());
    }

    #[test]
    fn id_filter_string_id_matches_number() {
        // server ids are numbers; id may arrive as string "2"
        let manifest = Manifest { id: Some(json!("2")), ..make_manifest() };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], json!("Bob"));
    }

    // --- filter_rows ---

    #[test]
    fn filter_rows_substring() {
        let mut filter = IndexMap::new();
        filter.insert("name".into(), json!("ali"));
        let manifest = Manifest { filter, ..make_manifest() };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], json!("Alice"));
    }

    #[test]
    fn filter_rows_no_match() {
        let mut filter = IndexMap::new();
        filter.insert("name".into(), json!("xyz"));
        let manifest = Manifest { filter, ..make_manifest() };
        assert!(apply_view(rows(), &manifest).is_empty());
    }

    #[test]
    fn filter_rows_empty_filter_returns_all() {
        let manifest = make_manifest();
        assert_eq!(apply_view(rows(), &manifest).len(), 3);
    }

    #[test]
    fn filter_rows_reference_matches_by_value_not_label() {
        // Row stores the reference id (e.g. "admin"), not the display label ("Admin").
        // A filter of "admin" must match; a filter of "Admin" must also match (case-insensitive
        // substring), but crucially the filter value coming from a <select value="admin"> is
        // the id string — this test confirms the id path works.
        let mut row_a = IndexMap::new();
        row_a.insert("id".into(), json!(1));
        row_a.insert("role".into(), json!("admin"));
        let mut row_b = IndexMap::new();
        row_b.insert("id".into(), json!(2));
        row_b.insert("role".into(), json!("editor"));

        let rows = vec![row_a, row_b];

        let mut filter = IndexMap::new();
        filter.insert("role".into(), json!("admin")); // value (id), not label ("Admin")
        let result = filter_rows(&rows, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], json!("admin"));
    }

    #[test]
    fn filter_rows_array_reference_single_value() {
        // Single-value array filter: filter value "rust" matches rows whose array contains "rust".
        let mut row_a = IndexMap::new();
        row_a.insert("id".into(), json!(1));
        row_a.insert("tags".into(), json!(["rust", "web"]));
        let mut row_b = IndexMap::new();
        row_b.insert("id".into(), json!(2));
        row_b.insert("tags".into(), json!(["web"]));
        let mut row_c = IndexMap::new();
        row_c.insert("id".into(), json!(3));
        row_c.insert("tags".into(), json!(Vec::<String>::new()));

        let rows = vec![row_a, row_b, row_c];

        // String filter "rust" — comes from no-JS form ?tags=rust
        let mut filter_str = IndexMap::new();
        filter_str.insert("tags".into(), json!("rust")); // id, not label "Rust"
        let result = filter_rows(&rows, &filter_str);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["id"], json!(1));

        // Array filter ["rust"] — comes from JS checkbox selection
        let mut filter_arr = IndexMap::new();
        filter_arr.insert("tags".into(), json!(["rust"]));
        let result = filter_rows(&rows, &filter_arr);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["id"], json!(1));
    }

    #[test]
    fn filter_rows_array_reference_multi_value() {
        // Multi-value array filter: all selected ids must be in the row's array.
        let mut row_a = IndexMap::new();
        row_a.insert("id".into(), json!(1));
        row_a.insert("tags".into(), json!(["rust", "web"]));
        let mut row_b = IndexMap::new();
        row_b.insert("id".into(), json!(2));
        row_b.insert("tags".into(), json!(["rust"]));
        let mut row_c = IndexMap::new();
        row_c.insert("id".into(), json!(3));
        row_c.insert("tags".into(), json!(["web"]));

        let rows = vec![row_a, row_b, row_c];

        // Comma-joined string "rust,web" — from no-JS repeated params ?tags=rust&tags=web
        let mut filter_str = IndexMap::new();
        filter_str.insert("tags".into(), json!("rust,web"));
        let result = filter_rows(&rows, &filter_str);
        assert_eq!(result.len(), 1, "only row with both rust and web should match");
        assert_eq!(result[0]["id"], json!(1));

        // Array ["rust","web"] — from JS multi-checkbox selection
        let mut filter_arr = IndexMap::new();
        filter_arr.insert("tags".into(), json!(["rust", "web"]));
        let result = filter_rows(&rows, &filter_arr);
        assert_eq!(result.len(), 1, "only row with both rust and web should match");
        assert_eq!(result[0]["id"], json!(1));
    }

    #[test]
    fn filter_rows_ignores_empty_array_filter() {
        // An empty array filter should return all rows.
        let mut row_a = IndexMap::new();
        row_a.insert("id".into(), json!(1));
        row_a.insert("tags".into(), json!(["rust"]));
        let mut row_b = IndexMap::new();
        row_b.insert("id".into(), json!(2));
        row_b.insert("tags".into(), json!(Vec::<String>::new()));

        let rows = vec![row_a, row_b];

        let mut filter = IndexMap::new();
        filter.insert("tags".into(), json!(Vec::<String>::new()));
        let result = filter_rows(&rows, &filter);
        assert_eq!(result.len(), 2, "empty array filter should match all rows");
    }

    // --- sort ---

    #[test]
    fn sort_numeric_ascending() {
        let manifest = Manifest {
            sort: Some(Sort { field: "age".into(), ascending: true }),
            ..make_manifest()
        };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result[0]["name"], json!("Bob"));   // 25
        assert_eq!(result[1]["name"], json!("Alice")); // 30
        assert_eq!(result[2]["name"], json!("Charlie")); // 35
    }

    #[test]
    fn sort_numeric_descending() {
        let manifest = Manifest {
            sort: Some(Sort { field: "age".into(), ascending: false }),
            ..make_manifest()
        };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result[0]["name"], json!("Charlie")); // 35
        assert_eq!(result[1]["name"], json!("Alice"));   // 30
        assert_eq!(result[2]["name"], json!("Bob"));     // 25
    }

    #[test]
    fn sort_string_ascending() {
        let manifest = Manifest {
            sort: Some(Sort { field: "name".into(), ascending: true }),
            ..make_manifest()
        };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result[0]["name"], json!("Alice"));
        assert_eq!(result[1]["name"], json!("Bob"));
        assert_eq!(result[2]["name"], json!("Charlie"));
    }

    #[test]
    fn sort_string_descending() {
        let manifest = Manifest {
            sort: Some(Sort { field: "name".into(), ascending: false }),
            ..make_manifest()
        };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result[0]["name"], json!("Charlie"));
        assert_eq!(result[1]["name"], json!("Bob"));
        assert_eq!(result[2]["name"], json!("Alice"));
    }

    #[test]
    fn sort_natural_numeric_order() {
        // "10" > "9" numerically but "10" < "9" lexicographically
        let mut r: Vec<Row> = (1..=10)
            .map(|i| {
                let mut m = Row::new();
                m.insert("id".into(), json!(i));
                m.insert("age".into(), json!(i));
                m
            })
            .collect();
        r.reverse(); // start in reverse order
        let manifest = Manifest {
            sort: Some(Sort { field: "age".into(), ascending: true }),
            ..make_manifest()
        };
        let result = apply_view(r, &manifest);
        for (i, row) in result.iter().enumerate() {
            assert_eq!(row["age"], json!(i + 1));
        }
    }

    // --- paginate ---

    #[test]
    fn paginate_page1() {
        let manifest = Manifest { page: Some(1), per_page: Some(2), ..make_manifest() };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["name"], json!("Alice"));
        assert_eq!(result[1]["name"], json!("Bob"));
    }

    #[test]
    fn paginate_page2() {
        let manifest = Manifest { page: Some(2), per_page: Some(2), ..make_manifest() };
        let result = apply_view(rows(), &manifest);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], json!("Charlie"));
    }

    #[test]
    fn paginate_beyond_last_page_returns_empty() {
        let manifest = Manifest { page: Some(5), per_page: Some(2), ..make_manifest() };
        assert!(apply_view(rows(), &manifest).is_empty());
    }

    // --- combined ---

    #[test]
    fn combined_filter_sort_paginate() {
        // 6 rows; filter keeps those with "o" in name; sort by age asc; page 1 size 2
        let r = vec![
            row(1, "Bob", 25),
            row(2, "Tom", 40),
            row(3, "Joe", 22),
            row(4, "Alice", 30),
            row(5, "Zoe", 28),
            row(6, "Dot", 35),
        ];
        let mut filter = IndexMap::new();
        filter.insert("name".into(), json!("o"));
        let manifest = Manifest {
            filter,
            sort: Some(Sort { field: "age".into(), ascending: true }),
            page: Some(1),
            per_page: Some(2),
            ..make_manifest()
        };
        // matching: Bob(25), Tom(40), Joe(22), Zoe(28), Dot(35) — sorted by age: Joe(22), Bob(25), Zoe(28), Dot(35), Tom(40)
        // page 1 of 2: Joe, Bob
        let result = apply_view(r, &manifest);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["name"], json!("Joe"));
        assert_eq!(result[1]["name"], json!("Bob"));
    }

    // ── display_value ─────────────────────────────────────────────────────────

    fn make_blueprint_with_role() -> Blueprint {
        use crate::blueprint::{Blueprint, Model};
        use crate::field::Field;
        Blueprint::new()
            .model(
                "user",
                Model::new()
                    .field("name", Field::string("Name"))
                    .field("active", Field::boolean("Active"))
                    .field("role", Field::reference("Role", "role")),
            )
            .model("role", Model::new().field("name", Field::string("Name")))
    }

    fn empty_outcome() -> Outcome {
        Outcome {
            rows: crate::raw::ModelRows::Many(vec![]),
            total: 0,
            lookups: IndexMap::new(),
        }
    }

    #[test]
    fn display_value_string_primitive() {
        let bp = make_blueprint_with_role();
        let outcome = empty_outcome();
        let val = json!("Alice");
        assert_eq!(display_value(&bp, &outcome, "user", "name", &val), "Alice");
    }

    #[test]
    fn display_value_boolean_true() {
        let bp = make_blueprint_with_role();
        let outcome = empty_outcome();
        let val = json!(true);
        assert_eq!(display_value(&bp, &outcome, "user", "active", &val), "Yes");
    }

    #[test]
    fn display_value_boolean_false() {
        let bp = make_blueprint_with_role();
        let outcome = empty_outcome();
        let val = json!(false);
        assert_eq!(display_value(&bp, &outcome, "user", "active", &val), "No");
    }

    #[test]
    fn display_value_reference_resolves_from_lookup() {
        let bp = make_blueprint_with_role();
        let mut lookup: IndexMap<String, Row> = IndexMap::new();
        let mut role_row = Row::new();
        role_row.insert("id".into(), json!("admin"));
        role_row.insert("name".into(), json!("Admin"));
        lookup.insert("admin".into(), role_row);
        let outcome = Outcome {
            rows: crate::raw::ModelRows::Many(vec![]),
            total: 0,
            lookups: indexmap::indexmap! { "role".to_owned() => lookup },
        };
        let val = json!("admin");
        assert_eq!(display_value(&bp, &outcome, "user", "role", &val), "Admin");
    }

    #[test]
    fn display_value_null_returns_empty() {
        let bp = make_blueprint_with_role();
        let outcome = empty_outcome();
        assert_eq!(display_value(&bp, &outcome, "user", "name", &json!(null)), "");
    }

    // ── display_value_from_outcome ────────────────────────────────────────────

    #[test]
    fn display_value_from_outcome_resolves_reference() {
        let mut lookup: IndexMap<String, Row> = IndexMap::new();
        let mut role_row = Row::new();
        role_row.insert("id".into(), json!("admin"));
        role_row.insert("name".into(), json!("Admin"));
        lookup.insert("admin".into(), role_row);
        let outcome = Outcome {
            rows: crate::raw::ModelRows::Many(vec![]),
            total: 0,
            lookups: indexmap::indexmap! { "role".to_owned() => lookup },
        };
        let val = json!("admin");
        assert_eq!(display_value_from_outcome(&outcome, "role", &val), "Admin");
    }

    #[test]
    fn display_value_from_outcome_string_no_lookup_returns_self() {
        let outcome = empty_outcome();
        let val = json!("hello");
        assert_eq!(display_value_from_outcome(&outcome, "x", &val), "hello");
    }

    #[test]
    fn display_value_from_outcome_bool() {
        let outcome = empty_outcome();
        assert_eq!(display_value_from_outcome(&outcome, "x", &json!(true)), "Yes");
        assert_eq!(display_value_from_outcome(&outcome, "x", &json!(false)), "No");
    }

    #[test]
    fn display_value_from_outcome_null_returns_empty() {
        let outcome = empty_outcome();
        assert_eq!(display_value_from_outcome(&outcome, "x", &json!(null)), "");
    }

    // ── derive_columns ────────────────────────────────────────────────────────

    #[test]
    fn derive_columns_returns_columns_in_order() {
        use crate::blueprint::{Blueprint, Model};
        use crate::field::Field;
        let bp = Blueprint::new().model(
            "user",
            Model::new()
                .field("name", Field::string("Name"))
                .field("email", Field::string("Email")),
        );
        let manifest = Manifest {
            base: "user".into(),
            fields: vec!["name".into(), "email".into()],
            ..make_manifest()
        };
        let cols = derive_columns(&bp, &manifest).unwrap();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].key, "name");
        assert_eq!(cols[1].key, "email");
        assert_eq!(cols[0].label, "Name");
    }

    #[test]
    fn derive_columns_unknown_model_errors() {
        use crate::blueprint::Blueprint;
        let bp = Blueprint::new();
        let manifest = Manifest { base: "ghost".into(), fields: vec!["x".into()], ..make_manifest() };
        assert!(derive_columns(&bp, &manifest).is_err());
    }

    // ── derive_filter_layout ──────────────────────────────────────────────────

    #[test]
    fn derive_filter_layout_groups_by_row() {
        use crate::blueprint::{Blueprint, Model};
        use crate::field::Field;
        let bp = Blueprint::new().model(
            "user",
            Model::new()
                .field("name", Field::string("Name"))
                .field("email", Field::string("Email"))
                .field("role", Field::string("Role")),
        );
        let manifest = Manifest {
            base: "user".into(),
            filter_fields: vec![
                vec!["name".into(), "email".into()],
                vec!["role".into()],
            ],
            ..make_manifest()
        };
        let layout = derive_filter_layout(&bp, &manifest).unwrap();
        assert_eq!(layout.len(), 2);
        assert_eq!(layout[0].len(), 2);
        assert_eq!(layout[1].len(), 1);
        assert_eq!(layout[0][0].key, "name");
        assert_eq!(layout[1][0].key, "role");
    }

    #[test]
    fn derive_filter_layout_skips_unknown_fields() {
        use crate::blueprint::{Blueprint, Model};
        use crate::field::Field;
        let bp = Blueprint::new().model(
            "user",
            Model::new().field("name", Field::string("Name")),
        );
        let manifest = Manifest {
            base: "user".into(),
            filter_fields: vec![vec!["name".into(), "ghost".into()]],
            ..make_manifest()
        };
        let layout = derive_filter_layout(&bp, &manifest).unwrap();
        assert_eq!(layout[0].len(), 1); // "ghost" was skipped
        assert_eq!(layout[0][0].key, "name");
    }

    #[test]
    fn derive_columns_from_blueprint_and_manifest_via_query() {
        use crate::blueprint::{Blueprint, Model};
        use crate::field::Field;
        use crate::query::Query;
        let bp = Blueprint::new().model(
            "user",
            Model::new()
                .field("name", Field::string("Name"))
                .field("role", Field::reference("Role", "role")),
        ).model("role", Model::new().field("name", Field::string("Role name")));
        let plan = bp.manifest(Query::new("user").select(["name", "role"])).unwrap();
        let columns = derive_columns(&bp, &plan).unwrap();
        assert_eq!(columns[0].key, "name");
        assert_eq!(columns[0].label, "Name");
        assert_eq!(columns[1].key, "role");
        assert_eq!(columns[1].label, "Role");
    }

    #[test]
    fn derive_filter_layout_returns_2d_column_grid_via_query() {
        use crate::blueprint::{Blueprint, Model};
        use crate::field::Field;
        use crate::query::Query;
        let bp = Blueprint::new().model(
            "user",
            Model::new()
                .field("name", Field::string("Name"))
                .field("role", Field::reference("Role", "role"))
                .field("active", Field::boolean("Active")),
        ).model("role", Model::new().field("name", Field::string("Role name")));
        let manifest = bp.manifest(
            Query::new("user")
                .select(["name", "role", "active"])
                .filter_layout(vec![vec!["name", "role"], vec!["active"]]),
        ).unwrap();
        let layout = derive_filter_layout(&bp, &manifest).unwrap();
        assert_eq!(layout.len(), 2);
        assert_eq!(layout[0].len(), 2);
        assert_eq!(layout[0][0].key, "name");
        assert_eq!(layout[0][1].key, "role");
        assert_eq!(layout[1].len(), 1);
        assert_eq!(layout[1][0].key, "active");
    }
}
