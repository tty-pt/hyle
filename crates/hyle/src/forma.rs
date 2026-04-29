use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::field::{Field, ShapeField};
use crate::query::Query;

/// How a forma field type is described in a runtime forma definition.
/// Strings correspond to primitive names or entity names (for references).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormaFieldType {
    /// "string" | "number" | "boolean" | "file" | "<entity-name>"
    Named(String),
    Array {
        array: Box<FormaFieldType>,
    },
    Shape {
        shape: Vec<FormaField>,
    },
}

impl Default for FormaFieldType {
    fn default() -> Self {
        FormaFieldType::Named("string".to_owned())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormaField {
    pub name: String,
    pub label: String,
    #[serde(default)]
    pub field_type: FormaFieldType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_type: Option<FormaFieldType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form_type: Option<FormaFieldType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column_type: Option<FormaFieldType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_value: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Forma {
    #[serde(default)]
    pub fields: Vec<FormaField>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub form: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<Vec<String>>>,
}

/// Which rendering context to use when deriving a query from a forma.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FormaContext {
    Detail,
    Form,
    Column,
}

impl Default for FormaContext {
    fn default() -> Self {
        FormaContext::Column
    }
}

/// Derive a `Query` from a `Forma` definition.
///
/// - `context` selects which field subset list to use (`detail`, `form`, or `column`).
/// - `id` is placed in `where.id` when provided (single-record fetch).
pub fn forma_to_query(
    forma: &Forma,
    table_name: &str,
    context: &FormaContext,
    id: Option<&JsonValue>,
) -> Query {
    // Pick which field names are active for this context
    let context_names: Option<&Vec<String>> = match context {
        FormaContext::Detail => forma.detail.as_ref(),
        FormaContext::Form => forma.form.as_ref(),
        FormaContext::Column => forma.column.as_ref(),
    };

    let active_fields: Vec<&FormaField> = if let Some(names) = context_names {
        names
            .iter()
            .filter_map(|n| forma.fields.iter().find(|f| &f.name == n))
            .collect()
    } else {
        forma.fields.iter().collect()
    };

    let select: Vec<String> = active_fields.iter().map(|f| f.name.clone()).collect();

    let mut where_ = IndexMap::new();
    if let Some(id_val) = id {
        where_.insert("id".to_owned(), id_val.clone());
    }

    Query {
        model: table_name.to_owned(),
        select,
        where_,
        filters: forma.filters.clone().unwrap_or_default(),
        page: None,
        per_page: None,
        sort: None,
        method: id.map(|_| "one".to_owned()),
    }
}

/// Map a `FormaField` to a hyle `Field`.
#[allow(dead_code)]
pub(crate) fn forma_field_to_field(sf: &FormaField, context: &FormaContext) -> Field {
    // Pick context-specific type override if present
    let ftype = match context {
        FormaContext::Detail => sf.detail_type.as_ref().unwrap_or(&sf.field_type),
        FormaContext::Form => sf.form_type.as_ref().unwrap_or(&sf.field_type),
        FormaContext::Column => sf.column_type.as_ref().unwrap_or(&sf.field_type),
    };

    let mut field = forma_field_type_to_field(&sf.label, ftype);

    if let Some(fixed) = &sf.fixed_value {
        field.options.fixed_value = Some(fixed.clone());
    }
    if let Some(rule) = &sf.rule {
        field.options.rule = Some(rule.clone());
    }

    field
}

#[allow(dead_code)]
fn forma_field_type_to_field(label: &str, ftype: &FormaFieldType) -> Field {
    match ftype {
        FormaFieldType::Named(name) => match name.as_str() {
            "string" => Field::string(label),
            "number" => Field::number(label),
            "boolean" => Field::boolean(label),
            "file" => Field::file(label),
            entity => Field::reference(label, entity),
        },
        FormaFieldType::Array { array } => {
            let item_field = forma_field_type_to_field(label, array);
            Field::array(label, item_field.field_type)
        }
        FormaFieldType::Shape { shape } => {
            let mut shape_fields = IndexMap::new();
            for sf in shape {
                let f = forma_field_type_to_field(&sf.label, &sf.field_type);
                shape_fields.insert(
                    sf.name.clone(),
                    ShapeField::new(&sf.label, f.field_type),
                );
            }
            Field::shape(label, shape_fields)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn forma_to_query_column_context() {
        let forma = Forma {
            fields: vec![
                FormaField {
                    name: "name".to_owned(),
                    label: "Name".to_owned(),
                    field_type: FormaFieldType::Named("string".to_owned()),
                    ..Default::default()
                },
                FormaField {
                    name: "role".to_owned(),
                    label: "Role".to_owned(),
                    field_type: FormaFieldType::Named("role".to_owned()),
                    ..Default::default()
                },
            ],
            column: Some(vec!["name".to_owned()]),
            ..Default::default()
        };

        let query = forma_to_query(&forma, "user", &FormaContext::Column, None);
        assert_eq!(query.model, "user");
        assert_eq!(query.select, vec!["name"]);
        assert!(query.where_.is_empty());
        assert!(query.method.is_none());
    }

    #[test]
    fn forma_to_query_with_id() {
        let forma = Forma {
            fields: vec![FormaField {
                name: "name".to_owned(),
                label: "Name".to_owned(),
                field_type: FormaFieldType::Named("string".to_owned()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let query = forma_to_query(&forma, "user", &FormaContext::Form, Some(&json!(42)));
        assert_eq!(query.where_.get("id"), Some(&json!(42)));
        assert_eq!(query.method, Some("one".to_owned()));
    }

    #[test]
    fn forma_field_to_field_maps_primitive_and_reference_kinds() {
        use crate::field::FieldType;
        let ctx = FormaContext::Column;
        let cases: &[(&str, bool)] = &[
            ("string",  false),
            ("number",  false),
            ("boolean", false),
            ("file",    false),
            ("role",    true),
        ];
        for (kind, is_ref) in cases {
            let sf = FormaField {
                name: "f".into(),
                label: "F".into(),
                field_type: FormaFieldType::Named(kind.to_string()),
                ..Default::default()
            };
            let field = forma_field_to_field(&sf, &ctx);
            match &field.field_type {
                FieldType::Reference { reference } => {
                    assert!(is_ref, "expected primitive for kind={kind}");
                    assert_eq!(reference.entity, *kind);
                }
                FieldType::Primitive { .. } => {
                    assert!(!is_ref, "expected reference for kind={kind}");
                }
                other => panic!("unexpected field type for kind={kind}: {other:?}"),
            }
        }
    }
}

impl Default for FormaField {
    fn default() -> Self {
        Self {
            name: String::new(),
            label: String::new(),
            field_type: FormaFieldType::default(),
            detail_type: None,
            form_type: None,
            column_type: None,
            fixed_value: None,
            rule: None,
        }
    }
}
