use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortType {
    String,
    Numeric,
    Date,
    None,
}

impl Default for SortType {
    fn default() -> Self {
        Self::String
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Primitive {
    String,
    Number,
    Boolean,
    File,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub entity: String,
    #[serde(default = "default_display_field")]
    pub display_field: String,
}

fn default_display_field() -> String {
    "name".to_owned()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FieldType {
    Primitive {
        primitive: Primitive,
    },
    Reference {
        reference: Reference,
    },
    Array {
        item: Box<FieldType>,
    },
    Shape {
        fields: IndexMap<String, ShapeField>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeField {
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub options: FieldOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputHint {
    pub kind: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub props: IndexMap<String, JsonValue>,
}

impl InputHint {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            props: IndexMap::new(),
        }
    }

    pub fn with_prop(mut self, key: impl Into<String>, value: impl Into<JsonValue>) -> Self {
        self.props.insert(key.into(), value.into());
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldOptions {
    #[serde(default)]
    pub sort: SortType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<InputHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_value: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, JsonValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub options: FieldOptions,
}

impl Field {
    pub fn new(label: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            label: label.into(),
            field_type,
            options: FieldOptions::default(),
        }
    }

    pub fn string(label: impl Into<String>) -> Self {
        Self::new(
            label,
            FieldType::Primitive {
                primitive: Primitive::String,
            },
        )
        .with_input(InputHint::new("text"))
    }

    pub fn number(label: impl Into<String>) -> Self {
        Self::new(
            label,
            FieldType::Primitive {
                primitive: Primitive::Number,
            },
        )
        .with_sort(SortType::Numeric)
        .with_input(InputHint::new("number"))
    }

    pub fn boolean(label: impl Into<String>) -> Self {
        Self::new(
            label,
            FieldType::Primitive {
                primitive: Primitive::Boolean,
            },
        )
        .with_sort(SortType::None)
        .with_input(InputHint::new("checkbox"))
    }

    pub fn file(label: impl Into<String>) -> Self {
        Self::new(
            label,
            FieldType::Primitive {
                primitive: Primitive::File,
            },
        )
        .with_sort(SortType::None)
        .with_input(InputHint::new("file"))
    }

    pub fn reference(label: impl Into<String>, entity: impl Into<String>) -> Self {
        Self::new(
            label,
            FieldType::Reference {
                reference: Reference {
                    entity: entity.into(),
                    display_field: default_display_field(),
                },
            },
        )
        .with_input(InputHint::new("select"))
    }

    pub fn array(label: impl Into<String>, item: FieldType) -> Self {
        Self::new(
            label,
            FieldType::Array {
                item: Box::new(item),
            },
        )
        .with_sort(SortType::None)
    }

    pub fn shape(label: impl Into<String>, fields: IndexMap<String, ShapeField>) -> Self {
        Self::new(label, FieldType::Shape { fields }).with_sort(SortType::None)
    }

    pub fn with_sort(mut self, sort: SortType) -> Self {
        self.options.sort = sort;
        self
    }

    pub fn with_input(mut self, input: InputHint) -> Self {
        self.options.input = Some(input);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: JsonValue) -> Self {
        self.options.metadata.insert(key.into(), value);
        self
    }
}

impl ShapeField {
    pub fn new(label: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            label: label.into(),
            field_type,
            options: FieldOptions::default(),
        }
    }
}

/// Build a `Field` by kind name. Used by the WASM `make_field` export so
/// the JS `field` builder helpers can delegate entirely to Rust.
///
/// `kind`: `"string"` | `"number"` | `"boolean"` | `"file"` | `"ref"`
/// `entity`: required when `kind == "ref"`, ignored otherwise.
pub fn make_field(kind: &str, label: &str, entity: Option<&str>) -> Field {
    match kind {
        "number" => Field::number(label),
        "boolean" => Field::boolean(label),
        "file" => Field::file(label),
        "ref" => Field::reference(label, entity.unwrap_or("")),
        _ => Field::string(label),
    }
}
