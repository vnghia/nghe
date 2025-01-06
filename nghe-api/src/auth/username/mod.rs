pub mod token;
use std::borrow::Cow;

use nghe_proc_macro::api_derive;

#[api_derive]
#[derive(Clone)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Auth<'s, 'p> {
    Token(token::Auth<'s>),
    Password {
        #[serde(rename = "p")]
        password: Cow<'p, str>,
    },
}

#[api_derive]
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Username<'u, 's, 'p> {
    #[serde(rename = "u")]
    pub username: Cow<'u, str>,
    #[serde(flatten)]
    pub auth: Auth<'s, 'p>,
}

mod convert {
    use super::*;

    impl<'s> From<token::Auth<'s>> for Auth<'s, '_> {
        fn from(value: token::Auth<'s>) -> Self {
            Self::Token(value)
        }
    }

    impl<'p, C: Into<Cow<'p, str>>> From<C> for Auth<'_, 'p> {
        fn from(value: C) -> Self {
            Self::Password { password: value.into() }
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[api_derive]
    #[derive(PartialEq)]
    pub struct Test<'u, 's, 'p> {
        value: Option<u32>,
        #[serde(flatten)]
        username: Username<'u, 's, 'p>,
    }

    #[rstest]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&\
        u=username&s=c19b2d&value=10",
        Some(Test {
            value: Some(10),
            username: Username {
                username: "username".into(),
                auth: token::Auth {
                    salt: "c19b2d".into(),
                    token: token::Token::new(b"sesame", "c19b2d")
                }.into()
            }
        }
    ))]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&u=username&s=c19b2d",
        Some(Test {
            value: None,
            username: Username {
                username: "username".into(),
                auth: token::Auth {
                    salt: "c19b2d".into(),
                    token: token::Token::new(b"sesame", "c19b2d")
                }.into()
            }
        }
    ))]
    #[case(
        "u=username&p=password&value=10",
        Some(Test {
            value: Some(10),
            username: Username {
                username: "username".into(),
                auth: "password".into()
            }
        }
    ))]
    #[case(
        "u=username&p=password",
        Some(Test {
            value: None,
            username: Username {
                username: "username".into(),
                auth: "password".into()
            }
        }
    ))]
    #[case("u=username&s=c19b2d", None)]
    fn test_deserialize(#[case] input: &str, #[case] result: Option<Test<'_, '_, '_>>) {
        assert_eq!(serde_html_form::from_str(input).ok(), result);
    }
}
