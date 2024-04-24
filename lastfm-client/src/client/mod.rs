use concat_string::concat_string;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ClientError;
use crate::params::MethodName;
use crate::response::LastfmResponse;

#[derive(Clone)]
pub struct Client {
    client: reqwest::Client,
    key: String,
}

impl Client {
    const LASTFM_ROOT_URL: &'static str = "https://ws.audioscrobbler.com/2.0/?";

    pub fn new(key: String) -> Self {
        Self { client: Default::default(), key }
    }

    pub fn new_with_client(client: reqwest::Client, key: String) -> Self {
        Self { client, key }
    }

    #[cfg(lastfm_env)]
    pub fn new_from_env() -> Self {
        Self::new(env!("LASTFM_KEY").to_string())
    }

    fn to_query_str<P: Serialize + MethodName>(&self, params: &P) -> Result<String, ClientError> {
        serde_html_form::to_string(params)
            .map(|q| {
                concat_string!(
                    "method=",
                    P::method_name(),
                    "&",
                    q,
                    "&",
                    "api_key=",
                    &self.key,
                    "&format=json"
                )
            })
            .map_err(ClientError::from)
    }

    pub async fn send<P: Serialize + MethodName, R: DeserializeOwned>(
        &self,
        params: &P,
    ) -> Result<R, ClientError> {
        self.client
            .get(concat_string!(Self::LASTFM_ROOT_URL, self.to_query_str(params)?))
            .send()
            .await?
            .error_for_status()?
            .json::<LastfmResponse<R>>()
            .await?
            .into()
    }
}

#[cfg(all(test, lastfm_env))]
mod tests {
    use fake::{Dummy, Fake, Faker};

    use super::*;

    #[tokio::test]
    async fn test_client_send_err() {
        #[derive(Serialize, Dummy)]
        struct TestParams {
            invalid: String,
        }

        impl MethodName for TestParams {
            fn method_name() -> &'static str {
                "invalid"
            }
        }

        assert!(
            Client::new("invalid-api-key".into())
                .send::<_, ()>(&TestParams { ..Faker.fake() })
                .await
                .is_err()
        )
    }
}
