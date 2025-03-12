use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
pub use nghe_api::user::get::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::Error;
use crate::database::Database;
use crate::orm::users;

#[handler(internal = true)]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    request: Request,
) -> Result<Response, Error> {
    let user_id = if let Some(id) = request.id {
        users::Role::check_admin(database, user_id).await?;
        id
    } else {
        user_id
    };

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
        let response = handler(mock.database(), user.id(), Request { id: None }).await.unwrap();
        assert_eq!(user.id(), response.id);
        assert_eq!(user.username(), response.username);
    }

    #[rstest]
    #[tokio::test]
    async fn test_handler_admin(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
        #[values(true, false)] admin: bool,
    ) {
        let user_1 = mock.add_user().role(users::Role { admin }).call().await.user(0).await;
        let user_2 = mock.add_user().role(users::Role { admin: false }).call().await.user(1).await;
        let response =
            handler(mock.database(), user_1.id(), Request { id: Some(user_2.id()) }).await;

        if admin {
            let response = response.unwrap();
            assert_eq!(user_2.id(), response.id);
            assert_eq!(user_2.username(), response.username);
            assert_eq!(user_2.role(), response.role.into());
        } else {
            assert!(response.is_err());
        }
    }
}
