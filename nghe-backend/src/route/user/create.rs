use diesel_async::RunQueryDsl;
pub use nghe_api::user::create::{Request, Response};
use nghe_proc_macro::handler;

use crate::Error;
use crate::database::Database;
use crate::orm::users;
use crate::route::permission;

#[handler(role = admin, internal = true)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { username, password, email, role, allow } = request;
    let password = database.encrypt(password);

    let user_id = diesel::insert_into(users::table)
        .values(users::Data {
            info: users::Info { username: username.into(), email: email.into(), role: role.into() },
            password: password.into(),
        })
        .returning(users::id)
        .get_result(&mut database.get().await?)
        .await?;

    if allow {
        permission::add::handler(
            database,
            permission::add::Request {
                user_id: Some(user_id),
                music_folder_id: None,
                permission: nghe_api::permission::Permission::default(),
            },
        )
        .await?;
    }
    Ok(Response { user_id })
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
    async fn test_create_user(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        let user_id = handler(mock.database(), Faker.fake()).await.unwrap().user_id;
        assert_eq!(mock.user_id(0).await, user_id);
    }
}
