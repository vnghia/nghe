use nghe_proc_macro::api_derive;
use serde::Deserialize;

use super::username;

#[api_derive]
#[derive(Clone)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Form<'u, 's, 'p> {
    Username(username::Username<'u, 's, 'p>),
}

pub trait Trait<'u, 's, 'p, 'de: 'u + 's + 'p, R>: Deserialize<'de> {
    fn new(request: R, auth: Form<'u, 's, 'p>) -> Self;
    fn auth<'form>(&'form self) -> &'form Form<'u, 's, 'p>;
    fn request(self) -> R;
}

impl<'u, 's, 'p> From<username::Username<'u, 's, 'p>> for Form<'u, 's, 'p> {
    fn from(value: username::Username<'u, 's, 'p>) -> Self {
        Self::Username(value)
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
        form: Form<'u, 's, 'p>,
    }

    #[rstest]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&\
        u=username&s=c19b2d&value=10",
        Some(Test {
            value: Some(10),
            form: username::Username {
                username: "username".into(),
                auth: username::token::Auth {
                    salt: "c19b2d".into(),
                    token: username::token::Token::new(b"sesame", "c19b2d")
                }.into()
            }.into()
        }
    ))]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&u=username&s=c19b2d",
        Some(Test {
            value: None,
            form: username::Username {
                username: "username".into(),
                auth: username::token::Auth {
                    salt: "c19b2d".into(),
                    token: username::token::Token::new(b"sesame", "c19b2d")
                }.into()
            }.into()
        }
    ))]
    #[case(
        "u=username&p=password&value=10",
        Some(Test {
            value: Some(10),
            form: username::Username {
                username: "username".into(),
                auth: "password".into()
            }.into()
        }
    ))]
    #[case(
        "u=username&p=password",
        Some(Test {
            value: None,
            form: username::Username {
                username: "username".into(),
                auth: "password".into()
            }.into()
        }
    ))]
    #[case("u=username&s=c19b2d", None)]
    fn test_deserialize(#[case] input: &str, #[case] result: Option<Test<'_, '_, '_>>) {
        assert_eq!(serde_html_form::from_str(input).ok(), result);
    }
}
