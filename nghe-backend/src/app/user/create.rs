use diesel_async::RunQueryDsl;
pub use nghe_api::user::create::{Request, Response};
use nghe_proc_macro::handler;

use crate::app::permission;
use crate::app::state::Database;
use crate::orm::users;
use crate::Error;

#[handler(role = admin)]
pub async fn handler(database: &Database, request: Request) -> Result<Response, Error> {
    let Request { username, password, email, admin, stream, download, share, allow } = request;
    let password = database.encrypt(password);

    let user_id = diesel::insert_into(users::table)
        .values(users::Data {
            username: username.into(),
            email: email.into(),
            password: password.into(),
            role: users::Role { admin, stream, download, share },
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
