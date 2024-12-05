// use axum::extract::{FromRef, FromRequest, Request};
// use axum_extra::extract::Form;
// use nghe_api::auth;

// use crate::database::Database;
// use crate::Error;

// impl<S, F> FromRequest<S> for F
// where
//     S: Send + Sync,
//     Database: FromRef<S>,
//     F: for<'u, 's> auth::form::Trait<'u, 's> + Send,
// {
//     type Rejection = Error;

//     async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
//         let Form(form) = Form::from_request(request, &()).await?;
//     }
// }
