use fairjax::*;
use rand::Rng;
use rand::rng;

#[derive(Clone, Debug, Copy, PartialEq, Message)]
pub enum Msg {
    A(i32),
}

use Msg::*;

#[macro_export]
macro_rules! declare_transitive {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::match_fairest_case!(
                    Msg,
                    msg >> mailbox,
                    case::<$strategy>(A(x) && A(y) && A(z), *x + 100 < *y && *y + 200 < *z, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), A(y), A(z)]));
                    })
                );
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize) -> Vec<Msg> {
    (0..(size * 3))
        .map(|_| A(rng().random_range(-500..500)))
        .collect()
}
