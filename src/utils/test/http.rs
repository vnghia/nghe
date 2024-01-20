use crate::{
    config::EncryptionKey,
    open_subsonic::common::request::{Validate, ValidatedForm},
    DbPool,
};

use axum::{body::Bytes, response::Response};
use http_body_util::BodyExt;

pub async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}

pub async fn to_validated_form<P: Validate + Sync>(
    pool: &DbPool,
    key: &EncryptionKey,
    params: P,
) -> ValidatedForm<P> {
    let user = params.validate(pool, key).await.unwrap();
    ValidatedForm { params, user }
}
