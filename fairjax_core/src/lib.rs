pub mod mailbox;
pub mod matchgroup;
pub mod pattern;
pub mod permute;
pub mod tree;

pub type MessageId = usize;

pub type Store<M> = std::collections::HashMap<MessageId, M>;

pub type GuardFn<M> = Box<dyn Fn(&Vec<&M>) -> bool>;
pub type BodyFn<M, R> = Box<dyn Fn(&Vec<&M>) -> Option<R>>;

pub trait Message: Clone + std::fmt::Debug {}
