use fairjax::*;
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

fn main() {
    let messages = vec![A, B, A, B, A, B, C];

    fairjax_manager!(manager, Msg, 3);

    for incoming_msg in messages {
        manager.process_incoming(incoming_msg);

        // Extract from inner queue
        while let Some(msg) = manager.next() {
            match manager.active_matcher() {
                (ActiveMatcher::Matcher0, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        C => {
                            println!("Received C, switch to inbox 1");

                            manager.switch_to(ActiveMatcher::Matcher1, mailbox);
                        }
                    });
                }
                (ActiveMatcher::Matcher1, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        A => {
                            println!("Received A, switch to inbox 2");

                            manager.switch_to(ActiveMatcher::Matcher2, mailbox);
                        }
                    });
                }
                (ActiveMatcher::Matcher2, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        B => {
                            println!("Received B, switch to inbox 1");

                            manager.switch_to(ActiveMatcher::Matcher1, mailbox);
                        }
                    });
                }
            }
        }
    }
}
