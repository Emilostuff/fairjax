use crate::Store;
use crate::tree::Node;
use crate::{BodyFn, GuardFn, Message, MessageId, matchgroup::MatchGroup};

pub trait Pattern<M: Message, R> {
    fn consume(&mut self, message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>>;
    fn execute(&self, messages: &Vec<MessageId>, store: &Store<M>) -> Option<R>;
    fn remove(&mut self, messages: &Vec<MessageId>);
}

pub struct PatternMatcher<G: MatchGroup<M>, M: Message, R> {
    tree: Node<G, M>,
    guard_fn: GuardFn<M>,
    body_fn: BodyFn<M, R>,
}

impl<G: MatchGroup<M>, M: Message, R> PatternMatcher<G, M, R> {
    pub fn new(guard_fn: GuardFn<M>, body_fn: BodyFn<M, R>) -> Self {
        Self {
            tree: Node::<G, M>::new(),
            guard_fn,
            body_fn,
        }
    }
}

impl<G: MatchGroup<M>, M: Message, R> Pattern<M, R> for PatternMatcher<G, M, R> {
    fn consume(&mut self, message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>> {
        let response = self.tree.ramification(message, id, store, &self.guard_fn);
        println!();
        self.tree.print_tree(0);
        response
    }

    fn execute(&self, messages: &Vec<MessageId>, store: &Store<M>) -> Option<R> {
        // Extract actual messages
        let matched_messages: Vec<_> = messages.iter().map(|id| &store[id]).collect();

        // Call body function and return response
        (self.body_fn)(&matched_messages)
    }

    fn remove(&mut self, messages: &Vec<MessageId>) {
        self.tree.remove(messages)
    }
}
