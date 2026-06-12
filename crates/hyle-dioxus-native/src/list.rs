use dioxus::prelude::*;
use dioxus_signals::ReadSignal;
use indexmap::IndexMap;

use hyle::{ModelResult, Row, RowItem, Source};
use hyle_dioxus::{
    DioxusMutationOptions, HyleAdapter, HyleSourceState,
    use_dioxus_mutation,
};

pub fn items_to_source(model: &str, items: &[RowItem]) -> Source {
    let rows: Vec<Row> = items.iter().map(|item| item.into_row()).collect();
    let mut source: Source = IndexMap::new();
    source.insert(model.to_owned(), ModelResult::many(rows));
    source
}

pub fn use_static_adapter(source: Source) -> HyleAdapter {
    let source_signal: ReadSignal<HyleSourceState> =
        use_memo(move || HyleSourceState::Ready(source.clone())).into();
    let noop = || {
        use_dioxus_mutation(
            |_| async { Ok::<(), String>(()) },
            DioxusMutationOptions::default(),
        )
    };
    HyleAdapter {
        source: source_signal,
        create: noop(),
        update: noop(),
        delete: noop(),
    }
}


