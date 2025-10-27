use fairjax::*;
use rand::seq::SliceRandom;

#[derive(Clone, Debug, Copy, PartialEq, Message)]
pub enum Msg {
    R(usize), // Worker Ready
    J(usize), // Job
}

use Msg::*;

#[macro_export]
macro_rules! declare_workers {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::match_fairest_case!(
                    Msg,
                    msg >> mailbox,
                    case::<$strategy>(R(w0) && R(w1) && R(w2) && R(w3) && J(job), true, {
                        output.push(test_suite::MatchTrace::new(
                            0,
                            vec![R(w0), R(w1), R(w2), R(w3), J(job)],
                        ));
                    })
                );
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize, seed: Option<u64>) -> Vec<Msg> {
    let mut rng = crate::get_rng(seed);
    let mut messages: Vec<_> = (0..size)
        .flat_map(|i| [R(i), R(i), R(i), R(i), J(i)])
        .collect();

    messages.shuffle(&mut rng);
    messages
}
