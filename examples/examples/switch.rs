use fairjax::*;
use fairjax_core::MailBox;
use std::{
    array,
    cell::{RefCell, RefMut},
    collections::VecDeque,
};

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A,
    B,
    C,
}

use Msg::*;

enum ActiveMatcher {
    Matcher0,
    Matcher1,
    Matcher2,
}

use ActiveMatcher::*;

struct FairjaxManager<T> {
    active_matcher: RefCell<ActiveMatcher>,
    mailboxes: [RefCell<MailBox<T>>; 3],
    queue: RefCell<VecDeque<T>>,
}

impl<T> FairjaxManager<T> {
    pub fn new() -> Self {
        FairjaxManager {
            active_matcher: RefCell::new(ActiveMatcher::Matcher0),
            mailboxes: array::from_fn(|_| RefCell::new(MailBox::<T>::default())),
            queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn process_incoming(&self, message: T) {
        self.queue.borrow_mut().push_back(message);
    }

    pub fn next(&self) -> Option<T> {
        self.queue.borrow_mut().pop_front()
    }

    pub fn active_matcher<'a>(&'a self) -> (ActiveMatcher, RefMut<'a, MailBox<T>>) {
        match *self.active_matcher.borrow() {
            Matcher0 => (Matcher0, self.mailboxes[0].borrow_mut()),
            Matcher1 => (Matcher1, self.mailboxes[1].borrow_mut()),
            Matcher2 => (Matcher2, self.mailboxes[2].borrow_mut()),
        }
    }

    pub fn switch_to(&self, target: ActiveMatcher, mut current: RefMut<MailBox<T>>) {
        // Extract messages and push to processing queue
        let previous_messages = current.extract();
        for m in previous_messages.into_iter().rev() {
            self.queue.borrow_mut().push_front(m);
        }

        // Set active matcher:
        *self.active_matcher.borrow_mut() = target;
    }
}

fn main() {
    let messages = vec![A, B, A, B, A, B, C];
    let manager = FairjaxManager::new();

    for incoming_msg in messages {
        manager.process_incoming(incoming_msg);

        // Extract from inner queue
        while let Some(msg) = manager.next() {
            match manager.active_matcher() {
                (Matcher0, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        C => {
                            println!("Received C, switch to inbox 1");

                            manager.switch_to(Matcher1, mailbox);
                        }
                    });
                }
                (Matcher1, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        A => {
                            println!("Received A, switch to inbox 2");

                            manager.switch_to(Matcher2, mailbox);
                        }
                    });
                }
                (Matcher2, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        B => {
                            println!("Received B, switch to inbox 1");

                            manager.switch_to(Matcher1, mailbox);
                        }
                    });
                }
            }
        }
    }
}
