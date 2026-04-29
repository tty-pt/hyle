use axum::{
    body::Body,
    extract::State,
    http::Request,
    response::{IntoResponse, Response},
};
use dioxus::server::{FullstackState, ServeConfig};
use hyle::FormErrors;

/// Axum extension for re-rendering Dioxus pages with [`FormErrors`] injected.
///
/// Register one instance as an Axum [`axum::Extension`] at startup, then
/// extract it in POST handlers to re-render the form page on validation failure.
///
/// # Example
/// ```rust,ignore
/// // server setup
/// .layer(Extension(HyleRenderer))
///
/// // post handler
/// async fn handle_create(
///     State(fs): State<FullstackState>,
///     Extension(renderer): Extension<HyleRenderer>,
///     Form(form): Form<IndexMap<String, String>>,
/// ) -> Response {
///     match validate(&form) {
///         Err(errors) => renderer.render_with_errors(fs, "/items/new", errors).await,
///         Ok(_)       => { /* persist */ Redirect::to("/").into_response() }
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct HyleRenderer;

impl HyleRenderer {
    /// Re-render `uri` with `errors` injected as a [`FormErrors`] context value.
    pub async fn render_with_errors(
        self,
        fullstack_state: FullstackState,
        uri: &str,
        errors: FormErrors,
    ) -> Response {
        let error_state = fullstack_state
            .with_config(ServeConfig::new().context_provider(move || errors.clone()));

        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .unwrap();

        FullstackState::render_handler(State(error_state), req)
            .await
            .into_response()
    }
}
