use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

use crate::error::{Error, HyleResult};
use crate::field::{Field, FieldType};
use crate::query::{Manifest, Query};
use crate::raw::{ModelRows, Outcome, Row, Source, value_to_lookup_key};
use crate::view::{apply_view, derive_columns, Column};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub fields: IndexMap<String, Field>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            fields: IndexMap::new(),
        }
    }

    pub fn field(mut self, name: impl Into<String>, field: Field) -> Self {
        self.fields.insert(name.into(), field);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blueprint {
    #[serde(default)]
    pub models: IndexMap<String, Model>,
}

/// Output of [`Blueprint::resolve_and_view`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedView {
    pub outcome: Outcome,
    pub rows: Vec<Row>,
    pub is_single: bool,
    pub columns: Vec<Column>,
}

impl Blueprint {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn model(mut self, name: impl Into<String>, model: Model) -> Self {
        self.models.insert(name.into(), model);
        self
    }

    pub fn manifest(&self, query: Query) -> HyleResult<Manifest> {
        let model = self
            .models
            .get(&query.model)
            .ok_or_else(|| Error::UnknownModel(query.model.clone()))?;

        let mut fields = if query.select.is_empty() {
            model.fields.keys().cloned().collect::<Vec<_>>()
        } else {
            query.select.clone()
        };

        if fields.is_empty() {
            return Err(Error::EmptySelection);
        }

        for field in &fields {
            if !model.fields.contains_key(field) {
                return Err(Error::UnknownField {
                    model: query.model.clone(),
                    field: field.clone(),
                });
            }
        }

        let id = query.where_.get("id").cloned();
        let filter = query
            .where_
            .iter()
            .filter(|(key, _)| key.as_str() != "id")
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<IndexMap<_, _>>();

        let explicit_filter_fields = query
            .filters
            .iter()
            .flatten()
            .cloned()
            .collect::<IndexSet<_>>();

        for field in &explicit_filter_fields {
            if !model.fields.contains_key(field) {
                return Err(Error::UnknownField {
                    model: query.model.clone(),
                    field: field.clone(),
                });
            }
        }

        let mut lookups = IndexSet::new();
        let mut inlines = IndexSet::new();

        for field_name in &fields {
            let field = &model.fields[field_name];
            collect_references(
                self,
                &query.model,
                field_name,
                &field.field_type,
                explicit_filter_fields.contains(field_name),
                &mut lookups,
                &mut inlines,
            )?;
        }

        for field_name in &explicit_filter_fields {
            if fields.contains(field_name) {
                continue;
            }

            let field = &model.fields[field_name];
            collect_references(
                self,
                &query.model,
                field_name,
                &field.field_type,
                true,
                &mut lookups,
                &mut inlines,
            )?;
        }

        fields.shrink_to_fit();

        Ok(Manifest {
            base: query.model,
            id,
            fields,
            filter,
            lookups: lookups.into_iter().collect(),
            inlines: inlines.into_iter().collect(),
            page: query.page,
            per_page: query.per_page,
            sort: query.sort,
            method: query.method,
            filter_fields: query.filters,
        })
    }

    /// Convenience: manifest + resolve + normalise rows in one call.
    pub fn resolve_query(&self, query: Query, source: &Source) -> HyleResult<(Manifest, Outcome, Vec<Row>)> {
        let manifest = self.manifest(query)?;
        let outcome = self.resolve(&manifest, source)?;
        let rows = outcome.rows.rows();
        Ok((manifest, outcome, rows))
    }

    /// resolve + apply_view + is_single + derive_columns in one call.
    ///
    /// Returns a [`ResolvedView`] containing the filtered/sorted/paginated rows,
    /// whether the result represents a single record, and the column metadata —
    /// collapsing what would otherwise be 4–5 separate WASM round-trips.
    pub fn resolve_and_view(&self, manifest: &Manifest, source: &Source) -> HyleResult<ResolvedView> {
        let outcome = self.resolve(manifest, source)?;
        let all_rows = outcome.rows.rows();
        let rows = apply_view(all_rows, manifest);
        let is_single = crate::raw::is_single(manifest, &outcome);
        let columns = derive_columns(self, manifest)?;
        Ok(ResolvedView { outcome, rows, is_single, columns })
    }

