#[test]
fn ui() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/safe_instrument_pass.rs");
    tests.pass("tests/ui/safe_instrument_ret_pass.rs");
    tests.pass("tests/ui/safe_instrument_err_pass.rs");
    tests.pass("tests/ui/safe_instrument_async_ret_err_pass.rs");
    #[cfg(feature = "trusted-literal")]
    tests.pass("tests/ui/safe_instrument_trusted_literal_pass.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_debug.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_err.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_ret_type.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_err_type.rs");
    tests.compile_fail("tests/ui/safe_instrument_fail_err_non_result.rs");
}
