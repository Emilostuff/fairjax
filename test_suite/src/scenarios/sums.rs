use fairjax::*;
use rand::Rng;
use rand::rng;

#[derive(Clone, Debug, Copy, PartialEq, Message)]
pub enum Msg {
    A(usize),
}

use Msg::*;

#[macro_export]
macro_rules! declare_sum {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::match_fairest_case!(
                    Msg,
                    msg >> mailbox,
                    case::<$strategy>(A(x) && A(y), x + y == 2, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), A(y)]));
                    }),
                    case::<$strategy>(A(x) && A(y), x + y == 8, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), A(y)]));
                    }),
                    case::<$strategy>(A(x) && A(y), x + y == 11, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), A(y)]));
                    }),
                    case::<$strategy>(A(x) && A(y), x + y == 16, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), A(y)]));
                    }),
                    case::<$strategy>(A(x) && A(y), x + y == 20, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), A(y)]));
                    })
                );
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize) -> Vec<Msg> {
    (0..size).map(|_| A(rng().random_range(1..=10))).collect()
}
