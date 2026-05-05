use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// ── Mutation input ─────────────────────────────────────────────────────────────

/// The input passed to a mutation (create, update, or delete).
///
/// `model` is the model name (mirrors `Query.model`), allowing generic adapters
/// to route to the correct endpoint without knowing the model at compile time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MutateInput {
    /// The model name — e.g. `"user"`.
    pub model: String,
    /// The row id when editing or deleting an existing record; `None` for creates.
    pub id: Option<JsonValue>,
    /// The form field values to submit.
    pub data: IndexMap<String, String>,
}

// ── URL query string parsing ───────────────────────────────────────────────────

/// Parse a URL query string into `(page, per_page, filters)`.
///
/// - `page` defaults to `1`; `per_page` defaults to `default_per_page`.
/// - Keys `"page"` and `"per_page"` are consumed; all others go into `filters`.
/// - Empty-value keys are omitted from `filters`.
/// - Percent-encoding and `+`-as-space are decoded.
///
/// # Example
/// ```
/// let (page, per_page, filters) = hyle::parse_query_params("page=2&name=Ali", 5);
/// assert_eq!(page, 2);
/// assert_eq!(per_page, 5);
/// assert_eq!(filters["name"], "Ali");
/// ```
pub fn parse_query_params(
    query_str: &str,
    default_per_page: usize,
) -> (usize, usize, IndexMap<String, String>) {
    let mut page = 1usize;
    let mut per_page = default_per_page;
    let mut filters: IndexMap<String, String> = IndexMap::new();

    for part in query_str.split('&').filter(|s| !s.is_empty()) {
        let mut kv = part.splitn(2, '=');
        let k = match kv.next() {
            Some(k) => k,
            None => continue,
        };
        let v = percent_decode(kv.next().unwrap_or(""));
        match k {
            "page"     => { if let Ok(n) = v.parse::<usize>() { page     = n.max(1); } }
            "per_page" => { if let Ok(n) = v.parse::<usize>() { per_page = n.max(1); } }
            _ => {
                if !v.is_empty() {
                    // Repeated keys (e.g. tags=rust&tags=web) are joined with commas
                    // so that filter_rows can match against the comma-joined value.
                    filters
                        .entry(k.to_owned())
                        .and_modify(|existing| {
                            existing.push(',');
                            existing.push_str(&v);
                        })
                        .or_insert(v);
                }
            }
        }
    }

    (page, per_page, filters)
}

fn percent_decode(s: &str) -> String {
    let s = s.replace('+', " ");
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sort {
    pub field: String,
    #[serde(default)]
    pub ascending: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub model: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub select: Vec<String>,
    #[serde(rename = "where", default, skip_serializing_if = "IndexMap::is_empty")]
    pub where_: IndexMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_page: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

impl Query {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Self::default()
        }
    }

    pub fn select(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.select = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn filter_layout<I, J, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = J>,
        J: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.filters = rows
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect())
            .collect();
        self
    }

    pub fn where_eq(mut self, field: impl Into<String>, value: JsonValue) -> Self {
        self.where_.insert(field.into(), value);
        self
    }

    pub fn page(mut self, page: usize, per_page: usize) -> Self {
        self.page = Some(page);
        self.per_page = Some(per_page);
        self
    }

    pub fn sort_by(mut self, field: impl Into<String>, ascending: bool) -> Self {
        self.sort = Some(Sort {
            field: field.into(),
            ascending,
        });
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonValue>,
    pub fields: Vec<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub filter: IndexMap<String, JsonValue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lookups: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inlines: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_page: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// 2-D filter layout carried from `query.filters` for UI layout derivation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filter_fields: Vec<Vec<String>>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_string() {
        let (page, per_page, filters) = parse_query_params("", 5);
        assert_eq!(page, 1);
        assert_eq!(per_page, 5);
        assert!(filters.is_empty());
    }

    #[test]
    fn parse_page_and_per_page() {
        let (page, per_page, filters) = parse_query_params("page=3&per_page=10", 5);
        assert_eq!(page, 3);
        assert_eq!(per_page, 10);
        assert!(filters.is_empty());
    }

    #[test]
    fn parse_page_zero_clamps_to_one() {
        let (page, _, _) = parse_query_params("page=0", 5);
        assert_eq!(page, 1);
    }

    #[test]
    fn parse_filters() {
        let (page, per_page, filters) = parse_query_params("name=Ali&role=admin", 5);
        assert_eq!(page, 1);
        assert_eq!(per_page, 5);
        assert_eq!(filters["name"], "Ali");
        assert_eq!(filters["role"], "admin");
    }

    #[test]
    fn parse_empty_value_omitted_from_filters() {
        let (_, _, filters) = parse_query_params("name=", 5);
        assert!(!filters.contains_key("name"));
    }

    #[test]
    fn parse_percent_encoding() {
        let (_, _, filters) = parse_query_params("name=Ali%20Smith", 5);
        assert_eq!(filters["name"], "Ali Smith");
    }

    #[test]
    fn parse_plus_as_space() {
        let (_, _, filters) = parse_query_params("name=Ali+Smith", 5);
        assert_eq!(filters["name"], "Ali Smith");
    }

    #[test]
    fn parse_default_per_page_respected() {
        let (_, per_page, _) = parse_query_params("page=2", 20);
        assert_eq!(per_page, 20);
    }

    #[test]
    fn parse_percent_encoding_multibyte_utf8() {
        // "héllo" encoded: é = %C3%A9
        let (_, _, filters) = parse_query_params("name=h%C3%A9llo", 5);
        assert_eq!(filters["name"], "héllo");
    }

    #[test]
    fn parse_repeated_key_joins_with_comma() {
        // Repeated URL params (e.g. ?tags=rust&tags=web from checkboxes) must be
        // joined with a comma so filter_rows can split them back.
        let (_, _, filters) = parse_query_params("tags=rust&tags=web", 5);
        assert_eq!(filters["tags"], "rust,web");
    }

    #[test]
    fn parse_single_array_key_not_joined() {
        // A single occurrence of a key is stored as-is (no trailing comma).
        let (_, _, filters) = parse_query_params("tags=rust", 5);
        assert_eq!(filters["tags"], "rust");
    }

    #[test]
    fn where_serialises_as_where_not_where_underscore() {
        use serde_json::json;
        let q = Query::new("user").where_eq("name", json!("Alice"));
        let serialised = serde_json::to_value(&q).unwrap();
        assert!(serialised.get("where").is_some(), "expected 'where' key");
        assert!(serialised.get("where_").is_none(), "unexpected 'where_' key");
        let round_trip: Query = serde_json::from_value(serialised).unwrap();
        assert_eq!(round_trip.where_.get("name"), Some(&json!("Alice")));
    }
}
