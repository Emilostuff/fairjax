use fairjax::*;
use rand::seq::SliceRandom;
use rand::{Rng, rng};

#[derive(Clone, Debug, Copy, PartialEq, Message)]
pub enum Msg {
    E(usize),
    I(usize),
}

use Msg::*;
const MAX_SINGLE_VAL: usize = 10000;

#[macro_export]
macro_rules! declare_many_to_one {
    ($fn_name:ident, $strategy:ident) => {
        fn $fn_name(messages: &[Msg]) -> Vec<test_suite::MatchTrace<Msg>> {
            let mut mailbox: fairjax_core::MailBox<Msg> = fairjax_core::MailBox::new();
            let mut output = vec![];

            use Msg::*;
            for msg in messages {
                fairjax::match_fairest_case!(
                    Msg,
                    msg >> mailbox,
                    case::<$strategy>(E(x) && I(total), x == total, {
                        output.push(test_suite::MatchTrace::new(0, vec![E(x), I(total)]));
                    }),
                    case::<$strategy>(E(x) && E(y) && I(total), *x + *y == *total, {
                        output.push(test_suite::MatchTrace::new(1, vec![E(x), E(y), I(total)]));
                    }),
                    case::<$strategy>(E(x) && E(y) && E(z) && I(total), *x + *y + *z == *total, {
                        output.push(test_suite::MatchTrace::new(
                            2,
                            vec![E(x), E(y), E(z), I(total)],
                        ));
                    })
                );
            }
            output
        }
    };
}

pub fn generate_random_messages(size: usize) -> Vec<Msg> {
    let mut invoices = Vec::<Msg>::new();
    let mut expenses = Vec::<Msg>::new();

    for _ in 0..size {
        match rng().random_range(0..3) {
            0 => {
                let val = rng().random_range(0..MAX_SINGLE_VAL);
                expenses.push(E(val));
                invoices.push(I(val));
            }
            1 => {
                let val0 = rng().random_range(0..MAX_SINGLE_VAL);
                let val1 = rng().random_range(0..MAX_SINGLE_VAL);
                expenses.push(E(val0));
                expenses.push(E(val1));
                invoices.push(I(val0 + val1));
            }
            _ => {
                let val0 = rng().random_range(0..MAX_SINGLE_VAL);
                let val1 = rng().random_range(0..MAX_SINGLE_VAL);
                let val2 = rng().random_range(0..MAX_SINGLE_VAL);
                expenses.push(E(val0));
                expenses.push(E(val1));
                expenses.push(E(val2));
                invoices.push(I(val0 + val1 + val2));
            }
        }
    }

    expenses.shuffle(&mut rng());
    invoices.extend(expenses);
    invoices
}
