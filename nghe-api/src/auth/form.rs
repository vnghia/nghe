use nghe_proc_macro::api_derive;
use serde::Deserialize;

use super::{ApiKey, Username};

#[api_derive]
#[derive(Clone)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Form<'u, 'c, 's, 'p> {
    Username(Username<'u, 'c, 's, 'p>),
    ApiKey(ApiKey),
}

pub trait Trait<'u, 'c, 's, 'p, 'de: 'u + 'c + 's + 'p, R>: Deserialize<'de> {
    fn new(request: R, auth: Form<'u, 'c, 's, 'p>) -> Self;
    fn auth<'form>(&'form self) -> &'form Form<'u, 'c, 's, 'p>;
    fn request(self) -> R;
}

mod convert {
    use uuid::Uuid;

    use super::*;

    impl<'u, 'c, 's, 'p> From<Username<'u, 'c, 's, 'p>> for Form<'u, 'c, 's, 'p> {
        fn from(value: Username<'u, 'c, 's, 'p>) -> Self {
            Self::Username(value)
        }
    }

    impl From<Uuid> for Form<'_, '_, '_, '_> {
        fn from(value: Uuid) -> Self {
            Self::ApiKey(value.into())
        }
    }
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;
    use uuid::uuid;

    use super::*;
    use crate::auth::username;

    #[api_derive]
    #[derive(PartialEq)]
    pub struct Test<'u, 'c, 's, 'p> {
        value: Option<u32>,
        #[serde(flatten)]
        form: Form<'u, 'c, 's, 'p>,
    }

    #[rstest]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&\
        u=username&s=c19b2d&value=10&c=client",
        Some(Test {
            value: Some(10),
            form: Username {
                username: "username".into(),
                client: "client".into(),
                auth: username::token::Auth {
                    salt: "c19b2d".into(),
                    token: username::Token::new(b"sesame", "c19b2d")
                }.into()
            }.into()
        }
    ))]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&\
        u=username&s=c19b2d&c=client",
        Some(Test {
            value: None,
            form: Username {
                username: "username".into(),
                client: "client".into(),
                auth: username::token::Auth {
                    salt: "c19b2d".into(),
                    token: username::Token::new(b"sesame", "c19b2d")
                }.into()
            }.into()
        }
    ))]
    #[case(
        "u=username&p=password&value=10&c=client",
        Some(Test {
            value: Some(10),
            form: Username {
                username: "username".into(),
                client: "client".into(),
                auth: "password".into()
            }.into()
        }
    ))]
    #[case(
        "u=username&p=password&c=client",
        Some(Test {
            value: None,
            form: Username {
                username: "username".into(),
                client: "client".into(),
                auth: "password".into()
            }.into()
        }
    ))]
    #[case(
        "apiKey=ce8216ee-c293-4285-8847-2268e6704a19&value=10",
        Some(Test {
            value: Some(10),
            form: uuid!("ce8216ee-c293-4285-8847-2268e6704a19").into()
        }
    ))]
    #[case(
        "apiKey=ce8216ee-c293-4285-8847-2268e6704a19",
        Some(Test {
            value: None,
            form: uuid!("ce8216ee-c293-4285-8847-2268e6704a19").into()
        }
    ))]
    #[case("u=username&s=c19b2d", None)]
    fn test_deserialize(#[case] input: &str, #[case] result: Option<Test<'_, '_, '_, '_>>) {
        assert_eq!(serde_html_form::from_str(input).ok(), result);
    }
}
