use hyle::{Blueprint, Field, FieldType, Model, Reference};
use serde_json::json;

pub fn make_blueprint() -> Blueprint {
    Blueprint::new()
        .model(
            "user",
            Model::new()
                .field("name", Field::string("Name")
                    .with_metadata("required", json!(true))
                    .with_metadata("minLength", json!(2)))
                .field("email", Field::string("Email"))
                .field("role", Field::reference("Role", "role"))
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
                )
                .field("active", Field::boolean("Active")),
        )
        .model("role", Model::new().field("name", Field::string("Role name")))
        .model("tag", Model::new().field("name", Field::string("Tag name")))
}
