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

enum ActiveMailBox {
    Mailbox0,
    Mailbox1,
    Mailbox2,
}

use ActiveMailBox::*;

struct MailBoxManager<T> {
    active_mailbox: RefCell<ActiveMailBox>,
    mailboxes: [RefCell<MailBox<T>>; 3],
    queue: RefCell<VecDeque<T>>,
}

impl<T> MailBoxManager<T> {
    pub fn new() -> Self {
        MailBoxManager {
            active_mailbox: RefCell::new(ActiveMailBox::Mailbox0),
            mailboxes: array::from_fn(|_| RefCell::new(MailBox::<T>::new())),
            queue: RefCell::new(VecDeque::new()),
        }
    }

    pub fn process_incoming(&self, message: T) {
        self.queue.borrow_mut().push_back(message);
    }

    pub fn next(&self) -> Option<T> {
        self.queue.borrow_mut().pop_front()
    }

    pub fn active_mailbox(&self) -> (ActiveMailBox, RefMut<MailBox<T>>) {
        match *self.active_mailbox.borrow() {
            Mailbox0 => (Mailbox0, self.mailboxes[0].borrow_mut()),
            Mailbox1 => (Mailbox1, self.mailboxes[1].borrow_mut()),
            Mailbox2 => (Mailbox2, self.mailboxes[2].borrow_mut()),
        }
    }

    pub fn switch_to(&self, target: ActiveMailBox, mut current: RefMut<MailBox<T>>) {
        // Extract messages and push to processing queue
        let previous_messages = current.extract();
        for m in previous_messages.into_iter().rev() {
            self.queue.borrow_mut().push_front(m);
        }

        // Set active mailbox:
        *self.active_mailbox.borrow_mut() = target;
    }
}

fn main() {
    let messages = vec![A, B, A, B, A, B, C];
    let manager = MailBoxManager::new();

    for incoming_msg in messages {
        manager.process_incoming(incoming_msg);

        // Extract from inner queue
        while let Some(msg) = manager.next() {
            match manager.active_mailbox() {
                (Mailbox0, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        C => {
                            println!("Received C, switch to inbox 1");

                            manager.switch_to(Mailbox1, mailbox);
                        }
                    });
                }
                (Mailbox1, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        A => {
                            println!("Received A, switch to inbox 2");

                            manager.switch_to(Mailbox2, mailbox);
                        }
                    });
                }
                (Mailbox2, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        B => {
                            println!("Received B, switch to inbox 1");

                            manager.switch_to(Mailbox1, mailbox);
                        }
                    });
                }
            }
        }
    }
}
