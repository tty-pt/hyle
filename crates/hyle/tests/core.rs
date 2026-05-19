use hyle::{
	Blueprint, Field, FieldType, Forma, FormaContext, FormaField, Model,
	ModelResult, Outcome, Primitive, Query, Reference, Row, ShapeField, Source, Value,
};
use indexmap::IndexMap;

fn blueprint() -> Blueprint {
	Blueprint::new()
		.model(
			"user",
			Model::new()
				.field("name", Field::string("Name"))
				.field("email", Field::string("Email"))
				.field("role", Field::reference("Role", "role"))
				.field("active", Field::boolean("Active")),
		)
		.model(
			"role",
			Model::new().field("name", Field::string("Role name")),
		)
}

#[test]
fn derives_query_plan_with_selected_fields_and_enum_dependencies() {
	let plan = blueprint()
		.manifest(
			Query::new("user")
				.select(["name", "role", "active"])
				.filter_layout(vec![vec!["name", "role"], vec!["active"]])
				.where_eq("active", Value::Bool(true))
				.page(2, 25)
				.sort_by("name", true),
		)
		.unwrap();

	assert_eq!(plan.base, "user");
	assert_eq!(plan.fields, vec!["name", "role", "active"]);
	assert_eq!(plan.filter["active"], Value::Bool(true));
	assert_eq!(plan.lookups, vec!["role"]);
	assert!(plan.inlines.is_empty());
	assert_eq!(plan.page, Some(2));
	assert_eq!(plan.per_page, Some(25));
	assert_eq!(plan.sort.unwrap().field, "name");
}

#[test]
fn marks_selected_references_as_joins_when_not_used_as_filters() {
	let plan = blueprint()
		.manifest(Query::new("user").select(["name", "role"]))
		.unwrap();

	assert!(plan.lookups.is_empty());
	assert_eq!(plan.inlines, vec!["role"]);
}

#[test]
fn resolves_raw_data_into_rows_and_lookup_maps() {
	let blueprint = blueprint();
	let plan = blueprint
		.manifest(Query::new("user").select(["name", "role"]))
		.unwrap();

	let mut user = IndexMap::new();
	user.insert("id".to_owned(), Value::Int(1));
	user.insert("name".to_owned(), Value::from("Alice"));
	user.insert("role".to_owned(), Value::from("admin"));

	let mut role = IndexMap::new();
	role.insert("id".to_owned(), Value::from("admin"));
	role.insert("name".to_owned(), Value::from("Admin"));

	let mut raw = Source::new();
	raw.insert("user".to_owned(), ModelResult::many(vec![user]));
	raw.insert("role".to_owned(), ModelResult::many(vec![role]));

	let resolved = blueprint.resolve(&plan, &raw).unwrap();

	assert_eq!(resolved.total, 1);
	assert_eq!(resolved.lookups["role"]["admin"]["name"], Value::from("Admin"));
}

#[test]
fn serializes_to_js_friendly_json() {
	let blueprint = blueprint();
	let plan = blueprint
		.manifest(Query::new("user").select(["name", "email"]))
		.unwrap();

	let json = serde_json::to_value(plan).unwrap();
	assert_eq!(json["base"], "user");
	assert_eq!(json["fields"], serde_json::json!(["name", "email"]));
}

#[test]
fn displays_reference_values_from_lookup_tables() {
	let blueprint = blueprint();
	let plan = blueprint
		.manifest(Query::new("user").select(["name", "role"]))
		.unwrap();

	let user = IndexMap::from([
		("id".to_owned(), Value::Int(1)),
		("name".to_owned(), Value::from("Alice")),
		("role".to_owned(), Value::from("admin")),
	]);
	let role = IndexMap::from([
		("id".to_owned(), Value::from("admin")),
		("name".to_owned(), Value::from("Admin")),
	]);

	let mut source = Source::new();
	source.insert("user".to_owned(), ModelResult::many(vec![user]));
	source.insert("role".to_owned(), ModelResult::many(vec![role]));

	let outcome = blueprint.resolve(&plan, &source).unwrap();
	let displayed = hyle::display_value(&blueprint, &outcome, "user", "role", &Value::from("admin"));

	assert_eq!(displayed, "Admin");
}

// ─── Group 1: plan errors and defaults ───────────────────────────────────────

