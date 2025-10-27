use crate::*;
use std::collections::HashMap;

pub struct MailBox<M> {
    store: Store<M>,
    init: bool,
    id_factory: MessageIdFactory,
    cases: Vec<Box<dyn CaseHandler<M>>>,
}

impl<M> MailBox<M> {
    pub fn new() -> Self {
        MailBox {
            store: HashMap::new(),
            init: false,
            id_factory: MessageIdFactory::new(),
            cases: Vec::new(),
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.init
    }

    pub fn is_modified(&self) -> bool {
        !self.cases.is_empty() || !self.store.is_empty()
    }

    pub fn init(&mut self) {
        self.init = true;
    }

    pub fn add_case(&mut self, case: Box<dyn CaseHandler<M>>) {
        if !self.init {
            self.cases.push(case);
        } else {
            panic!("Mailbox must not be modifed");
        }
    }

    fn remove_message_ids_from_cases(&mut self, messages: &MatchedIds) {
        for case in &mut self.cases {
            case.remove(messages);
        }
    }

    pub fn process(&mut self, message: M) -> Option<MatchedMessages<M>> {
        // Generate new id for incoming message
        let message_id = self.id_factory.next();

        // Store message
        self.store.insert(message_id, message);

        // Compute fairest match for each case (if possible)
        let matches: Vec<CaseMatch> = self
            .cases
            .iter_mut()
            .map(|case| case.consume(message_id, &self.store))
            .enumerate()
            .filter_map(|e| match e {
                (id, Some(matched_ids)) => Some(CaseMatch::new(CaseId(id), matched_ids)),
                _ => None,
            })
            .collect();

        let fairest_match = CaseMatch::get_fairest(matches)?;

        self.remove_message_ids_from_cases(&fairest_match.messages);

        Some(fairest_match.to_messages(&mut self.store))
    }

    pub fn unmatched_messages(&self) -> Vec<MessageId> {
        self.store.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal mock implementations for required traits and types
    type TestMessage = String;

    // Mock cases
    struct MockCase0;
    impl CaseHandler<TestMessage> for MockCase0 {
        fn consume(&mut self, id: MessageId, _store: &Store<TestMessage>) -> Option<MatchedIds> {
            if id == id!(6) {
                Some(MatchedIds::new(vec![id!(5), id!(6), id!(2)]))
            } else {
                None
            }
        }
        fn remove(&mut self, _messages: &MatchedIds) {}
    }

    struct MockCase1;
    impl CaseHandler<TestMessage> for MockCase1 {
        fn consume(&mut self, id: MessageId, _store: &Store<TestMessage>) -> Option<MatchedIds> {
            if id == id!(6) {
                Some(MatchedIds::new(vec![id!(6), id!(4), id!(2), id!(5)]))
            } else {
                None
            }
        }
        fn remove(&mut self, _messages: &MatchedIds) {}
    }

    #[test]
    fn test_mailbox_init() {
        // Init mailbox
        let mut mailbox = MailBox::<TestMessage>::new();
        assert!(!mailbox.is_initialized());
        assert!(!mailbox.is_modified());

        // Add case and check that it recognizes that it is modified
        mailbox.add_case(Box::new(MockCase0));
        assert!(mailbox.is_modified());

        // Add the other case and finalize init
        mailbox.add_case(Box::new(MockCase1));
        mailbox.init();
        assert!(mailbox.is_initialized());
    }

    #[test]
    fn test_mailbox_process() {
        // Init mailbox with two mock cases
        let mut mailbox = MailBox::<TestMessage>::new();
        mailbox.add_case(Box::new(MockCase0));
        mailbox.add_case(Box::new(MockCase1));
        mailbox.init();

        // process messages 1-9
        for msg in ["1", "2", "3", "4", "5"] {
            assert!(mailbox.process(msg.into()).is_none());
        }

        // process message 10
        let result = mailbox.process("6".into()).unwrap();

        // Check that the correct match is found
        assert_eq!(&CaseId(1), result.case_id());
        assert_eq!(
            (
                String::from("6"),
                String::from("4"),
                String::from("2"),
                String::from("5")
            ),
            result.into_4_tuple()
        );

        // Check that exactly the matched messages are removed from store
        let remaining = mailbox.unmatched_messages();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.contains(&id!(1)));
        assert!(!remaining.contains(&id!(2)));
        assert!(remaining.contains(&id!(3)));
        assert!(!remaining.contains(&id!(4)));
        assert!(!remaining.contains(&id!(5)));
        assert!(!remaining.contains(&id!(6)));
    }
}
