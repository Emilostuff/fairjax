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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GuardEval, Message, Store};
    use std::cell::RefCell;

    #[derive(Debug, Clone)]
    struct Msg(usize);

    impl Message for Msg {}

    #[test]
    fn test_brute_force_matcher_pattern_size_3() {
        thread_local! {
            static TRIED_SLICES: RefCell<Vec<[usize; 3]>> = RefCell::new(Vec::new());
        }

        pub fn guard_fn(_messages: &Vec<&Msg>) -> GuardEval {
            TRIED_SLICES
                .with_borrow_mut(|ts| ts.push([_messages[0].0, _messages[1].0, _messages[2].0]));
            GuardEval::False
        }

        let mut store = Store::<Msg>::default();
        let mut matcher = BruteForceMatcher::new(guard_fn, 3);

        store.insert(1, Msg(1));
        store.insert(2, Msg(2));
        store.insert(3, Msg(3));

        matcher.consume(&Msg(1), 1, &store);
        matcher.consume(&Msg(2), 2, &store);
        matcher.consume(&Msg(3), 3, &store);

        // define expected order for evaluation of slices
        let expected: Vec<[usize; 3]> = vec![
            // slex: [1, 2, 3]
            [1, 2, 3],
            [1, 3, 2],
            [2, 1, 3],
            [2, 3, 1],
            [3, 1, 2],
            [3, 2, 1],
        ];

        TRIED_SLICES.with_borrow(|ts| assert_eq!(&expected, ts))
    }

    #[test]
    fn test_brute_force_matcher_pattern_size_2() {
        thread_local! {
            static TRIED_SLICES: RefCell<Vec<[usize; 2]>> = RefCell::new(Vec::new());
        }

        pub fn guard_fn(_messages: &Vec<&Msg>) -> GuardEval {
            TRIED_SLICES.with_borrow_mut(|ts| ts.push([_messages[0].0, _messages[1].0]));
            GuardEval::False
        }

        let mut store = Store::<Msg>::default();
        let mut matcher = BruteForceMatcher::new(guard_fn, 2);

        store.insert(1, Msg(1));
        store.insert(2, Msg(2));
        store.insert(3, Msg(3));
        store.insert(4, Msg(4));

        matcher.consume(&Msg(1), 1, &store);
        matcher.consume(&Msg(2), 2, &store);
        matcher.consume(&Msg(3), 3, &store);
        matcher.consume(&Msg(4), 4, &store);

        // define expected order for evaluation of slices
        let expected: Vec<[usize; 2]> = vec![
            // ROUND 1: Mailbox = [Msg(1)]
            // ROUND 2: Mailbox = [Msg(1), Msg(2)]
            // slex group: [1, 2]
            [1, 2],
            [2, 1],
            // ROUND 3: Mailbox = [Msg(1), Msg(2), Msg(3)]:
            // slex group: [1, 2]
            [1, 2],
            [2, 1],
            // slex group: [1, 3]
            [1, 3],
            [3, 1],
            // slex group: [2, 3]
            [2, 3],
            [3, 2],
            // ROUND 4: Mailbox = [Msg(1), Msg(2), Msg(3), Msg(4)]:
            // slex group: [1, 2]
            [1, 2],
            [2, 1],
            // slex group: [1, 3]
            [1, 3],
            [3, 1],
            // slex group: [1, 4]
            [1, 4],
            [4, 1],
            // slex group: [2, 3]
            [2, 3],
            [3, 2],
            // slex group: [2, 4]
            [2, 4],
            [4, 2],
            // slex group: [3, 4]
            [3, 4],
            [4, 3],
        ];
        TRIED_SLICES.with_borrow(|ts| assert_eq!(&expected, ts))
    }
}
