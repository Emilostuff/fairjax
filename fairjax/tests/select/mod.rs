#[test]
fn consume_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/select/pass/*.rs");
    t.compile_fail("tests/select/fail/*.rs");
}
