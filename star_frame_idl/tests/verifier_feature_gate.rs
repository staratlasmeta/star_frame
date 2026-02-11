#[test]
fn verifier_module_requires_verifier_feature() {
    if cfg!(feature = "verifier") {
        return;
    }

    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/verifier_feature_off.rs");
}
