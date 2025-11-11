#[derive(Clone, Debug, Copy, PartialEq)]

pub enum Msg {
    A(usize),
    B(usize),
}

use Msg::*;

fn main() {
    let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();

    let messages = vec![A(1), B(2), A(3), B(3), A(2), B(1)];
    for msg in messages {
        fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
            (A(id), B(id)) => {
                println!("Matched A({}) and B({})", id, id);
            }
        });
    }
}
