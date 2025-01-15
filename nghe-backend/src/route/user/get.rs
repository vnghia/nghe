use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
pub use nghe_api::user::get::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

#[handler(internal = true)]
pub async fn handler(database: &Database, user_id: Uuid) -> Result<Response, Error> {
    users::table
        .filter(users::id.eq(user_id))
        .select(users::User::as_select())
        .first(&mut database.get().await?)
        .await
        .map(users::User::into)
        .map_err(Error::from)
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(#[future(awt)] mock: Mock) {
        let user = mock.user(0).await;
        let response = handler(mock.database(), user.id()).await.unwrap();
        assert_eq!(user.id(), response.id);
        assert_eq!(user.username(), response.username);
    }
}
