#[test]
fn ui() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/derive_pass.rs");
    tests.compile_fail("tests/ui/derive_fail_raw_string.rs");
}
