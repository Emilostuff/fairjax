pub mod definition;
pub mod matchgroup;
pub mod message;
pub mod pattern;
pub mod permute;
pub mod tree;

pub type MessageId = usize;

pub type Store<M> = std::collections::HashMap<MessageId, M>;

pub type GuardFn<M> = Box<dyn Fn(&Vec<&M>) -> bool>;
pub type BodyFn<M> = Box<dyn Fn(&Vec<&M>) -> Option<Response>>;

pub enum Response {
    Continue,
    Stop,
}
