use std::{
    future::Future,
    hash::{Hash, Hasher},
    pin::Pin,
    rc::Rc,
    sync::Arc,
};

use dioxus::prelude::*;
use dioxus_fullstack_core::{use_server_future, ServerFnError};
use dioxus_query::prelude::*;
use dioxus_signals::{Signal, WritableExt};

use crate::{HyleAdapter, HyleMutation, HyleSourceState, UseSource};
use hyle::{MutateInput, Source};

pub type InvalidationSignal = Signal<u32>;

type MutateFn =
    Arc<dyn Fn(MutateInput) -> Pin<Box<dyn Future<Output = Result<(), String>>>> + 'static>;

#[derive(Clone)]
struct HyleMutationQuery {
    mutate: MutateFn,
    on_success: Option<Rc<dyn Fn()>>,
}

impl PartialEq for HyleMutationQuery {
    fn eq(&self, _other: &Self) -> bool { true }
}
impl Eq for HyleMutationQuery {}
impl Hash for HyleMutationQuery {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl MutationCapability for HyleMutationQuery {
    type Ok = ();
    type Err = String;
    type Keys = String;

    async fn run(&self, input_json: &Self::Keys) -> Result<Self::Ok, Self::Err> {
        let input: MutateInput =
            serde_json::from_str(input_json).map_err(|e| e.to_string())?;
        (self.mutate)(input).await
    }

    async fn on_settled(&self, _input_json: &Self::Keys, result: &Result<Self::Ok, Self::Err>) {
        if result.is_ok() {
            if let Some(cb) = &self.on_success {
                cb();
            }
        }
    }
}

#[derive(Default)]
pub struct DioxusMutationOptions {
    pub on_success: Option<Rc<dyn Fn()>>,
}

pub fn use_dioxus_mutation<F, Fut>(
    mutate_fn: F,
    options: DioxusMutationOptions,
) -> HyleMutation
where
    F: Fn(MutateInput) -> Fut + 'static + Clone,
    Fut: Future<Output = Result<(), String>> + 'static,
{
    let mutate: MutateFn = Arc::new(move |input| Box::pin(mutate_fn(input)));
    let on_success = options.on_success;

    let capability = HyleMutationQuery {
        mutate: mutate.clone(),
        on_success: on_success.clone(),
    };
    let dq_mutation = use_mutation(Mutation::new(capability));

    let mut is_pending = use_signal(|| false);
    let mut is_success = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let mutate_cb = Callback::new(move |input: MutateInput| {
        is_pending.set(true);
        is_success.set(false);
        error.set(None);
        spawn(async move {
            let input_json = serde_json::to_string(&input).unwrap_or_default();
            dq_mutation.mutate_async(input_json).await;
            let reader = dq_mutation.peek();
            let state = reader.state();
            is_pending.set(false);
            match &*state {
                MutationStateData::Settled { res: Ok(()), .. } => {
                    is_success.set(true);
                    if let Some(mut inv) = try_consume_context::<InvalidationSignal>() {
                        *inv.write() += 1;
                    }
                }
                MutationStateData::Settled { res: Err(e), .. } => {
                    error.set(Some(e.clone()));
                }
                _ => {}
            }
        });
    });

    let reset_cb = Callback::new(move |_: ()| {
        is_pending.set(false);
        is_success.set(false);
        error.set(None);
    });

    HyleMutation {
        mutate: mutate_cb,
        reset: reset_cb,
        is_pending,
        is_success,
        error,
    }
}

pub fn use_fullstack_source<F, Fut>(fetch_fn: F) -> UseSource
where
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<Source, ServerFnError>> + 'static,
{
    let invalidation = try_consume_context::<InvalidationSignal>();

    let future = use_server_future(move || {
        if let Some(inv) = invalidation {
            let _ = inv.read();
        }
        fetch_fn()
    });

    use_memo(move || match &future {
        Ok(f) => match &*f.read() {
            Some(Ok(src))  => HyleSourceState::Ready(src.clone()),
            Some(Err(e))   => HyleSourceState::Error(e.to_string()),
            None           => HyleSourceState::Loading,
        },
        Err(_suspended) => HyleSourceState::Loading,
    }).into()
}

pub fn make_fullstack_adapter<SF, SFut, CF, CFut, UF, UFut, DF, DFut>(
    source_fn: SF,
    create_fn: CF,
    update_fn: UF,
    delete_fn: DF,
) -> HyleAdapter
where
    SF: Fn() -> SFut + Clone + 'static,
    SFut: Future<Output = Result<Source, ServerFnError>> + 'static,
    CF: Fn(MutateInput) -> CFut + Clone + 'static,
    CFut: Future<Output = Result<(), String>> + 'static,
    UF: Fn(MutateInput) -> UFut + Clone + 'static,
    UFut: Future<Output = Result<(), String>> + 'static,
    DF: Fn(MutateInput) -> DFut + Clone + 'static,
    DFut: Future<Output = Result<(), String>> + 'static,
{
    let source = use_fullstack_source(source_fn);
    let create = use_dioxus_mutation(create_fn, DioxusMutationOptions::default());
    let update = use_dioxus_mutation(update_fn, DioxusMutationOptions::default());
    let delete = use_dioxus_mutation(delete_fn, DioxusMutationOptions::default());
    HyleAdapter { source, create, update, delete }
}