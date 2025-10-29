use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Msg {
    A(usize),
    B(usize),
}

use Msg::*;

#[macro_export]
macro_rules! declare_permutations {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::fairjax!(match msg.clone() >> [mailbox, Msg] {
                    #[$strategy]
                    (A(a), A(b), A(c), A(d), B(e), B(f), B(g), B(h))
                        if a < b && b < c && c < d && e < f && f < g && g < h =>
                    {
                        output.push(test_suite::MatchTrace::new(
                            0,
                            vec![A(a), A(b), A(c), A(d), B(e), B(f), B(g), B(h)],
                        ));
                    }
                });
            }
            output
        }
    };
}

pub fn generate_random_messages(seed: Option<u64>) -> Vec<Msg> {
    let mut rng = crate::get_rng(seed);
    let x = rng.random_range(1..1000);
    let mut messages: Vec<_> = vec![
        A(x),
        A(2 * x),
        A(3 * x),
        A(4 * x),
        B(5 * x),
        B(6 * x),
        B(7 * x),
        B(8 * x),
    ];
    messages.shuffle(&mut rng);
    messages
}
