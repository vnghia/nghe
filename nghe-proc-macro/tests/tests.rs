#[test]
pub fn api() {
    macrotest::expand("tests/api/**/*.rs");
}

#[test]
pub fn backend() {
    macrotest::expand("tests/backend/**/*.rs");
}

#[test]
pub fn orm() {
    macrotest::expand("tests/orm/**/*.rs");
}

#[test]
pub fn command() {
    macrotest::expand("tests/command/**/*.rs");
}
