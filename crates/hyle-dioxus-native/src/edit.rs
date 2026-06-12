use indexmap::IndexMap;
use std::collections::BTreeSet;

use hyle::{Blueprint, FieldType, ModelResult, Row, Source, Value};

pub fn item_to_source(
    blueprint: &Blueprint,
    model: &str,
    id: &str,
    fields: &IndexMap<String, String>,
) -> Source {
    let mut row = Row::new();
    row.insert("id".to_owned(), Value::String(id.to_owned()));
    for (k, v) in fields {
        row.insert(k.clone(), Value::String(v.clone()));
    }
    let mut source: Source = Source::new();
    source.insert(model.to_owned(), ModelResult::one(row));

    if let Some(m) = blueprint.models.get(model) {
        for (field_name, field) in &m.fields {
            if let FieldType::Reference { reference } = &field.field_type {
                if let Some(val) = fields.get(field_name) {
                    let mut seen = BTreeSet::new();
                    for v in val.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        seen.insert(v.to_owned());
                    }
                    if !seen.is_empty() {
                        let lookup_rows: Vec<Row> = seen
                            .into_iter()
                            .map(|v| {
                                let mut r = Row::new();
                                r.insert("id".to_owned(), Value::String(v.clone()));
                                r.insert(reference.display_field.clone(), Value::String(v));
                                r
                            })
                            .collect();
                        source.insert(reference.entity.clone(), ModelResult::many(lookup_rows));
                    }
                }
            }
        }
    }

    source
}
