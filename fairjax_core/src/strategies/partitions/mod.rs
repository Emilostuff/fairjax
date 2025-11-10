use crate::{AnyKeyBox, KeyFn};
use crate::{CaseHandler, MatchedIds, MessageId, Store};
use std::collections::BTreeMap;

/// Matcher backend for a case/join pattern based on the stateful tree algorithm
pub struct PartitionsMatcher<M> {
    matcher_factory: Box<dyn Fn() -> Box<dyn CaseHandler<M>>>,
    partition_store: BTreeMap<AnyKeyBox, Box<dyn CaseHandler<M>>>,
    key_fn: KeyFn<M>,
}

impl<M> PartitionsMatcher<M> {
    pub fn new(
        matcher_factory: Box<dyn Fn() -> Box<dyn CaseHandler<M>>>,
        key_extraction_fn: fn(&M) -> AnyKeyBox,
    ) -> Self {
        Self {
            matcher_factory,
            partition_store: BTreeMap::new(),
            key_fn: key_extraction_fn,
        }
    }
}

impl<M> CaseHandler<M> for PartitionsMatcher<M> {
    fn consume(&mut self, id: MessageId, store: &Store<M>) -> Option<MatchedIds> {
        let message = &store[&id];
        let key = (self.key_fn)(message);

        self.partition_store
            .entry(key)
            .or_insert_with(|| (self.matcher_factory)())
            .consume(id, store)
    }

    fn remove(&mut self, messages: &MatchedIds, store: &Store<M>) {
        for id in &messages.0 {
            let message = &store[&id];
            let key = (self.key_fn)(message);

            if let Some(handler) = self.partition_store.get_mut(&key) {
                handler.remove(&messages, store);
            }
        }
    }
}
