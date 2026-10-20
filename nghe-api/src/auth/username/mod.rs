pub mod token;
use std::borrow::Cow;

#[cfg(any(test, feature = "test"))]
use fake::{Fake, Faker};
use nghe_proc_macro::api_derive;
pub use token::Token;

#[api_derive]
#[derive(Clone)]
#[serde(untagged)]
pub enum Auth<'s, 'p> {
    Token(token::Auth<'s>),
    Password {
        #[serde(rename = "p")]
        #[dummy(expr = "Faker.fake::<String>().into()")]
        password: Cow<'p, str>,
    },
}

#[api_derive]
#[derive(Clone)]
pub struct Username<'u, 'c, 's, 'p> {
    #[serde(rename = "u")]
    #[dummy(expr = "Faker.fake::<String>().into()")]
    pub username: Cow<'u, str>,
    #[serde(rename = "c")]
    #[dummy(expr = "Faker.fake::<String>().into()")]
    pub client: Cow<'c, str>,
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
