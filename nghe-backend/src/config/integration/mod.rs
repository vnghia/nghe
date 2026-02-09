use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_path::Utf8PlatformPathBuf;
use typed_path::utils::utf8_temp_dir;

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
        expression = Some(utf8_temp_dir().unwrap().join("nghe").join("spotify").join("token.json"))
    ))]
    pub token_path: Option<Utf8PlatformPathBuf>,
}

#[derive(Clone, Default, Serialize, Deserialize, Educe)]
#[educe(Debug)]
pub struct Lastfm {
    #[educe(Debug(ignore))]
    pub key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Integration {
    pub spotify: Spotify,
    pub lastfm: Lastfm,
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

    impl Lastfm {
        #[cfg(not(lastfm_env))]
        pub fn from_env() -> Self {
            Self { key: None }
        }

        #[cfg(lastfm_env)]
        pub fn from_env() -> Self {
            Self { key: Some(env!("LASTFM_KEY").to_owned()) }
        }
    }

    impl Integration {
        pub fn from_env() -> Self {
            Self { spotify: Spotify::from_env(), lastfm: Lastfm::from_env() }
        }
    }
}
