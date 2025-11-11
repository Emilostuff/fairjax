use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(usize),
    B(usize),
    C(usize),
    D(usize),
    E(usize),
}

use Msg::*;

#[macro_export]
macro_rules! declare_groups {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    #[$strategy]
                    (A(a), B(b), C(c), D(d), E(e)) if a == b && b == c && c == d && d == e => {
                        output.push(test_suite::MatchTrace::new(
                            0,
                            vec![A(a), B(b), C(c), D(d), E(e)],
                        ));
                    }
                });
            }
            output
        }
    };
}

#[macro_export]
macro_rules! partitions_declare_groups {
    ($fn_name:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    #[Partitions]
                    (A(x), B(x), C(x), D(x), E(x)) => {
                        output.push(test_suite::MatchTrace::new(
                            0,
                            vec![A(x), B(x), C(x), D(x), E(x)],
                        ));
                    }
                });
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize, seed: Option<u64>) -> Vec<Msg> {
    let mut rng = crate::get_rng(seed);
    let mut messages: Vec<_> = (0..size)
        .flat_map(|i| [A(i), B(i), C(i), D(i), E(i)])
        .collect();
    messages.shuffle(&mut rng);
    messages
}
