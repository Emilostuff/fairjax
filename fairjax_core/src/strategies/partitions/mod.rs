use crate::{AnyKeyBox, KeyFn};
use crate::{CaseHandler, MatchedIds, MessageId, Store};
use std::collections::BTreeMap;

/// Matcher backend for a case/join pattern based on the stateful tree algorithm
pub struct PartitionsMatcher<M, F: Fn() -> Box<dyn CaseHandler<M>>> {
    matcher_factory: F,
    partition_store: BTreeMap<AnyKeyBox, Box<dyn CaseHandler<M>>>,
    key_fn: KeyFn<M>,
}

impl<M, F: Fn() -> Box<dyn CaseHandler<M>>> PartitionsMatcher<M, F> {
    pub fn new(matcher_factory: F, key_extraction_fn: KeyFn<M>) -> Self {
        Self {
            matcher_factory,
            partition_store: BTreeMap::new(),
            key_fn: key_extraction_fn,
        }
    }
}

impl<M, F: Fn() -> Box<dyn CaseHandler<M>>> CaseHandler<M> for PartitionsMatcher<M, F> {
    fn consume(&mut self, id: MessageId, store: &Store<M>) -> Option<MatchedIds> {
        let message = &store[&id];
        if let Some(key) = (self.key_fn)(message) {
            self.partition_store
                .entry(key)
                .or_insert_with(|| (self.matcher_factory)())
                .consume(id, store)
        } else {
            None
        }
    }

    fn remove(&mut self, messages: &MatchedIds, store: &Store<M>) {
        for id in &messages.0 {
            let message = &store[&id];
            if let Some(key) = (self.key_fn)(message) {
                if let Some(handler) = self.partition_store.get_mut(&key) {
                    handler.remove(&messages, store);
                    if handler.is_empty() {
                        self.partition_store.remove(&key);
                    }
                }
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.partition_store.is_empty()
    }
}
