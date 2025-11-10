pub mod permute;
pub mod tree;

use crate::{AcceptFn, MatchedIds};

use crate::{CaseHandler, GuardFn, Mapping, MessageId, Store};
use std::fmt::Debug;
use tree::Node;

pub trait PartialMatch<const C: usize, M>: Sized + Debug + Default {
    fn extend(&self, message: &M, id: MessageId) -> Option<Self>;
    fn is_complete(&self) -> bool;
    fn message_ids(&self) -> &[Option<MessageId>; C];
    fn final_message_ids(&self) -> [MessageId; C] {
        std::array::from_fn(|i| self.message_ids()[i].unwrap())
    }
    fn has_common_id(&self, matched_ids: &MatchedIds) -> bool {
        self.message_ids()
            .iter()
            .filter_map(|id| *id)
            .all(|id| !matched_ids.contains(&id))
    }
}

/// Matcher backend for a case/join pattern based on the stateful tree algorithm
pub struct StatefulTreeMatcher<const C: usize, P: PartialMatch<C, M>, M> {
    tree: Node<C, P, M>,
    guard_fn: GuardFn<C, M>,
    accept_fn: AcceptFn<M>,
    mappings: &'static [Mapping<C>],
}

impl<const C: usize, P: PartialMatch<C, M>, M> StatefulTreeMatcher<C, P, M> {
    pub fn new(
        guard_fn: GuardFn<C, M>,
        accept_fn: AcceptFn<M>,
        mappings: &'static [Mapping<C>],
    ) -> Self {
        Self {
            tree: Node::<C, P, M>::new(),
            guard_fn,
            accept_fn,
            mappings,
        }
    }
}

impl<const C: usize, P: PartialMatch<C, M>, M> CaseHandler<M> for StatefulTreeMatcher<C, P, M> {
    fn consume(&mut self, id: MessageId, store: &Store<M>) -> Option<MatchedIds> {
        let message = &store[&id];
        if (self.accept_fn)(message) {
            return self
                .tree
                .ramification(message, id, store, &self.guard_fn, self.mappings);
        }
        None
    }

    fn remove(&mut self, messages: &MatchedIds, _store: &Store<M>) {
        self.tree.remove(messages)
    }
}
