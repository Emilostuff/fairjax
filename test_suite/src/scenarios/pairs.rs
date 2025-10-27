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
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages.to_owned() {
                fairjax::match_fairest_case!(
                    Msg,
                    msg >> mailbox,
                    case::<$strategy>(A(x) && B(y), x == y, {
                        output.push(test_suite::MatchTrace::new(0, vec![A(x), B(y)]));
                    })
                );
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
