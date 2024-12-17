pub mod form;
pub mod header;

use axum::extract::FromRef;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
pub use form::Form;
pub use header::Header;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

pub trait AuthN {
    fn username(&self) -> &str;
    fn is_authenticated(&self, password: impl AsRef<[u8]>) -> bool;
}

pub trait AuthZ {
    fn is_authorized(role: users::Role) -> bool;
}

async fn login<R: AuthZ, S>(state: &S, authn: &impl AuthN) -> Result<Uuid, Error>
where
    Database: FromRef<S>,
{
    let database = Database::from_ref(state);
    let users::Auth { id, password, role } = users::table
        .filter(users::username.eq(authn.username()))
        .select(users::Auth::as_select())
        .first(&mut database.get().await?)
        .await
        .map_err(|_| error::Kind::WrongUsernameOrPassword)?;

    if !authn.is_authenticated(database.decrypt(password)?) {
        error::Kind::WrongUsernameOrPassword.into()
    } else if !R::is_authorized(role) {
        error::Kind::Forbidden.into()
    } else {
        Ok(id)
    }
}
