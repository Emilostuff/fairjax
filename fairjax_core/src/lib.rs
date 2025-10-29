pub mod id;
pub mod mailbox;
pub mod mapping;
pub mod strategies {
    pub mod brute_force;
    pub mod stateful_tree;
}

pub mod matching {
    pub mod case_match;
    pub mod matched_ids;
    pub mod matched_messages;
}

pub use id::{CaseId, MessageId, MessageIdFactory};
pub use mailbox::MailBox;
pub use mapping::Mapping;

pub use matching::case_match::CaseMatch;
pub use matching::matched_ids::{MatchedIds, MatchedIdsSorted};
pub use matching::matched_messages::MatchedMessages;

/// A store for received unmatched messages of type M identified by a unique MessageId
pub type Store<M> = std::collections::HashMap<MessageId, M>;

/// Guard function, for a case with C messages of type M in its pattern.
pub type GuardFn<const C: usize, M> = fn(&[&M; C], &Mapping<C>) -> bool;

/// Top interface for interacting with a case on messages of type M.
/// K is the maximum pattern size across all cases.
pub trait CaseHandler<M> {
    fn consume(&mut self, id: MessageId, store: &Store<M>) -> Option<MatchedIds>;
    fn remove(&mut self, messages: &MatchedIds);
}

#[cfg(test)]
#[macro_export]
macro_rules! id {
    ($id:expr) => {
        $crate::MessageId::new($id)
    };
}
