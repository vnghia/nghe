use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
pub use nghe_api::user::update_password::{Request, Response};
use nghe_proc_macro::handler;
use uuid::Uuid;

use crate::database::Database;
use crate::orm::users;
use crate::{Error, error};

#[handler(internal = true)]
pub async fn handler(
    database: &Database,
    user_id: Uuid,
    user_role: users::Role,
    request: Request,
) -> Result<Response, Error> {
    let user_id = if let Some(id) = request.id {
        if !user_role.admin {
            return error::Kind::Forbidden.into();
        }
        id
    } else {
        user_id
    };

    diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .set(users::password.eq(database.encrypt(request.password)))
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
        let password: String = Faker.fake();

        handler(
            mock.database(),
            user.id(),
            users::Role { admin: false, ..users::Role::default() },
            Request { id: None, password: password.clone() },
        )
        .await
        .unwrap();

        let user = mock.user(0).await;
        assert_eq!(user.password(), password);
    }

    #[rstest]
    #[tokio::test]
    async fn test_handler_admin(
        #[future(awt)]
        #[with(2, 0)]
        mock: Mock,
        #[values(true, false)] admin: bool,
    ) {
        let user_1 = mock.user(0).await;
        let user_2 = mock.user(1).await;
        let password: String = Faker.fake();

        let response = handler(
            mock.database(),
            user_1.id(),
            users::Role { admin, ..users::Role::default() },
            Request { id: Some(user_2.id()), password: password.clone() },
        )
        .await;

        if admin {
            response.unwrap();
            let user_2 = mock.user(1).await;
            assert_eq!(user_2.password(), password);
        } else {
            assert!(response.is_err());
        }
    }
}
