use crate::Store;
use crate::tree::Node;
use crate::{GuardFn, Message, MessageId, matchgroup::MatchGroup};

pub trait Pattern<M: Message> {
    fn consume(&mut self, message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>>;
    fn remove(&mut self, messages: &Vec<MessageId>);
}

pub struct PatternMatcher<G: MatchGroup<M>, M: Message> {
    tree: Node<G, M>,
    guard_fn: GuardFn<M>,
}

impl<G: MatchGroup<M>, M: Message> PatternMatcher<G, M> {
    pub fn new(guard_fn: GuardFn<M>) -> Self {
        Self {
            tree: Node::<G, M>::new(),
            guard_fn,
        }
    }
}

impl<G: MatchGroup<M>, M: Message> Pattern<M> for PatternMatcher<G, M> {
    fn consume(&mut self, message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>> {
        let response = self.tree.ramification(message, id, store, &self.guard_fn);
        println!();
        self.tree.print_tree(0);
        response
    }

    fn remove(&mut self, messages: &Vec<MessageId>) {
        self.tree.remove(messages)
    }
}
