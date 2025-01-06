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
