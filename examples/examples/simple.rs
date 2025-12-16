use fairjax::*;
use fairjax_core::MailBox;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(usize),
    B(usize),
}

use Msg::*;

fn main() {
    let mut mailbox = MailBox::default();

    let messages = vec![A(1), B(2), A(3), B(3), A(2), B(1)];
    for msg in messages {
        fairjax!(match msg >> [mailbox, Msg] {
            (A(x), B(y)) if x == y => {
                println!("Matched A({}) and B({})", x, y);
            }
        });
    }
}
