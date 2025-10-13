use super::{CaseHandler, GuardFn, Message, MessageId, Store};
use crate::GuardEval;

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

    pub fn try_all_combinations(&self, store: &Store<M>) -> Option<Vec<MessageId>> {
        fn recurse<'a, M: Message>(
            matcher: &'a BruteForceMatcher<M>,
            store: &'a Store<M>,
            slice: &mut Vec<MessageId>,
        ) -> Option<Vec<MessageId>> {
            if slice.len() == matcher.patterns_size {
                let message_refs: Vec<_> = slice.iter().map(|id| &store[id]).collect();
                if (matcher.guard_fn)(&message_refs) == GuardEval::True {
                    return Some(slice.clone());
                }
            }
            for i in 0..matcher.message_ids.len() {
                if !slice.contains(&matcher.message_ids[i]) {
                    slice.push(matcher.message_ids[i]);
                    if let Some(result) = recurse(matcher, store, slice) {
                        return Some(result);
                    }
                    slice.pop();
                }
            }
            None
        }

        let mut slice = Vec::with_capacity(self.patterns_size);
        recurse(&self, store, &mut slice)
    }
}

impl<M: Message> CaseHandler<M> for BruteForceMatcher<M> {
    fn consume(&mut self, _message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>> {
        self.message_ids.push(id);
        self.try_all_combinations(store)
    }

    fn remove(&mut self, messages: &Vec<MessageId>) {
        self.message_ids.retain(|id| !messages.contains(id));
    }
}
