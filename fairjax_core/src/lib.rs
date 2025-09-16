pub mod mailbox;
pub mod matchgroup;
pub mod pattern;
pub mod permute;
pub mod tree;

pub type MessageId = usize;

pub type Store<M> = std::collections::HashMap<MessageId, M>;

pub type GuardFn<M> = fn(&Vec<&M>) -> bool;

pub trait Message: Clone + std::fmt::Debug {}

pub use mailbox::MailBox;
