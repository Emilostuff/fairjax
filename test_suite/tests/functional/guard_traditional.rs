use fairjax::*;
use fairjax_core::MailBox;

#[derive(Clone, Debug, Copy, PartialEq)]
enum MyMsg {
    A { x: usize, y: &'static str, z: bool },
}

fn new_a(x: usize, y: &'static str, z: bool) -> MyMsg {
    MyMsg::A { x, y, z }
}

use MyMsg::*;

#[allow(unused_variables, unreachable_patterns)]
fn run_test(message: &MyMsg) -> usize {
    let mut mailbox: MailBox<MyMsg> = MailBox::new();
    let mut output = usize::MAX;

    fairjax!(match message.clone() >> [mailbox, MyMsg] {
        // Exact value
        A { x, .. } if *x == 0 => output = 0,
        // Range (inclusive boundaries)
        A { x, .. } if *x >= 1 && *x <= 5 => output = 1,
        // String constant
        A { y, .. } if *y == "test" => output = 2,
        // Boolean constant
        A { z, .. } if *z == false => output = 3,
        // Specific combination
        A { x, y, z } if *x == 7 && *y == "val" && *z == true => output = 4,
        // Catch-all
        A { .. } => panic!(),
    });

    output
}

#[test]
fn test_match_exact_value() {
    // Usize constant: x = 0 should match A { x: 0, .. }
    let msg = new_a(0, "foo", true);

    // Verify correctnesss
    assert_eq!(0, run_test(&msg));
}

#[test]
fn test_match_range_lower() {
    // Range lower bound: x = 1 should match A { x: 1..=5, .. }
    let msg = new_a(1, "bar", true);

    // Verify correctnesss
    assert_eq!(1, run_test(&msg));
}

#[test]
fn test_match_range_upper() {
    // Range upper bound: x = 5 should match A { x: 1..=5, .. }
    let msg = new_a(5, "baz", false);

    // Verify correctnesss
    assert_eq!(1, run_test(&msg));
}

#[test]
fn test_match_string_constant() {
    // String constant: y = "test" should match A { y: "test", .. }
    let msg = new_a(42, "test", true);

    // Verify correctnesss
    assert_eq!(2, run_test(&msg));
}

#[test]
fn test_match_boolean_constant() {
    // Boolean constant: z = false should match A { z: false, .. }
    let msg = new_a(99, "hello", false);

    // Verify correctness
    assert_eq!(3, run_test(&msg));
}

#[test]
fn match_specific_combination() {
    // Specific combination: x = 7, y = "val", z = true should match A { x: 7, y: "val", z: true }
    let msg = new_a(7, "val", true);

    // Verify correctness
    assert_eq!(4, run_test(&msg));
}

#[test]
#[should_panic]
fn test_match_unhandled_case_panics() {
    // Unhandled case: should panic as it matches only the catch-all
    let msg = new_a(42, "other", true);
    run_test(&msg);
}
