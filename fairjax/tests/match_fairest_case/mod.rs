#[test]
fn codegen_output_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/match_fairest_case/pass/*.rs");
    t.compile_fail("tests/match_fairest_case/fail/*.rs");
}
