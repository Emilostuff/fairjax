use super::PartialMatch;
use crate::{GuardFn, Mapping, MatchedIds, MessageId, Store};

#[derive(Default)]
pub struct Node<const C: usize, P: PartialMatch<C, M> + Default, M> {
    group: P,
    children: Vec<Node<C, P, M>>,
    phantom: std::marker::PhantomData<M>,
}

impl<const C: usize, P: PartialMatch<C, M> + Default, M> Node<C, P, M> {
    pub fn new() -> Self {
        Self {
            group: P::default(),
            children: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn ramification(
        &mut self,
        message: &M,
        id: MessageId,
        store: &Store<M>,
        guard_fn: &GuardFn<C, M>,
        mappings: &[Mapping<C>],
    ) -> Option<MatchedIds> {
        if let Some(new_group) = self.group.extend(&message, id) {
            // Run ramification DFS style
            for child in &mut self.children {
                if let Some(result) = child.ramification(message, id, store, guard_fn, &mappings) {
                    return Some(result);
                }
            }

            // Check if match is complete (i.e. not partial)
            if new_group.is_complete() {
                let message_ids: [MessageId; C] = new_group.final_message_ids();

                let messages: [&M; C] = std::array::from_fn(|i| {
                    let id = message_ids[i];
                    &store[&id]
                });

                // Find fairest match that satisfies guard
                for mapping in mappings {
                    if guard_fn(&messages, &mapping) {
                        return Some(MatchedIds::from(message_ids, mapping.clone()));
                    }
                }
            } else {
                // Add child with partial match
                self.children.push(Node {
                    group: new_group,
                    children: Vec::new(),
                    phantom: std::marker::PhantomData,
                });
            }
        }
        None
    }

    pub fn remove(&mut self, messages: &MatchedIds) {
        self.children
            .retain(|node| node.group.has_common_id(messages));

        for child in &mut self.children {
            child.remove(messages);
        }
    }
}
