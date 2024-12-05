use axum::extract::{FromRef, FromRequest, Request};
use nghe_api::auth;
use nghe_api::auth::form::Trait;
use nghe_api::common::FormRequest;
use uuid::Uuid;

use super::{login, AuthN, AuthZ};
use crate::database::Database;
use crate::Error;

pub struct Form<R> {
    pub id: Uuid,
    pub request: R,
}

impl AuthN for auth::Form<'_, '_> {
    fn username(&self) -> &str {
        match self {
            auth::Form::Token(auth) => auth.username.as_ref(),
        }
    }

    fn is_authenticated(&self, password: impl AsRef<[u8]>) -> bool {
        match self {
            auth::Form::Token(auth) => {
                let password = password.as_ref();
                let password_token = auth::Token::new(password, auth.salt.as_bytes());
                password_token == auth.token
            }
        }
    }
}

impl<S, R> FromRequest<S> for Form<R>
where
    S: Send + Sync,
    Database: FromRef<S>,
    R: for<'u, 's> FormRequest<'u, 's> + AuthZ + Send,
{
    type Rejection = Error;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let form: R::AuthForm = axum_extra::extract::Form::from_request(request, &()).await?.0;
        let id = login::<R, _>(state, form.auth()).await?;
        Ok(Self { id, request: form.request() })
    }
}
