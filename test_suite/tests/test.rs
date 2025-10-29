use fairjax_core::MailBox;
use test_suite::MatchTrace;

#[derive(Clone, Debug, Copy, PartialEq)]
enum MyMsg {
    A(u8),
    B(u8),
    C(u8, u8),
}

use MyMsg::*;

fn run_test(messages: &[MyMsg]) -> Vec<MatchTrace<MyMsg>> {
    let mut mailbox: MailBox<MyMsg> = MailBox::new();
    let mut output = vec![];

    for msg in messages.to_owned() {
        fairjax::fairjax!(match msg >> [mailbox, MyMsg] {
            (A(x), B(y)) if x >= y => {
                output.push(MatchTrace::new(0, vec![A(x), B(y)]));
            }
            #[BruteForce]
            (B(x1), C(y1, y2), B(x2)) if x1 == y1 && x2 == y2 => {
                output.push(MatchTrace::new(1, vec![B(x1), C(y1, y2), B(x2)]));
            }
            (C(x1, x2), C(y1, y2)) if x1 == y1 && x2 == y2 => {
                output.push(MatchTrace::new(2, vec![C(x1, x2), C(y1, y2)]));
            }
            A(x) if *x > 100 => {
                output.push(MatchTrace::new(3, vec![A(x)]));
            }
        });
    }
    output
}

#[test]
fn test1() {
    let input = [A(4), B(8), C(5, 8), B(5)];
    let expected = vec![MatchTrace::new(1, vec![B(5), C(5, 8), B(8)])];

    assert_eq!(expected, run_test(&input));
}

#[test]
fn test2() {
    let input = [A(4), A(101), B(8), C(5, 8), B(5)];
    let expected = vec![
        MatchTrace::new(3, vec![A(101)]),
        MatchTrace::new(1, vec![B(5), C(5, 8), B(8)]),
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
        MatchTrace::new(3, vec![A(120)]),
        MatchTrace::new(1, vec![B(120), C(120, 120), B(120)]),
        MatchTrace::new(1, vec![B(5), C(5, 8), B(8)]),
    ];

    assert_eq!(expected, run_test(&input));
}
