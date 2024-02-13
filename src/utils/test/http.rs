use super::db::TemporaryDatabase;
use crate::open_subsonic::common::request::{Validate, ValidatedForm};

use axum::{body::Bytes, response::Response};
use http_body_util::BodyExt;

pub async fn to_bytes(res: Response) -> Bytes {
    res.into_body().collect().await.unwrap().to_bytes()
}

pub async fn to_validated_form<P: Validate + Sync>(
    db: &TemporaryDatabase,
    params: P,
) -> ValidatedForm<P> {
    let user = params.validate(db.get_pool(), db.get_key()).await.unwrap();
    ValidatedForm { params, user }
}