    pub fn resolve(&self, manifest: &Manifest, source: &Source) -> HyleResult<Outcome> {
        let base = source
            .get(&manifest.base)
            .ok_or_else(|| Error::MissingBaseModel(manifest.base.clone()))?;

        let mut lookups = IndexMap::new();

        for model_name in manifest.lookups.iter().chain(manifest.inlines.iter()) {
            if let Some(result) = source.get(model_name) {
                lookups.insert(model_name.clone(), rows_by_id(result.rows()));
            }
        }

        Ok(Outcome {
            rows: base.result.clone(),
            total: base.total,
            lookups,
        })
    }
}

fn collect_references(
    blueprint: &Blueprint,
    source_model: &str,
    source_field: &str,
    field_type: &FieldType,
    explicit_need: bool,
    lookups: &mut IndexSet<String>,
    inlines: &mut IndexSet<String>,
) -> HyleResult<()> {
    match field_type {
        FieldType::Primitive { .. } => Ok(()),
        FieldType::Reference { reference } => {
            if !blueprint.models.contains_key(&reference.entity) {
                return Err(Error::UnknownReference {
                    model: source_model.to_owned(),
                    field: source_field.to_owned(),
                    target: reference.entity.clone(),
                });
            }

            if explicit_need {
                lookups.insert(reference.entity.clone());
            } else {
                inlines.insert(reference.entity.clone());
            }

            Ok(())
        }
        FieldType::Array { item } => collect_references(
            blueprint,
            source_model,
            source_field,
            item,
            explicit_need,
            lookups,
            inlines,
        ),
        FieldType::Shape { fields } => {
            for (name, field) in fields {
                collect_references(
                    blueprint,
                    source_model,
                    name,
                    &field.field_type,
                    explicit_need,
                    lookups,
                    inlines,
                )?;
            }
            Ok(())
        }
    }
}

fn rows_by_id(rows: Vec<Row>) -> IndexMap<String, Row> {
    rows.into_iter()
        .filter_map(|row| {
            let id = row.get("id").and_then(value_to_lookup_key)?;
            Some((id.clone(), row))
        })
        .collect()
}

