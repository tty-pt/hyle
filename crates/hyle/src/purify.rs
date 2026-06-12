use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurifyError {
	pub field: String,
	pub rule: String,
	pub message: String,
}
