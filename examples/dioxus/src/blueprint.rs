use hyle::{Blueprint, Field, Model};
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
                .field("active", Field::boolean("Active")),
        )
        .model("role", Model::new().field("name", Field::string("Role name")))
}
