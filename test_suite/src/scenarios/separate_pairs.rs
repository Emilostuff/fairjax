use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(u8, usize),
    B(u8, usize),
}

use Msg::*;

#[macro_export]
macro_rules! declare_separate_pairs {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    #[$strategy]
                    (A(0, a), B(0, b)) if a == b =>
                        output.push(test_suite::MatchTrace::new(0, vec![A(0, a), B(0, b)])),
                    #[$strategy]
                    (A(1, a), B(1, b)) if a == b =>
                        output.push(test_suite::MatchTrace::new(0, vec![A(1, a), B(1, b)])),
                    #[$strategy]
                    (A(2, a), B(2, b)) if a == b =>
                        output.push(test_suite::MatchTrace::new(0, vec![A(2, a), B(2, b)])),
                    #[$strategy]
                    (A(3, a), B(3, b)) if a == b =>
                        output.push(test_suite::MatchTrace::new(0, vec![A(3, a), B(3, b)])),
                });
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize, seed: Option<u64>) -> Vec<Msg> {
    let mut rng = crate::get_rng(seed);
    let mut messages: Vec<_> = (0..size)
        .flat_map(|i| {
            [
                A(0, i),
                B(0, i),
                A(1, i),
                B(1, i),
                A(2, i),
                B(2, i),
                A(3, i),
                B(3, i),
            ]
        })
        .collect();
    messages.shuffle(&mut rng);
    messages
}
