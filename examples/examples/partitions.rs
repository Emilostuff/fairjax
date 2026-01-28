use fairjax_core::MailBox;

/// Message type
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(usize),
    B(usize),
}

use Msg::*;

fn main() {
    // Create mailbox
    let mut mailbox = MailBox::default();

    // Mock incoming messges
    let messages = vec![A(1), B(2), A(3), B(3), A(2), B(1)];

    // Receive messages one by one
    for msg in messages {
        fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
            // 'id' is used across all messages in the pattern (uniting variable).
            // Same outcome as: (A(id1), B(id2)) if id1 == id2,
            // but can optimised to run much faster.
            (A(id), B(id)) => {
                println!("Matched A({}) and B({})", id, id);
            }
        });
    }
}
