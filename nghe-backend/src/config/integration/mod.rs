use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_path::utils::utf8_temp_dir;
use typed_path::Utf8PlatformPathBuf;

#[serde_as]
#[derive(Clone, Serialize, Deserialize, Educe)]
#[educe(Debug, Default)]
pub struct Spotify {
    #[educe(Debug(ignore))]
    pub id: Option<String>,
    #[educe(Debug(ignore))]
    pub secret: Option<String>,
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[educe(Default(
        expression = Some(
            utf8_temp_dir()
                .unwrap()
                .join("nghe")
                .join("spotify")
                .join("token.json")
                .with_platform_encoding_checked()
                .unwrap()
        )
    ))]
    pub token_path: Option<Utf8PlatformPathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Integration {
    pub spotify: Spotify,
}

#[cfg(test)]
#[coverage(off)]
mod test {
    use super::*;

    impl Spotify {
        #[cfg(not(spotify_env))]
        pub fn from_env() -> Self {
            Self { id: None, secret: None, token_path: None }
        }

        #[cfg(spotify_env)]
        pub fn from_env() -> Self {
            Self {
                id: Some(env!("SPOTIFY_ID").to_owned()),
                secret: Some(env!("SPOTIFY_SECRET").to_owned()),
                token_path: None,
            }
        }
    }

    impl Integration {
        pub fn from_env() -> Self {
            Self { spotify: Spotify::from_env() }
        }
    }
}
