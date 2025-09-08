use crate::permute::Element;

use super::*;
use std::fmt::Debug;

pub trait MatchGroup<M: Clone>: Sized + Debug + Default {
    fn extend(&self, message: &M, id: MessageId) -> Option<Self>;
    fn is_complete(&self) -> bool;
    fn message_ids(&self) -> Vec<MessageId>;
    fn to_elements(&self) -> Vec<Element>;
}