#[test]
fn plan_selects_all_fields_when_select_is_empty() {
	let manifest = blueprint().manifest(Query::new("user")).unwrap();
	assert_eq!(manifest.fields, vec!["name", "email", "role", "active"]);
}

#[test]
fn plan_errors_on_unknown_model() {
	let result = blueprint().manifest(Query::new("ghost"));
	assert!(result.is_err());
}

#[test]
fn plan_errors_on_unknown_field_in_select() {
	let result = blueprint().manifest(Query::new("user").select(["name", "ghost"]));
	assert!(result.is_err());
}

#[test]
fn plan_errors_on_unknown_field_in_filter_layout() {
	let result = blueprint().manifest(
		Query::new("user")
			.select(["name"])
			.filter_layout([["ghost"]]),
	);
	assert!(result.is_err());
}

#[test]
fn plan_errors_on_unknown_reference_target() {
	let bp = Blueprint::new().model(
		"m",
		Model::new().field("x", Field::reference("X", "nonexistent")),
	);
	let result = bp.manifest(Query::new("m").select(["x"]));
	assert!(result.is_err());
}

// ─── Group 2: resolve error path ─────────────────────────────────────────────

#[test]
fn resolve_errors_when_base_model_missing_from_source() {
	let blueprint = blueprint();
	let manifest = blueprint.manifest(Query::new("user").select(["name"])).unwrap();
	let result = blueprint.resolve(&manifest, &Source::new());
	assert!(result.is_err());
}

// ─── Group 3: resolve_query ───────────────────────────────────────────────────

#[test]
fn resolve_query_returns_manifest_outcome_and_rows() {
	let blueprint = blueprint();

	let user = IndexMap::from([
		("id".to_owned(), Value::Int(1)),
		("name".to_owned(), Value::from("Alice")),
	]);
	let mut source = Source::new();
	source.insert("user".to_owned(), ModelResult::many(vec![user]));

	let (manifest, outcome, rows) = blueprint
		.resolve_query(Query::new("user").select(["name"]), &source)
		.unwrap();

	assert_eq!(manifest.base, "user");
	assert_eq!(outcome.total, 1);
	assert_eq!(rows.len(), 1);
	assert_eq!(rows[0]["name"], Value::from("Alice"));
}

// ─── Group 4: display_value variants ─────────────────────────────────────────

#[test]
fn display_value_formats_boolean_as_yes_no() {
	let blueprint = blueprint();
	let outcome = Outcome::empty();
	assert_eq!(
		hyle::display_value(&blueprint, &outcome, "user", "active", &Value::Bool(true)),
		"Yes"
	);
	assert_eq!(
		hyle::display_value(&blueprint, &outcome, "user", "active", &Value::Bool(false)),
		"No"
	);
}

#[test]
fn display_value_returns_empty_string_for_null() {
	let blueprint = blueprint();
	let outcome = Outcome::empty();
	assert_eq!(
		hyle::display_value(&blueprint, &outcome, "user", "name", &Value::Null),
		""
	);
}

#[test]
fn display_value_falls_back_for_unknown_field() {
	let blueprint = blueprint();
	let outcome = Outcome::empty();
	assert_eq!(
		hyle::display_value(&blueprint, &outcome, "user", "nonexistent", &Value::from("raw")),
		"raw"
	);
}

#[test]
fn display_value_renders_array_as_comma_separated() {
	let bp = Blueprint::new().model(
		"item",
		Model::new().field(
			"tags",
			Field::array(
				"Tags",
				FieldType::Primitive {
					primitive: Primitive::String,
				},
			),
		),
	);
	let outcome = Outcome::empty();
	let displayed = hyle::display_value(&bp, &outcome, "item", "tags", &Value::Array(vec![Value::from("a"), Value::from("b"), Value::from("c")]));
	assert_eq!(displayed, "a, b, c");
}

#[test]
fn display_value_renders_shape_as_label_colon_value() {
	let mut shape_fields = IndexMap::new();
	shape_fields.insert(
		"street".to_owned(),
		ShapeField::new("Street", FieldType::Primitive { primitive: Primitive::String }),
	);
	shape_fields.insert(
		"city".to_owned(),
		ShapeField::new("City", FieldType::Primitive { primitive: Primitive::String }),
	);

	let bp = Blueprint::new().model(
		"contact",
		Model::new().field("address", Field::shape("Address", shape_fields)),
	);
	let outcome = Outcome::empty();
	let displayed = hyle::display_value(
		&bp,
		&outcome,
		"contact",
		"address",
		&Value::Map(IndexMap::from([
			("street".to_owned(), Value::from("123 Main St")),
			("city".to_owned(), Value::from("Springfield")),
		])),
	);
	assert_eq!(displayed, "Street: 123 Main St; City: Springfield");
}

