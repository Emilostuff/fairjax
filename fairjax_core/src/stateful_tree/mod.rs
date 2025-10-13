pub mod permute;
pub mod tree;

use super::{CaseHandler, GuardFn, Message, MessageId, Store};
use permute::Element;
use std::fmt::Debug;
use tree::Node;

pub trait PartialMatch<M: Clone>: Sized + Debug + Default {
    fn extend(&self, message: &M, id: MessageId) -> Option<Self>;
    fn is_complete(&self) -> bool;
    fn message_ids(&self) -> Vec<MessageId>;
    fn to_elements(&self) -> Vec<Element>;
}

/// Matcher backend for a case/join pattern based on the stateful tree algorithm
pub struct StatefulTreeMatcher<P: PartialMatch<M>, M: Message> {
    tree: Node<P, M>,
    guard_fn: GuardFn<M>,
}

impl<P: PartialMatch<M>, M: Message> StatefulTreeMatcher<P, M> {
    pub fn new(guard_fn: GuardFn<M>) -> Self {
        Self {
            tree: Node::<P, M>::new(),
            guard_fn,
        }
    }
}

impl<P: PartialMatch<M>, M: Message> CaseHandler<M> for StatefulTreeMatcher<P, M> {
    fn consume(&mut self, message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>> {
        let response = self.tree.ramification(message, id, store, &self.guard_fn);
        response
    }

    fn remove(&mut self, messages: &Vec<MessageId>) {
        self.tree.remove(messages)
    }
}
