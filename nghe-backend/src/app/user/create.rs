use diesel_async::RunQueryDsl;
pub use nghe_api::user::create::{Request, Response};
use nghe_proc_macro::handler;

use crate::app::permission;
use crate::app::state::Database;
use crate::orm::users;
use crate::Error;

#[handler(role = admin)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { username, password, email, role, allow } = request;
    let password = database.encrypt(password);

    let user_id = diesel::insert_into(users::table)
        .values(users::Data {
            username: username.into(),
            password: password.into(),
            email: email.into(),
            role: role.into(),
        })
        .returning(users::schema::id)
        .get_result(&mut database.get().await?)
        .await?;

    if allow {
        permission::add::handler(
            database,
            permission::add::Request { user_id: Some(user_id), music_folder_id: None },
        )
        .await?;
    }
    Ok(Response { user_id })
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use rstest::rstest;

    use super::*;
    use crate::test::{mock, Mock};

    #[rstest]
    #[tokio::test]
    async fn test_create_user(
        #[future(awt)]
        #[with(0, 0)]
        mock: Mock,
    ) {
        let user_id = handler(mock.database(), Faker.fake()).await.unwrap().user_id;
        assert_eq!(mock.user(0).await.user.id, user_id);
    }
}
