use crate::{Blueprint, Forma, FormaContext, Manifest, Outcome, Query, Row, Source, Value};
use crate::raw::rows_from_outcome;
use crate::view::{derive_columns, derive_filter_layout};
use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value as JsonValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn manifest(blueprint_json: &str, query_json: &str) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let query: Query = parse_json(query_json, "query")?;
    let manifest = blueprint.manifest(query).map_err(to_string)?;
    serde_json::to_string(&manifest).map_err(to_string)
}

#[wasm_bindgen]
pub fn resolve(
    blueprint_json: &str,
    manifest_json: &str,
    source_json: &str,
) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let manifest: Manifest = parse_json(manifest_json, "manifest")?;
    let source: Source = parse_json(source_json, "source")?;
    let result = blueprint.resolve(&manifest, &source).map_err(to_string)?;
    serde_json::to_string(&result).map_err(to_string)
}

/// Manifest + resolve + normalise rows in one call.
/// Returns `{ manifest, result, rows }` JSON.
#[wasm_bindgen]
pub fn resolve_query(
    blueprint_json: &str,
    query_json: &str,
    source_json: &str,
) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let query: Query = parse_json(query_json, "query")?;
    let source: Source = parse_json(source_json, "source")?;
    let (manifest, outcome, rows) = blueprint.resolve_query(query, &source).map_err(to_string)?;
    #[derive(Serialize)]
    struct Out<'a> {
        manifest: &'a Manifest,
        result: &'a Outcome,
        rows: &'a Vec<Row>,
    }
    serde_json::to_string(&Out { manifest: &manifest, result: &outcome, rows: &rows })
        .map_err(to_string)
}

/// Normalise `result.rows` (one-or-many) into a flat JSON array of rows.
#[wasm_bindgen]
pub fn rows_array(result_json: &str) -> std::result::Result<String, String> {
    let outcome: Outcome = parse_json(result_json, "result")?;
    let rows = rows_from_outcome(&outcome);
    serde_json::to_string(&rows).map_err(to_string)
}

/// Returns `true` when the outcome represents a single record.
#[wasm_bindgen]
pub fn is_single(manifest_json: &str, outcome_json: &str) -> std::result::Result<bool, String> {
    let manifest: Manifest = parse_json(manifest_json, "manifest")?;
    let outcome: Outcome = parse_json(outcome_json, "outcome")?;
    Ok(crate::is_single(&manifest, &outcome))
}

#[wasm_bindgen]
pub fn columns(blueprint_json: &str, manifest_json: &str) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let manifest: Manifest = parse_json(manifest_json, "manifest")?;
    let columns = derive_columns(&blueprint, &manifest).map_err(to_string)?;
    serde_json::to_string(&columns).map_err(to_string)
}

#[wasm_bindgen]
pub fn filter_rows(rows_json: &str, filters_json: &str) -> std::result::Result<String, String> {
    let rows: Vec<Row> = parse_json(rows_json, "rows")?;
    let filters: IndexMap<String, Value> = parse_json(filters_json, "filters")?;
    let filtered = crate::filter_rows(&rows, &filters);
    serde_json::to_string(&filtered).map_err(to_string)
}

#[wasm_bindgen]
pub fn display_value(
    blueprint_json: &str,
    outcome_json: &str,
    model_name: &str,
    field_name: &str,
    value_json: &str,
) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let outcome: Outcome = parse_json(outcome_json, "outcome")?;
    let value: Value = parse_json(value_json, "value")?;
    Ok(crate::display_value(
        &blueprint, &outcome, model_name, field_name, &value,
    ))
}

/// Build a `Field` by kind. `kind`: `"string"|"number"|"boolean"|"file"|"ref"`.
/// `entity_json`: JSON string for the entity name (required for `"ref"`, else `"null"`).
#[wasm_bindgen]
pub fn make_field(
    kind: &str,
    label: &str,
    entity_json: &str,
) -> std::result::Result<String, String> {
    let entity: Option<String> = parse_json(entity_json, "entity")?;
    let field = crate::make_field(kind, label, entity.as_deref());
    serde_json::to_string(&field).map_err(to_string)
}

/// Derive the 2-D filter layout from a manifest's `filter_fields`.
#[wasm_bindgen]
pub fn filter_layout(
    blueprint_json: &str,
    manifest_json: &str,
) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let manifest: Manifest = parse_json(manifest_json, "manifest")?;
    let layout = derive_filter_layout(&blueprint, &manifest).map_err(to_string)?;
    serde_json::to_string(&layout).map_err(to_string)
}

/// Derive a Query from a Forma definition.
#[wasm_bindgen]
pub fn forma_to_query(
    forma_json: &str,
    table_name: &str,
    context_json: &str,
    id_json: &str,
) -> std::result::Result<String, String> {
    let forma: Forma = parse_json(forma_json, "forma")?;
    let context: FormaContext = parse_json(context_json, "context")?;
    let id: Option<JsonValue> = {
        let v: JsonValue = parse_json(id_json, "id")?;
        if v.is_null() { None } else { Some(v) }
    };
    let query = crate::forma_to_query(&forma, table_name, &context, id.as_ref());
    serde_json::to_string(&query).map_err(to_string)
}

/// Synchronously validate a row against blueprint field metadata rules.
/// Returns `null` (JSON) if valid, or a JSON array of `PurifyError` objects.
#[wasm_bindgen]
pub fn purify_row(
    blueprint_json: &str,
    model_name: &str,
    row_json: &str,
) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let row: Row = parse_json(row_json, "row")?;
    match crate::purify_row_sync(&blueprint, model_name, &row) {
        Ok(()) => Ok("null".to_owned()),
        Err(errors) => serde_json::to_string(&errors).map_err(to_string),
    }
}

/// resolve + apply_view + is_single + columns in one call.
/// Returns `{ result, rows, isSingle, columns }` JSON.
#[wasm_bindgen]
pub fn resolve_and_view(
    blueprint_json: &str,
    manifest_json: &str,
    source_json: &str,
) -> std::result::Result<String, String> {
    let blueprint: Blueprint = parse_json(blueprint_json, "blueprint")?;
    let manifest: Manifest = parse_json(manifest_json, "manifest")?;
    let source: Source = parse_json(source_json, "source")?;
    let view = blueprint.resolve_and_view(&manifest, &source).map_err(to_string)?;
    serde_json::to_string(&view).map_err(to_string)
}

/// Apply id-filter, filter_rows, sort, and pagination in one call.
#[wasm_bindgen]
pub fn apply_view(rows_json: &str, manifest_json: &str) -> std::result::Result<String, String> {
    let rows: Vec<Row> = parse_json(rows_json, "rows")?;
    let manifest: Manifest = parse_json(manifest_json, "manifest")?;
    let result = crate::apply_view(rows, &manifest);
    serde_json::to_string(&result).map_err(to_string)
}

fn parse_json<T: serde::de::DeserializeOwned>(
    input: &str,
    name: &str,
) -> std::result::Result<T, String> {
    serde_json::from_str(input).map_err(|err| format!("invalid {name} JSON: {err}"))
}

fn to_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}
