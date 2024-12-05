use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use nghe_api::common::BinaryRequest;

use crate::Error;

pub struct Binary<R>(pub R);

impl<S, R> FromRequest<S> for Binary<R>
where
    S: Send + Sync,
    R: BinaryRequest + Send,
{
    type Rejection = Error;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            bitcode::deserialize(&Bytes::from_request(request, state).await?)
                .map_err(|_| Error::SerializeBinaryRequest)?,
        ))
    }
}
