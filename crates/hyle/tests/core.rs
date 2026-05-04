use hyle::{
    Blueprint, Field, FieldType, Forma, FormaContext, FormaField, FormaFieldType, Model,
    ModelResult, Outcome, Primitive, Query, Reference, Row, ShapeField, Source, Value,
};
use indexmap::IndexMap;
use serde_json::json;

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

fn two_named_rows() -> Vec<Row> {
    vec![
        IndexMap::from([("name".to_owned(), json!("Alice"))]),
        IndexMap::from([("name".to_owned(), json!("Bruno"))]),
    ]
}

#[test]
fn derives_query_plan_with_selected_fields_and_enum_dependencies() {
    let plan = blueprint()
        .manifest(
            Query::new("user")
                .select(["name", "role", "active"])
                .filter_layout(vec![vec!["name", "role"], vec!["active"]])
                .where_eq("active", json!(true))
                .page(2, 25)
                .sort_by("name", true),
        )
        .unwrap();

    assert_eq!(plan.base, "user");
    assert_eq!(plan.fields, vec!["name", "role", "active"]);
    assert_eq!(plan.filter["active"], json!(true));
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
    user.insert("id".to_owned(), Value::from(1));
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
    assert_eq!(resolved.lookups["role"]["admin"]["name"], json!("Admin"));
}

#[test]
fn serializes_to_js_friendly_json() {
    let blueprint = blueprint();
    let plan = blueprint
        .manifest(Query::new("user").select(["name", "email"]))
        .unwrap();

    let json = serde_json::to_value(plan).unwrap();
    assert_eq!(json["base"], "user");
    assert_eq!(json["fields"], json!(["name", "email"]));
}

#[test]
fn filters_rows_using_manifest_filter_semantics() {
    let rows = vec![
        IndexMap::from([
            ("name".to_owned(), json!("Alice")),
            ("active".to_owned(), json!(true)),
        ]),
        IndexMap::from([
            ("name".to_owned(), json!("Bruno")),
            ("active".to_owned(), json!(false)),
        ]),
    ];

    let filters = IndexMap::from([("name".to_owned(), json!("ali"))]);
    let filtered = hyle::filter_rows(&rows, &filters);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0]["name"], json!("Alice"));
}

#[test]
fn displays_reference_values_from_lookup_tables() {
    let blueprint = blueprint();
    let plan = blueprint
        .manifest(Query::new("user").select(["name", "role"]))
        .unwrap();

    let user = IndexMap::from([
        ("id".to_owned(), json!(1)),
        ("name".to_owned(), json!("Alice")),
        ("role".to_owned(), json!("admin")),
    ]);
    let role = IndexMap::from([
        ("id".to_owned(), json!("admin")),
        ("name".to_owned(), json!("Admin")),
    ]);

    let mut source = Source::new();
    source.insert("user".to_owned(), ModelResult::many(vec![user]));
    source.insert("role".to_owned(), ModelResult::many(vec![role]));

    let outcome = blueprint.resolve(&plan, &source).unwrap();
    let displayed = hyle::display_value(&blueprint, &outcome, "user", "role", &json!("admin"));

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
        ("id".to_owned(), json!(1)),
        ("name".to_owned(), json!("Alice")),
    ]);
    let mut source = Source::new();
    source.insert("user".to_owned(), ModelResult::many(vec![user]));

    let (manifest, outcome, rows) = blueprint
        .resolve_query(Query::new("user").select(["name"]), &source)
        .unwrap();

    assert_eq!(manifest.base, "user");
    assert_eq!(outcome.total, 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["name"], json!("Alice"));
}

// ─── Group 4: display_value variants ─────────────────────────────────────────

#[test]
fn display_value_formats_boolean_as_yes_no() {
    let blueprint = blueprint();
    let outcome = Outcome::empty();
    assert_eq!(
        hyle::display_value(&blueprint, &outcome, "user", "active", &json!(true)),
        "Yes"
    );
    assert_eq!(
        hyle::display_value(&blueprint, &outcome, "user", "active", &json!(false)),
        "No"
    );
}

#[test]
fn display_value_returns_empty_string_for_null() {
    let blueprint = blueprint();
    let outcome = Outcome::empty();
    assert_eq!(
        hyle::display_value(&blueprint, &outcome, "user", "name", &json!(null)),
        ""
    );
}

