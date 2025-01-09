pub mod token;
use std::borrow::Cow;

#[cfg(feature = "fake")]
use fake::{Fake, Faker};
use nghe_proc_macro::api_derive;
pub use token::Token;

#[api_derive(fake = true)]
#[derive(Clone)]
#[serde(untagged)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Auth<'s, 'p> {
    Token(token::Auth<'s>),
    Password {
        #[serde(rename = "p")]
        #[cfg_attr(feature = "fake", dummy(expr = "Faker.fake::<String>().into()"))]
        password: Cow<'p, str>,
    },
}

#[api_derive(fake = true)]
#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Username<'u, 'c, 's, 'p> {
    #[serde(rename = "u")]
    #[cfg_attr(feature = "fake", dummy(expr = "Faker.fake::<String>().into()"))]
    pub username: Cow<'u, str>,
    #[serde(rename = "c")]
    #[cfg_attr(feature = "fake", dummy(expr = "Faker.fake::<String>().into()"))]
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
