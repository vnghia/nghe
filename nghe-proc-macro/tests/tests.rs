#[test]
pub fn api() {
    macrotest::expand("tests/api/**/*.rs");
}

#[test]
pub fn backend() {
    macrotest::expand("tests/backend/**/*.rs");
}