#[allow(dead_code)]
fn _assert_rows_send_sync(_: ModelRows) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{Field, Reference};
    use crate::raw::ModelResult;
    use serde_json::json;

    fn simple_blueprint() -> Blueprint {
        Blueprint::new()
            .model(
                "user",
                Model::new()
                    .field("name", Field::string("Name"))
                    .field("email", Field::string("Email"))
                    .field("role", Field::reference("Role", "role")),
            )
            .model(
                "role",
                Model::new()
                    .field("name", Field::string("Name")),
            )
    }

    fn user_source() -> Source {
        let mut src = Source::new();
        src.insert(
            "user".into(),
            ModelResult::many(vec![
                indexmap::indexmap! {
                    "id".to_owned()   => json!(1),
                    "name".to_owned() => json!("Alice"),
                    "email".to_owned()=> json!("alice@example.test"),
                    "role".to_owned() => json!("admin"),
                },
            ]),
        );
        src.insert(
            "role".into(),
            ModelResult::many(vec![
                indexmap::indexmap! {
                    "id".to_owned()   => json!("admin"),
                    "name".to_owned() => json!("Admin"),
                },
            ]),
        );
        src
    }

    // ── manifest ──────────────────────────────────────────────────────────────

    #[test]
    fn manifest_happy_path() {
        let bp = simple_blueprint();
        let q = Query::new("user").select(["name", "email"]);
        let m = bp.manifest(q).unwrap();
        assert_eq!(m.base, "user");
        assert_eq!(m.fields, vec!["name", "email"]);
        assert!(m.lookups.is_empty());
        assert!(m.inlines.is_empty());
    }

    #[test]
    fn manifest_empty_select_uses_all_fields() {
        let bp = simple_blueprint();
        let q = Query::new("user");
        let m = bp.manifest(q).unwrap();
        assert!(m.fields.contains(&"name".to_owned()));
        assert!(m.fields.contains(&"role".to_owned()));
    }

    #[test]
    fn manifest_unknown_model_errors() {
        let bp = simple_blueprint();
        let q = Query::new("ghost");
        assert!(matches!(bp.manifest(q), Err(Error::UnknownModel(_))));
    }

    #[test]
    fn manifest_unknown_field_errors() {
        let bp = simple_blueprint();
        let q = Query::new("user").select(["ghost"]);
        assert!(matches!(bp.manifest(q), Err(Error::UnknownField { .. })));
    }

    #[test]
    fn manifest_reference_field_goes_in_inlines() {
        let bp = simple_blueprint();
        let q = Query::new("user").select(["name", "role"]);
        let m = bp.manifest(q).unwrap();
        assert!(m.inlines.contains(&"role".to_owned()));
        assert!(m.lookups.is_empty());
    }

    #[test]
    fn manifest_reference_field_in_filter_goes_in_lookups() {
        let bp = simple_blueprint();
        let q = Query::new("user")
            .select(["name", "role"])
            .filter_layout([["role"]]);
        let m = bp.manifest(q).unwrap();
        assert!(m.lookups.contains(&"role".to_owned()));
        assert!(!m.inlines.contains(&"role".to_owned()));
    }

    #[test]
    fn manifest_unknown_reference_errors() {
        let bp = Blueprint::new().model(
            "user",
            Model::new().field("dept", Field::reference("Dept", "department")),
        );
        let q = Query::new("user").select(["dept"]);
        assert!(matches!(bp.manifest(q), Err(Error::UnknownReference { .. })));
    }

    // ── resolve ───────────────────────────────────────────────────────────────

    #[test]
    fn resolve_happy_path() {
        let bp = simple_blueprint();
        let q = Query::new("user").select(["name", "role"]);
        let m = bp.manifest(q).unwrap();
        let src = user_source();
        let outcome = bp.resolve(&m, &src).unwrap();
        assert_eq!(outcome.rows.rows().len(), 1);
        assert!(outcome.lookups.contains_key("role"));
    }

    #[test]
    fn resolve_missing_base_model_errors() {
        let bp = simple_blueprint();
        let manifest = Manifest {
            base: "ghost".into(),
            id: None,
            fields: vec![],
            filter: Default::default(),
            lookups: vec![],
            inlines: vec![],
            page: None,
            per_page: None,
            sort: None,
            method: None,
            filter_fields: vec![],
        };
        let src = user_source();
        assert!(matches!(bp.resolve(&manifest, &src), Err(Error::MissingBaseModel(_))));
    }

    // ── resolve_and_view ──────────────────────────────────────────────────────

    #[test]
    fn resolve_and_view_end_to_end() {
        let bp = simple_blueprint();
        let q = Query::new("user").select(["name", "email"]);
        let m = bp.manifest(q).unwrap();
        let src = user_source();
        let view = bp.resolve_and_view(&m, &src).unwrap();
        assert_eq!(view.rows.len(), 1);
        assert_eq!(view.rows[0]["name"], json!("Alice"));
        assert_eq!(view.columns.len(), 2);
        assert!(!view.is_single);
    }

    #[test]
    fn resolve_and_view_single_record() {
        let bp = simple_blueprint();
        let q = Query {
            model: "user".into(),
            select: vec!["name".into()],
            method: Some("one".into()),
            ..Default::default()
        };
        let m = bp.manifest(q).unwrap();
        let src = user_source();
        let view = bp.resolve_and_view(&m, &src).unwrap();
        assert!(view.is_single);
    }
}
