pub mod brute_force;
pub mod mailbox;
pub mod stateful_tree;

pub type MessageId = usize;

pub type Store<M> = std::collections::HashMap<MessageId, M>;

pub type GuardFn<M> = fn(&Vec<&M>) -> GuardEval;

pub trait Message: Clone + std::fmt::Debug {}

pub use mailbox::MailBox;

pub trait CaseHandler<M: Message> {
    fn consume(&mut self, message: &M, id: MessageId, store: &Store<M>) -> Option<Vec<MessageId>>;
    fn remove(&mut self, messages: &Vec<MessageId>);
}

#[derive(PartialEq)]
pub enum GuardEval {
    True,
    False,
    Mismatch,
}
