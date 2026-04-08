#[test]
fn ui() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/safe_instrument_pass.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_debug.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_err.rs");
}
