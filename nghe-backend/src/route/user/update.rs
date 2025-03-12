use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::user::update::{Request, Response};
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

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set((users::username.eq(request.username), users::email.eq(request.email)))
        .execute(&mut database.get().await?)
        .await?;

    Ok(Response)
}

#[cfg(test)]
#[coverage(off)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::{Mock, mock};

    #[rstest]
    #[tokio::test]
    async fn test_handler(#[future(awt)] mock: Mock) {
        let user = mock.user(0).await;
        let username: String = Faker.fake();

        handler(
            mock.database(),
            user.id(),
            Request { id: None, username: username.clone(), email: Faker.fake() },
        )
        .await
        .unwrap();

        let user = mock.user(0).await;
        assert_eq!(user.username(), username);
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
        let username: String = Faker.fake();

        let response = handler(
            mock.database(),
            user_1.id(),
            Request { id: Some(user_2.id()), username: username.clone(), email: Faker.fake() },
        )
        .await;

        if admin {
            response.unwrap();
            let user_2 = mock.user(1).await;
            assert_eq!(user_2.username(), username);
        } else {
            assert!(response.is_err());
        }
    }
}
