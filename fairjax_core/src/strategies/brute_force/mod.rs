use crate::{CaseHandler, GuardFn, Mapping, MatchedIds, MessageId, Store};
use itertools::Itertools;

#[derive(Clone)]
pub struct BruteForceMatcher<const C: usize, M> {
    message_ids: Vec<MessageId>,
    guard_fn: GuardFn<C, M>,
    permutations: Vec<Mapping<C>>,
}

impl<const C: usize, M> BruteForceMatcher<C, M> {
    pub fn new(guard_fn: GuardFn<C, M>) -> Self {
        let permutations = (0..C)
            .permutations(C)
            .map(|indices| {
                let mut arr = [0; C];
                for (i, v) in indices.into_iter().enumerate() {
                    arr[i] = v;
                }
                Mapping::new(arr)
            })
            .collect();

        Self {
            message_ids: Vec::new(),
            guard_fn,
            permutations,
        }
    }

    pub fn all_mappings(&self) -> &[Mapping<C>] {
        &self.permutations
    }

    pub fn try_all_slices_in_sorted_lex_order(&self, store: &Store<M>) -> Option<MatchedIds> {
        // Get all possible permutations as mappings
        let mappings = self.all_mappings();

        // Loop through all message sets of size C in sorted lexicographical order
        for message_set in self.message_ids.iter().combinations(C) {
            let message_ids: [MessageId; C] = std::array::from_fn(|i| *message_set[i]);

            // Fetch messages for message set
            let messages: [&M; C] = message_ids.map(|id| &store[&id]);

            // Try all mappings in order
            for mapping in mappings {
                if (self.guard_fn)(&messages, &mapping) {
                    return Some(MatchedIds::from(message_ids, mapping.clone()));
                }
            }
        }
        None
    }
}

impl<const C: usize, M> CaseHandler<M> for BruteForceMatcher<C, M> {
    fn consume(&mut self, id: MessageId, store: &Store<M>) -> Option<MatchedIds> {
        self.message_ids.push(id);
        self.try_all_slices_in_sorted_lex_order(store)
    }

    fn remove(&mut self, messages: &MatchedIds, _store: &Store<M>) {
        self.message_ids.retain(|id| !messages.contains(id));
    }

    fn is_empty(&self) -> bool {
        self.message_ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use std::cell::RefCell;

    #[derive(Debug, Clone)]
    struct Msg(usize);

    #[test]
    fn test_brute_force_matcher_pattern_size_3() {
        thread_local! {
            static TRIED_SLICES: RefCell<Vec<[usize; 3]>> = RefCell::new(Vec::new());
        }

        pub fn guard_fn(messages: &[&Msg; 3], mapping: &Mapping<3>) -> bool {
            TRIED_SLICES.with_borrow_mut(|ts| {
                ts.push([
                    messages[mapping.get(0)].0,
                    messages[mapping.get(1)].0,
                    messages[mapping.get(2)].0,
                ])
            });
            false
        }

        let mut store = Store::<Msg>::default();
        let mut matcher = BruteForceMatcher::new(guard_fn);

        store.insert(id!(1), Msg(1));
        store.insert(id!(2), Msg(2));
        store.insert(id!(3), Msg(3));

        matcher.consume(id!(1), &store);
        matcher.consume(id!(2), &store);
        matcher.consume(id!(3), &store);

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

        pub fn guard_fn(messages: &[&Msg; 2], mapping: &Mapping<2>) -> bool {
            TRIED_SLICES.with_borrow_mut(|ts| {
                ts.push([messages[mapping.get(0)].0, messages[mapping.get(1)].0])
            });
            false
        }

        let mut store = Store::<Msg>::default();
        let mut matcher = BruteForceMatcher::new(guard_fn);

        store.insert(id!(1), Msg(1));
        store.insert(id!(2), Msg(2));
        store.insert(id!(3), Msg(3));
        store.insert(id!(4), Msg(4));

        matcher.consume(id!(1), &store);
        matcher.consume(id!(2), &store);
        matcher.consume(id!(3), &store);
        matcher.consume(id!(4), &store);

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
