use educe::Educe;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typed_path::utils::utf8_temp_dir;
use typed_path::Utf8NativePathBuf;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct Spotify {
    pub id: Option<String>,
    pub secret: Option<String>,
    #[serde(with = "crate::filesystem::path::serde::option")]
    #[educe(Default(
        expression = Some(utf8_temp_dir().unwrap().join("nghe").join("spotify").join("token.json"))
    ))]
    pub token_path: Option<Utf8NativePathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Integration {
    pub spotify: Spotify,
}

#[cfg(test)]
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
