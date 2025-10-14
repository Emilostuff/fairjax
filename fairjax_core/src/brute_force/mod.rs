use super::{CaseHandler, GuardFn, Message, MessageId, Store};
use crate::GuardEval;
use itertools::Itertools;

pub struct BruteForceMatcher<M: Message> {
    message_ids: Vec<MessageId>,
    patterns_size: usize,
    guard_fn: GuardFn<M>,
}

impl<M: Message> BruteForceMatcher<M> {
    pub fn new(guard_fn: GuardFn<M>, patterns_size: usize) -> Self {
        Self {
            message_ids: Vec::new(),
            patterns_size,
            guard_fn,
        }
    }

    pub fn try_all_slices_in_sorted_lex_order(&self, store: &Store<M>) -> Option<Vec<MessageId>> {
        for message_set in self.message_ids.iter().combinations(self.patterns_size) {
            for slice in message_set.iter().permutations(self.patterns_size) {
                let message_refs: Vec<_> = slice.iter().map(|id| &store[id]).collect();
                if (self.guard_fn)(&message_refs) == GuardEval::True {
                    return Some(slice.iter().map(|&&&i| i).collect());
                }
            }
        }
        None
    }
}

impl<M: Message> CaseHandler<M> for BruteForceMatcher<M> {
    fn consume(&mut self, _message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>> {
        self.message_ids.push(id);
        self.try_all_slices_in_sorted_lex_order(store)
    }

    fn remove(&mut self, messages: &Vec<MessageId>) {
        self.message_ids.retain(|id| !messages.contains(id));
    }
}
