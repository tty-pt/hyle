use hyle::{ModelResult, Row, Source, Value};
use indexmap::IndexMap;
use qmap::Qmap;
use std::ffi::CStr;
use libc::c_char;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String = 0,
    Int = 1,
    Bool = 2,
    NullableString = 3,
    Reference = 4,
    MultiReference = 5,
}

pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub target_dataset: Option<String>,
    pub inverse_name: Option<String>,
}

pub struct DatasetDef {
    pub id: String,
    pub fields: Vec<FieldDef>,
    pub source_hd: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelationItem {
    pub id: String,
    #[serde(flatten)]
    pub inverse: IndexMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Relation {
    pub target: String,
    pub inverse: Option<String>,
    pub items: Vec<RelationItem>,
}

/// Builds a Hyle Source directly from qmap, assuming the qmap value is a JSON object string.
/// This matches the current behavior of common_dataset.c while we transition.
pub fn build_source_from_json_qmap(qmap: &Qmap, def: &DatasetDef) -> Source {
    let mut rows = Vec::new();
    let mut cursor = qmap.iter(std::ptr::null(), 0);

    while let Some((_key_ptr, val_ptr)) = cursor.next() {
        let c_str = unsafe { CStr::from_ptr(val_ptr as *const c_char) };
        if let Ok(json_str) = c_str.to_str() {
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(json_str) {
                let mut row: Row = IndexMap::new();
                for (k, v) in map {
                    row.insert(k, json_to_hyle_value(v));
                }
                rows.push(row);
            }
        }
    }

    let mut source = Source::new();
    source.insert(def.id.clone(), ModelResult::many(rows));
    
    // Build relations
    let mut relations = IndexMap::new();
    for field in &def.fields {
        if let Some(target) = &field.target_dataset {
            if let Some(rel) = build_relation(qmap, def, field, target) {
                relations.insert(field.name.clone(), rel);
            }
        }
    }
    
    if !relations.is_empty() {
        if let Ok(rel_val) = serde_json::to_value(relations) {
            source.insert("relations".to_string(), ModelResult::one(json_to_hyle_row(rel_val)));
        }
    }

    source
}

fn build_relation(qmap: &Qmap, _def: &DatasetDef, field: &FieldDef, target: &str) -> Option<Relation> {
    // We need to build the inverse mapping
    // This replicates dataset_build_relations_json from C
    
    // 1. Collect all references from all rows
    let mut inverse_map: IndexMap<String, Vec<String>> = IndexMap::new();
    let mut cursor = qmap.iter(std::ptr::null(), 0);
    while let Some((key_ptr, val_ptr)) = cursor.next() {
        let id = unsafe { CStr::from_ptr(key_ptr as *const c_char) }.to_string_lossy().into_owned();
        let row_json = unsafe { CStr::from_ptr(val_ptr as *const c_char) }.to_string_lossy();
        
        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(&row_json) {
            if let Some(val) = map.get(&field.name) {
                match val {
                    serde_json::Value::String(s) => {
                        if !s.is_empty() {
                            inverse_map.entry(s.clone()).or_default().push(id);
                        }
                    }
                    serde_json::Value::Array(arr) => {
                        for v in arr {
                            if let serde_json::Value::String(s) = v {
                                if !s.is_empty() {
                                    inverse_map.entry(s.clone()).or_default().push(id.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    if inverse_map.is_empty() {
        return None;
    }
    
    let mut items = Vec::new();
    for (target_id, source_ids) in inverse_map {
        let mut inverse = IndexMap::new();
        if let Some(inv_name) = &field.inverse_name {
            inverse.insert(inv_name.clone(), source_ids);
        }
        items.push(RelationItem {
            id: target_id,
            inverse,
        });
    }
    
    Some(Relation {
        target: target.to_string(),
        inverse: field.inverse_name.clone(),
        items,
    })
}

fn json_to_hyle_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Number(n) => Value::String(n.to_string()),
        serde_json::Value::Bool(b) => Value::String(b.to_string()),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_to_hyle_value).collect())
        }
        serde_json::Value::Null => Value::String(String::new()),
        serde_json::Value::Object(obj) => Value::String(serde_json::to_string(&obj).unwrap_or_default()),
    }
}

fn json_to_hyle_row(v: serde_json::Value) -> Row {
    if let serde_json::Value::Object(map) = v {
        let mut row = IndexMap::new();
        for (k, val) in map {
            row.insert(k, json_to_hyle_value(val));
        }
        row
    } else {
        IndexMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_source_from_json_qmap() {
        let q = Qmap::open(None, None, 2, 2, 0xFF, 0).unwrap();
        q.put_str("song1", "{\"id\":\"song1\", \"title\":\"Song One\", \"type\":\"Rock\"}");
        q.put_str("song2", "{\"id\":\"song2\", \"title\":\"Song Two\", \"type\":[\"Jazz\",\"Blues\"]}");

        let def = DatasetDef {
            id: "song.items".to_string(),
            fields: vec![
                FieldDef {
                    name: "id".to_string(),
                    field_type: FieldType::String,
                    target_dataset: None,
                    inverse_name: None,
                },
                FieldDef {
                    name: "title".to_string(),
                    field_type: FieldType::String,
                    target_dataset: None,
                    inverse_name: None,
                },
                FieldDef {
                    name: "type".to_string(),
                    field_type: FieldType::MultiReference,
                    target_dataset: Some("song.types".to_string()),
                    inverse_name: Some("songs".to_string()),
                },
            ],
            source_hd: q.handle(),
        };

        let source = build_source_from_json_qmap(&q, &def);
        assert!(source.contains_key("song.items"));
        assert!(source.contains_key("relations"));

        let result = source.get("song.items").unwrap();
        let rows = result.rows();
        assert_eq!(rows.len(), 2);

        let song2 = rows.iter().find(|r| r.get("id") == Some(&Value::String("song2".to_string()))).unwrap();
        assert_eq!(song2.get("type"), Some(&Value::Array(vec![Value::String("Jazz".to_string()), Value::String("Blues".to_string())])));

        let rel_result = source.get("relations").unwrap();
        let rel_row = &rel_result.rows()[0];
        let type_rel_str = match rel_row.get("type").unwrap() {
            Value::String(s) => s,
            _ => panic!("Expected string JSON for relation"),
        };
        let type_rel: Relation = serde_json::from_str(type_rel_str).unwrap();
        assert_eq!(type_rel.target, "song.types");
        assert_eq!(type_rel.items.len(), 3); // Rock, Jazz, Blues
    }

    #[test]
    fn test_single_reference() {
        let q = Qmap::open(None, None, 2, 2, 0xFF, 0).unwrap();
        q.put_str("song1", "{\"id\":\"song1\", \"author\":\"author1\"}");
        q.put_str("song2", "{\"id\":\"song2\", \"author\":\"author1\"}");

        let def = DatasetDef {
            id: "song.items".to_string(),
            fields: vec![
                FieldDef {
                    name: "id".to_string(),
                    field_type: FieldType::String,
                    target_dataset: None,
                    inverse_name: None,
                },
                FieldDef {
                    name: "author".to_string(),
                    field_type: FieldType::Reference,
                    target_dataset: Some("author.items".to_string()),
                    inverse_name: Some("songs".to_string()),
                },
            ],
            source_hd: q.handle(),
        };

        let source = build_source_from_json_qmap(&q, &def);
        let rel_result = source.get("relations").unwrap();
        let rel_row = &rel_result.rows()[0];
        let author_rel_str = match rel_row.get("author").unwrap() {
            Value::String(s) => s,
            _ => panic!("Expected string JSON for relation"),
        };
        let author_rel: Relation = serde_json::from_str(author_rel_str).unwrap();
        assert_eq!(author_rel.target, "author.items");
        assert_eq!(author_rel.items.len(), 1); // Only author1
        assert_eq!(author_rel.items[0].id, "author1");
        assert_eq!(author_rel.items[0].inverse.get("songs").unwrap(), &vec!["song1".to_string(), "song2".to_string()]);
    }

    #[test]
    fn test_primitive_types() {
        let q = Qmap::open(None, None, 2, 2, 0xFF, 0).unwrap();
        q.put_str("item1", "{\"id\":\"item1\", \"count\":42, \"active\":true, \"price\":3.14}");

        let def = DatasetDef {
            id: "items".to_string(),
            fields: vec![
                FieldDef { name: "id".to_string(), field_type: FieldType::String, target_dataset: None, inverse_name: None },
                FieldDef { name: "count".to_string(), field_type: FieldType::Int, target_dataset: None, inverse_name: None },
                FieldDef { name: "active".to_string(), field_type: FieldType::Bool, target_dataset: None, inverse_name: None },
            ],
            source_hd: q.handle(),
        };

        let source = build_source_from_json_qmap(&q, &def);
        let result = source.get("items").unwrap();
        let row = &result.rows()[0];
        assert_eq!(row.get("count"), Some(&Value::String("42".to_string())));
        assert_eq!(row.get("active"), Some(&Value::String("true".to_string())));
        assert_eq!(row.get("price"), Some(&Value::String("3.14".to_string())));
    }

    #[test]
    fn test_null_and_empty_values() {
        let q = Qmap::open(None, None, 2, 2, 0xFF, 0).unwrap();
        q.put_str("item1", "{\"id\":\"item1\", \"ref\":null, \"multi_ref\":[], \"str\":\"\"}");
        q.put_str("item2", "{\"id\":\"item2\", \"multi_ref\":[\"\"]}");

        let def = DatasetDef {
            id: "items".to_string(),
            fields: vec![
                FieldDef { name: "ref".to_string(), field_type: FieldType::Reference, target_dataset: Some("target".to_string()), inverse_name: None },
                FieldDef { name: "multi_ref".to_string(), field_type: FieldType::MultiReference, target_dataset: Some("target".to_string()), inverse_name: None },
            ],
            source_hd: q.handle(),
        };

        let source = build_source_from_json_qmap(&q, &def);
        let result = source.get("items").unwrap();
        let rows = result.rows();
        let item1 = rows.iter().find(|r| r.get("id") == Some(&Value::String("item1".to_string()))).unwrap();
        assert_eq!(item1.get("ref"), Some(&Value::String("".to_string())));
        assert_eq!(item1.get("str"), Some(&Value::String("".to_string())));
        
        assert!(!source.contains_key("relations"));
    }

    #[test]
    fn test_multiple_relations() {
        let q = Qmap::open(None, None, 2, 2, 0xFF, 0).unwrap();
        q.put_str("item1", "{\"id\":\"item1\", \"ref1\":\"targetA1\", \"ref2\":\"targetB1\"}");

        let def = DatasetDef {
            id: "items".to_string(),
            fields: vec![
                FieldDef { name: "ref1".to_string(), field_type: FieldType::Reference, target_dataset: Some("targetA".to_string()), inverse_name: None },
                FieldDef { name: "ref2".to_string(), field_type: FieldType::Reference, target_dataset: Some("targetB".to_string()), inverse_name: None },
            ],
            source_hd: q.handle(),
        };

        let source = build_source_from_json_qmap(&q, &def);
        let rel_result = source.get("relations").unwrap();
        let rel_row = &rel_result.rows()[0];
        
        assert!(rel_row.contains_key("ref1"));
        assert!(rel_row.contains_key("ref2"));
    }

    #[test]
    fn test_fault_tolerance() {
        let q = Qmap::open(None, None, 2, 2, 0xFF, 0).unwrap();
        q.put_str("item1", "{\"id\":\"item1\"}");
        q.put_str("item2", "INVALID JSON");
        q.put_str("item3", "{\"id\":\"item3\"}");

        let def = DatasetDef {
            id: "items".to_string(),
            fields: vec![],
            source_hd: q.handle(),
        };

        let source = build_source_from_json_qmap(&q, &def);
        let result = source.get("items").unwrap();
        assert_eq!(result.rows().len(), 2);
        assert_eq!(result.rows()[0].get("id"), Some(&Value::String("item1".to_string())));
        assert_eq!(result.rows()[1].get("id"), Some(&Value::String("item3".to_string())));
    }
}
