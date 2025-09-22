use fairjax::*;
use fairjax_core::MailBox;

#[derive(Clone, Debug, Copy, PartialEq, Message)]
enum MyMsg {
    A(u8),
    B(u8),
    C(u8, u8),
}

use MyMsg::*;

#[derive(Clone, Debug, PartialEq)]
struct Match {
    pattern_no: usize,
    messages: Vec<MyMsg>,
}

impl Match {
    fn new(pattern_no: usize, messages: Vec<MyMsg>) -> Self {
        Match {
            pattern_no,
            messages,
        }
    }
}

fn run_test(messages: &[MyMsg]) -> Vec<Match> {
    let mut mailbox: MailBox<MyMsg> = MailBox::new();
    let mut output = vec![];

    for msg in messages {
        match_fairest_case!(
            MyMsg,
            msg >> mailbox,
            case(A(x) && B(y), x >= y, {
                output.push(Match {
                    pattern_no: 0,
                    messages: vec![A(x), B(y)],
                });
            }),
            case(B(x1) && C(y1, y2) && B(x2), x1 == y1 && x2 == y2, {
                output.push(Match {
                    pattern_no: 1,
                    messages: vec![B(x1), C(y1, y2), B(x2)],
                });
            }),
            case(C(x1, x2) && C(y1, y2), x1 == y1 && x2 == y2, {
                output.push(Match {
                    pattern_no: 2,
                    messages: vec![C(x1, x2), C(y1, y2)],
                });
            }),
            case(A(x), *x > 100, {
                output.push(Match {
                    pattern_no: 3,
                    messages: vec![A(x)],
                });
            })
        );
    }
    output
}

#[test]
fn test() {
    let input = [A(4), B(8), C(5, 8), B(5)];
    let expected = vec![Match::new(1, vec![B(5), C(5, 8), B(8)])];

    assert_eq!(expected, run_test(&input));
}

#[test]
fn test2() {
    let input = [A(4), A(101), B(8), C(5, 8), B(5)];
    let expected = vec![
        Match::new(3, vec![A(101)]),
        Match::new(1, vec![B(5), C(5, 8), B(8)]),
    ];

    assert_eq!(expected, run_test(&input));
}

#[test]
fn complex_consumption_order_test() {
    let input = [
        A(120),      // trigger pattern_no 3
        B(120),      // mailbox = B(120)
        C(120, 120), // mailbox = B(120), C(120, 120)
        B(120),      // trigger pattern_no 1
        A(4),        // now mailbox has A(4)
        B(8),        // now mailbox has A(4), B(8)
        C(5, 8),     // now mailbox has A(4), B(8), C(5, 8)
        B(5),        // trigger pattern_no 1
    ];
    let expected = vec![
        Match::new(3, vec![A(120)]),
        Match::new(1, vec![B(120), C(120, 120), B(120)]),
        Match::new(1, vec![B(5), C(5, 8), B(8)]),
    ];

    assert_eq!(expected, run_test(&input));
}
