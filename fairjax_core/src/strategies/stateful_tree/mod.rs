pub mod permute;
pub mod tree;

use crate::MatchedIds;

use crate::{CaseHandler, GuardFn, MessageId, Store};
use permute::Element;
use std::fmt::Debug;
use tree::Node;

pub trait PartialMatch<const C: usize, M>: Sized + Debug + Default {
    fn extend(&self, message: &M, id: MessageId) -> Option<Self>;
    fn is_complete(&self) -> bool;
    fn message_ids(&self) -> &[Option<MessageId>; C];
    fn to_elements(&self) -> [Element; C];
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
}

impl<const C: usize, P: PartialMatch<C, M>, M> StatefulTreeMatcher<C, P, M> {
    pub fn new(guard_fn: GuardFn<C, M>) -> Self {
        Self {
            tree: Node::<C, P, M>::new(),
            guard_fn,
        }
    }
}

impl<const C: usize, P: PartialMatch<C, M>, M> CaseHandler<M> for StatefulTreeMatcher<C, P, M> {
    fn consume(&mut self, id: MessageId, store: &Store<M>) -> Option<MatchedIds> {
        let message = &store[&id];
        self.tree.ramification(message, id, store, &self.guard_fn)
    }

    fn remove(&mut self, messages: &MatchedIds) {
        self.tree.remove(messages)
    }
}
