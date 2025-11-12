use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(usize),
    B(usize),
    C(usize),
}

use Msg::*;

#[macro_export]
macro_rules! declare_equal_cases {
    (
        $fn_name:ident,
        $strategy:ident,
        [$($idx:expr),* $(,)?]
    ) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::default();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    $(
                        #[$strategy]
                        (A(a), B(b1), B(b2), C(c)) if a < b1 && b1 < b2 && b2 < c => {
                            let idx = $idx;
                            output.push(test_suite::MatchTrace::new(idx, vec![A(a), B(b1), B(b2), C(c)]));
                        },
                    )*
                });
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize, seed: Option<u64>) -> Vec<Msg> {
    let mut rng = crate::get_rng(seed);
    let mut messages: Vec<_> = (0..size)
        .flat_map(|i| [A(i), B(i * i), B(i * i * i), C(i * i * i * i)])
        .collect();
    messages.shuffle(&mut rng);
    messages
}
