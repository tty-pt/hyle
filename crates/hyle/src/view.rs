use serde::{Deserialize, Serialize};

use crate::blueprint::Blueprint;
use crate::error::{Error, HyleResult};
use crate::field::{Field, FieldType, Primitive};
use crate::query::Manifest;
use crate::raw::{Outcome, Value, value_to_lookup_key};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
	pub key: String,
	pub field: Field,
	pub label: String,
}

pub fn derive_columns(blueprint: &Blueprint, manifest: &Manifest) -> HyleResult<Vec<Column>> {
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

#[cfg(feature = "wasm")]
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

pub fn display_value(
	blueprint: &Blueprint,
	outcome: &Outcome,
	model_name: &str,
	field_name: &str,
	value: &Value,
) -> String {
	if matches!(value, Value::Null) {
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
				if let Value::Bool(b) = value {
					return if *b { "Yes" } else { "No" }.to_owned();
				}
				value_to_display_text(value)
			}
			_ => value_to_display_text(value),
		},

		FieldType::Array { item } => {
			if let Value::Array(arr) = value {
				arr.iter()
					.map(|v| display_value_for_type(blueprint, outcome, model_name, item, v))
					.collect::<Vec<_>>()
					.join(", ")
			} else {
				value_to_display_text(value)
			}
		}

		FieldType::Shape { fields } => {
			if let Value::Map(map) = value {
				fields
					.iter()
					.filter_map(|(key, shape_field)| {
						let sub_val = map.get(key)?;
						if matches!(sub_val, Value::Null) {
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
		Value::Int(n) => n.to_string(),
		Value::Float(n) => n.to_string(),
		Value::Null => String::new(),
		Value::String(s) => s.clone(),
		Value::Array(a) => {
			#[cfg(feature = "json")]
			{ serde_json::to_string(a).unwrap_or_default() }
			#[cfg(not(feature = "json"))]
			{ a.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ") }
		}
		Value::Map(m) => {
			#[cfg(feature = "json")]
			{ serde_json::to_string(m).unwrap_or_default() }
			#[cfg(not(feature = "json"))]
			{ format!("{{{}}}", m.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(", ")) }
		}
		Value::Bytes(b) => {
			b.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ")
		}
	}
}

pub fn value_to_display_text(value: &Value) -> String {
	match value {
		Value::Null => String::new(),
		Value::Bool(b) => b.to_string(),
		Value::Int(n) => n.to_string(),
		Value::Float(n) => n.to_string(),
		Value::String(s) => s.clone(),
		Value::Bytes(b) => b.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(""),
		Value::Array(arr) => arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", "),
		Value::Map(m) => format!("{{{}}}", m.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(", ")),
	}
}