#[test]
fn display_value_falls_back_for_unknown_field() {
    let blueprint = blueprint();
    let outcome = Outcome::empty();
    assert_eq!(
        hyle::display_value(&blueprint, &outcome, "user", "nonexistent", &json!("raw")),
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
    let displayed = hyle::display_value(&bp, &outcome, "item", "tags", &json!(["a", "b", "c"]));
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
        &json!({ "street": "123 Main St", "city": "Springfield" }),
    );
    assert_eq!(displayed, "Street: 123 Main St; City: Springfield");
}

// ─── Group 6: filter_rows edge cases ─────────────────────────────────────────

#[test]
fn filter_rows_ignores_empty_string_filter() {
    let rows = two_named_rows();
    let filters = IndexMap::from([("name".to_owned(), json!(""))]);
    let filtered = hyle::filter_rows(&rows, &filters);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn filter_rows_ignores_null_filter() {
    let rows = two_named_rows();
    let filters = IndexMap::from([("name".to_owned(), json!(null))]);
    let filtered = hyle::filter_rows(&rows, &filters);
    assert_eq!(filtered.len(), 2);
}

// ─── Group 7: ModelResult::one ───────────────────────────────────────────────

#[test]
fn model_result_one_has_total_of_one() {
    let row: Row = IndexMap::from([("id".to_owned(), json!(42))]);
    let mr = ModelResult::one(row.clone());
    assert_eq!(mr.total, 1);
    assert_eq!(mr.rows(), vec![row]);
}

// ─── Group 8: purify numeric and maxLength ────────────────────────────────────

#[test]
fn purify_enforces_numeric_max() {
    let bp = Blueprint::new().model(
        "m",
        Model::new().field("score", Field::number("Score").with_metadata("max", json!(10))),
    );
    let row = IndexMap::from([("score".to_owned(), json!(11))]);
    let result = hyle::purify_row_sync(&bp, "m", &row);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err()[0].rule, "max");
}

#[test]
fn purify_enforces_max_length() {
    let bp = Blueprint::new().model(
        "m",
        Model::new()
            .field("code", Field::string("Code").with_metadata("maxLength", json!(3))),
    );
    let row = IndexMap::from([("code".to_owned(), json!("toolong"))]);
    let result = hyle::purify_row_sync(&bp, "m", &row);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err()[0].rule, "maxLength");
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
        ("id".to_owned(), json!(1)),
        ("name".to_owned(), json!("Alice")),
        ("tags".to_owned(), json!(["rust", "web"])),
    ]);
    let tag_rust = IndexMap::from([("id".to_owned(), json!("rust")), ("name".to_owned(), json!("Rust"))]);
    let tag_web  = IndexMap::from([("id".to_owned(), json!("web")),  ("name".to_owned(), json!("Web"))]);
    let mut source = Source::new();
    source.insert("user".to_owned(), ModelResult::many(vec![user]));
    source.insert("tag".to_owned(), ModelResult::many(vec![tag_rust, tag_web]));

    let outcome = bp.resolve(&manifest, &source).unwrap();
    let displayed = hyle::display_value(&bp, &outcome, "user", "tags", &json!(["rust", "web"]));
    assert_eq!(displayed, "Rust, Web");
}

#[test]
fn build_filter_fields_array_reference_has_options() {
    use hyle::{build_filter_fields, compute_manifest};
    let bp = blueprint_with_tags();
    let manifest_state = compute_manifest(&bp, &Query::new("user").select(["name", "tags"]));
    let manifest = if let hyle::HyleManifestState::Ready { manifest } = manifest_state {
        manifest
    } else {
        panic!("expected ready manifest");
    };

    let tag = IndexMap::from([("id".to_owned(), json!("rust")), ("name".to_owned(), json!("Rust"))]);
    let mut source = Source::new();
    source.insert("user".to_owned(), ModelResult::many(vec![]));
    source.insert("tag".to_owned(), ModelResult::many(vec![tag]));
    let outcome = bp.resolve(&manifest, &source).unwrap();

    let fields = build_filter_fields(&bp, &manifest, &outcome);
    let tags_field = fields.iter().find(|f| f.key == "tags").unwrap();
    assert!(tags_field.options.is_some());
    let opts = tags_field.options.as_ref().unwrap();
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].0, "rust");
    assert_eq!(opts[0].1, "Rust");
}
