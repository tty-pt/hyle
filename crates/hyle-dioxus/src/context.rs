use std::sync::Arc;

use dioxus_hooks::{try_use_context, use_context};

use hyle::Blueprint;

use crate::types::HyleComponents;

/// Hyle configuration stored in Dioxus context.
///
/// Provide this at your root component:
/// ```rust,ignore
/// use_context_provider(|| HyleConfig { blueprint: Arc::new(my_blueprint()) });
/// ```
///
/// The blueprint is wrapped in `Arc` so that cloning the context value (which
/// Dioxus does on every `use_context` call) stays cheap.
#[derive(Clone)]
pub struct HyleConfig {
    pub blueprint: Arc<Blueprint>,
}

/// Read the `HyleConfig` from Dioxus context.
///
/// Panics if `HyleConfig` has not been provided by an ancestor component.
pub(crate) fn use_hyle_config() -> HyleConfig {
    use_context::<HyleConfig>()
}

/// Read `HyleComponents` from Dioxus context, if provided.
///
/// Returns `None` when no ancestor has called
/// `use_context_provider(|| HyleComponents { .. })`.
pub fn use_hyle_components() -> Option<HyleComponents> {
    try_use_context::<HyleComponents>()
}

/// Convenience re-export so callers don't need to import `use_context_provider` directly.
#[allow(unused_imports)]
pub use dioxus_hooks::use_context_provider;
