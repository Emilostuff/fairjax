use fairjax::*;

/// Define message enum
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A,
    B,
    C,
}

use Msg::*;

// Delare FairjaxManager struct (with 3 internal mailboxes) and the Matcer enum
fairjax_switch!(3);

fn main() {
    // Demo messages to process
    let messages = vec![A, B, A, B, A, B, C];

    // Construct the fairjax manager
    let manager = FairjaxManager::new();

    // Process each of the incoming messages
    for incoming_msg in messages {
        manager.process_incoming(incoming_msg);

        // Extract message(s) to process from inner queue
        while let Some(msg) = manager.next() {
            // Route message to appropriate matcher
            match manager.active_matcher() {
                (Matcher::Fairjax0, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        C => {
                            println!("Received C, switching to inbox 1");

                            // Switch to matcher 1. Specify the mailbox to extract messages from
                            // when replaying messages into the new mailbox
                            manager.switch_to(Matcher::Fairjax1, mailbox);
                        }
                    });
                }
                (Matcher::Fairjax1, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        A => {
                            println!("Received A, switching to inbox 2");

                            // Switch to matcher 2. Specify the mailbox to extract messages from
                            // when replaying messages into the new mailbox
                            manager.switch_to(Matcher::Fairjax2, mailbox);
                        }
                    });
                }
                (Matcher::Fairjax2, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        B => {
                            println!("Received B, switching to inbox 1");

                            // Switch to matcher 1. Specify the mailbox to extract messages from
                            // when replaying messages into the new mailbox
                            manager.switch_to(Matcher::Fairjax1, mailbox);
                        }
                    });
                }
            }
        }
    }
}
