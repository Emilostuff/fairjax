use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(usize),
    B(usize),
}

use Msg::*;

#[macro_export]
macro_rules! declare_pairs {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::default();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    #[$strategy]
                    (A(x), B(y)) if x == y => {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), B(y)]));
                    }
                });
            }
            output
        }
    };
}

#[macro_export]
macro_rules! partitions_declare_pairs {
    ($fn_name:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::default();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    #[Partitions]
                    (A(x), B(x)) => {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), B(x)]));
                    }
                });
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize, seed: Option<u64>) -> Vec<Msg> {
    let mut rng = crate::get_rng(seed);
    let mut messages: Vec<_> = (0..size).flat_map(|i| [A(i), B(i)]).collect();
    messages.shuffle(&mut rng);
    messages
}
