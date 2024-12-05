use nghe_proc_macro::api_derive;
use serde::Deserialize;

use super::token;

#[api_derive(copy = false)]
#[serde(untagged)]
pub enum Form<'u, 's> {
    Token(token::Auth<'u, 's>),
}

pub trait Trait<'u, 's, R>: for<'de> Deserialize<'de> {
    fn new(request: R, auth: Form<'u, 's>) -> Self;
    fn auth<'form>(&'form self) -> &'form Form<'u, 's>;
    fn request(self) -> R;
}

impl<'u, 't> From<token::Auth<'u, 't>> for Form<'u, 't> {
    fn from(value: token::Auth<'u, 't>) -> Self {
        Self::Token(value)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::auth::token::Token;

    #[api_derive]
    pub struct Test<'u, 't> {
        value: Option<u32>,
        #[serde(flatten, borrow)]
        form: Form<'u, 't>,
    }

    #[rstest]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&\
        u=username&s=c19b2d&value=10",
        Some(Test {
            value: Some(10),
            form: token::Auth {
                username: "username".into(),
                salt: "c19b2d".into(),
                token: Token::new(b"sesame", "c19b2d")
            }.into()
        }
    ))]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&u=username&s=c19b2d",
        Some(Test {
            value: None,
            form: token::Auth {
                username: "username".into(),
                salt: "c19b2d".into(),
                token: Token::new(b"sesame", "c19b2d")
            }.into()
        }
    ))]
    #[case("u=username&s=c19b2d", None)]
    fn test_deserialize(#[case] input: &str, #[case] result: Option<Test<'_, '_>>) {
        assert_eq!(serde_html_form::from_str(input).ok(), result);
    }
}
