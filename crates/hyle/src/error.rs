use thiserror::Error;

pub type HyleResult<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("unknown model: {0}")]
    UnknownModel(String),

    #[error("unknown field `{field}` on model `{model}`")]
    UnknownField { model: String, field: String },

    #[error("field `{field}` on model `{model}` references unknown model `{target}`")]
    UnknownReference {
        model: String,
        field: String,
        target: String,
    },

    #[error("query must select at least one field")]
    EmptySelection,

    #[error("source is missing base model `{0}`")]
    MissingBaseModel(String),
}
