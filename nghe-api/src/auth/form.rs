use std::borrow::Cow;

use nghe_proc_macro::api_derive;

use super::Token;

#[api_derive]
#[derive(Clone)]
#[cfg_attr(any(test, feature = "test"), derive(Default))]
pub struct Form<'u, 't> {
    #[serde(rename = "u")]
    pub username: Cow<'u, str>,
    #[serde(rename = "s")]
    pub salt: Cow<'t, str>,
    #[serde(rename = "t")]
    pub token: Token,
}

pub trait Trait: Sized {
    fn auth(&self) -> &Form<'_, '_>;
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

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
            form: Form {
                username: "username".into(),
                salt: "c19b2d".into(),
                token: Token::new(b"sesame", "c19b2d")
            }
        }
    ))]
    #[case(
        "t=26719a1196d2a940705a59634eb18eab&u=username&s=c19b2d",
        Some(Test {
            value: None,
            form: Form {
                username: "username".into(),
                salt: "c19b2d".into(),
                token: Token::new(b"sesame", "c19b2d")
            }
        }
    ))]
    #[case("u=username&s=c19b2d", None)]
    fn test_deserialize(#[case] input: &str, #[case] result: Option<Test<'_, '_>>) {
        assert_eq!(serde_html_form::from_str(input).ok(), result);
    }
}
