use crate::{CaseHandler, Message, MessageId, Store};
use std::collections::HashMap;

pub struct MailBox<M: Message> {
    store: Store<M>,
    init: bool,
    id_counter: usize,
    cases: Vec<Box<dyn CaseHandler<M>>>,
}

impl<M: Message> MailBox<M> {
    pub fn new() -> Self {
        MailBox {
            store: HashMap::new(),
            init: false,
            id_counter: 0,
            cases: Vec::new(),
        }
    }

    fn create_id(&mut self) -> MessageId {
        self.id_counter += 1;
        self.id_counter
    }

    fn get_fairest_match(results: &Vec<Option<Vec<MessageId>>>) -> Option<usize> {
        // find size of longest vec
        let mut matches = results
            .iter()
            .enumerate()
            .filter_map(|e| match e {
                (i, Some(v)) => Some((i, v.clone())),
                _ => None,
            })
            .collect::<Vec<_>>();

        let max_len = matches.iter().map(|(_, v)| v.len()).max()?;

        // sort and pad all vectors to max length
        for (_, v) in matches.iter_mut() {
            v.sort();
            while v.len() < max_len {
                v.push(usize::MAX);
            }
        }

        matches.sort_by(|a, b| a.1.cmp(&b.1));
        matches.first().map(|e| e.0)
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

    pub fn consume(&mut self, message: M) -> Option<(usize, Vec<M>)> {
        // Generate new id for incoming message
        let id = self.create_id();

        // Store message
        self.store.insert(id, message.clone());

        // Compute fairest match for each case
        let matches: Vec<_> = self
            .cases
            .iter_mut()
            .map(|p| p.consume(&message, id, &self.store))
            .collect();

        // Find fairest match across cases
        let fairest_match = Self::get_fairest_match(&matches)?;
        let matched_message_ids = matches[fairest_match].as_ref().unwrap();

        // Extract actual messages
        let matched_messages: Vec<M> = matched_message_ids
            .iter()
            .map(|id| self.store[id].clone())
            .collect();

        // Remove messages from store
        for id in matched_message_ids {
            self.store.remove(&id);
        }

        // remove messages from each case
        for case in &mut self.cases {
            case.remove(&matched_message_ids);
        }

        Some((fairest_match, matched_messages))
    }
}
