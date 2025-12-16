use fairjax::*;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A,
    B,
    C,
}

use Msg::*;

fairjax_switch!(3);

fn main() {
    let messages = vec![A, B, A, B, A, B, C];

    let manager = FairjaxManager::new();

    for incoming_msg in messages {
        manager.process_incoming(incoming_msg);

        // Extract from inner queue
        while let Some(msg) = manager.next() {
            match manager.active_matcher() {
                (Matcher::Fairjax0, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        C => {
                            println!("Received C, switch to inbox 1");

                            manager.switch_to(Matcher::Fairjax1, mailbox);
                        }
                    });
                }
                (Matcher::Fairjax1, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        A => {
                            println!("Received A, switch to inbox 2");

                            manager.switch_to(Matcher::Fairjax2, mailbox);
                        }
                    });
                }
                (Matcher::Fairjax2, mut mailbox) => {
                    fairjax!(match msg >> [mailbox, Msg] {
                        B => {
                            println!("Received B, switch to inbox 1");

                            manager.switch_to(Matcher::Fairjax1, mailbox);
                        }
                    });
                }
            }
        }
    }
}