// ─── Group 7: ModelResult::one ───────────────────────────────────────────────

#[test]
fn model_result_one_has_total_of_one() {
	let row: Row = IndexMap::from([("id".to_owned(), Value::Int(42))]);
	let mr = ModelResult::one(row.clone());
	assert_eq!(mr.total, 1);
	assert_eq!(mr.rows(), vec![row]);
}

// ─── Group 9: forma context, filters, and field mapping ──────────────────────

#[test]
fn forma_to_query_uses_form_context_fields() {
	let forma = Forma {
		fields: vec![
			FormaField { name: "name".into(), label: "Name".into(), ..Default::default() },
			FormaField { name: "role".into(), label: "Role".into(), ..Default::default() },
		],
		form: Some(vec!["name".into()]),
		..Default::default()
	};
	let query = hyle::forma_to_query(&forma, "user", &FormaContext::Form, None);
	assert_eq!(query.select, vec!["name"]);
}

#[test]
fn forma_to_query_seeds_filters_from_forma() {
	let forma = Forma {
		fields: vec![
			FormaField { name: "name".into(), label: "Name".into(), ..Default::default() },
			FormaField { name: "role".into(), label: "Role".into(), ..Default::default() },
		],
		filters: Some(vec![vec!["name".into(), "role".into()]]),
		..Default::default()
	};
	let query = hyle::forma_to_query(&forma, "user", &FormaContext::Column, None);
	assert_eq!(query.filters, vec![vec!["name", "role"]]);

	let manifest = blueprint().manifest(query).unwrap();
	assert_eq!(manifest.filter_fields, vec![vec!["name", "role"]]);
}

// ─── Array<Reference> field type ─────────────────────────────────────────────

fn blueprint_with_tags() -> Blueprint {
	Blueprint::new()
		.model(
			"user",
			Model::new()
				.field("name", Field::string("Name"))
				.field(
					"tags",
					Field::array(
						"Tags",
						FieldType::Reference {
							reference: Reference {
								entity: "tag".into(),
								display_field: "name".into(),
							},
						},
					),
				),
		)
		.model("tag", Model::new().field("name", Field::string("Tag name")))
}

#[test]
fn array_reference_goes_in_lookups() {
	let bp = blueprint_with_tags();
	let manifest = bp
		.manifest(Query::new("user").select(["name", "tags"]))
		.unwrap();
	assert!(manifest.lookups.contains(&"tag".to_owned()));
}

#[test]
fn array_reference_unknown_entity_errors() {
	let bp = Blueprint::new().model(
		"user",
		Model::new().field(
			"tags",
			Field::array(
				"Tags",
				FieldType::Reference {
					reference: Reference {
						entity: "nonexistent".into(),
						display_field: "name".into(),
					},
				},
			),
		),
	);
	let result = bp.manifest(Query::new("user").select(["tags"]));
	assert!(result.is_err());
}

#[test]
fn display_value_array_reference_resolves_labels() {
	let bp = blueprint_with_tags();
	let manifest = bp
		.manifest(Query::new("user").select(["name", "tags"]))
		.unwrap();

	let user = IndexMap::from([
		("id".to_owned(), Value::Int(1)),
		("name".to_owned(), Value::from("Alice")),
		("tags".to_owned(), Value::Array(vec![Value::from("rust"), Value::from("web")])),
	]);
	let tag_rust = IndexMap::from([("id".to_owned(), Value::from("rust")), ("name".to_owned(), Value::from("Rust"))]);
	let tag_web  = IndexMap::from([("id".to_owned(), Value::from("web")),  ("name".to_owned(), Value::from("Web"))]);
	let mut source = Source::new();
	source.insert("user".to_owned(), ModelResult::many(vec![user]));
	source.insert("tag".to_owned(), ModelResult::many(vec![tag_rust, tag_web]));

	let outcome = bp.resolve(&manifest, &source).unwrap();
	let displayed = hyle::display_value(&bp, &outcome, "user", "tags", &Value::Array(vec![Value::from("rust"), Value::from("web")]));
	assert_eq!(displayed, "Rust, Web");
}
