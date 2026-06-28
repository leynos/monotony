//! Compile-time public API contracts.

#[test]
fn public_clock_api_compiles_for_downstream_crates() {
    let cases = trybuild::TestCases::new();

    cases.pass("tests/trybuild/public_clock_api.rs");
}

#[cfg(feature = "test-util")]
#[test]
fn test_util_clock_api_compiles_for_downstream_crates() {
    let cases = trybuild::TestCases::new();

    cases.pass("tests/trybuild/test_util_clock_api.rs");
}
