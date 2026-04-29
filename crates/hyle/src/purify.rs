use std::collections::HashMap;

use garde::rules::{length, pattern, range};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::blueprint::Blueprint;
use crate::raw::{Row, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurifyError {
    pub field: String,
    pub rule: String,
    pub message: String,
}

/// A synchronous custom validation rule, dispatched by name via the
/// `"purifyAs": "<rule_name>"` field metadata key.
///
/// Register instances on a [`Purifier`] with [`Purifier::register_rule`].
pub trait SyncRule: Send + Sync {
    fn validate(&self, field_name: &str, value: &Value) -> Result<(), String>;
}

/// Metadata-driven synchronous validator.
///
/// Built-in rules read from `Field::options.metadata`:
/// - `"required": true`
/// - `"min": <number>` / `"max": <number>`
/// - `"minLength": <uint>` / `"maxLength": <uint>`
/// - `"pattern": "<regex>"`
/// - `"purifyAs": "<rule_name>"` — dispatches to a registered [`SyncRule`]
///
/// Built-in rules use [`garde`] under the hood. Custom rules are registered
/// by name and referenced from field metadata via `"purifyAs"`.
///
/// WASM-compatible: no async, no platform-specific deps.
pub struct Purifier {
    custom_rules: HashMap<String, Box<dyn SyncRule>>,
}

impl Default for Purifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Purifier {
    pub fn new() -> Self {
        Self {
            custom_rules: HashMap::new(),
        }
    }

    /// Register a named custom rule. Reference it from a field's metadata
    /// with `"purifyAs": "<name>"`.
    pub fn register_rule(&mut self, name: impl Into<String>, rule: Box<dyn SyncRule>) {
        self.custom_rules.insert(name.into(), rule);
    }

    /// Validate all fields of `model_name` in `row` against the blueprint's
    /// field metadata rules. Returns `Ok(())` if all pass, or
    /// `Err(Vec<PurifyError>)` with every violation found.
    pub fn purify_row(
        &self,
        blueprint: &Blueprint,
        model_name: &str,
        row: &Row,
    ) -> Result<(), Vec<PurifyError>> {
        let model = match blueprint.models.get(model_name) {
            Some(m) => m,
            None => {
                return Err(vec![PurifyError {
                    field: "model".to_owned(),
                    rule: "unknown_model".to_owned(),
                    message: format!("Unknown model: {}", model_name),
                }]);
            }
        };

        let mut errors = Vec::new();

        for (field_name, field) in &model.fields {
            let value = row.get(field_name);
            let metadata = &field.options.metadata;

            // 1. Required
            let is_required = metadata
                .get("required")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if is_required && value.map(Value::is_null).unwrap_or(true) {
                errors.push(PurifyError {
                    field: field_name.clone(),
                    rule: "required".to_owned(),
                    message: format!("Field '{}' is required", field_name),
                });
                continue;
            }

            let Some(value) = value else { continue };
            if value.is_null() {
                continue;
            }

            // 2. Numeric range via garde::rules::range
            if let Some(val) = value.as_f64() {
                let min = metadata.get("min").and_then(|v| v.as_f64());
                let max = metadata.get("max").and_then(|v| v.as_f64());

                if min.is_some() || max.is_some() {
                    if let Err(e) = range::apply(&val, (min, max)) {
                        let rule = if val < min.unwrap_or(f64::NEG_INFINITY) { "min" } else { "max" };
                        errors.push(PurifyError {
                            field: field_name.clone(),
                            rule: rule.to_owned(),
                            message: e.to_string(),
                        });
                    }
                }
            }

            // 3. String length + pattern via garde::rules::{length, pattern}
            if let Some(s) = value.as_str() {
                let min_len = metadata.get("minLength").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let max_len = metadata
                    .get("maxLength")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(usize::MAX as u64) as usize;

                if metadata.contains_key("minLength") || metadata.contains_key("maxLength") {
                    if let Err(e) = length::simple::apply(&s, (min_len, max_len)) {
                        let rule = if s.len() < min_len { "minLength" } else { "maxLength" };
                        errors.push(PurifyError {
                            field: field_name.clone(),
                            rule: rule.to_owned(),
                            message: e.to_string(),
                        });
                    }
                }

                if let Some(pat) = metadata.get("pattern").and_then(|v| v.as_str()) {
                    match Regex::new(pat) {
                        Ok(re) => {
                            if let Err(e) = pattern::apply(&s, (&re,)) {
                                errors.push(PurifyError {
                                    field: field_name.clone(),
                                    rule: "pattern".to_owned(),
                                    message: e.to_string(),
                                });
                            }
                        }
                        Err(e) => {
                            errors.push(PurifyError {
                                field: field_name.clone(),
                                rule: "pattern".to_owned(),
                                message: format!("Invalid pattern '{}': {}", pat, e),
                            });
                        }
                    }
                }
            }

            // 4. Custom sync rule via "purifyAs" metadata key
            if let Some(rule_name) = metadata.get("purifyAs").and_then(|v| v.as_str()) {
                if let Some(rule) = self.custom_rules.get(rule_name) {
                    if let Err(msg) = rule.validate(field_name, value) {
                        errors.push(PurifyError {
                            field: field_name.clone(),
                            rule: rule_name.to_owned(),
                            message: msg,
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Convenience wrapper: validate with a default (no custom rules) [`Purifier`].
/// Used by the WASM export and for simple cases that don't need custom rules.
pub fn purify_row_sync(
    blueprint: &Blueprint,
    model_name: &str,
    row: &Row,
) -> Result<(), Vec<PurifyError>> {
    Purifier::default().purify_row(blueprint, model_name, row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Blueprint, Field, Model};
    use indexmap::IndexMap;
    use serde_json::json;

    #[test]
    fn test_required() {
        let blueprint = Blueprint::new().model(
            "user",
            Model::new().field(
                "name",
                Field::string("Name").with_metadata("required", json!(true)),
            ),
        );

        let row: Row = IndexMap::new();
        let result = purify_row_sync(&blueprint, "user", &row);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0].rule, "required");
    }

    #[test]
    fn test_min_length() {
        let blueprint = Blueprint::new().model(
            "user",
            Model::new().field(
                "name",
                Field::string("Name")
                    .with_metadata("required", json!(true))
                    .with_metadata("minLength", json!(2)),
            ),
        );

        let row = IndexMap::from([("name".to_owned(), json!("A"))]);
        let result = purify_row_sync(&blueprint, "user", &row);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0].rule, "minLength");
    }

    #[test]
    fn test_valid_row() {
        let blueprint = Blueprint::new().model(
            "user",
            Model::new()
                .field(
                    "name",
                    Field::string("Name")
                        .with_metadata("required", json!(true))
                        .with_metadata("minLength", json!(2)),
                )
                .field(
                    "age",
                    Field::number("Age").with_metadata("min", json!(18)),
                ),
        );

        let row = IndexMap::from([
            ("name".to_owned(), json!("Alice")),
            ("age".to_owned(), json!(25)),
        ]);
        assert!(purify_row_sync(&blueprint, "user", &row).is_ok());
    }

    #[test]
    fn test_pattern() {
        let blueprint = Blueprint::new().model(
            "user",
            Model::new().field(
                "code",
                Field::string("Code").with_metadata("pattern", json!(r"^\d{4}$")),
            ),
        );

        let row = IndexMap::from([("code".to_owned(), json!("1234"))]);
        assert!(purify_row_sync(&blueprint, "user", &row).is_ok());

        let row = IndexMap::from([("code".to_owned(), json!("abcd"))]);
        let result = purify_row_sync(&blueprint, "user", &row);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0].rule, "pattern");
    }

    #[test]
    fn test_custom_sync_rule() {
        struct AllowListRule(Vec<String>);
        impl SyncRule for AllowListRule {
            fn validate(&self, _field_name: &str, value: &Value) -> Result<(), String> {
                let s = value.as_str().unwrap_or("");
                if self.0.iter().any(|v| v == s) {
                    Ok(())
                } else {
                    Err(format!("'{}' is not an allowed value", s))
                }
            }
        }

        let blueprint = Blueprint::new().model(
            "item",
            Model::new().field(
                "status",
                Field::string("Status").with_metadata("purifyAs", json!("allowlist")),
            ),
        );

        let mut purifier = Purifier::new();
        purifier.register_rule(
            "allowlist",
            Box::new(AllowListRule(vec!["active".into(), "inactive".into()])),
        );

        let row = IndexMap::from([("status".to_owned(), json!("active"))]);
        assert!(purifier.purify_row(&blueprint, "item", &row).is_ok());

        let row = IndexMap::from([("status".to_owned(), json!("unknown"))]);
        let result = purifier.purify_row(&blueprint, "item", &row);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err()[0].rule, "allowlist");
    }
}
