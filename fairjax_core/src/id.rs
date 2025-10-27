#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
/// ID to uniquely identify messages, and determine their age (lower is older)
pub struct MessageId(usize);

impl MessageId {
    #[cfg(test)]
    /// Create new MessageId with specific `id` value
    pub fn new(id: usize) -> Self {
        MessageId(id)
    }

    #[inline]
    /// Create a MessageId for padding containing the maximum possible id value
    pub fn max() -> Self {
        MessageId(usize::MAX)
    }
}

pub struct MessageIdFactory {
    id_counter: usize,
}

impl MessageIdFactory {
    pub fn new() -> Self {
        MessageIdFactory { id_counter: 0 }
    }

    pub fn next(&mut self) -> MessageId {
        self.id_counter += 1;
        MessageId(self.id_counter)
    }
}

/// ID to uniquely identify a case
#[derive(Debug, PartialEq, Eq)]
pub struct CaseId(pub usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_new_and_equality() {
        let id1 = MessageId::new(5);
        let id2 = MessageId::new(5);
        let id3 = MessageId::new(10);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_message_id_ordering() {
        let older = MessageId::new(1);
        let newer = MessageId::new(2);
        assert!(older < newer);
        assert!(newer > older);
    }

    #[test]
    fn test_message_id_max() {
        let max_id = MessageId::max();
        assert_eq!(max_id, MessageId(usize::MAX));
    }

    #[test]
    fn test_message_id_factory() {
        let mut factory = MessageIdFactory::new();
        let id1 = factory.next();
        let id2 = factory.next();
        assert_eq!(id1, MessageId(1));
        assert_eq!(id2, MessageId(2));
    }
}
