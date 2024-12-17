use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use nghe_api::common::BinaryRequest;

use crate::{Error, error};

pub struct Binary<R>(pub R);

impl<S, R> FromRequest<S> for Binary<R>
where
    S: Send + Sync,
    R: BinaryRequest + Send,
{
    type Rejection = Error;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            bitcode::deserialize(
                &Bytes::from_request(request, state).await.map_err(error::Kind::from)?,
            )
            .map_err(|_| error::Kind::DeserializeBinary)?,
        ))
    }
}

#[cfg(test)]
#[coverage(off)]
mod test {
    #![allow(unexpected_cfgs)]

    use axum::body::Body;
    use axum::http;
    use fake::{Fake, Faker};
    use nghe_proc_macro::api_derive;

    use super::*;

    #[api_derive(fake = true)]
    #[endpoint(path = "test", url_only = true, internal = true, same_crate = false)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct Request {
        param_one: i32,
        param_two: u32,
    }

    #[tokio::test]
    async fn test_from_request() {
        let request: Request = Faker.fake();
        let http_request = http::Request::builder()
            .method(http::Method::POST)
            .body(Body::from(bitcode::serialize(&request).unwrap()))
            .unwrap();
        let binary = Binary::<Request>::from_request(http_request, &()).await.unwrap();
        assert_eq!(binary.0, request);
    }
}
