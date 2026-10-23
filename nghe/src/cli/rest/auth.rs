use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug)]
pub enum Auth {
    Basic { username: String, password: String },
    Bearer(String),
}

impl FromStr for Auth {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((username, password)) = s.split_once(':') {
            Self::Basic { username: username.into(), password: password.into() }
        } else {
            Self::Bearer(s.into())
        })
    }
}
