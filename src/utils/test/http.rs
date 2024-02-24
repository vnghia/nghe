use axum::{body::Bytes, response::Response};
use http_body_util::BodyExt;

pub async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}
