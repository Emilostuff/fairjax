use super::PartialMatch;
use super::permute::Permutations;
use crate::{GuardEval, GuardFn, MessageId, Store};
use std::fmt::Debug;

#[derive(Default)]
pub struct Node<P: PartialMatch<M> + Debug + Default, M: Clone> {
    match_group: P,
    children: Vec<Node<P, M>>,
    phantom: std::marker::PhantomData<M>, // Added PhantomData to use type parameter U
}

impl<P: PartialMatch<M> + Debug + Default, M: Clone> Node<P, M> {
    pub fn new() -> Self {
        Self {
            match_group: P::default(),
            children: Vec::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn ramification(
        &mut self,
        message: &M,
        id: MessageId,
        store: &Store<M>,
        guard_fn: &GuardFn<M>,
    ) -> Option<Vec<MessageId>> {
        if let Some(new_group) = self.match_group.extend(&message, id) {
            // Run ramification DFS style
            for child in &mut self.children {
                if let Some(result) = child.ramification(message, id, store, guard_fn) {
                    return Some(result);
                }
            }

            // Check if match is complete (i.e. not partial)
            if new_group.is_complete() {
                // Create permutation elements and get all message permutations
                let permutations = Permutations::get_permutations(new_group.to_elements());

                // Find fairest match that satisfies guard
                for permutation in permutations {
                    let message_refs: Vec<_> = permutation.iter().map(|id| &store[id]).collect();

                    match guard_fn(&message_refs) {
                        GuardEval::True => return Some(permutation),
                        GuardEval::False => continue,
                        GuardEval::Mismatch => unreachable!(),
                    }
                }
            } else {
                // Add child with partial match
                self.children.push(Node {
                    match_group: new_group,
                    children: Vec::new(),
                    phantom: std::marker::PhantomData,
                });
            }
        }
        None
    }

    pub fn remove(&mut self, messages: &Vec<MessageId>) {
        self.children.retain({
            |node| {
                node.match_group
                    .message_ids()
                    .iter()
                    .all(|id| !messages.contains(id))
            }
        });
        for child in &mut self.children {
            child.remove(messages);
        }
    }

    pub fn print_tree(&self, indent: usize) {
        println!("{:indent$}{:?}", "", self.match_group);
        for child in &self.children {
            child.print_tree(indent + 4);
        }
    }
}
